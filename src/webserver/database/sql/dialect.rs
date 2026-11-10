//! Database-specific parser dialects and bind-placeholder syntax.

use sqlparser::ast::{Expr, Value};
use sqlparser::dialect::{
    Dialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::{Span, Token};
use sqlx::any::AnyKind;

use crate::webserver::database::SupportedDatabase;

/// Native bind-placeholder forms supported by `sqlx` database backends.
#[derive(Clone, Copy)]
pub(super) enum PlaceholderStyle {
    Numbered { prefix: &'static str },
    Positional { token: &'static str },
}

/// Returns the `sqlparser` dialect matching the configured database.
pub(super) fn parser_dialect(database: SupportedDatabase) -> Box<dyn Dialect> {
    match database {
        SupportedDatabase::Duckdb => Box::new(extended::SqlPageDuckDbDialect::new()),
        SupportedDatabase::Oracle => Box::new(extended::SqlPageOracleDialect::new()),
        SupportedDatabase::Postgres => Box::new(extended::SqlPagePostgreSqlDialect::new()),
        SupportedDatabase::Generic => Box::new(extended::SqlPageGenericDialect::new()),
        SupportedDatabase::Mssql => Box::new(extended::SqlPageMsSqlDialect::new()),
        SupportedDatabase::MySql => Box::new(extended::SqlPageMySqlDialect::new()),
        SupportedDatabase::Sqlite => Box::new(extended::SqlPageSQLiteDialect::new()),
        SupportedDatabase::Snowflake => Box::new(extended::SqlPageSnowflakeDialect::new()),
    }
}

// sqlparser 0.64 only accepts unquoted placeholder names. SQLPage also accepts
// quoted form inputs such as :"First name". Handle these at expression boundaries
// so a JSON_OBJECT key/value separator followed by a quoted column stays intact.
fn parse_quoted_form_parameter(parser: &mut Parser<'_>) -> Option<Result<Expr, ParserError>> {
    let prefix = parser.peek_token();
    if prefix.token != Token::Colon {
        return None;
    }
    let name = parser.peek_nth_token(1);
    let Token::Word(word) = &name.token else {
        return None;
    };
    if word.quote_style.is_none() || prefix.span.end != name.span.start {
        return None;
    }
    let value = Value::Placeholder(format!(":{}", word.value));
    parser.next_token();
    parser.next_token();
    Some(Ok(Expr::Value(
        value.with_span(Span::new(prefix.span.start, name.span.end)),
    )))
}

// derive_dialect! always emits public types and constructors.
#[allow(unreachable_pub)]
mod extended {
    use super::{
        DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
        PostgreSqlDialect, SQLiteDialect, SnowflakeDialect, parse_quoted_form_parameter,
    };

    macro_rules! sqlpage_dialect {
        ($name:ident, $base:ty) => {
            sqlparser::derive_dialect!(
                $name,
                $base,
                preserve_type_id = true,
                overrides = {
                    parse_prefix = parse_quoted_form_parameter(_parser)
                        .or_else(|| self.dialect.parse_prefix(_parser))
                }
            );
        };
    }

    sqlpage_dialect!(SqlPageDuckDbDialect, DuckDbDialect);
    sqlpage_dialect!(SqlPageOracleDialect, OracleDialect);
    sqlpage_dialect!(SqlPagePostgreSqlDialect, PostgreSqlDialect);
    sqlpage_dialect!(SqlPageGenericDialect, GenericDialect);
    sqlpage_dialect!(SqlPageMsSqlDialect, MsSqlDialect);
    sqlpage_dialect!(SqlPageMySqlDialect, MySqlDialect);
    sqlpage_dialect!(SqlPageSQLiteDialect, SQLiteDialect);
    sqlpage_dialect!(SqlPageSnowflakeDialect, SnowflakeDialect);
}

/// Returns the native placeholder syntax emitted into rewritten SQL.
pub(super) fn placeholder_style(kind: AnyKind) -> PlaceholderStyle {
    match kind {
        AnyKind::Sqlite => PlaceholderStyle::Numbered { prefix: "?" },
        AnyKind::Postgres => PlaceholderStyle::Numbered { prefix: "$" },
        AnyKind::Mssql => PlaceholderStyle::Numbered { prefix: "@p" },
        AnyKind::MySql | AnyKind::Odbc => PlaceholderStyle::Positional { token: "?" },
    }
}
