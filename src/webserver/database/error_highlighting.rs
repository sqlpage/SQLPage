use std::fmt::Write;

/// Display a database error with a highlighted line and character offset.
#[must_use]
pub fn display_db_error(context: &str, query: &str, db_err: sqlx::error::Error) -> anyhow::Error {
    let mut msg = format!("{context}:\n");
    let offset = if let sqlx::error::Error::Database(db_err) = &db_err {
        db_err.offset()
    } else {
        None
    };
    if let Some(mut offset) = offset {
        for (line_no, line) in query.lines().enumerate() {
            if offset > line.len() {
                offset -= line.len() + 1;
            } else {
                highlight_line_offset(&mut msg, line, offset);
                write!(msg, "line {}, character {offset}", line_no + 1).unwrap();
                break;
            }
        }
    } else {
        write!(msg, "{}", query.lines().next().unwrap_or_default()).unwrap();
    }
    anyhow::Error::new(db_err).context(msg)
}

/// Highlight a line with a character offset.
pub fn highlight_line_offset(msg: &mut String, line: &str, offset: usize) {
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
        current_line_num += 1;
        if current_line_num + 1 == line_num || current_line_num == line_num + 1 {
            writeln!(msg, "{line}").unwrap();
        } else if current_line_num == line_num {
            highlight_line_offset(&mut msg, line, col_num_usize);
        } else if current_line_num > line_num + 1 {
            break;
        }
    }
    msg
}

#[test]
fn test_quote_source_with_highlight() {
    let source = "SELECT *\nFROM table\nWHERE <syntax error>";
    let expected = "FROM table\nWHERE <syntax error>\n     ⬆️\n";
    assert_eq!(quote_source_with_highlight(source, 3, 6), expected);
}
