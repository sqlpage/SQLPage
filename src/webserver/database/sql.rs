use super::PreparedStatement;
use crate::file_cache::AsyncFromStrWithState;
use crate::webserver::database::StmtParam;
use crate::{AppState, Database};
use async_trait::async_trait;
use sqlparser::ast::{DataType, Expr, Value, VisitMut, VisitorMut};
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
    pub(super) statements: Vec<anyhow::Result<PreparedStatement>>,
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
            let db_kind = db.connection.any_kind();
            let param_names = ParameterExtractor::extract_parameters(&mut stmt, db_kind);
            let parameters = map_params(param_names);
            let query = stmt.to_string();
            let param_types = get_param_types(&parameters);
            let stmt_res = db.prepare_with(&query, &param_types).await;
            match &stmt_res {
                Ok(_) => log::debug!("Successfully prepared SQL statement '{query}'"),
                Err(err) => log::warn!("{err:#}"),
            }
            let statement_result = stmt_res.map(|statement| PreparedStatement {
                statement,
                parameters,
            });
            statements.push(statement_result);
        }
        statements.shrink_to_fit();
        ParsedSqlFile { statements }
    }

    fn finish_with_error(
        err: ParserError,
        mut parser: Parser,
        mut statements: Vec<anyhow::Result<PreparedStatement>>,
    ) -> ParsedSqlFile {
        let mut err_msg = "SQL syntax error before: ".to_string();
        for _ in 0..32 {
            let next_token = parser.next_token();
            if next_token == EOF {
                break;
            }
            let _ = write!(&mut err_msg, "{next_token} ");
        }
        let error = anyhow::Error::from(err).context(err_msg);
        statements.push(Err(error));
        ParsedSqlFile { statements }
    }

    fn from_err(e: impl Into<anyhow::Error>) -> Self {
        Self {
            statements: vec![Err(e
                .into()
                .context("SQLPage could not parse the SQL file"))],
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

fn map_params(names: Vec<String>) -> Vec<StmtParam> {
    names
        .into_iter()
        .map(|name| {
            let (prefix, name) = name.split_at(1);
            let name = name.to_owned();
            match prefix {
                "$" => StmtParam::GetOrPost(name),
                ":" => StmtParam::Post(name),
                _ => StmtParam::Get(name),
            }
        })
        .collect()
}

struct ParameterExtractor {
    db_kind: AnyKind,
    parameters: Vec<String>,
}

impl ParameterExtractor {
    fn extract_parameters(
        sql_ast: &mut sqlparser::ast::Statement,
        db_kind: AnyKind,
    ) -> Vec<String> {
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
    fn post_visit_expr(&mut self, value: &mut Expr) -> ControlFlow<Self::Break> {
        if let Expr::Value(Value::Placeholder(param)) = value {
            let new_expr = self.make_placeholder();
            let name = std::mem::take(param);
            self.parameters.push(name);
            *value = new_expr;
        }
        ControlFlow::<()>::Continue(())
    }
}

#[test]
fn test_statement_rewrite() {
    let sql = "select $a from t where $x > $a OR $x = 0";
    let mut ast = Parser::parse_sql(&GenericDialect, sql).unwrap();
    let parameters = ParameterExtractor::extract_parameters(&mut ast[0], AnyKind::Postgres);
    assert_eq!(
        ast[0].to_string(),
        "SELECT CAST($1 AS TEXT) FROM t WHERE CAST($2 AS TEXT) > CAST($3 AS TEXT) OR CAST($4 AS TEXT) = 0"
    );
    assert_eq!(parameters, ["$a", "$x", "$a", "$x"]);
}
