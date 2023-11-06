use super::sql_pseudofunctions::{func_call_to_param, StmtParam};
use super::PreparedStatement;
use crate::file_cache::AsyncFromStrWithState;
use crate::utils::add_value_to_map;
use crate::{AppState, Database};
use anyhow::Context;
use async_trait::async_trait;
use sqlparser::ast::{
    BinaryOperator, CharacterLength, DataType, Expr, Function, FunctionArg, FunctionArgExpr, Ident,
    ObjectName, Statement, Value, VisitMut, VisitorMut,
};
use sqlparser::dialect::{Dialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::Token::{SemiColon, EOF};
use sqlparser::tokenizer::Tokenizer;
use sqlx::any::{AnyKind, AnyTypeInfo};
use sqlx::Postgres;
use std::fmt::Write;
use std::ops::ControlFlow;

#[derive(Default)]
pub struct ParsedSqlFile {
    pub(super) statements: Vec<ParsedSQLStatement>,
}

pub(super) enum ParsedSQLStatement {
    Statement(PreparedStatement),
    StaticSimpleSelect(serde_json::Map<String, serde_json::Value>),
    Error(anyhow::Error),
    SetVariable {
        variable: StmtParam,
        value: PreparedStatement,
    },
}

impl ParsedSqlFile {
    pub async fn new(db: &Database, sql: &str) -> ParsedSqlFile {
        let dialect = dialect_for_db(db.connection.any_kind());
        let parsed_statements = match parse_sql(dialect.as_ref(), sql) {
            Ok(parsed) => parsed,
            Err(err) => return Self::from_err(err),
        };
        let mut statements = Vec::with_capacity(8);
        for parsed in parsed_statements {
            statements.push(match parsed {
                ParsedStatement::StaticSimpleSelect(s) => ParsedSQLStatement::StaticSimpleSelect(s),
                ParsedStatement::Error(e) => ParsedSQLStatement::Error(e),
                ParsedStatement::StmtWithParams(stmt_with_params) => {
                    prepare_query_with_params(db, stmt_with_params).await
                }
                ParsedStatement::SetVariable { variable, value } => {
                    match prepare_query_with_params(db, value).await {
                        ParsedSQLStatement::Statement(value) => {
                            ParsedSQLStatement::SetVariable { variable, value }
                        }
                        err => err,
                    }
                }
            });
        }
        statements.shrink_to_fit();
        ParsedSqlFile { statements }
    }

    fn from_err(e: impl Into<anyhow::Error>) -> Self {
        Self {
            statements: vec![ParsedSQLStatement::Error(
                e.into().context("SQLPage could not parse the SQL file"),
            )],
        }
    }
}

async fn prepare_query_with_params(
    db: &Database,
    StmtWithParams { query, params }: StmtWithParams,
) -> ParsedSQLStatement {
    let param_types = get_param_types(&params);
    match db.prepare_with(&query, &param_types).await {
        Ok(statement) => {
            log::debug!("Successfully prepared SQL statement '{query}'");
            ParsedSQLStatement::Statement(PreparedStatement {
                statement,
                parameters: params,
            })
        }
        Err(err) => {
            log::warn!("Failed to prepare {query:?}: {err:#}");
            ParsedSQLStatement::Error(err.context(format!(
                "The database returned an error when preparing this SQL statement: {query}"
            )))
        }
    }
}

#[async_trait(? Send)]
impl AsyncFromStrWithState for ParsedSqlFile {
    async fn from_str_with_state(app_state: &AppState, source: &str) -> anyhow::Result<Self> {
        Ok(ParsedSqlFile::new(&app_state.db, source).await)
    }
}

#[derive(Debug, PartialEq)]
struct StmtWithParams {
    query: String,
    params: Vec<StmtParam>,
}

#[derive(Debug)]
enum ParsedStatement {
    StmtWithParams(StmtWithParams),
    StaticSimpleSelect(serde_json::Map<String, serde_json::Value>),
    SetVariable {
        variable: StmtParam,
        value: StmtWithParams,
    },
    Error(anyhow::Error),
}

fn parse_sql<'a>(
    dialect: &'a dyn Dialect,
    sql: &'a str,
) -> anyhow::Result<impl Iterator<Item = ParsedStatement> + 'a> {
    let tokens = Tokenizer::new(dialect, sql)
        .tokenize_with_location()
        .with_context(|| "SQLPage's SQL parser could not tokenize the sql file")?;
    let mut parser = Parser::new(dialect).with_tokens_with_locations(tokens);
    let db_kind = kind_of_dialect(dialect);
    Ok(std::iter::from_fn(move || {
        parse_single_statement(&mut parser, db_kind)
    }))
}

fn parse_single_statement(parser: &mut Parser<'_>, db_kind: AnyKind) -> Option<ParsedStatement> {
    if parser.peek_token() == EOF {
        return None;
    }
    let mut stmt = match parser.parse_statement() {
        Ok(stmt) => stmt,
        Err(err) => return Some(syntax_error(err, parser)),
    };
    while parser.consume_token(&SemiColon) {}
    if let Some(static_statement) = extract_static_simple_select(&stmt) {
        log::debug!("Optimised a static simple select to avoid a trivial database query: {stmt} optimized to {static_statement:?}");
        return Some(ParsedStatement::StaticSimpleSelect(static_statement));
    }
    let params = ParameterExtractor::extract_parameters(&mut stmt, db_kind);
    if let Some((variable, query)) = extract_set_variable(&mut stmt) {
        return Some(ParsedStatement::SetVariable {
            variable,
            value: StmtWithParams { query, params },
        });
    }
    Some(ParsedStatement::StmtWithParams(StmtWithParams {
        query: stmt.to_string(),
        params,
    }))
}

fn syntax_error(err: ParserError, parser: &mut Parser) -> ParsedStatement {
    let mut err_msg = String::with_capacity(128);
    parser.prev_token(); // go back to the token that caused the error
    for i in 0..32 {
        let next_token = parser.next_token();
        if i == 0 {
            writeln!(
                &mut err_msg,
                "SQLPage found a syntax error on line {}, character {}:",
                next_token.location.line, next_token.location.column
            )
            .unwrap();
        }
        if next_token == EOF {
            break;
        }
        write!(&mut err_msg, "{next_token} ").unwrap();
    }
    ParsedStatement::Error(anyhow::Error::from(err).context(err_msg))
}

fn dialect_for_db(db_kind: AnyKind) -> Box<dyn Dialect> {
    match db_kind {
        AnyKind::Postgres => Box::new(PostgreSqlDialect {}),
        AnyKind::Mssql => Box::new(MsSqlDialect {}),
        AnyKind::MySql => Box::new(MySqlDialect {}),
        AnyKind::Sqlite => Box::new(SQLiteDialect {}),
    }
}

fn kind_of_dialect(dialect: &dyn Dialect) -> AnyKind {
    if dialect.is::<PostgreSqlDialect>() {
        AnyKind::Postgres
    } else if dialect.is::<MsSqlDialect>() {
        AnyKind::Mssql
    } else if dialect.is::<MySqlDialect>() {
        AnyKind::MySql
    } else if dialect.is::<SQLiteDialect>() {
        AnyKind::Sqlite
    } else {
        unreachable!("Unknown dialect")
    }
}

fn get_param_types(parameters: &[StmtParam]) -> Vec<AnyTypeInfo> {
    parameters
        .iter()
        .map(|_p| <str as sqlx::Type<Postgres>>::type_info().into())
        .collect()
}

fn map_param(mut name: String) -> StmtParam {
    if name.is_empty() {
        return StmtParam::GetOrPost(name);
    }
    let prefix = name.remove(0);
    match prefix {
        '$' => StmtParam::GetOrPost(name),
        ':' => StmtParam::Post(name),
        _ => StmtParam::Get(name),
    }
}

fn extract_static_simple_select(
    stmt: &Statement,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let set_expr = match stmt {
        Statement::Query(q)
            if q.limit.is_none()
                && q.fetch.is_none()
                && q.order_by.is_empty()
                && q.with.is_none()
                && q.offset.is_none()
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
                && matches!(&s.group_by, sqlparser::ast::GroupByExpr::Expressions(e) if e.is_empty())
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
    let mut map = serde_json::Map::with_capacity(select_items.len());
    for select_item in select_items {
        let sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } = select_item else {
            return None;
        };
        let value = match expr {
            Expr::Value(Value::Boolean(b)) => serde_json::Value::Bool(*b),
            Expr::Value(Value::Number(n, _)) => serde_json::Value::Number(n.parse().ok()?),
            Expr::Value(Value::SingleQuotedString(s)) => serde_json::Value::String(s.clone()),
            Expr::Value(Value::Null) => serde_json::Value::Null,
            _ => return None,
        };
        let key = alias.value.clone();
        map = add_value_to_map(map, (key, value));
    }
    Some(map)
}

fn extract_set_variable(stmt: &mut Statement) -> Option<(StmtParam, String)> {
    if let Statement::SetVariable {
        variable: ObjectName(name),
        value,
        local: false,
        hivevar: false,
    } = stmt
    {
        if let ([ident], [value]) = (name.as_mut_slice(), value.as_mut_slice()) {
            let variable = if let Some(variable) = extract_ident_param(ident) {
                variable
            } else {
                StmtParam::GetOrPost(std::mem::take(&mut ident.value))
            };
            return Some((variable, format!("SELECT {value}")));
        }
    }
    None
}

struct ParameterExtractor {
    db_kind: AnyKind,
    parameters: Vec<StmtParam>,
}

const PLACEHOLDER_PREFIXES: [(AnyKind, &str); 2] =
    [(AnyKind::Postgres, "$"), (AnyKind::Mssql, "@p")];
const DEFAULT_PLACEHOLDER: &str = "?";

impl ParameterExtractor {
    fn extract_parameters(
        sql_ast: &mut sqlparser::ast::Statement,
        db_kind: AnyKind,
    ) -> Vec<StmtParam> {
        let mut this = Self {
            db_kind,
            parameters: vec![],
        };
        sql_ast.visit(&mut this);
        this.parameters
    }

    fn make_placeholder(&self) -> Expr {
        let name = make_placeholder(self.db_kind, self.parameters.len() + 1);
        // We cast our placeholders to TEXT even though we always bind TEXT data to them anyway
        // because that helps the database engine to prepare the query.
        // For instance in PostgreSQL, the query planner will not be able to use an index on a
        // column if the column is compared to a placeholder of type VARCHAR, but it will be able
        // to use the index if the column is compared to a placeholder of type TEXT.
        let data_type = match self.db_kind {
            // MySQL requires CAST(? AS CHAR) and does not understand CAST(? AS TEXT)
            AnyKind::MySql => DataType::Char(None),
            AnyKind::Mssql => DataType::Varchar(Some(CharacterLength {
                length: 8000,
                unit: None,
            })),
            _ => DataType::Text,
        };
        let value = Expr::Value(Value::Placeholder(name));
        Expr::Cast {
            expr: Box::new(value),
            data_type,
            format: None,
        }
    }

    fn handle_builtin_function(
        &mut self,
        func_name: &str,
        mut arguments: Vec<FunctionArg>,
    ) -> Expr {
        #[allow(clippy::single_match_else)]
        let placeholder = self.make_placeholder();
        let param = func_call_to_param(func_name, &mut arguments);
        self.parameters.push(param);
        placeholder
    }

    fn is_own_placeholder(&self, param: &str) -> bool {
        if let Some((_, prefix)) = PLACEHOLDER_PREFIXES
            .iter()
            .find(|(kind, _prefix)| *kind == self.db_kind)
        {
            if let Some(param) = param.strip_prefix(prefix) {
                if let Ok(index) = param.parse::<usize>() {
                    return index <= self.parameters.len() + 1;
                }
            }
            return false;
        }
        param == DEFAULT_PLACEHOLDER
    }
}

/** This is a helper struct to format a list of arguments for an error message. */
pub(super) struct FormatArguments<'a>(pub &'a [FunctionArg]);
impl std::fmt::Display for FormatArguments<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self.0.iter();
        if let Some(arg) = args.next() {
            write!(f, "{arg}")?;
        }
        for arg in args {
            write!(f, ", {arg}")?;
        }
        Ok(())
    }
}

pub(super) fn extract_single_quoted_string_optional(
    arguments: &mut [FunctionArg],
) -> Option<String> {
    if let Some(Expr::Value(Value::SingleQuotedString(param_value))) =
        arguments.first_mut().and_then(function_arg_expr)
    {
        return Some(std::mem::take(param_value));
    }
    None
}

pub(super) fn extract_single_quoted_string(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> Result<String, String> {
    extract_single_quoted_string_optional(arguments).ok_or_else(|| {
        format!(
            "{func_name}({}) is not a valid call. Expected a literal single quoted string.",
            FormatArguments(arguments)
        )
    })
}

pub(super) fn extract_integer(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> Result<usize, String> {
    match arguments.first_mut().and_then(function_arg_expr) {
        Some(Expr::Value(Value::Number(param_value, _b))) => param_value
            .parse::<usize>()
            .map_err(|e| format!("{func_name}({param_value}) failed: {e}")),
        _ => Err(format!(
            "{func_name}({}) is not a valid call. Expected a literal integer",
            FormatArguments(arguments)
        )),
    }
}

pub(super) fn function_arg_to_stmt_param(arg: &mut FunctionArg) -> Option<StmtParam> {
    match function_arg_expr(arg) {
        Some(Expr::Value(Value::Placeholder(placeholder))) => {
            Some(map_param(std::mem::take(placeholder)))
        }
        Some(Expr::Identifier(ident)) => extract_ident_param(ident),
        Some(Expr::Function(Function {
            name: ObjectName(func_name_parts),
            args,
            ..
        })) if is_sqlpage_func(func_name_parts) => Some(func_call_to_param(
            sqlpage_func_name(func_name_parts),
            args.as_mut_slice(),
        )),
        Some(Expr::Value(Value::SingleQuotedString(param_value))) => {
            Some(StmtParam::Literal(std::mem::take(param_value)))
        }
        _ => None,
    }
}

pub(super) fn stmt_param_error_invalid_arguments(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> StmtParam {
    StmtParam::Error(format!(
        "{func_name}({}) is not a valid call. \
        Only variables (such as $my_variable) \
        and sqlpage function calls (such as sqlpage.header('my_header')) \
        are supported as arguments to sqlpage functions.",
        FormatArguments(arguments)
    ))
}

pub(super) fn extract_optional_variable_argument(
    arguments: &mut [FunctionArg],
) -> Option<StmtParam> {
    arguments.first_mut().and_then(function_arg_to_stmt_param)
}

pub(super) fn extract_variable_argument(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> StmtParam {
    extract_optional_variable_argument(arguments)
        .unwrap_or_else(|| stmt_param_error_invalid_arguments(func_name, arguments))
}

fn function_arg_expr(arg: &mut FunctionArg) -> Option<&mut Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
        _ => None,
    }
}

#[inline]
#[must_use]
pub fn make_placeholder(db_kind: AnyKind, arg_number: usize) -> String {
    if let Some((_, prefix)) = PLACEHOLDER_PREFIXES
        .iter()
        .find(|(kind, _)| *kind == db_kind)
    {
        return format!("{prefix}{arg_number}");
    }
    DEFAULT_PLACEHOLDER.to_string()
}

fn extract_ident_param(Ident { value, .. }: &mut Ident) -> Option<StmtParam> {
    if value.starts_with('$') || value.starts_with(':') {
        let name = std::mem::take(value);
        Some(map_param(name))
    } else {
        None
    }
}

impl VisitorMut for ParameterExtractor {
    type Break = ();
    fn pre_visit_expr(&mut self, value: &mut Expr) -> ControlFlow<Self::Break> {
        match value {
            Expr::Identifier(ident) => {
                if let Some(param) = extract_ident_param(ident) {
                    *value = self.make_placeholder();
                    self.parameters.push(param);
                }
            }
            Expr::Value(Value::Placeholder(param)) if !self.is_own_placeholder(param) =>
            // this check is to avoid recursively replacing placeholders in the form of '?', or '$1', '$2', which we emit ourselves
            {
                let new_expr = self.make_placeholder();
                let name = std::mem::take(param);
                self.parameters.push(map_param(name));
                *value = new_expr;
            }
            Expr::Function(Function {
                name: ObjectName(func_name_parts),
                args,
                special: false,
                distinct: false,
                over: None,
                ..
            }) if is_sqlpage_func(func_name_parts) => {
                let func_name = sqlpage_func_name(func_name_parts);
                log::debug!("Handling builtin function: {func_name}");
                let arguments = std::mem::take(args);
                *value = self.handle_builtin_function(func_name, arguments);
            }
            // Replace 'str1' || 'str2' with CONCAT('str1', 'str2') for MSSQL
            Expr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } if self.db_kind == AnyKind::Mssql => {
                let left = std::mem::replace(left.as_mut(), Expr::Value(Value::Null));
                let right = std::mem::replace(right.as_mut(), Expr::Value(Value::Null));
                *value = Expr::Function(Function {
                    name: ObjectName(vec![Ident::new("CONCAT")]),
                    args: vec![
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(left)),
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(right)),
                    ],
                    over: None,
                    filter: None,
                    null_treatment: None,
                    distinct: false,
                    special: false,
                    order_by: vec![],
                });
            }
            _ => (),
        }
        ControlFlow::<()>::Continue(())
    }
}

fn is_sqlpage_func(func_name_parts: &[Ident]) -> bool {
    if let [Ident { value, .. }, Ident { .. }] = func_name_parts {
        value == "sqlpage"
    } else {
        false
    }
}

fn sqlpage_func_name(func_name_parts: &[Ident]) -> &str {
    if let [Ident { .. }, Ident { value, .. }] = func_name_parts {
        value
    } else {
        debug_assert!(
            false,
            "sqlpage function name should have been checked by is_sqlpage_func"
        );
        ""
    }
}

#[cfg(test)]
mod test {
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
        let parameters = ParameterExtractor::extract_parameters(&mut ast, AnyKind::Postgres);
        assert_eq!(
        ast.to_string(),
        "SELECT CAST($1 AS TEXT) FROM t WHERE CAST($2 AS TEXT) > CAST($3 AS TEXT) OR CAST($4 AS TEXT) = CAST($5 AS TEXT)"
    );
        assert_eq!(
            parameters,
            [
                StmtParam::GetOrPost("a".to_string()),
                StmtParam::GetOrPost("x".to_string()),
                StmtParam::GetOrPost("a".to_string()),
                StmtParam::GetOrPost("x".to_string()),
                StmtParam::Cookie("cookoo".to_string()),
            ]
        );
    }

    #[test]
    fn test_statement_rewrite_sqlite() {
        let mut ast = parse_stmt("select $x, :y from t", &SQLiteDialect {});
        let parameters = ParameterExtractor::extract_parameters(&mut ast, AnyKind::Sqlite);
        assert_eq!(
            ast.to_string(),
            "SELECT CAST(? AS TEXT), CAST(? AS TEXT) FROM t"
        );
        assert_eq!(
            parameters,
            [
                StmtParam::GetOrPost("x".to_string()),
                StmtParam::Post("y".to_string()),
            ]
        );
    }

    const ALL_DIALECTS: &[(&dyn Dialect, AnyKind)] = &[
        (&PostgreSqlDialect {}, AnyKind::Postgres),
        (&MsSqlDialect {}, AnyKind::Mssql),
        (&MySqlDialect {}, AnyKind::MySql),
        (&SQLiteDialect {}, AnyKind::Sqlite),
    ];

    #[test]
    fn test_sqlpage_function_with_argument() {
        for &(dialect, kind) in ALL_DIALECTS {
            let mut ast = parse_stmt("select sqlpage.hash_password($x)", dialect);
            let parameters = ParameterExtractor::extract_parameters(&mut ast, kind);
            assert_eq!(
                parameters,
                [StmtParam::HashPassword(Box::new(StmtParam::GetOrPost(
                    "x".to_string()
                )))],
                "Failed for dialect {dialect:?}"
            );
        }
    }

    #[test]
    fn test_set_variable() {
        let sql = "set x = $y";
        for &(dialect, db_kind) in ALL_DIALECTS {
            let mut parser = Parser::new(dialect).try_with_sql(sql).unwrap();
            let stmt = parse_single_statement(&mut parser, db_kind);
            if let Some(ParsedStatement::SetVariable {
                variable,
                value: StmtWithParams { query, params },
            }) = stmt
            {
                assert_eq!(
                    variable,
                    StmtParam::GetOrPost("x".to_string()),
                    "{dialect:?}"
                );
                assert!(query.starts_with("SELECT "));
                assert_eq!(params, [StmtParam::GetOrPost("y".to_string())]);
            } else {
                panic!("Failed for dialect {dialect:?}: {stmt:#?}",);
            }
        }
    }

    #[test]
    fn is_own_placeholder() {
        assert!(ParameterExtractor {
            db_kind: AnyKind::Postgres,
            parameters: vec![]
        }
        .is_own_placeholder("$1"));

        assert!(ParameterExtractor {
            db_kind: AnyKind::Postgres,
            parameters: vec![StmtParam::Get('x'.to_string())]
        }
        .is_own_placeholder("$2"));

        assert!(!ParameterExtractor {
            db_kind: AnyKind::Postgres,
            parameters: vec![]
        }
        .is_own_placeholder("$2"));

        assert!(ParameterExtractor {
            db_kind: AnyKind::Sqlite,
            parameters: vec![]
        }
        .is_own_placeholder("?"));

        assert!(!ParameterExtractor {
            db_kind: AnyKind::Sqlite,
            parameters: vec![]
        }
        .is_own_placeholder("$1"));
    }

    #[test]
    fn test_mssql_statement_rewrite() {
        let mut ast = parse_stmt(
            "select '' || $1 from [a schema].[a table]",
            &MsSqlDialect {},
        );
        let parameters = ParameterExtractor::extract_parameters(&mut ast, AnyKind::Mssql);
        assert_eq!(
            ast.to_string(),
            "SELECT CONCAT('', CAST(@p1 AS VARCHAR(8000))) FROM [a schema].[a table]"
        );
        assert_eq!(parameters, [StmtParam::GetOrPost("1".to_string()),]);
    }

    #[test]
    fn test_static_extract() {
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer, null as nothing, 'world' as hello"
            )),
            Some(
                serde_json::json!({
                    "hello": ["hello", "world"],
                    "answer": 42,
                    "nothing": (),
                })
                .as_object()
                .unwrap()
                .clone()
            )
        );
    }

    #[test]
    fn test_static_extract_doesnt_match() {
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer limit 0"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer order by 1"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer offset 1"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer where 1 = 0"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select 'hello' as hello, 42 as answer FROM t"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_postgres_stmt(
                "select x'CAFEBABE' as hello, 42 as answer"
            )),
            None
        );
    }
}
