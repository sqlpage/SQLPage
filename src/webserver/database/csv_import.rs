use std::collections::HashMap;

use anyhow::Context;
use futures_util::StreamExt;
use sqlparser::ast::{
    CopyLegacyCsvOption, CopyLegacyOption, CopyOption, CopySource, CopyTarget, Statement,
};
use sqlx::{
    any::{AnyArguments, AnyConnectionKind},
    AnyConnection, Arguments, Executor, PgConnection,
};
use crate::webserver::database::SupportedDatabase;
use tokio::io::AsyncRead;

use crate::webserver::http_request_info::RequestInfo;

use super::make_placeholder;

#[derive(Debug, PartialEq)]
pub(super) struct CsvImport {
    /// Used only in postgres
    pub query: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub delimiter: Option<char>,
    pub quote: Option<char>,
    // If true, the first line of the CSV file will be interpreted as a header
    // If false, then the column order will be determined by the order of the columns in the table
    pub header: Option<bool>,
    // A string that will be interpreted as null
    pub null_str: Option<String>,
    pub escape: Option<char>,
    /// Reference the the uploaded file name
    pub uploaded_file: String,
}

enum CopyCsvOption<'a> {
    Legacy(&'a sqlparser::ast::CopyLegacyOption),
    CopyLegacyCsvOption(&'a sqlparser::ast::CopyLegacyCsvOption),
    New(&'a sqlparser::ast::CopyOption),
}

impl CopyCsvOption<'_> {
    fn delimiter(&self) -> Option<char> {
        match self {
            CopyCsvOption::Legacy(CopyLegacyOption::Delimiter(c))
            | CopyCsvOption::New(CopyOption::Delimiter(c)) => Some(*c),
            _ => None,
        }
    }

    fn quote(&self) -> Option<char> {
        match self {
            CopyCsvOption::CopyLegacyCsvOption(CopyLegacyCsvOption::Quote(c))
            | CopyCsvOption::New(CopyOption::Quote(c)) => Some(*c),
            _ => None,
        }
    }

    fn header(&self) -> Option<bool> {
        match self {
            CopyCsvOption::CopyLegacyCsvOption(CopyLegacyCsvOption::Header) => Some(true),
            CopyCsvOption::New(CopyOption::Header(b)) => Some(*b),
            _ => None,
        }
    }

    fn null(&self) -> Option<String> {
        match self {
            CopyCsvOption::New(CopyOption::Null(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn escape(&self) -> Option<char> {
        match self {
            CopyCsvOption::New(CopyOption::Escape(c))
            | CopyCsvOption::CopyLegacyCsvOption(CopyLegacyCsvOption::Escape(c)) => Some(*c),
            _ => None,
        }
    }
}

pub(super) fn extract_csv_copy_statement(stmt: &mut Statement) -> Option<CsvImport> {
    if let Statement::Copy {
        source: CopySource::Table {
            table_name,
            columns,
        },
        to: false,
        target: source,
        options,
        legacy_options,
        values,
    } = stmt
    {
        if !values.is_empty() {
            log::warn!("COPY ... VALUES not compatible with SQLPage: {stmt}");
            return None;
        }
        let uploaded_file = match std::mem::replace(source, CopyTarget::Stdin) {
            CopyTarget::File { filename } => filename,
            other => {
                log::warn!("COPY from {other} not compatible with SQLPage: {stmt}");
                return None;
            }
        };

        let all_options: Vec<CopyCsvOption> = legacy_options
            .iter()
            .flat_map(|o| match o {
                CopyLegacyOption::Csv(o) => {
                    o.iter().map(CopyCsvOption::CopyLegacyCsvOption).collect()
                }
                o => vec![CopyCsvOption::Legacy(o)],
            })
            .chain(options.iter().map(CopyCsvOption::New))
            .collect();

        let table_name = table_name.to_string();
        let columns = columns.iter().map(|ident| ident.value.clone()).collect();
        let delimiter = all_options.iter().find_map(CopyCsvOption::delimiter);
        let quote = all_options.iter().find_map(CopyCsvOption::quote);
        let header = all_options.iter().find_map(CopyCsvOption::header);
        let null = all_options.iter().find_map(CopyCsvOption::null);
        let escape = all_options.iter().find_map(CopyCsvOption::escape);
        let query = stmt.to_string();

        Some(CsvImport {
            query,
            table_name,
            columns,
            delimiter,
            quote,
            header,
            null_str: null,
            escape,
            uploaded_file,
        })
    } else {
        None
    }
}

pub(super) async fn run_csv_import(
    db: &mut AnyConnection,
    dbms: SupportedDatabase,
    csv_import: &CsvImport,
    request: &RequestInfo,
) -> anyhow::Result<()> {
    let named_temp_file = &request
        .uploaded_files
        .get(&csv_import.uploaded_file)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "The request does not contain a field named {:?} with an uploaded file.\n\
                Please check that :\n\
                 - you have selected a file to upload, \n\
                 - the form field name is correct.",
                csv_import.uploaded_file
            )
        })?
        .file;
    let file_path = named_temp_file.path();
    let file = tokio::fs::File::open(file_path).await.with_context(|| {
        format!(
            "The CSV file {} was uploaded correctly, but could not be opened",
            file_path.display()
        )
    })?;
    let buffered = tokio::io::BufReader::new(file);
    // private_get_mut is not supposed to be used outside of sqlx, but it is the only way to
    // access the underlying connection
    match db.private_get_mut() {
        AnyConnectionKind::Postgres(pg_connection) => {
            run_csv_import_postgres(pg_connection, csv_import, buffered).await
        }
        _ => run_csv_import_insert(db, dbms, csv_import, buffered).await,
    }
    .with_context(|| {
        let table_name = &csv_import.table_name;
        format!(
            "{} was uploaded correctly, but its records could not be imported into the table {}",
            file_path.display(),
            table_name
        )
    })
}

/// This function does not parse the CSV file, it only sends it to postgres.
/// This is the fastest way to import a CSV file into postgres
async fn run_csv_import_postgres(
    db: &mut PgConnection,
    csv_import: &CsvImport,
    file: impl AsyncRead + Unpin + Send,
) -> anyhow::Result<()> {
    log::debug!("Running CSV import with postgres");
    let mut copy_transact = db
        .copy_in_raw(csv_import.query.as_str())
        .await
        .with_context(|| "The postgres COPY FROM STDIN command failed.")?;
    log::debug!("Copy transaction created");
    match copy_transact.read_from(file).await {
        Ok(_) => {
            log::debug!("Copy transaction finished successfully");
            copy_transact.finish().await?;
            Ok(())
        }
        Err(e) => {
            log::debug!("Copy transaction failed with error: {e}");
            copy_transact
                .abort("The COPY FROM STDIN command failed.")
                .await?;
            Err(e.into())
        }
    }
}

async fn run_csv_import_insert(
    db: &mut AnyConnection,
    dbms: SupportedDatabase,
    csv_import: &CsvImport,
    file: impl AsyncRead + Unpin + Send,
) -> anyhow::Result<()> {
    let insert_stmt = create_insert_stmt(dbms, csv_import);
    log::debug!("CSV data insert statement: {insert_stmt}");
    let mut reader = make_csv_reader(csv_import, file);
    let col_idxs = compute_column_indices(&mut reader, csv_import).await?;
    let mut records = reader.into_records();
    while let Some(record) = records.next().await {
        let r = record.with_context(|| "reading csv record")?;
        process_csv_record(r, db, &insert_stmt, csv_import, &col_idxs).await?;
    }
    Ok(())
}

async fn compute_column_indices<R: AsyncRead + Unpin + Send>(
    reader: &mut csv_async::AsyncReader<R>,
    csv_import: &CsvImport,
) -> anyhow::Result<Vec<usize>> {
    let mut col_idxs = Vec::with_capacity(csv_import.columns.len());
    if csv_import.header.unwrap_or(true) {
        let headers = reader
            .headers()
            .await?
            .iter()
            .enumerate()
            .map(|(i, h)| (h, i))
            .collect::<HashMap<&str, usize>>();
        for column in &csv_import.columns {
            let &idx = headers
                .get(column.as_str())
                .ok_or_else(|| anyhow::anyhow!("CSV Column not found: {column}"))?;
            col_idxs.push(idx);
        }
    } else {
        col_idxs.extend(0..csv_import.columns.len());
    }
    Ok(col_idxs)
}

fn create_insert_stmt(dbms: SupportedDatabase, csv_import: &CsvImport) -> String {
    let columns = csv_import.columns.join(", ");
    let placeholders = csv_import
        .columns
        .iter()
        .enumerate()
        .map(|(i, _)| make_placeholder(dbms, i + 1))
        .fold(String::new(), |mut acc, f| {
            if !acc.is_empty() {
                acc.push_str(", ");
            }
            acc.push_str(&f);
            acc
        });
    let table_name = &csv_import.table_name;
    format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})")
}

async fn process_csv_record(
    record: csv_async::StringRecord,
    db: &mut AnyConnection,
    insert_stmt: &str,
    csv_import: &CsvImport,
    column_indices: &[usize],
) -> anyhow::Result<()> {
    let mut arguments = AnyArguments::default();
    let null_str = csv_import.null_str.as_deref().unwrap_or_default();
    for (&i, column) in column_indices.iter().zip(csv_import.columns.iter()) {
        let value = record.get(i).unwrap_or_default();
        let value = if value == null_str { None } else { Some(value) };
        log::trace!("CSV value: {column}={value:?}");
        arguments.add(value);
    }
    db.execute((insert_stmt, Some(arguments))).await?;
    Ok(())
}

fn make_csv_reader<R: AsyncRead + Unpin + Send>(
    csv_import: &CsvImport,
    file: R,
) -> csv_async::AsyncReader<R> {
    let delimiter = csv_import
        .delimiter
        .and_then(|c| u8::try_from(c).ok())
        .unwrap_or(b',');
    let quote = csv_import
        .quote
        .and_then(|c| u8::try_from(c).ok())
        .unwrap_or(b'"');
    let has_headers = csv_import.header.unwrap_or(true);
    let escape = csv_import.escape.and_then(|c| u8::try_from(c).ok());
    csv_async::AsyncReaderBuilder::new()
        .delimiter(delimiter)
        .quote(quote)
        .has_headers(has_headers)
        .escape(escape)
        .create_reader(file)
}

#[test]
fn test_make_statement() {
    let csv_import = CsvImport {
        query: "COPY my_table (col1, col2) FROM 'my_file.csv' WITH (DELIMITER ';', HEADER)".into(),
        table_name: "my_table".into(),
        columns: vec!["col1".into(), "col2".into()],
        delimiter: Some(';'),
        quote: None,
        header: Some(true),
        null_str: None,
        escape: None,
        uploaded_file: "my_file.csv".into(),
    };
    let insert_stmt = create_insert_stmt(SupportedDatabase::Postgres, &csv_import);
    assert_eq!(
        insert_stmt,
        "INSERT INTO my_table (col1, col2) VALUES ($1, $2)"
    );
}

#[actix_web::test]
async fn test_end_to_end() {
    use sqlx::ConnectOptions;

    let mut copy_stmt = sqlparser::parser::Parser::parse_sql(
        &sqlparser::dialect::GenericDialect {},
        "COPY my_table (col1, col2) FROM 'my_file.csv' (DELIMITER ';', HEADER)",
    )
    .unwrap()
    .into_iter()
    .next()
    .unwrap();
    let csv_import = extract_csv_copy_statement(&mut copy_stmt).unwrap();
    assert_eq!(
        csv_import,
        CsvImport {
            query: "COPY my_table (col1, col2) FROM STDIN (DELIMITER ';', HEADER)".into(),
            table_name: "my_table".into(),
            columns: vec!["col1".into(), "col2".into()],
            delimiter: Some(';'),
            quote: None,
            header: Some(true),
            null_str: None,
            escape: None,
            uploaded_file: "my_file.csv".into(),
        }
    );
    let mut conn = "sqlite::memory:"
        .parse::<sqlx::any::AnyConnectOptions>()
        .unwrap()
        .connect()
        .await
        .unwrap();
    conn.execute("CREATE TABLE my_table (col1 TEXT, col2 TEXT)")
        .await
        .unwrap();
    let csv = "col2;col1\na;b\nc;d"; // order is different from the table
    let file = csv.as_bytes();
    run_csv_import_insert(&mut conn, &csv_import, file)
        .await
        .unwrap();
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT * FROM my_table")
        .fetch_all(&mut conn)
        .await
        .unwrap();
    assert_eq!(
        rows,
        vec![("b".into(), "a".into()), ("d".into(), "c".into())]
    );
}
