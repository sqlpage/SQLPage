use std::fmt::Write;

#[derive(Debug)]
struct NiceDatabaseError {
    db_err: sqlx::error::Error,
    query: String,
}

impl std::fmt::Display for NiceDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.db_err.fmt(f)?;
        if let sqlx::error::Error::Database(db_err) = &self.db_err {
            let Some(mut offset) = db_err.offset() else {
                return Ok(());
            };
            write!(f, "\n\nError in query:\n")?;
            for (line_no, line) in self.query.lines().enumerate() {
                if offset > line.len() {
                    offset -= line.len() + 1;
                } else {
                    highlight_line_offset(f, line, offset);
                    write!(f, "line {}, character {offset}", line_no + 1)?;
                    break;
                }
            }
        } else {
            write!(f, "{}", self.query.lines().next().unwrap_or_default())?;
        }
        Ok(())
    }
}

impl std::error::Error for NiceDatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.db_err.source()
    }
}

/// Display a database error with a highlighted line and character offset.
#[must_use]
pub fn display_db_error(query: &str, db_err: sqlx::error::Error) -> anyhow::Error {
    anyhow::Error::new(NiceDatabaseError {
        db_err,
        query: query.to_string(),
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
