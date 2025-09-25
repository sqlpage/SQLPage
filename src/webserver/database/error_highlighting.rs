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
                return write!(f, "{}", self.query);
            };
            for line in self.query.lines() {
                if offset > line.len() {
                    offset -= line.len() + 1;
                } else {
                    highlight_line_offset(f, line, offset);
                    if let Some(query_position) = self.query_position {
                        let start_line = query_position.start.line;
                        let end_line = query_position.end.line;
                        if start_line == end_line {
                            write!(f, "{}: line {}", self.source_file.display(), start_line)?;
                        } else {
                            write!(
                                f,
                                "{}: lines {} to {}",
                                self.source_file.display(),
                                start_line,
                                end_line
                            )?;
                        }
                    }
                    break;
                }
            }
            Ok(())
        } else {
            write!(f, "{}", self.query)
        }
    }
}

impl std::error::Error for NiceDatabaseError {}

/// Display a database error with a highlighted line and character offset.
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
fn test_quote_source_with_highlight() {
    let source = "SELECT *\nFROM table\nWHERE <syntax error>";
    let expected = "FROM table\nWHERE <syntax error>\n     ⬆️\n";
    assert_eq!(quote_source_with_highlight(source, 3, 6), expected);
}
