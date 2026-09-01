//! SQL-file front end that turns source text into a cacheable execution plan.
//!
//! A routed `.sql` file reaches this module before request-specific values are
//! available. It selects the connected DBMS's parser dialect, tokenizes and
//! parses statements with source locations, and classifies each statement as a
//! query, `SQLPage` `SET` assignment, CSV import, or error. Parse and rewrite
//! errors are retained as statements so they can flow through `SQLPage`'s normal
//! execution and rendering error path, while the resulting [`SqlFile`] remains
//! request-independent and safe to reuse from the file cache.
//!
//! Ordinary statements and `SET` value queries are delegated to [`rewrite`],
//! which lowers the parsed AST into database SQL plus SQLPage-owned expressions.
//! The immutable statement types live in [`statement`]; `execute_queries` later
//! supplies request variables, runs the database work, and evaluates the
//! `SQLPage` expressions. This module therefore owns parsing and statement
//! classification, but not query execution or database-row decoding.

use std::fmt::Write as _;
use std::path::Path;

use async_trait::async_trait;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::{
    Expr, Ident, ObjectName, ObjectNamePart, SelectFlavor, SelectItem, Set, SetExpr, Spanned,
    Statement, Value,
};
use sqlparser::dialect::Dialect;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::Token::{self, EOF, SemiColon};
use sqlparser::tokenizer::{Location, Span, TokenWithSpan, Tokenizer};

#[cfg(test)]
use super::SupportedDatabase;
use super::csv_import::extract_csv_copy_statement;
use super::{Database, DbInfo};
use crate::AppState;
use crate::file_cache::AsyncFromStrWithState;
use crate::webserver::database::error_highlighting::quote_source_with_highlight;

mod dialect;
mod rewrite;
mod statement;

#[cfg(test)]
pub(super) use statement::SourceLocation;
pub use statement::SqlFile;
pub(super) use statement::{
    DatabaseQuery, FileStatement, OutputColumn, Query, QueryBody, SourceSpan, StaticSimpleSelect,
    VariableName,
};

impl SqlFile {
    #[must_use]
    pub fn new(db: &Database, sql: &str, source_path: &Path) -> Self {
        let dialect = dialect::parser_dialect(db.info.database_type);
        log::debug!(
            "Parsing SQL file {} using dialect {:?}",
            source_path.display(),
            dialect
        );
        let statements = match parse_sql(&db.info, dialect.as_ref(), sql) {
            Ok(statements) => statements.collect::<Vec<_>>().into_boxed_slice(),
            Err(error) => {
                return Self::from_error(error, source_path);
            }
        };
        Self {
            statements,
            source_path: source_path.to_path_buf(),
        }
    }

    fn from_error(error: impl Into<anyhow::Error>, source_path: &Path) -> Self {
        Self {
            statements: vec![FileStatement::Error(
                error
                    .into()
                    .context(format!("Error parsing file {}", source_path.display())),
            )]
            .into_boxed_slice(),
            source_path: source_path.to_path_buf(),
        }
    }
}

#[async_trait(?Send)]
impl AsyncFromStrWithState for SqlFile {
    async fn from_str_with_state(
        app_state: &AppState,
        source: &str,
        source_path: &Path,
    ) -> anyhow::Result<Self> {
        Ok(Self::new(&app_state.db, source, source_path))
    }
}

fn parse_sql<'a>(
    database: &'a DbInfo,
    dialect: &'a dyn Dialect,
    sql: &'a str,
) -> anyhow::Result<impl Iterator<Item = FileStatement> + 'a> {
    log::trace!("Parsing {} SQL: {sql}", database.dbms_name);
    let tokens = Tokenizer::new(dialect, sql)
        .tokenize_with_location()
        .map_err(|error| {
            let location = error.location;
            anyhow::Error::new(error).context(format!(
                "The SQLPage parser could not understand the SQL file. Tokenization failed. Please check for syntax errors:\n{}",
                quote_source_with_highlight(sql, location.line, location.column)
            ))
        })?;
    let mut parser = Parser::new(dialect).with_tokens_with_locations(tokens);
    let mut has_error = false;
    Ok(std::iter::from_fn(move || {
        if has_error {
            return None;
        }
        let statement = parse_single_statement(&mut parser, database, sql);
        if matches!(statement, Some(FileStatement::Error(_))) {
            has_error = true;
        }
        statement
    }))
}

fn parse_single_statement(
    parser: &mut Parser<'_>,
    database: &DbInfo,
    source_sql: &str,
) -> Option<FileStatement> {
    if parser.peek_token() == EOF {
        return None;
    }
    let mut statement = match parser.parse_statement() {
        Ok(statement) => statement,
        Err(error) => return Some(syntax_error(error, parser, source_sql)),
    };
    let mut semicolon = false;
    while parser.consume_token(&SemiColon) {
        semicolon = true;
    }

    if let Some(statement) = extract_set_variable(&mut statement, database) {
        return Some(statement);
    }
    if let Some(csv_import) = extract_csv_copy_statement(&mut statement) {
        return Some(FileStatement::CsvImport(csv_import));
    }

    Some(
        match rewrite::rewrite_query(statement, database, semicolon) {
            Ok(query) => FileStatement::Query(query),
            Err(error) => FileStatement::Error(error),
        },
    )
}

fn extract_set_variable(statement: &mut Statement, database: &DbInfo) -> Option<FileStatement> {
    let Statement::Set(Set::SingleAssignment {
        variable: ObjectName(name),
        values,
        scope: None,
        hivevar: false,
    }) = statement
    else {
        return None;
    };
    let ([ObjectNamePart::Identifier(identifier)], [value]) =
        (name.as_mut_slice(), values.as_mut_slice())
    else {
        return None;
    };

    let mut target = std::mem::take(&mut identifier.value);
    if target.starts_with(['$', ':', '?']) {
        target.remove(0);
    }
    let expression = std::mem::replace(value, Expr::value(Value::Null));
    let value_statement = expression_to_query(expression);
    Some(
        match rewrite::rewrite_query(value_statement, database, false) {
            Ok(value) => FileStatement::SetVariable {
                target: VariableName(target),
                value,
            },
            Err(error) => FileStatement::Error(error),
        },
    )
}

fn syntax_error(error: ParserError, parser: &Parser<'_>, sql: &str) -> FileStatement {
    let Span {
        start: Location {
            line: start_line,
            column: start_column,
        },
        end: Location { line: end_line, .. },
    } = parser.peek_token_no_skip().span;
    let mut message = String::from(
        "Parsing failed: SQLPage couldn't understand the SQL file. Please check for syntax errors on ",
    );
    if start_line == end_line {
        write!(&mut message, "line {start_line}:").unwrap();
    } else {
        write!(&mut message, "lines {start_line} to {end_line}:").unwrap();
    }
    write!(
        &mut message,
        "\n{}",
        quote_source_with_highlight(sql, start_line, start_column)
    )
    .unwrap();
    FileStatement::Error(anyhow::Error::from(error).context(message))
}

fn expression_to_query(expression: Expr) -> Statement {
    if let Expr::Subquery(query) = expression {
        return Statement::Query(query);
    }
    Statement::Query(Box::new(sqlparser::ast::Query {
        with: None,
        body: Box::new(SetExpr::Select(Box::new(sqlparser::ast::Select {
            select_token: AttachedToken(TokenWithSpan::new(
                Token::make_keyword("SELECT"),
                expression.span(),
            )),
            distinct: None,
            top: None,
            projection: vec![SelectItem::ExprWithAlias {
                expr: expression,
                alias: Ident::new("sqlpage_set_expr"),
            }],
            into: None,
            from: vec![],
            lateral_views: vec![],
            selection: None,
            group_by: sqlparser::ast::GroupByExpr::Expressions(vec![], vec![]),
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            named_window: vec![],
            qualify: None,
            top_before_distinct: false,
            prewhere: None,
            window_before_qualify: false,
            value_table_mode: None,
            connect_by: Vec::new(),
            optimizer_hints: vec![],
            select_modifiers: None,
            flavor: SelectFlavor::Standard,
            exclude: None,
        }))),
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: vec![],
        for_clause: None,
        settings: None,
        format_clause: None,
        pipe_operators: Vec::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webserver::database::sqlpage_expr::{
        ConcatNullBehavior, RowInputId, SqlPageExpr, VariableRef, VariableSource,
    };
    use crate::webserver::database::sqlpage_functions::functions::SqlPageFunctionName;
    use sqlparser::dialect::{MySqlDialect, PostgreSqlDialect};
    use sqlx::any::AnyKind;

    fn database(database_type: SupportedDatabase) -> DbInfo {
        let kind = match database_type {
            SupportedDatabase::Postgres => AnyKind::Postgres,
            SupportedDatabase::Mssql => AnyKind::Mssql,
            SupportedDatabase::MySql => AnyKind::MySql,
            SupportedDatabase::Sqlite => AnyKind::Sqlite,
            _ => AnyKind::Odbc,
        };
        DbInfo {
            dbms_name: database_type.display_name().to_owned(),
            database_type,
            kind,
        }
    }

    fn one(sql: &str) -> FileStatement {
        one_for(SupportedDatabase::Postgres, sql)
    }

    fn one_for(database_type: SupportedDatabase, sql: &str) -> FileStatement {
        let database = database(database_type);
        parse_sql(&database, &PostgreSqlDialect {}, sql)
            .unwrap()
            .next()
            .unwrap()
    }

    fn rewrite_database(sql: &str) -> DatabaseQuery {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one(sql)
        else {
            panic!("expected database query");
        };
        query
    }

    fn call<Input, const N: usize>(
        function: SqlPageFunctionName,
        arguments: [SqlPageExpr<Input>; N],
    ) -> SqlPageExpr<Input> {
        SqlPageExpr::Call {
            function,
            arguments: Box::new(arguments),
        }
    }

    fn coalesce<Input, const N: usize>(arguments: [SqlPageExpr<Input>; N]) -> SqlPageExpr<Input> {
        SqlPageExpr::Coalesce(Box::new(arguments))
    }

    fn concat<Input, const N: usize>(arguments: [SqlPageExpr<Input>; N]) -> SqlPageExpr<Input> {
        SqlPageExpr::Concat {
            arguments: Box::new(arguments),
            null_behavior: ConcatNullBehavior::PropagateNull,
        }
    }

    fn variable<Input>(name: &str) -> SqlPageExpr<Input> {
        SqlPageExpr::Variable(VariableRef {
            name: name.into(),
            source: VariableSource::SetOrUrl,
        })
    }

    fn row(index: usize) -> SqlPageExpr<RowInputId> {
        SqlPageExpr::Input(RowInputId::new(index))
    }

    fn text<Input>(value: &str) -> SqlPageExpr<Input> {
        SqlPageExpr::Literal(serde_json::Value::String(value.into()))
    }

    #[test]
    fn database_only_parent_forces_binding() {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one("select upper(sqlpage.url_encode('x'))")
        else {
            panic!("expected database query");
        };
        assert_eq!(query.bindings.len(), 1);
        assert!(query.computed_columns.is_empty());
        assert!(query.sql.contains("upper($1)"));
    }

    #[test]
    fn emulated_parent_keeps_nested_call_per_row() {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one("select coalesce(sqlpage.url_encode(value), '') as encoded from t")
        else {
            panic!("expected database query");
        };
        assert!(query.bindings.is_empty());
        assert_eq!(query.row_input_json.len(), 1);
        assert_eq!(query.computed_columns.len(), 1);
        assert!(!query.sql.contains("sqlpage."));
    }

    #[test]
    fn parentheses_keep_nested_call_per_row() {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one("select (sqlpage.url_encode(value)) as encoded from t")
        else {
            panic!("expected database query");
        };
        assert!(query.bindings.is_empty());
        assert_eq!(query.row_input_json.len(), 1);
        assert_eq!(query.computed_columns.len(), 1);
        assert!(!query.sql.contains("sqlpage."));
    }

    #[test]
    fn row_value_cannot_cross_database_only_parent() {
        let FileStatement::Error(error) = one("select upper(sqlpage.url_encode(value)) from t")
        else {
            panic!("expected rewrite error");
        };
        assert!(format!("{error:#}").contains("required before the query"));
    }

    #[test]
    fn positional_bindings_follow_source_order() {
        let database = database(SupportedDatabase::MySql);
        let mut statements = parse_sql(
            &database,
            &MySqlDialect {},
            "select $a, upper(sqlpage.url_encode($b))",
        )
        .unwrap();
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = statements.next().unwrap()
        else {
            panic!("expected database query");
        };
        assert_eq!(query.bindings.len(), 2);
        assert_eq!(query.sql.matches('?').count(), 2);
        assert_eq!(
            query.bindings.as_ref(),
            [
                variable("a"),
                call(SqlPageFunctionName::url_encode, [variable("b")])
            ]
        );
    }

    #[test]
    fn positional_bindings_follow_cte_rendering_order() {
        let database = database(SupportedDatabase::MySql);
        let statement = parse_sql(
            &database,
            &MySqlDialect {},
            "with c as (select $a as x) select $b as y from c",
        )
        .unwrap()
        .next()
        .unwrap();
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = statement
        else {
            panic!("expected database query");
        };
        assert_eq!(query.sql, "WITH c AS (SELECT ? AS x) SELECT ? AS y FROM c");
        assert_eq!(query.bindings.as_ref(), [variable("a"), variable("b")]);
    }

    #[test]
    fn positional_bindings_follow_rendered_projection_order() {
        let database = database(SupportedDatabase::MySql);
        let statement = parse_sql(
            &database,
            &MySqlDialect {},
            "select $a as a, sqlpage.url_encode(upper(col || sqlpage.url_encode($b))) as b, $c as c from t",
        )
        .unwrap()
        .next()
        .unwrap();
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = statement
        else {
            panic!("expected database query");
        };
        assert_eq!(
            query.bindings.as_ref(),
            [
                variable("a"),
                variable("c"),
                call(SqlPageFunctionName::url_encode, [variable("b")]),
            ]
        );
    }

    #[test]
    fn numbered_bindings_keep_source_projection_order() {
        let query = rewrite_database(
            "select $a as a, sqlpage.url_encode(upper(col || sqlpage.url_encode($b))) as b, $c as c from t",
        );
        assert_eq!(
            query.bindings.as_ref(),
            [
                variable("a"),
                call(SqlPageFunctionName::url_encode, [variable("b")]),
                variable("c"),
            ]
        );
    }

    #[test]
    fn numbered_bindings_keep_source_argument_order() {
        let query = rewrite_database(
            "select coalesce(upper(sqlpage.url_encode($a)), sqlpage.url_encode(upper(sqlpage.url_encode($b)))) from t",
        );
        assert_eq!(
            query.bindings.as_ref(),
            [
                call(SqlPageFunctionName::url_encode, [variable("a")]),
                call(SqlPageFunctionName::url_encode, [variable("b")]),
            ]
        );
    }

    #[test]
    fn database_cannot_order_by_computed_column() {
        let FileStatement::Error(error) =
            one("select sqlpage.url_encode(value) as encoded from t order by encoded")
        else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("ORDER BY"));
    }

    #[test]
    fn database_cannot_order_by_ordinal_with_computed_columns() {
        let FileStatement::Error(error) =
            one("select a, sqlpage.url_encode(b) as encoded, c from t order by 3")
        else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("ORDER BY"));
    }

    #[test]
    fn database_cannot_group_by_computed_column() {
        let FileStatement::Error(error) = one(
            "select coalesce(sqlpage.url_encode(name), '') as enc, count(*) from users group by enc",
        ) else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("GROUP BY"));
    }

    #[test]
    fn database_cannot_group_by_ordinal_with_computed_columns() {
        let FileStatement::Error(error) =
            one("select a, sqlpage.url_encode(b) as encoded from t group by 1")
        else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("GROUP BY"));
    }

    #[test]
    fn database_cannot_filter_by_computed_column_in_having() {
        let FileStatement::Error(error) = one(
            "select sqlpage.url_encode(name) as enc, count(*) from users group by name having enc <> ''",
        ) else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("HAVING"));
    }

    #[test]
    fn database_cannot_filter_by_computed_column_in_where() {
        let FileStatement::Error(error) =
            one("select sqlpage.url_encode(name) as enc from users where enc <> ''")
        else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("WHERE"));
    }

    #[test]
    fn database_cannot_group_by_computed_column_in_expression() {
        let FileStatement::Error(error) =
            one("select sqlpage.url_encode(name) as enc, count(*) from users group by lower(enc)")
        else {
            panic!("expected rewrite error");
        };
        assert!(error.to_string().contains("GROUP BY"));
    }

    #[test]
    fn placeholder_like_literal_is_not_rewritten() {
        let database = database(SupportedDatabase::MySql);
        let statement = parse_sql(
            &database,
            &MySqlDialect {},
            "select '@SQLPAGE_TEMP1' where id = $id",
        )
        .unwrap()
        .next()
        .unwrap();
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = statement
        else {
            panic!("expected database query");
        };
        assert!(query.sql.contains("'@SQLPAGE_TEMP1'"));
        assert_eq!(query.bindings.len(), 1);
    }

    #[test]
    fn effectful_bindings_are_not_deduplicated() {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one("select 1 as value where sqlpage.random_string(1) <> sqlpage.random_string(1)")
        else {
            panic!("expected database query");
        };
        assert_eq!(query.bindings.len(), 2);
    }

    #[test]
    fn nested_run_sql_requires_buffering() {
        let FileStatement::Query(Query {
            body: QueryBody::Database(query),
            ..
        }) = one("select coalesce(sqlpage.run_sql(path), '') from files")
        else {
            panic!("expected database query");
        };
        assert!(query.must_buffer_rows());
    }

    #[test]
    fn static_simple_select_has_no_database_query() {
        let FileStatement::Query(Query {
            body: QueryBody::StaticSimpleSelect(query),
            ..
        }) = one("select sqlpage.url_encode('a b') as value")
        else {
            panic!("expected a single SQLPage-owned row");
        };
        assert_eq!(query.columns.len(), 1);
    }

    #[test]
    fn concat_operator_uses_backend_null_behavior_in_sqlpage_expressions() {
        for database_type in [SupportedDatabase::Oracle, SupportedDatabase::Mssql] {
            let FileStatement::Query(Query {
                body: QueryBody::StaticSimpleSelect(query),
                ..
            }) = one_for(database_type, "select '/' || null as path")
            else {
                panic!("expected a single SQLPage-owned row");
            };
            assert!(matches!(
                &query.columns[0].value,
                SqlPageExpr::Concat {
                    null_behavior: ConcatNullBehavior::IgnoreNull,
                    ..
                }
            ));

            let FileStatement::Query(Query {
                body: QueryBody::Database(query),
                ..
            }) = one_for(
                database_type,
                "select sqlpage.url_encode('/' || nullable_col) as path from input_rows",
            )
            else {
                panic!("expected a database query");
            };
            let SqlPageExpr::Call { arguments, .. } = &query.computed_columns[0].value else {
                panic!("expected a SQLPage function call");
            };
            assert!(matches!(
                &arguments[0],
                SqlPageExpr::Concat {
                    null_behavior: ConcatNullBehavior::IgnoreNull,
                    ..
                }
            ));
        }
    }

    #[test]
    fn unquoted_sqlpage_names_are_case_insensitive() {
        let FileStatement::Query(Query {
            body: QueryBody::StaticSimpleSelect(query),
            ..
        }) = one("select SQLPAGE.URL_ENCODE('a b') as value")
        else {
            panic!("expected a single SQLPage-owned row");
        };
        assert_eq!(query.columns.len(), 1);
    }

    #[test]
    fn mixed_database_and_row_boundaries_are_rewritten_together() {
        assert_eq!(
            rewrite_database(
                "select coalesce(upper(sqlpage.url_encode($prefix)), sqlpage.url_encode(value)) as result from t"
            ),
            DatabaseQuery {
                sql: "SELECT upper($1) AS \"__sqlpage_input_0\", value AS \"__sqlpage_input_1\" FROM t".into(),
                bindings: Box::new([call(SqlPageFunctionName::url_encode, [variable("prefix")])]),
                row_input_json: Box::new([false, false]),
                computed_columns: Box::new([OutputColumn {
                    name: "result".into(),
                    value: coalesce([
                        row(0),
                        call(SqlPageFunctionName::url_encode, [row(1)]),
                    ]),
                }]),
                json_columns: Box::new([]),
            }
        );
    }

    #[test]
    fn database_fragment_promoted_to_row_input_keeps_source_variables() {
        assert_eq!(
            rewrite_database("select concat(1 + 1, sqlpage.request_method(), $x) as result from t"),
            DatabaseQuery {
                sql: "SELECT 1 + 1 AS \"__sqlpage_input_0\" FROM t".into(),
                bindings: Box::new([]),
                row_input_json: Box::new([false]),
                computed_columns: Box::new([OutputColumn {
                    name: "result".into(),
                    value: SqlPageExpr::Concat {
                        arguments: Box::new([
                            row(0),
                            call(SqlPageFunctionName::request_method, []),
                            variable("x"),
                        ]),
                        null_behavior: ConcatNullBehavior::IgnoreNull,
                    },
                }]),
                json_columns: Box::new([]),
            }
        );
    }

    #[test]
    fn private_row_input_json_flags_follow_row_input_ids() {
        let query = rewrite_database(
            "select concat(to_json(value), sqlpage.url_encode(other)) as result from t",
        );
        assert_eq!(query.row_input_json.as_ref(), [true, false]);
        let SqlPageExpr::Concat { arguments, .. } = &query.computed_columns[0].value else {
            panic!("expected a concatenated per-row expression");
        };
        assert_eq!(
            arguments.as_ref(),
            [row(0), call(SqlPageFunctionName::url_encode, [row(1)])]
        );
    }

    #[test]
    fn predicate_call_is_standalone_while_projection_call_is_per_row() {
        assert_eq!(
            rewrite_database(
                "select sqlpage.url_encode(value) as encoded from t where sqlpage.url_encode($expected) = 'x'"
            ),
            DatabaseQuery {
                sql: "SELECT value AS \"__sqlpage_input_0\" FROM t WHERE $1 = 'x'".into(),
                bindings: Box::new([call(
                    SqlPageFunctionName::url_encode,
                    [variable("expected")]
                )]),
                row_input_json: Box::new([false]),
                computed_columns: Box::new([OutputColumn {
                    name: "encoded".into(),
                    value: call(SqlPageFunctionName::url_encode, [row(0)]),
                }]),
                json_columns: Box::new([]),
            }
        );
    }

    #[test]
    fn request_and_row_values_share_one_per_row_expression() {
        assert_eq!(
            rewrite_database(
                "select coalesce(sqlpage.url_encode($prefix || value), '') as encoded from t"
            ),
            DatabaseQuery {
                sql: "SELECT value AS \"__sqlpage_input_0\" FROM t".into(),
                bindings: Box::new([]),
                row_input_json: Box::new([false]),
                computed_columns: Box::new([OutputColumn {
                    name: "encoded".into(),
                    value: coalesce([
                        call(
                            SqlPageFunctionName::url_encode,
                            [concat([variable("prefix"), row(0)])],
                        ),
                        text(""),
                    ]),
                }]),
                json_columns: Box::new([]),
            }
        );
    }

    fn sql_for_dbinfo(info: &DbInfo, sql: &str) -> String {
        match parse_sql(info, &PostgreSqlDialect {}, sql).unwrap().next() {
            Some(FileStatement::Query(Query {
                body: QueryBody::Database(q),
                ..
            })) => q.sql,
            other => panic!("Expected database query for `{sql}`\nGot: {other:?}"),
        }
    }

    fn sql_for(db: SupportedDatabase, sql: &str) -> String {
        sql_for_dbinfo(&database(db), sql)
    }

    fn odbc_sql_for(db: SupportedDatabase, sql: &str) -> String {
        sql_for_dbinfo(
            &DbInfo {
                dbms_name: db.display_name().to_owned(),
                database_type: db,
                kind: AnyKind::Odbc,
            },
            sql,
        )
    }

    #[test]
    fn variables_keep_cast_only_where_typing_is_unpredictable() {
        use SupportedDatabase::*;
        let src = "SELECT $a";
        assert_eq!(sql_for(Sqlite, src), "SELECT CAST(?1 AS TEXT)");
        assert_eq!(sql_for(Oracle, src), "SELECT CAST(? AS VARCHAR(4000))");
        assert_eq!(sql_for(Snowflake, src), "SELECT CAST(? AS VARCHAR)");
        assert_eq!(sql_for(Generic, src), "SELECT CAST(? AS VARCHAR)");
        assert_eq!(sql_for(Postgres, src), "SELECT $1");
        assert_eq!(sql_for(MySql, src), "SELECT ?");
        assert_eq!(sql_for(Mssql, src), "SELECT @p1");
        assert_eq!(sql_for(Duckdb, src), "SELECT ?");
    }

    #[test]
    fn odbc_cast_follows_database() {
        use SupportedDatabase::*;
        for db in [Postgres, Sqlite] {
            assert_eq!(odbc_sql_for(db, "select $a"), "SELECT CAST(? AS TEXT)");
        }
        for db in [MySql, Mssql, Duckdb] {
            assert_eq!(odbc_sql_for(db, "select $a"), "SELECT ?");
        }
    }

    #[test]
    fn limit_uses_bare_parameter() {
        assert_eq!(
            sql_for(SupportedDatabase::Postgres, "select value from t limit $n"),
            "SELECT value FROM t LIMIT $1"
        );
    }
}
