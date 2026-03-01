use super::csv_import::{extract_csv_copy_statement, CsvImport};
use super::sqlpage_functions::functions::SqlPageFunctionName;
use super::syntax_tree::StmtParam;
use super::SupportedDatabase;
use crate::file_cache::AsyncFromStrWithState;
use crate::webserver::database::error_highlighting::quote_source_with_highlight;
use crate::webserver::database::DbInfo;
use crate::{AppState, Database};
use async_trait::async_trait;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::{
    CastKind, DataType, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArgumentList,
    FunctionArguments, Ident, ObjectName, ObjectNamePart, SelectFlavor, SelectItem, Set, SetExpr,
    Spanned, Statement, Value, ValueWithSpan,
};
use sqlparser::dialect::{
    Dialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, OracleDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::Token::{self, SemiColon, EOF};
use sqlparser::tokenizer::{Location, Span, TokenWithSpan, Tokenizer};
use sqlx::any::AnyKind;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod parameter_extraction;
use self::parameter_extraction::{
    extract_ident_param, validate_function_calls, ParameterExtractor, TEMP_PLACEHOLDER_PREFIX,
};
pub(super) use self::parameter_extraction::{
    function_args_to_stmt_params, DbPlaceHolder, ParamExtractContext, SqlPageFunctionError,
    DB_PLACEHOLDERS,
};

#[derive(Default)]
pub struct ParsedSqlFile {
    pub(super) statements: Vec<ParsedStatement>,
    pub source_path: PathBuf,
}

impl ParsedSqlFile {
    #[must_use]
    pub fn new(db: &Database, sql: &str, source_path: &Path) -> ParsedSqlFile {
        let dialect = dialect_for_db(db.info.database_type);
        log::debug!(
            "Parsing SQL file {} using dialect {:?}",
            source_path.display(),
            dialect
        );
        let parsed_statements = match parse_sql(&db.info, dialect.as_ref(), sql) {
            Ok(parsed) => parsed,
            Err(err) => return Self::from_err(err, source_path),
        };
        let statements = parsed_statements.collect();
        ParsedSqlFile {
            statements,
            source_path: source_path.to_path_buf(),
        }
    }

    fn from_err(e: impl Into<anyhow::Error>, source_path: &Path) -> Self {
        Self {
            statements: vec![ParsedStatement::Error(
                e.into()
                    .context(format!("Error parsing file {}", source_path.display())),
            )],
            source_path: source_path.to_path_buf(),
        }
    }
}

#[async_trait(? Send)]
impl AsyncFromStrWithState for ParsedSqlFile {
    async fn from_str_with_state(
        app_state: &AppState,
        source: &str,
        source_path: &Path,
    ) -> anyhow::Result<Self> {
        Ok(ParsedSqlFile::new(&app_state.db, source, source_path))
    }
}

/// A single SQL statement that has been parsed from a SQL file.
#[derive(Debug, PartialEq)]
pub(super) struct StmtWithParams {
    /// The SQL query with placeholders for parameters.
    pub query: String,
    /// The line and column of the first token in the query.
    pub query_position: SourceSpan,
    /// Parameters that should be bound to the query.
    /// They can contain functions that will be called before the query is executed,
    /// the result of which will be bound to the query.
    pub params: Vec<StmtParam>,
    /// Functions that are called on the result set after the query has been executed,
    /// and which can be passed the result of the query as an argument.
    pub delayed_functions: Vec<DelayedFunctionCall>,
    /// Columns that are JSON columns, and which should be converted to JSON objects after the query is executed.
    /// Only relevant for databases that do not have a native JSON type, and which return JSON values as text.
    pub json_columns: Vec<String>,
}

/// A location in the source code.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

/// A location in the source code.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub(super) enum ParsedStatement {
    StmtWithParams(StmtWithParams),
    StaticSimpleSelect(Vec<(String, SimpleSelectValue)>),
    SetVariable {
        variable: StmtParam,
        value: StmtWithParams,
    },
    StaticSimpleSet {
        variable: StmtParam,
        value: SimpleSelectValue,
    },
    CsvImport(CsvImport),
    Error(anyhow::Error),
}

#[derive(Debug, PartialEq)]
pub(super) enum SimpleSelectValue {
    Static(serde_json::Value),
    Dynamic(StmtParam),
}

fn parse_sql<'a>(
    db_info: &'a DbInfo,
    dialect: &'a dyn Dialect,
    sql: &'a str,
) -> anyhow::Result<impl Iterator<Item = ParsedStatement> + 'a> {
    log::trace!("Parsing {} SQL: {sql}", db_info.dbms_name);

    let tokens = Tokenizer::new(dialect, sql)
        .tokenize_with_location()
        .map_err(|err| {
            let location = err.location;
            anyhow::Error::new(err).context(format!("The SQLPage parser could not understand the SQL file. Tokenization failed. Please check for syntax errors:\n{}", quote_source_with_highlight(sql, location.line, location.column)))
        })?;
    let mut parser = Parser::new(dialect).with_tokens_with_locations(tokens);
    let mut has_error = false;
    Ok(std::iter::from_fn(move || {
        if has_error {
            // Return the first error and ignore the rest
            return None;
        }
        let statement = parse_single_statement(&mut parser, db_info, sql);
        log::debug!("Parsed statement: {statement:?}");
        if let Some(ParsedStatement::Error(_)) = &statement {
            has_error = true;
        }
        statement
    }))
}

fn transform_to_positional_placeholders(stmt: &mut StmtWithParams, kind: AnyKind) {
    if let Some((_, DbPlaceHolder::Positional { placeholder })) = DB_PLACEHOLDERS
        .iter()
        .find(|(placeholder_kind, _)| *placeholder_kind == kind)
    {
        let mut new_params = Vec::new();
        let mut query = stmt.query.clone();
        while let Some(pos) = query.find(TEMP_PLACEHOLDER_PREFIX) {
            let start_of_number = pos + TEMP_PLACEHOLDER_PREFIX.len();
            let end = query[start_of_number..]
                .find(|c: char| !c.is_ascii_digit())
                .map_or(query.len(), |i| start_of_number + i);
            let param_idx = query[start_of_number..end].parse::<usize>().unwrap_or(1) - 1;
            query.replace_range(pos..end, placeholder);
            new_params.push(stmt.params[param_idx].clone());
        }
        stmt.query = query;
        stmt.params = new_params;
    }
}

fn parse_single_statement(
    parser: &mut Parser<'_>,
    db_info: &DbInfo,
    source_sql: &str,
) -> Option<ParsedStatement> {
    if parser.peek_token() == EOF {
        return None;
    }
    let mut stmt = match parser.parse_statement() {
        Ok(stmt) => stmt,
        Err(err) => return Some(syntax_error(err, parser, source_sql)),
    };
    let mut semicolon = false;
    while parser.consume_token(&SemiColon) {
        semicolon = true;
    }

    let mut params = match ParameterExtractor::extract_parameters(&mut stmt, db_info.clone()) {
        Ok(p) => p,
        Err(err) => return Some(ParsedStatement::Error(err)),
    };
    let dbms = db_info.database_type;
    if let Some(parsed) = extract_set_variable(&mut stmt, &mut params, db_info) {
        return Some(parsed);
    }
    if let Some(csv_import) = extract_csv_copy_statement(&mut stmt) {
        return Some(ParsedStatement::CsvImport(csv_import));
    }
    if let Some(static_statement) = extract_static_simple_select(&stmt, &params) {
        log::debug!("Optimised a static simple select to avoid a trivial database query: {stmt} optimized to {static_statement:?}");
        return Some(ParsedStatement::StaticSimpleSelect(static_statement));
    }

    let delayed_functions = extract_toplevel_functions(&mut stmt);

    if let Err(err) = validate_function_calls(&stmt) {
        return Some(ParsedStatement::Error(err));
    }
    let json_columns = extract_json_columns(&stmt, dbms);
    let query = format!(
        "{stmt}{semicolon}",
        semicolon = if semicolon { ";" } else { "" }
    );
    let mut stmt_with_params = StmtWithParams {
        query,
        query_position: extract_query_start(&stmt),
        params,
        delayed_functions,
        json_columns,
    };
    transform_to_positional_placeholders(&mut stmt_with_params, db_info.kind);
    log::debug!("Final transformed statement: {}", stmt_with_params.query);
    Some(ParsedStatement::StmtWithParams(stmt_with_params))
}

fn extract_query_start(stmt: &impl Spanned) -> SourceSpan {
    let location = stmt.span();
    SourceSpan {
        start: SourceLocation {
            line: usize::try_from(location.start.line).unwrap_or(0),
            column: usize::try_from(location.start.column).unwrap_or(0),
        },
        end: SourceLocation {
            line: usize::try_from(location.end.line).unwrap_or(0),
            column: usize::try_from(location.end.column).unwrap_or(0),
        },
    }
}

fn syntax_error(err: ParserError, parser: &Parser, sql: &str) -> ParsedStatement {
    let Span {
        start: Location {
            line: start_line,
            column: start_column,
        },
        end: Location { line: end_line, .. },
    } = parser.peek_token_no_skip().span;

    let mut msg = String::from(
        "Parsing failed: SQLPage couldn't understand the SQL file. Please check for syntax errors on ",
    );
    if start_line == end_line {
        write!(&mut msg, "line {start_line}:").unwrap();
    } else {
        write!(&mut msg, "lines {start_line} to {end_line}:").unwrap();
    }
    write!(
        &mut msg,
        "\n{}",
        quote_source_with_highlight(sql, start_line, start_column)
    )
    .unwrap();
    ParsedStatement::Error(anyhow::Error::from(err).context(msg))
}

fn dialect_for_db(dbms: SupportedDatabase) -> Box<dyn Dialect> {
    match dbms {
        SupportedDatabase::Duckdb => Box::new(DuckDbDialect {}),
        SupportedDatabase::Oracle => Box::new(OracleDialect {}),
        SupportedDatabase::Postgres => Box::new(PostgreSqlDialect {}),
        SupportedDatabase::Generic => Box::new(GenericDialect {}),
        SupportedDatabase::Mssql => Box::new(MsSqlDialect {}),
        SupportedDatabase::MySql => Box::new(MySqlDialect {}),
        SupportedDatabase::Sqlite => Box::new(SQLiteDialect {}),
        SupportedDatabase::Snowflake => Box::new(SnowflakeDialect {}),
    }
}

#[derive(Debug, PartialEq)]
pub struct DelayedFunctionCall {
    pub function: SqlPageFunctionName,
    pub argument_col_names: Vec<String>,
    pub target_col_name: String,
}

/// The execution of top-level functions is delayed until after the query has been executed.
/// For instance, `SELECT sqlpage.fetch(x) FROM t` will be executed as `SELECT x as _sqlpage_f0_a0 FROM t`
/// and the `sqlpage.fetch` function will be called with the value of `_sqlpage_f0_a0` after the query has been executed,
/// on each row of the result set.
fn extract_toplevel_functions(stmt: &mut Statement) -> Vec<DelayedFunctionCall> {
    struct SelectItemToAdd {
        expr_to_insert: SelectItem,
        position: usize,
    }
    let mut delayed_function_calls: Vec<DelayedFunctionCall> = Vec::new();
    let set_expr = match stmt {
        Statement::Query(q) => q.body.as_mut(),
        _ => return delayed_function_calls,
    };
    let select_items = match set_expr {
        sqlparser::ast::SetExpr::Select(s) => &mut s.projection,
        _ => return delayed_function_calls,
    };
    let mut select_items_to_add: Vec<SelectItemToAdd> = Vec::new();

    for (position, select_item) in select_items.iter_mut().enumerate() {
        let SelectItem::ExprWithAlias {
            expr:
                Expr::Function(Function {
                    name: ObjectName(func_name_parts),
                    args:
                        FunctionArguments::List(FunctionArgumentList {
                            args,
                            duplicate_treatment: None,
                            ..
                        }),
                    ..
                }),
            alias,
        } = select_item
        else {
            continue;
        };
        let Some(func_name) = extract_sqlpage_function_name(func_name_parts) else {
            continue;
        };
        func_name_parts.clear(); // mark the function for deletion
        let mut argument_col_names = Vec::with_capacity(args.len());
        for (arg_idx, arg) in args.iter_mut().enumerate() {
            match arg {
                FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
                | FunctionArg::Named {
                    arg: FunctionArgExpr::Expr(expr),
                    ..
                } => {
                    let func_idx = delayed_function_calls.len();
                    let argument_col_name = format!("_sqlpage_f{func_idx}_a{arg_idx}");
                    argument_col_names.push(argument_col_name.clone());
                    let expr_to_insert = SelectItem::ExprWithAlias {
                        expr: std::mem::replace(expr, Expr::value(Value::Null)),
                        alias: Ident::with_quote('"', argument_col_name),
                    };
                    select_items_to_add.push(SelectItemToAdd {
                        expr_to_insert,
                        position,
                    });
                }
                other => {
                    log::error!("Unsupported argument to {func_name}: {other}");
                }
            }
        }
        delayed_function_calls.push(DelayedFunctionCall {
            function: func_name,
            argument_col_names,
            target_col_name: alias.value.clone(),
        });
    }
    // Insert the new select items (the function arguments) at the positions where the function calls were
    let mut it = select_items_to_add.into_iter().peekable();
    *select_items = std::mem::take(select_items)
        .into_iter()
        .enumerate()
        .flat_map(|(position, item)| {
            let mut items = Vec::with_capacity(1);
            while it.peek().is_some_and(|x| x.position == position) {
                items.push(it.next().unwrap().expr_to_insert);
            }
            if items.is_empty() {
                items.push(item);
            }
            items
        })
        .collect();
    delayed_function_calls
}

fn extract_static_simple_select(
    stmt: &Statement,
    params: &[StmtParam],
) -> Option<Vec<(String, SimpleSelectValue)>> {
    let set_expr = match stmt {
        Statement::Query(q)
            if q.limit_clause.is_none()
                && q.fetch.is_none()
                && q.order_by.is_none()
                && q.with.is_none()
                && q.locks.is_empty() =>
        {
            q.body.as_ref()
        }
        _ => return None,
    };
    let select_items = match set_expr {
        sqlparser::ast::SetExpr::Select(s)
            if s.cluster_by.is_empty()
                && s.distinct.is_none()
                && s.distribute_by.is_empty()
                && s.from.is_empty()
                && s.group_by == sqlparser::ast::GroupByExpr::Expressions(vec![], vec![])
                && s.having.is_none()
                && s.into.is_none()
                && s.lateral_views.is_empty()
                && s.named_window.is_empty()
                && s.qualify.is_none()
                && s.selection.is_none()
                && s.sort_by.is_empty()
                && s.top.is_none() =>
        {
            &s.projection
        }
        _ => return None,
    };
    let mut items = Vec::with_capacity(select_items.len());
    let mut params_iter = params.iter().cloned();
    for select_item in select_items {
        let sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } = select_item else {
            return None;
        };
        let value = expr_to_simple_select_val(&mut params_iter, expr)?;
        let key = alias.value.clone();
        items.push((key, value));
    }
    if let Some(p) = params_iter.next() {
        log::error!("static select extraction failed because of extraneous parameter: {p:?}");
        return None;
    }
    Some(items)
}

fn expr_to_simple_select_val(
    params_iter: &mut impl Iterator<Item = StmtParam>,
    expr: &Expr,
) -> Option<SimpleSelectValue> {
    use serde_json::Value::{Bool, Null, Number, String};
    use SimpleSelectValue::{Dynamic, Static};
    Some(match expr {
        Expr::Value(ValueWithSpan {
            value: Value::Boolean(b),
            ..
        }) => Static(Bool(*b)),
        Expr::Value(ValueWithSpan {
            value: Value::Number(n, _),
            ..
        }) => Static(Number(n.parse().ok()?)),
        Expr::Value(ValueWithSpan {
            value: Value::SingleQuotedString(s),
            ..
        }) => Static(String(s.clone())),
        Expr::Value(ValueWithSpan {
            value: Value::Null, ..
        }) => Static(Null),
        e if is_simple_select_placeholder(e) => {
            if let Some(p) = params_iter.next() {
                Dynamic(p)
            } else {
                log::error!("Parameter not extracted for placehorder: {expr:?}");
                return None;
            }
        }
        other => {
            log::trace!("Cancelling simple select optimization because of expr: {other:?}");
            return None;
        }
    })
}

fn is_simple_select_placeholder(e: &Expr) -> bool {
    match e {
        Expr::Value(ValueWithSpan {
            value: Value::Placeholder(_),
            ..
        }) => true,
        Expr::Cast {
            expr,
            data_type: DataType::Text | DataType::Varchar(_) | DataType::Char(_),
            format: None,
            kind: CastKind::Cast,
        } if is_simple_select_placeholder(expr) => true,
        _ => false,
    }
}

fn extract_set_variable(
    stmt: &mut Statement,
    params: &mut Vec<StmtParam>,
    db_info: &DbInfo,
) -> Option<ParsedStatement> {
    if let Statement::Set(Set::SingleAssignment {
        variable: ObjectName(name),
        values,
        scope: None,
        hivevar: false,
    }) = stmt
    {
        if let ([ObjectNamePart::Identifier(ident)], [value]) =
            (name.as_mut_slice(), values.as_mut_slice())
        {
            let variable = if let Some(variable) = extract_ident_param(ident) {
                variable
            } else {
                StmtParam::PostOrGet(std::mem::take(&mut ident.value))
            };
            let owned_expr = std::mem::replace(value, Expr::value(Value::Null));
            let mut params_iter = params.iter().cloned();
            if let Some(value) = expr_to_simple_select_val(&mut params_iter, &owned_expr) {
                return Some(ParsedStatement::StaticSimpleSet { variable, value });
            }

            let mut select_stmt: Statement = expr_to_statement(owned_expr);
            let delayed_functions = extract_toplevel_functions(&mut select_stmt);
            if let Err(err) = validate_function_calls(&select_stmt) {
                return Some(ParsedStatement::Error(err));
            }
            let json_columns = extract_json_columns(&select_stmt, db_info.database_type);
            let mut value = StmtWithParams {
                query: select_stmt.to_string(),
                query_position: extract_query_start(&select_stmt),
                params: std::mem::take(params),
                delayed_functions,
                json_columns,
            };
            transform_to_positional_placeholders(&mut value, db_info.kind);
            return Some(ParsedStatement::SetVariable { variable, value });
        }
    }
    None
}

const SQLPAGE_FUNCTION_NAMESPACE: &str = "sqlpage";

fn is_sqlpage_func(func_name_parts: &[ObjectNamePart]) -> bool {
    if let [ObjectNamePart::Identifier(Ident { value, .. }), ObjectNamePart::Identifier(Ident { .. })] =
        func_name_parts
    {
        value == SQLPAGE_FUNCTION_NAMESPACE
    } else {
        false
    }
}

fn extract_sqlpage_function_name(
    func_name_parts: &[ObjectNamePart],
) -> Option<SqlPageFunctionName> {
    if let [ObjectNamePart::Identifier(Ident {
        value: namespace, ..
    }), ObjectNamePart::Identifier(Ident { value, .. })] = func_name_parts
    {
        if namespace == SQLPAGE_FUNCTION_NAMESPACE {
            return SqlPageFunctionName::from_str(value).ok();
        }
    }
    None
}

fn sqlpage_func_name(func_name_parts: &[ObjectNamePart]) -> &str {
    if let [ObjectNamePart::Identifier(Ident { .. }), ObjectNamePart::Identifier(Ident { value, .. })] =
        func_name_parts
    {
        value
    } else {
        debug_assert!(
            false,
            "sqlpage function name should have been checked by is_sqlpage_func"
        );
        ""
    }
}

fn extract_json_columns(stmt: &Statement, dbms: SupportedDatabase) -> Vec<String> {
    // Only extract JSON columns for databases without native JSON support
    if matches!(dbms, SupportedDatabase::Postgres | SupportedDatabase::Mssql) {
        return Vec::new();
    }

    let mut json_columns = Vec::new();

    if let Statement::Query(query) = stmt {
        if let SetExpr::Select(select) = query.body.as_ref() {
            for item in &select.projection {
                if let SelectItem::ExprWithAlias { expr, alias } = item {
                    if is_json_function(expr) {
                        json_columns.push(alias.value.clone());
                        log::trace!("Found JSON column: {alias}");
                    }
                }
            }
        }
    }

    json_columns
}

fn is_json_function(expr: &Expr) -> bool {
    match expr {
        Expr::Function(function) => {
            if let [ObjectNamePart::Identifier(Ident { value, .. })] = function.name.0.as_slice() {
                [
                    "json_object",
                    "json_array",
                    "json_build_object",
                    "json_build_array",
                    "to_json",
                    "to_jsonb",
                    "json_agg",
                    "jsonb_agg",
                    "json_arrayagg",
                    "json_objectagg",
                    "json_group_array",
                    "json_group_object",
                    "json",
                    "jsonb",
                ]
                .iter()
                .any(|&func| value.eq_ignore_ascii_case(func))
            } else {
                false
            }
        }
        Expr::Cast { data_type, .. } => {
            if matches!(data_type, DataType::JSON | DataType::JSONB) {
                true
            } else if let DataType::Custom(ObjectName(parts), _) = data_type {
                if let [ObjectNamePart::Identifier(ident)] = parts.as_slice() {
                    ident.value.eq_ignore_ascii_case("json")
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn expr_to_statement(expr: Expr) -> Statement {
    Statement::Query(Box::new(sqlparser::ast::Query {
        with: None,
        body: Box::new(sqlparser::ast::SetExpr::Select(Box::new(
            sqlparser::ast::Select {
                select_token: AttachedToken(TokenWithSpan::new(
                    Token::make_keyword("SELECT"),
                    expr.span(),
                )),
                distinct: None,
                top: None,
                projection: vec![SelectItem::ExprWithAlias {
                    expr,
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
                connect_by: None,
                flavor: SelectFlavor::Standard,
                exclude: None,
            },
        ))),
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
mod test {
    use super::super::sqlpage_functions::functions::SqlPageFunctionName;
    use super::super::syntax_tree::SqlPageFunctionCall;

    use super::*;

    fn parse_stmt(sql: &str, dialect: &dyn Dialect) -> Statement {
        let mut ast = Parser::parse_sql(dialect, sql).unwrap();
        assert_eq!(ast.len(), 1);
        ast.pop().unwrap()
    }

    fn parse_postgres_stmt(sql: &str) -> Statement {
        parse_stmt(sql, &PostgreSqlDialect {})
    }

    #[test]
    fn test_statement_rewrite() {
        let mut ast =
            parse_postgres_stmt("select $a from t where $x > $a OR $x = sqlpage.cookie('cookoo')");
        let db_info = create_test_db_info(SupportedDatabase::Postgres);
        let parameters = ParameterExtractor::extract_parameters(&mut ast, db_info).unwrap();
        // $a -> $1
        // $x -> $2
        // sqlpage.cookie(...) -> $3
        assert_eq!(
        ast.to_string(),
        "SELECT CAST($1 AS TEXT) FROM t WHERE CAST($2 AS TEXT) > CAST($1 AS TEXT) OR CAST($2 AS TEXT) = CAST($3 AS TEXT)"
    );
        assert_eq!(
            parameters,
            [
                StmtParam::PostOrGet("a".to_string()),
                StmtParam::PostOrGet("x".to_string()),
                StmtParam::FunctionCall(SqlPageFunctionCall {
                    function: SqlPageFunctionName::cookie,
                    arguments: vec![StmtParam::Literal("cookoo".to_string())]
                }),
            ]
        );
    }

    #[test]
    fn test_statement_rewrite_sqlite() {
        let mut ast = parse_stmt("select $x, :y from t", &SQLiteDialect {});
        let db_info = create_test_db_info(SupportedDatabase::Sqlite);
        let parameters = ParameterExtractor::extract_parameters(&mut ast, db_info).unwrap();
        assert_eq!(
            ast.to_string(),
            "SELECT CAST(?1 AS TEXT), CAST(?2 AS TEXT) FROM t"
        );
        assert_eq!(
            parameters,
            [
                StmtParam::PostOrGet("x".to_string()),
                StmtParam::Post("y".to_string()),
            ]
        );
    }

    const ALL_DIALECTS: &[(&dyn Dialect, SupportedDatabase)] = &[
        (&PostgreSqlDialect {}, SupportedDatabase::Postgres),
        (&MsSqlDialect {}, SupportedDatabase::Mssql),
        (&MySqlDialect {}, SupportedDatabase::MySql),
        (&SQLiteDialect {}, SupportedDatabase::Sqlite),
    ];

    fn create_test_db_info(database_type: SupportedDatabase) -> DbInfo {
        let kind = match database_type {
            SupportedDatabase::Postgres => AnyKind::Postgres,
            SupportedDatabase::Mssql => AnyKind::Mssql,
            SupportedDatabase::MySql => AnyKind::MySql,
            SupportedDatabase::Sqlite => AnyKind::Sqlite,
            _ => AnyKind::Odbc,
        };
        DbInfo {
            dbms_name: database_type.display_name().to_string(),
            database_type,
            kind,
        }
    }

    #[test]
    fn test_duckdb_odbc_dialect_selection() {
        use std::any::Any;

        let dbms = SupportedDatabase::from_dbms_name("DuckDB");
        assert_eq!(dbms, SupportedDatabase::Duckdb);
        let dialect = dialect_for_db(dbms);
        assert_eq!(dialect.as_ref().type_id(), (DuckDbDialect {}).type_id());

        let sql = "select {'a': 1, 'b': 2} as payload";
        let db_info = create_test_db_info(dbms);
        let mut parsed = parse_sql(&db_info, dialect.as_ref(), sql).unwrap();
        let stmt = parsed.next().expect("expected one statement");
        assert!(
            !matches!(stmt, ParsedStatement::Error(_)),
            "duckdb dictionary literals should parse"
        );

        let pg_info = create_test_db_info(SupportedDatabase::Postgres);
        let mut parsed = parse_sql(&pg_info, &PostgreSqlDialect {}, sql).unwrap();
        let stmt = parsed.next().expect("expected one statement");
        assert!(
            matches!(stmt, ParsedStatement::Error(_)),
            "postgres should reject duckdb dictionary literals"
        );
    }

    #[test]
    fn test_extract_toplevel_delayed_functions() {
        let mut ast = parse_stmt(
            "select sqlpage.fetch($x) as x, sqlpage.persist_uploaded_file('a', 'b') as y from t",
            &PostgreSqlDialect {},
        );
        let functions = extract_toplevel_functions(&mut ast);
        assert_eq!(
            ast.to_string(),
            "SELECT $x AS \"_sqlpage_f0_a0\", 'a' AS \"_sqlpage_f1_a0\", 'b' AS \"_sqlpage_f1_a1\" FROM t"
        );
        assert_eq!(
            functions,
            vec![
                DelayedFunctionCall {
                    function: SqlPageFunctionName::fetch,
                    argument_col_names: vec!["_sqlpage_f0_a0".to_string()],
                    target_col_name: "x".to_string()
                },
                DelayedFunctionCall {
                    function: SqlPageFunctionName::persist_uploaded_file,
                    argument_col_names: vec![
                        "_sqlpage_f1_a0".to_string(),
                        "_sqlpage_f1_a1".to_string()
                    ],
                    target_col_name: "y".to_string()
                }
            ]
        );
    }

    #[test]
    fn test_extract_toplevel_delayed_functions_parameter_order() {
        // The order of the function arguments should be preserved
        // Otherwise the statement parameters will be bound to the wrong arguments
        let sql = "select $a as a, sqlpage.exec('xxx', x = $b) as b, $c as c from t";
        let mut ast = parse_postgres_stmt(sql);
        let delayed_functions = extract_toplevel_functions(&mut ast);
        assert_eq!(
            ast.to_string(),
            "SELECT $a AS a, 'xxx' AS \"_sqlpage_f0_a0\", x = $b AS \"_sqlpage_f0_a1\", $c AS c FROM t"
        );
        assert_eq!(
            delayed_functions,
            &[DelayedFunctionCall {
                function: SqlPageFunctionName::exec,
                argument_col_names: vec![
                    "_sqlpage_f0_a0".to_string(),
                    "_sqlpage_f0_a1".to_string()
                ],
                target_col_name: "b".to_string()
            }]
        );
    }

    #[test]
    fn test_sqlpage_function_with_argument() {
        for &(dialect, _kind) in ALL_DIALECTS {
            let sql = "select sqlpage.fetch($x)";
            let mut ast = parse_stmt(sql, dialect);
            let db_info = create_test_db_info(SupportedDatabase::Postgres);
            let parameters = ParameterExtractor::extract_parameters(&mut ast, db_info).unwrap();
            assert_eq!(
                parameters,
                [StmtParam::FunctionCall(SqlPageFunctionCall {
                    function: SqlPageFunctionName::fetch,
                    arguments: vec![StmtParam::PostOrGet("x".to_string())]
                })],
                "Failed for dialect {dialect:?}"
            );
        }
    }

    #[test]
    fn test_parse_sql_unsupported_expr_in_sqlpage_arg() {
        let sql = "SELECT sqlpage.link('x', json_build_object('k', c)) FROM (SELECT 1 AS c) t";
        let db_info = create_test_db_info(SupportedDatabase::Postgres);
        let mut parsed = parse_sql(&db_info, &PostgreSqlDialect {}, sql).unwrap();
        let stmt = parsed.next().expect("one statement");
        let ParsedStatement::Error(err) = stmt else {
            panic!("expected ParsedStatement::Error: {stmt:?}");
        };
        let err_msg = format!("{err:#}");
        assert!(
            err_msg.contains("Unsupported sqlpage function argument:"),
            "{err_msg}"
        );
        assert!(err_msg.contains("\"c\" is an sql expression, which cannot be passed as a nested sqlpage function argument."), "{err_msg}");
    }

    #[test]
    fn test_parse_sql_unemulated_function_in_sqlpage_arg() {
        let sql = "SELECT sqlpage.link('x', upper('a')) FROM (SELECT 1) t";
        let db_info = create_test_db_info(SupportedDatabase::Postgres);
        let mut parsed = parse_sql(&db_info, &PostgreSqlDialect {}, sql).unwrap();
        let stmt = parsed.next().expect("one statement");
        let ParsedStatement::Error(err) = stmt else {
            panic!("expected ParsedStatement::Error: {stmt:?}");
        };
        let err_msg = format!("{err:#}");
        assert!(
            err_msg.contains("Unsupported sqlpage function argument:"),
            "{err_msg}"
        );
        assert!(
            err_msg.contains("\"upper\" is not a supported sqlpage function"),
            "{err_msg}"
        );
    }

    #[test]
    fn test_set_variable_to_other_variable() {
        let sql = "set x = $y";
        for &(dialect, dbms) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let db_info = create_test_db_info(dbms);
            match parse_single_statement(&mut parser, &db_info, sql) {
                Some(ParsedStatement::StaticSimpleSet { variable, value }) => {
                    assert_eq!(
                        variable,
                        StmtParam::PostOrGet("x".to_string()),
                        "{dialect:?}"
                    );
                    assert_eq!(
                        value,
                        SimpleSelectValue::Dynamic(StmtParam::PostOrGet("y".to_string()))
                    );
                }
                other => panic!("Failed for dialect {dialect:?}: {other:#?}"),
            }
        }
    }

    #[test]
    fn is_own_placeholder() {
        assert!(ParameterExtractor {
            db_info: create_test_db_info(SupportedDatabase::Postgres),
            parameters: vec![],
            extract_error: None,
        }
        .is_own_placeholder("$1"));

        assert!(ParameterExtractor {
            db_info: create_test_db_info(SupportedDatabase::Postgres),
            parameters: vec![StmtParam::Get("x".to_string())],
            extract_error: None,
        }
        .is_own_placeholder("$2"));

        assert!(!ParameterExtractor {
            db_info: create_test_db_info(SupportedDatabase::Postgres),
            parameters: vec![],
            extract_error: None,
        }
        .is_own_placeholder("$2"));

        assert!(ParameterExtractor {
            db_info: create_test_db_info(SupportedDatabase::Sqlite),
            parameters: vec![],
            extract_error: None,
        }
        .is_own_placeholder("?1"));

        assert!(!ParameterExtractor {
            db_info: create_test_db_info(SupportedDatabase::Sqlite),
            parameters: vec![],
            extract_error: None,
        }
        .is_own_placeholder("$1"));
    }

    #[test]
    fn test_mssql_statement_rewrite() {
        let mut ast = parse_stmt(
            "select '' || $1 from [a schema].[a table]",
            &MsSqlDialect {},
        );
        let db_info = create_test_db_info(SupportedDatabase::Mssql);
        let parameters = ParameterExtractor::extract_parameters(&mut ast, db_info).unwrap();
        assert_eq!(
            ast.to_string(),
            "SELECT CONCAT('', CAST(@p1 AS VARCHAR(MAX))) FROM [a schema].[a table]"
        );
        assert_eq!(parameters, [StmtParam::PostOrGet("1".to_string()),]);
    }

    #[test]
    fn test_static_extract() {
        use SimpleSelectValue::Static;

        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt(
                    "select 'hello' as hello, 42 as answer, null as nothing, 'world' as hello"
                ),
                &[]
            ),
            Some(vec![
                ("hello".into(), Static("hello".into())),
                ("answer".into(), Static(42.into())),
                ("nothing".into(), Static(().into())),
                ("hello".into(), Static("world".into())),
            ])
        );
    }

    #[test]
    fn test_simple_select_with_sqlpage_pseudofunction() {
        let sql = "select 'text' as component, $x as contents, $y as title";
        let dialects: &[&dyn Dialect] = &[
            &PostgreSqlDialect {},
            &SQLiteDialect {},
            &MySqlDialect {},
            &MsSqlDialect {},
        ];
        for &dialect in dialects {
            use std::any::Any;
            use SimpleSelectValue::{Dynamic, Static};
            use StmtParam::PostOrGet;

            let db_info = if dialect.type_id() == (PostgreSqlDialect {}).type_id() {
                create_test_db_info(SupportedDatabase::Postgres)
            } else if dialect.type_id() == (SQLiteDialect {}).type_id() {
                create_test_db_info(SupportedDatabase::Sqlite)
            } else if dialect.type_id() == (MySqlDialect {}).type_id() {
                create_test_db_info(SupportedDatabase::MySql)
            } else if dialect.type_id() == (MsSqlDialect {}).type_id() {
                create_test_db_info(SupportedDatabase::Mssql)
            } else {
                create_test_db_info(SupportedDatabase::Generic)
            };
            let parsed: Vec<ParsedStatement> = parse_sql(&db_info, dialect, sql).unwrap().collect();
            match &parsed[..] {
                [ParsedStatement::StaticSimpleSelect(q)] => assert_eq!(
                    q,
                    &[
                        ("component".into(), Static("text".into())),
                        ("contents".into(), Dynamic(PostOrGet("x".into()))),
                        ("title".into(), Dynamic(PostOrGet("y".into()))),
                    ]
                ),
                other => panic!("failed to extract simple select in {dialect:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn test_simple_select_only_extraction() {
        use SimpleSelectValue::{Dynamic, Static};
        use StmtParam::PostOrGet;
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'text' as component, $1 as contents"),
                &[PostOrGet("cook".into())]
            ),
            Some(vec![
                ("component".into(), Static("text".into())),
                ("contents".into(), Dynamic(PostOrGet("cook".into()))),
            ])
        );
    }

    #[test]
    fn test_extract_set_variable() {
        let sql = "set x = CURRENT_TIMESTAMP";
        for &(dialect, dbms) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let db_info = create_test_db_info(dbms);
            let stmt = parse_single_statement(&mut parser, &db_info, sql);
            if let Some(ParsedStatement::SetVariable {
                variable,
                value: StmtWithParams { query, params, .. },
            }) = stmt
            {
                assert_eq!(
                    variable,
                    StmtParam::PostOrGet("x".to_string()),
                    "{dialect:?}"
                );
                assert_eq!(query, "SELECT CURRENT_TIMESTAMP AS sqlpage_set_expr");
                assert!(params.is_empty());
            } else {
                panic!("Failed for dialect {dialect:?}: {stmt:#?}",);
            }
        }
    }

    #[test]
    fn test_extract_set_variable_static() {
        let sql = "set x = 'hello'";
        for &(dialect, dbms) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let db_info = create_test_db_info(dbms);
            match parse_single_statement(&mut parser, &db_info, sql) {
                Some(ParsedStatement::StaticSimpleSet {
                    variable: StmtParam::PostOrGet(var_name),
                    value: SimpleSelectValue::Static(value),
                }) => {
                    assert_eq!(var_name, "x");
                    assert_eq!(value, "hello");
                }
                other => panic!("Failed for dialect {dialect:?}: {other:#?}"),
            }
        }
    }

    #[test]
    fn test_static_extract_doesnt_match() {
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'hello' as hello, 42 as answer limit 0"),
                &[]
            ),
            None
        );
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'hello' as hello, 42 as answer order by 1"),
                &[]
            ),
            None
        );
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'hello' as hello, 42 as answer offset 1"),
                &[]
            ),
            None
        );
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'hello' as hello, 42 as answer where 1 = 0"),
                &[]
            ),
            None
        );
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select 'hello' as hello, 42 as answer FROM t"),
                &[]
            ),
            None
        );
        assert_eq!(
            extract_static_simple_select(
                &parse_postgres_stmt("select x'CAFEBABE' as hello, 42 as answer"),
                &[]
            ),
            None
        );
    }

    #[test]
    fn test_extract_json_columns() {
        let sql = r"
            WITH json_cte AS (
                SELECT json_build_object('a', x, 'b', y) AS cte_json
                FROM generate_series(1, 3) x
                JOIN generate_series(4, 6) y ON true
            )
            SELECT
                json_object('key', 'value') AS json_col1,
                json_array(1, 2, 3) AS json_col2,
                (SELECT json_build_object('nested', subq.val)
                 FROM (SELECT AVG(x) AS val FROM generate_series(1, 5) x) subq
            ) AS json_col3, -- not supported because of the subquery
            CASE
                WHEN EXISTS (SELECT 1 FROM json_cte WHERE cte_json->>'a' = '2')
                THEN to_json(ARRAY(SELECT cte_json FROM json_cte))
                ELSE json_build_array()
            END AS json_col4, -- not supported because of the CASE
            json_unknown_fn(regular_column) AS non_json_col,
            CAST(json_col1 AS json) AS json_col6
        FROM some_table
        CROSS JOIN json_cte
        WHERE json_typeof(json_col1) = 'object'
    ";

        let stmt = parse_postgres_stmt(sql);
        let json_columns = extract_json_columns(&stmt, SupportedDatabase::Sqlite);

        assert_eq!(
            json_columns,
            vec![
                "json_col1".to_string(),
                "json_col2".to_string(),
                "json_col6".to_string()
            ]
        );
    }

    #[test]
    fn test_set_variable_with_sqlpage_function() {
        let sql = "set x = sqlpage.url_encode(some_db_function())";
        for &(dialect, dbms) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let db_info = create_test_db_info(dbms);
            let stmt = parse_single_statement(&mut parser, &db_info, sql);
            let Some(ParsedStatement::SetVariable {
                variable,
                value:
                    StmtWithParams {
                        query,
                        params,
                        delayed_functions,
                        json_columns,
                        ..
                    },
            }) = stmt
            else {
                panic!("for dialect {dialect:?}: {stmt:#?} instead of SetVariable");
            };
            assert_eq!(
                variable,
                StmtParam::PostOrGet("x".to_string()),
                "{dialect:?}"
            );
            assert_eq!(
                delayed_functions,
                [DelayedFunctionCall {
                    function: SqlPageFunctionName::url_encode,
                    argument_col_names: vec!["_sqlpage_f0_a0".to_string()],
                    target_col_name: "sqlpage_set_expr".to_string()
                }]
            );
            assert_eq!(query, "SELECT some_db_function() AS \"_sqlpage_f0_a0\"");
            assert_eq!(params, []);
            assert_eq!(json_columns, Vec::<String>::new());
        }
    }

    #[test]
    fn test_extract_json_columns_from_literal() {
        let sql = r#"
            SELECT
                'Pro Plan' as title,
                JSON('{"icon":"database","color":"blue","description":"1GB Database"}') as item,
                JSON('{"icon":"headset","color":"green","description":"Priority Support"}') as item
        "#;

        let stmt = parse_stmt(sql, &SQLiteDialect {});
        let json_columns = extract_json_columns(&stmt, SupportedDatabase::Sqlite);

        assert!(json_columns.contains(&"item".to_string()));
        assert!(!json_columns.contains(&"title".to_string()));
    }

    #[test]
    fn test_positional_placeholders() {
        let sql = "select \
        @SQLPAGE_TEMP10 as a1, \
        @SQLPAGE_TEMP9 as a2, \
        @SQLPAGE_TEMP8 as a3, \
        @SQLPAGE_TEMP7 as a4, \
        @SQLPAGE_TEMP6 as a5, \
        @SQLPAGE_TEMP5 as a6, \
        @SQLPAGE_TEMP4 as a7, \
        @SQLPAGE_TEMP3 as a8, \
        @SQLPAGE_TEMP2 as a9, \
        @SQLPAGE_TEMP1 as a10 \
        @SQLPAGE_TEMP10 as a1bis \
        from t";
        let mut stmt = StmtWithParams {
            query: sql.to_string(),
            query_position: SourceSpan {
                start: SourceLocation { line: 1, column: 1 },
                end: SourceLocation { line: 1, column: 1 },
            },
            params: vec![
                StmtParam::PostOrGet("x1".to_string()),
                StmtParam::PostOrGet("x2".to_string()),
                StmtParam::PostOrGet("x3".to_string()),
                StmtParam::PostOrGet("x4".to_string()),
                StmtParam::PostOrGet("x5".to_string()),
                StmtParam::PostOrGet("x6".to_string()),
                StmtParam::PostOrGet("x7".to_string()),
                StmtParam::PostOrGet("x8".to_string()),
                StmtParam::PostOrGet("x9".to_string()),
                StmtParam::PostOrGet("x10".to_string()),
            ],
            delayed_functions: vec![],
            json_columns: vec![],
        };
        transform_to_positional_placeholders(&mut stmt, AnyKind::MySql);
        assert_eq!(
            stmt.query,
            "select \
        ? as a1, \
        ? as a2, \
        ? as a3, \
        ? as a4, \
        ? as a5, \
        ? as a6, \
        ? as a7, \
        ? as a8, \
        ? as a9, \
        ? as a10 \
        ? as a1bis \
        from t"
        );
        assert_eq!(
            stmt.params,
            vec![
                StmtParam::PostOrGet("x10".to_string()),
                StmtParam::PostOrGet("x9".to_string()),
                StmtParam::PostOrGet("x8".to_string()),
                StmtParam::PostOrGet("x7".to_string()),
                StmtParam::PostOrGet("x6".to_string()),
                StmtParam::PostOrGet("x5".to_string()),
                StmtParam::PostOrGet("x4".to_string()),
                StmtParam::PostOrGet("x3".to_string()),
                StmtParam::PostOrGet("x2".to_string()),
                StmtParam::PostOrGet("x1".to_string()),
                StmtParam::PostOrGet("x10".to_string()),
            ]
        );
    }

    #[test]
    fn test_set_variable_error_handling() {
        let sql = "set x = db_function(sqlpage.fetch(other_db_function()))";
        for &(dialect, dbms) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let db_info = create_test_db_info(dbms);
            let stmt = parse_single_statement(&mut parser, &db_info, sql);
            if let Some(ParsedStatement::Error(err)) = stmt {
                assert!(
                    err.to_string()
                        .contains("Unsupported sqlpage function argument:"),
                    "Expected error for invalid function, got: {err}"
                );
            } else {
                panic!("Expected error for invalid function, got: {stmt:#?}");
            }
        }
    }
}
