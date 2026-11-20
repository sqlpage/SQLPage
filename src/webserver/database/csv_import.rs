use std::collections::HashMap;

use anyhow::Context;
use sqlparser::ast::{
    CopyLegacyCsvOption, CopyLegacyOption, CopyOption, CopySource, CopyTarget, Statement,
};
use sqlx::postgres::PgConnection;
use tokio::io::AsyncRead;

use crate::webserver::http_request_info::RequestInfo;

use super::{DatabaseConnection, DbInfo, Parameter};

#[derive(Debug, PartialEq)]
pub(super) struct CsvImport {
    /// Native PostgreSQL executes the COPY statement with STDIN as its source.
    query: String,
    pub(super) table_name: String,
    columns: Vec<String>,
    format: CsvFormat,
    pub(super) uploaded_file: String,
}

#[derive(Debug, PartialEq)]
struct CsvFormat {
    delimiter: u8,
    quote: u8,
    header: bool,
    null_str: String,
    escape: Option<u8>,
}

impl CsvFormat {
    fn from_options(options: &[CopyOption], legacy: &[CopyLegacyOption]) -> Self {
        let mut format = Self {
            delimiter: b',',
            quote: b'"',
            header: true,
            null_str: String::new(),
            escape: None,
        };
        // The first setting wins, with legacy COPY syntax taking precedence.
        for option in options.iter().rev() {
            match option {
                CopyOption::Delimiter(c) => format.delimiter = u8::try_from(*c).unwrap_or(b','),
                CopyOption::Quote(c) => format.quote = u8::try_from(*c).unwrap_or(b'"'),
                CopyOption::Header(b) => format.header = *b,
                CopyOption::Null(s) => format.null_str.clone_from(s),
                CopyOption::Escape(c) => format.escape = u8::try_from(*c).ok(),
                _ => {}
            }
        }
        for option in legacy.iter().rev() {
            match option {
                CopyLegacyOption::Delimiter(c) => {
                    format.delimiter = u8::try_from(*c).unwrap_or(b',')
                }
                CopyLegacyOption::Csv(options) => {
                    for option in options.iter().rev() {
                        match option {
                            CopyLegacyCsvOption::Header => format.header = true,
                            CopyLegacyCsvOption::Quote(c) => {
                                format.quote = u8::try_from(*c).unwrap_or(b'"')
                            }
                            CopyLegacyCsvOption::Escape(c) => format.escape = u8::try_from(*c).ok(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        format
    }
}

pub(super) fn extract_csv_copy_statement(stmt: &mut Statement) -> Option<CsvImport> {
    let Statement::Copy {
        source: CopySource::Table {
            table_name,
            columns,
        },
        to: false,
        target,
        options,
        legacy_options,
        values,
    } = stmt
    else {
        return None;
    };
    if !values.is_empty() {
        log::warn!("COPY ... VALUES not compatible with SQLPage: {stmt}");
        return None;
    }
    let uploaded_file = match std::mem::replace(target, CopyTarget::Stdin) {
        CopyTarget::File { filename } => filename,
        other => {
            log::warn!("COPY from {other} not compatible with SQLPage: {stmt}");
            return None;
        }
    };
    Some(CsvImport {
        table_name: table_name.to_string(),
        columns: columns.iter().map(|ident| ident.value.clone()).collect(),
        format: CsvFormat::from_options(options, legacy_options),
        query: stmt.to_string(),
        uploaded_file,
    })
}

pub(super) async fn run_csv_import(
    db: &mut dyn DatabaseConnection,
    driver: DbInfo,
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
    if let Some(pg) = db.native().downcast_mut::<PgConnection>() {
        run_csv_import_postgres(pg, csv_import, buffered).await
    } else {
        run_csv_import_insert(db, driver, csv_import, buffered).await
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
    db: &mut dyn DatabaseConnection,
    driver: DbInfo,
    csv_import: &CsvImport,
    file: impl AsyncRead + Unpin + Send,
) -> anyhow::Result<()> {
    let insert_stmt = create_insert_stmt(driver, csv_import);
    log::debug!("CSV data insert statement: {insert_stmt}");
    let format = &csv_import.format;
    let mut reader = csv_async::AsyncReaderBuilder::new()
        .delimiter(format.delimiter)
        .quote(format.quote)
        .escape(format.escape)
        .has_headers(format.header)
        .create_reader(file);
    let mut columns: Vec<_> = csv_import.columns.iter().enumerate().collect();
    if format.header {
        let by_name: HashMap<_, _> = reader.headers().await?.iter().zip(0..).collect();
        for (index, name) in &mut columns {
            *index = *by_name
                .get(name.as_str())
                .with_context(|| format!("CSV Column not found: {name}"))?;
        }
    }

    let mut record = csv_async::StringRecord::new();
    while reader
        .read_record(&mut record)
        .await
        .context("reading csv record")?
    {
        let mut arguments = Vec::with_capacity(columns.len());
        for &(index, name) in &columns {
            let text = record.get(index).unwrap_or_default();
            let value = (text != format.null_str).then_some(text);
            log::trace!("CSV value: {name}={value:?}");
            arguments.push(Parameter::Text(value));
        }
        db.execute_with(&insert_stmt, &arguments).await?;
    }
    Ok(())
}

fn create_insert_stmt(db_kind: DbInfo, csv_import: &CsvImport) -> String {
    let columns = csv_import.columns.join(", ");
    let placeholders = (1..=csv_import.columns.len())
        .map(|i| db_kind.placeholder(i))
        .collect::<Vec<_>>()
        .join(", ");
    let table_name = &csv_import.table_name;
    format!("INSERT INTO {table_name} ({columns}) VALUES ({placeholders})")
}

#[cfg(test)]
fn parse_import(sql: &str) -> CsvImport {
    let mut stmts =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, sql).unwrap();
    extract_csv_copy_statement(&mut stmts[0]).unwrap()
}

#[test]
fn test_make_statement() {
    let csv_import = parse_import(
        "COPY my_table (col1, col2) FROM 'my_file.csv'
        (DELIMITER '|', HEADER FALSE, NULL 'first', NULL 'second', QUOTE '+', ESCAPE '+')
        DELIMITER ';' DELIMITER '?' NULL 'legacy' CSV HEADER QUOTE '!' QUOTE '?' ESCAPE '^'",
    );
    assert_eq!(
        csv_import.format,
        CsvFormat {
            delimiter: b';',
            quote: b'!',
            escape: Some(b'^'),
            header: true,
            null_str: "first".into(),
        }
    );
    let insert_stmt = create_insert_stmt(
        DbInfo::Native(super::SupportedDatabase::Postgres),
        &csv_import,
    );
    assert_eq!(
        insert_stmt,
        "INSERT INTO my_table (col1, col2) VALUES ($1, $2)"
    );
}

#[actix_web::test]
async fn test_end_to_end() {
    let csv_import =
        parse_import("COPY my_table (col1, col2) FROM 'my_file.csv' (DELIMITER ';', HEADER)");
    assert_eq!(
        csv_import,
        CsvImport {
            query: "COPY my_table (col1, col2) FROM STDIN (DELIMITER ';', HEADER)".into(),
            table_name: "my_table".into(),
            columns: vec!["col1".into(), "col2".into()],
            format: CsvFormat {
                delimiter: b';',
                quote: b'"',
                escape: None,
                header: true,
                null_str: String::new(),
            },
            uploaded_file: "my_file.csv".into(),
        }
    );
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    conn.execute_with("CREATE TABLE my_table (col1 TEXT, col2 TEXT)", &[])
        .await
        .unwrap();
    let csv = "col2;col1\na;b\nc;d"; // order is different from the table
    run_csv_import_insert(
        &mut conn,
        DbInfo::Native(super::SupportedDatabase::Sqlite),
        &csv_import,
        csv.as_bytes(),
    )
    .await
    .unwrap();
    drop(conn);
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT * FROM my_table")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(
        rows,
        vec![("b".into(), "a".into()), ("d".into(), "c".into())]
    );
}
