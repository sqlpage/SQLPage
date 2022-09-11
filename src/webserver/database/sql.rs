use super::PreparedStatement;
use crate::file_cache::AsyncFromStrWithState;
use crate::webserver::database::StmtParam;
use crate::{AppState, Database};
use anyhow::Context;
use async_trait::async_trait;
use sqlparser::ast::{visitor_fn_mut, DataType, DriveMut, Expr, Value, VisitorEvent};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token::{SemiColon, EOF};
use sqlparser::tokenizer::Tokenizer;
use sqlx::any::{AnyKind, AnyTypeInfo};
use sqlx::postgres::types::Oid;
use sqlx::postgres::PgTypeInfo;
use sqlx::{Executor, Statement};

#[derive(Default)]
pub struct ParsedSqlFile {
    pub(super) statements: Vec<anyhow::Result<PreparedStatement>>,
}

impl ParsedSqlFile {
    pub(super) async fn new(db: &Database, sql: &str) -> ParsedSqlFile {
        let dialect = GenericDialect {};
        let tokens = Tokenizer::new(&dialect, sql).tokenize();
        let mut parser = match tokens {
            Ok(tokens) => Parser::new(tokens, &dialect),
            Err(e) => return Self::from_err(e),
        };
        let db_kind = db.connection.any_kind();
        let mut statements = Vec::with_capacity(8);
        while parser.peek_token() != EOF {
            let mut stmt = match parser.parse_statement() {
                Ok(stmt) => stmt,
                Err(err) => {
                    statements.push(Err(anyhow::Error::from(err)));
                    break;
                }
            };
            while parser.consume_token(&SemiColon) {}
            let param_names = extract_parameters(&mut stmt, db_kind);
            let parameters = map_params(param_names);
            let query = stmt.to_string();
            let param_types = get_param_types(&parameters);
            let stmt_res = db
                .connection
                .prepare_with(&query, &param_types)
                .await
                .with_context(|| format!("Preparing SQL statement: '{}'", query));
            statements.push(stmt_res.map(|statement| PreparedStatement {
                statement: statement.to_owned(),
                parameters,
            }));
        }
        statements.shrink_to_fit();
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

#[async_trait(?Send)]
impl AsyncFromStrWithState for ParsedSqlFile {
    async fn from_str_with_state(app_state: &AppState, source: &str) -> Self {
        ParsedSqlFile::new(&app_state.db, source).await
    }
}

fn get_param_types(parameters: &[StmtParam]) -> Vec<AnyTypeInfo> {
    parameters
        .iter()
        .map(|_p| PgTypeInfo::with_oid(Oid(25)).into())
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

fn extract_parameters(sql_ast: &mut sqlparser::ast::Statement, db: AnyKind) -> Vec<String> {
    let mut parameters: Vec<String> = Vec::new();
    sql_ast.drive_mut(&mut visitor_fn_mut(|value: &mut Expr, event| {
        // Only update the nodes AFTER they have been visited
        if let VisitorEvent::Enter = event {
            return;
        }
        if let Expr::Value(Value::Placeholder(param)) = value {
            let new_expr = make_placeholder(db, parameters.len());
            let name = std::mem::take(param);
            parameters.push(name);
            *value = new_expr
        }
    }));
    parameters
}

fn make_placeholder(db: AnyKind, current_count: usize) -> Expr {
    let name = match db {
        // Postgres only supports numbered parameters
        AnyKind::Postgres => format!("${}", current_count + 1),
        _ => '?'.to_string(),
    };
    let data_type = match db {
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

#[test]
fn test_statement_rewrite() {
    let sql = "select $a from t where $x > $a OR $x = 0";
    let mut ast = Parser::parse_sql(&GenericDialect, sql).unwrap();
    let parameters = extract_parameters(&mut ast[0], AnyKind::Postgres);
    assert_eq!(
        ast[0].to_string(),
        "SELECT CAST($1 AS TEXT) FROM t WHERE CAST($2 AS TEXT) > CAST($3 AS TEXT) OR CAST($4 AS TEXT) = 0"
    );
    assert_eq!(parameters, ["$a", "$x", "$a", "$x"]);
}
