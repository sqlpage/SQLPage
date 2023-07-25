use super::sql_pseudofunctions::{func_call_to_param, StmtParam};
use super::PreparedStatement;
use crate::file_cache::AsyncFromStrWithState;
use crate::{AppState, Database};
use async_trait::async_trait;
use sqlparser::ast::{
    DataType, Expr, Function, FunctionArg, FunctionArgExpr, Ident, ObjectName, Statement, Value,
    VisitMut, VisitorMut,
};
use sqlparser::dialect::GenericDialect;
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
}

impl ParsedSqlFile {
    pub async fn new(db: &Database, sql: &str) -> ParsedSqlFile {
        let dialect = GenericDialect {};
        let tokens = Tokenizer::new(&dialect, sql).tokenize_with_location();
        let mut parser = match tokens {
            Ok(tokens) => Parser::new(&dialect).with_tokens_with_locations(tokens),
            Err(e) => return Self::from_err(e),
        };
        let mut statements = Vec::with_capacity(8);
        while parser.peek_token() != EOF {
            let mut stmt = match parser.parse_statement() {
                Ok(stmt) => stmt,
                Err(err) => {
                    return Self::finish_with_error(err, parser, statements);
                }
            };
            while parser.consume_token(&SemiColon) {}
            if let Some(static_statement) = extract_static_simple_select(&stmt) {
                log::debug!("Optimised a static simple select to avoid a trivial database query: {stmt} optimized to {static_statement:?}");
                statements.push(ParsedSQLStatement::StaticSimpleSelect(static_statement));
                continue;
            }
            let db_kind = db.connection.any_kind();
            let parameters = ParameterExtractor::extract_parameters(&mut stmt, db_kind);
            let query = stmt.to_string();
            let param_types = get_param_types(&parameters);
            let stmt_res = db.prepare_with(&query, &param_types).await;
            let statement_result = match stmt_res {
                Ok(statement) => {
                    log::debug!("Successfully prepared SQL statement '{query}'");
                    ParsedSQLStatement::Statement(PreparedStatement {
                        statement,
                        parameters,
                    })
                }
                Err(err) => {
                    log::warn!("{err:#}");
                    ParsedSQLStatement::Error(err)
                }
            };
            statements.push(statement_result);
        }
        statements.shrink_to_fit();
        ParsedSqlFile { statements }
    }

    fn finish_with_error(
        err: ParserError,
        mut parser: Parser,
        mut statements: Vec<ParsedSQLStatement>,
    ) -> ParsedSqlFile {
        let mut err_msg = "SQL syntax error before: ".to_string();
        for _ in 0..32 {
            let next_token = parser.next_token();
            if next_token == EOF {
                break;
            }
            _ = write!(&mut err_msg, "{next_token} ");
        }
        let error = anyhow::Error::from(err).context(err_msg);
        statements.push(ParsedSQLStatement::Error(error));
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

#[async_trait(? Send)]
impl AsyncFromStrWithState for ParsedSqlFile {
    async fn from_str_with_state(app_state: &AppState, source: &str) -> anyhow::Result<Self> {
        Ok(ParsedSqlFile::new(&app_state.db, source).await)
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
                && s.group_by.is_empty()
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
        let sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } = select_item else { return None };
        let value = match expr {
            Expr::Value(Value::Boolean(b)) => serde_json::Value::Bool(*b),
            Expr::Value(Value::Number(n, _)) => serde_json::Value::Number(n.parse().ok()?),
            Expr::Value(Value::SingleQuotedString(s)) => serde_json::Value::String(s.clone()),
            Expr::Value(Value::Null) => serde_json::Value::Null,
            _ => return None,
        };
        map.insert(alias.value.clone(), value);
    }
    Some(map)
}

struct ParameterExtractor {
    db_kind: AnyKind,
    parameters: Vec<StmtParam>,
}

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
        let data_type = match self.db_kind {
            // MySQL requires CAST(? AS CHAR) and does not understand CAST(? AS TEXT)
            AnyKind::MySql => DataType::Char(None),
            _ => DataType::Text,
        };
        let value = Expr::Value(Value::Placeholder(name));
        Expr::Cast {
            expr: Box::new(value),
            data_type,
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

pub(super) fn extract_single_quoted_string(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> Result<String, String> {
    match arguments.first_mut().and_then(function_arg_expr) {
        Some(Expr::Value(Value::SingleQuotedString(param_value))) => {
            Ok(std::mem::take(param_value))
        }
        _ => Err(format!(
            "{func_name}({}) is not a valid call. Expected a literal single quoted string.",
            FormatArguments(arguments)
        )),
    }
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

pub(super) fn extract_variable_argument(
    func_name: &'static str,
    arguments: &mut [FunctionArg],
) -> Result<StmtParam, String> {
    match arguments.first_mut().and_then(function_arg_expr) {
        Some(Expr::Value(Value::Placeholder(placeholder))) => {
            Ok(map_param(std::mem::take(placeholder)))
        }
        Some(Expr::Function(Function {
            name: ObjectName(func_name_parts),
            args,
            ..
        })) if is_sqlpage_func(func_name_parts) => Ok(func_call_to_param(
            sqlpage_func_name(func_name_parts),
            args.as_mut_slice(),
        )),
        _ => Err(format!(
            "{func_name}({}) is not a valid call. Expected either a placeholder or a sqlpage function call as argument.",
            FormatArguments(arguments)
        )),
    }
}

fn function_arg_expr(arg: &mut FunctionArg) -> Option<&mut Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
        _ => None,
    }
}

#[inline]
pub fn make_placeholder(db_kind: AnyKind, arg_number: usize) -> String {
    match db_kind {
        // Postgres only supports numbered parameters
        AnyKind::Postgres => format!("${arg_number}"),
        _ => '?'.to_string(),
    }
}

impl VisitorMut for ParameterExtractor {
    type Break = ();
    fn pre_visit_expr(&mut self, value: &mut Expr) -> ControlFlow<Self::Break> {
        match value {
            Expr::Value(Value::Placeholder(param))
                if param.chars().nth(1).is_some_and(char::is_alphabetic) =>
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

    fn parse_stmt(sql: &str) -> Statement {
        let mut ast = Parser::parse_sql(&GenericDialect, sql).unwrap();
        assert_eq!(ast.len(), 1);
        ast.pop().unwrap()
    }

    #[test]
    fn test_statement_rewrite() {
        let mut ast = parse_stmt("select $a from t where $x > $a OR $x = sqlpage.cookie('cookoo')");
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
    fn test_static_extract() {
        assert_eq!(
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer, null as nothing"
            )),
            Some(
                serde_json::json!({
                    "hello": "hello",
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
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer limit 0"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer order by 1"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer offset 1"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer where 1 = 0"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_stmt(
                "select 'hello' as hello, 42 as answer FROM t"
            )),
            None
        );
        assert_eq!(
            extract_static_simple_select(&parse_stmt("select x'CAFEBABE' as hello, 42 as answer")),
            None
        );
    }
}
