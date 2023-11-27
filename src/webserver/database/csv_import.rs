use std::collections::HashMap;

use anyhow::Context;
use futures_util::StreamExt;
use sqlparser::ast::{
    CopyLegacyCsvOption, CopyLegacyOption, CopyOption, CopySource, CopyTarget, Statement,
};
use sqlx::{any::AnyArguments, AnyConnection, Arguments, Executor};

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

impl<'a> CopyCsvOption<'a> {
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

pub fn extract_csv_copy_statement(stmt: &mut Statement) -> Option<CsvImport> {
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
        log::warn!("COPY statement not compatible with SQLPage: {stmt}");
        None
    }
}

pub(super) async fn run_csv_import(
    db: &mut AnyConnection,
    csv_import: &CsvImport,
    request: &RequestInfo,
) -> anyhow::Result<()> {
    let file_path = request
        .uploaded_files
        .get(&csv_import.uploaded_file)
        .ok_or_else(|| anyhow::anyhow!("File not found"))?
        .file
        .path();
    let file = tokio::fs::File::open(file_path)
        .await
        .with_context(|| "opening csv")?;
    let insert_stmt = create_insert_stmt(db, csv_import);
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

async fn compute_column_indices(
    reader: &mut csv_async::AsyncReader<tokio::fs::File>,
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

fn create_insert_stmt(db: &mut AnyConnection, csv_import: &CsvImport) -> String {
    let kind = db.kind();
    let columns = csv_import.columns.join(", ");
    let placeholders = csv_import
        .columns
        .iter()
        .enumerate()
        .map(|(i, _)| make_placeholder(kind, i))
        .fold(String::new(), |mut acc, f| {
            acc.push_str(", ");
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

fn make_csv_reader(
    csv_import: &CsvImport,
    file: tokio::fs::File,
) -> csv_async::AsyncReader<tokio::fs::File> {
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
