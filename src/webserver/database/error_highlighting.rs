use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use super::sql::{SourceSpan, StmtWithParams};

#[derive(Debug)]
struct NiceDatabaseError {
    /// The source file that contains the query.
    source_file: PathBuf,
    /// The error that occurred.
    db_err: sqlx::error::Error,
    /// The query that was executed.
    query: String,
    /// The start location of the query in the source file, if the query was extracted from a larger file.
    query_position: Option<SourceSpan>,
}

fn write_source_position_info(
    f: &mut std::fmt::Formatter<'_>,
    source_file: &Path,
    query_position: Option<SourceSpan>,
) -> Result<(), std::fmt::Error> {
    write!(f, "\n{}", source_file.display())?;
    if let Some(query_position) = query_position {
        let start_line = query_position.start.line;
        let end_line = query_position.end.line;
        if start_line == end_line {
            write!(f, ": line {start_line}")?;
        } else {
            write!(f, ": lines {start_line} to {end_line}")?;
        }
    }
    Ok(())
}

impl std::fmt::Display for NiceDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "In \"{}\": The following error occurred while executing an SQL statement:\n{}\n\nThe SQL statement sent by SQLPage was:\n",
            self.source_file.display(),
            self.db_err
        )?;
        if let sqlx::error::Error::Database(db_err) = &self.db_err {
            let Some(mut offset) = db_err.offset() else {
                write!(f, "{}", self.query)?;
                self.show_position_info(f)?;
                return Ok(());
            };
            for line in self.query.lines() {
                if offset > line.len() {
                    offset -= line.len() + 1;
                } else {
                    highlight_line_offset(f, line, offset);
                    self.show_position_info(f)?;
                    break;
                }
            }
            Ok(())
        } else {
            write!(f, "{}", self.query)?;
            self.show_position_info(f)?;
            Ok(())
        }
    }
}

impl NiceDatabaseError {
    fn show_position_info(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write_source_position_info(f, &self.source_file, self.query_position)
    }
}

impl std::error::Error for NiceDatabaseError {}

#[derive(Debug)]
struct NicePositionedError {
    source_file: PathBuf,
    query_position: SourceSpan,
    error: anyhow::Error,
}

impl std::fmt::Display for NicePositionedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "In \"{}\": {}", self.source_file.display(), self.error)?;
        write_source_position_info(f, &self.source_file, Some(self.query_position))
    }
}

impl std::error::Error for NicePositionedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}

/// Display a database error without any position information
#[must_use]
pub fn display_db_error(
    source_file: &Path,
    query: &str,
    db_err: sqlx::error::Error,
) -> anyhow::Error {
    anyhow::Error::new(NiceDatabaseError {
        source_file: source_file.to_path_buf(),
        db_err,
        query: query.to_string(),
        query_position: None,
    })
}

/// Display a database error with a highlighted line and character offset.
#[must_use]
pub fn display_stmt_db_error(
    source_file: &Path,
    stmt: &StmtWithParams,
    db_err: sqlx::error::Error,
) -> anyhow::Error {
    anyhow::Error::new(NiceDatabaseError {
        source_file: source_file.to_path_buf(),
        db_err,
        query: stmt.query.clone(),
        query_position: Some(stmt.query_position),
    })
}

#[must_use]
pub fn display_stmt_error(
    source_file: &Path,
    query_position: SourceSpan,
    error: anyhow::Error,
) -> anyhow::Error {
    anyhow::Error::new(NicePositionedError {
        source_file: source_file.to_path_buf(),
        query_position,
        error,
    })
}

/// Highlight a line with a character offset.
pub fn highlight_line_offset<W: std::fmt::Write>(msg: &mut W, line: &str, offset: usize) {
    writeln!(msg, "{line}").unwrap();
    writeln!(msg, "{}⬆️", " ".repeat(offset)).unwrap();
}

/// Highlight an error given a line and a character offset
/// line and `col_num` are 1-based
pub fn quote_source_with_highlight(source: &str, line_num: u64, col_num: u64) -> String {
    let mut msg = String::new();
    let mut current_line_num: u64 = 1; // 1-based line number
    let col_num_usize = usize::try_from(col_num)
        .unwrap_or_default()
        .saturating_sub(1);
    for line in source.lines() {
        if current_line_num + 1 == line_num || current_line_num == line_num + 1 {
            writeln!(msg, "{line}").unwrap();
        } else if current_line_num == line_num {
            highlight_line_offset(&mut msg, line, col_num_usize);
        } else if current_line_num > line_num + 1 {
            break;
        }
        current_line_num += 1;
    }
    msg
}

#[test]
fn test_display_stmt_error_includes_file_and_line() {
    let err = display_stmt_error(
        Path::new("example.sql"),
        SourceSpan {
            start: super::sql::SourceLocation {
                line: 12,
                column: 3,
            },
            end: super::sql::SourceLocation {
                line: 12,
                column: 17,
            },
        },
        anyhow::anyhow!("boom"),
    );
    let message = err.to_string();
    assert!(message.contains("In \"example.sql\": boom"));
    assert!(message.contains("example.sql: line 12"));
}

#[test]
fn test_quote_source_with_highlight() {
    let source = "SELECT *\nFROM table\nWHERE <syntax error>";
    let expected = "FROM table\nWHERE <syntax error>\n     ⬆️\n";
    assert_eq!(quote_source_with_highlight(source, 3, 6), expected);
}
