//! Converts SQLPage-owned values used by database SQL into prepared bindings.

use std::ops::ControlFlow;

use anyhow::anyhow;
use sqlparser::ast::{
    BinaryOperator, CastKind, CharacterLength, DataType, Expr as SqlExpr, Function, FunctionArg,
    FunctionArgumentList, FunctionArguments, Ident, ObjectName, ObjectNamePart,
    Statement as SqlStatement, Value, ValueWithSpan, VisitMut, VisitorMut,
};
use sqlparser::tokenizer::Span;

use super::sqlpage_expression::{
    StandaloneContext, build_sqlpage_expr, recognize_sqlpage_function, variable_from_expr,
};
use crate::webserver::database::sqlpage_expr::{SqlPageExpr, StandaloneExpr};
use crate::webserver::database::{DbInfo, SupportedDatabase};

/// Lowers SQLPage-owned values in database AST fragments into bindings.
pub(super) struct DatabaseLowerer {
    database: DbInfo,
    bindings: Vec<StandaloneExpr>,
    error: Option<anyhow::Error>,
}

impl DatabaseLowerer {
    pub(super) fn new(database: DbInfo) -> Self {
        Self {
            database,
            bindings: Vec::new(),
            error: None,
        }
    }

    pub(super) fn lower_ast(&mut self, value: &mut impl VisitMut) -> anyhow::Result<()> {
        let _ = value.visit(self);
        self.error.take().map_or(Ok(()), Err)
    }

    fn add_binding(&mut self, value: StandaloneExpr) -> SqlExpr {
        let prefix = self.database.numbered_placeholder_prefix();
        // Variable identity includes its lookup source. Calls may have side effects,
        // and positional placeholders need a separate argument for each occurrence.
        let existing = if prefix.is_some() && matches!(value, SqlPageExpr::Variable(_)) {
            self.bindings.iter().position(|binding| binding == &value)
        } else {
            None
        };
        let sequence = existing.unwrap_or_else(|| {
            let sequence = self.bindings.len();
            self.bindings.push(value);
            sequence
        });
        // Positional drivers use temporary $n markers until SQL rendering order is known.
        let placeholder = format!("{}{}", prefix.unwrap_or("$"), sequence + 1);
        cast_placeholder(placeholder, self.database)
    }

    pub(super) fn finish_bindings(
        self,
        statement: &mut SqlStatement,
    ) -> anyhow::Result<Box<[StandaloneExpr]>> {
        if self.database.numbered_placeholder_prefix().is_some() {
            return Ok(self.bindings.into_boxed_slice());
        }
        let mut finalizer = PositionalBindingFinalizer {
            binding_count: self.bindings.len(),
            order: Vec::with_capacity(self.bindings.len()),
            error: None,
        };
        let _ = statement.visit(&mut finalizer);
        finalizer.error.map_or(Ok(()), Err)?;
        let mut bindings = self.bindings.into_iter().map(Some).collect::<Vec<_>>();
        finalizer
            .order
            .into_iter()
            .map(|index| {
                bindings[index].take().ok_or_else(|| {
                    anyhow!("Generated binding placeholder ${} was repeated", index + 1)
                })
            })
            .collect()
    }
}

impl VisitorMut for DatabaseLowerer {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &mut SqlExpr) -> ControlFlow<Self::Break> {
        if self.error.is_some() {
            return ControlFlow::Break(());
        }
        let replacement = match expression {
            SqlExpr::Value(ValueWithSpan {
                value: Value::Placeholder(_),
                span,
            }) if *span == Span::empty() => None,
            SqlExpr::Value(ValueWithSpan {
                value: Value::Placeholder(_),
                ..
            })
            | SqlExpr::Identifier(_) => variable_from_expr(expression)
                .map(|variable| self.add_binding(SqlPageExpr::Variable(variable))),
            SqlExpr::Function(function) => match recognize_sqlpage_function(function) {
                Ok(Some(_)) => {
                    let owned = std::mem::replace(expression, SqlExpr::value(Value::Null));
                    match build_sqlpage_expr(
                        self.database.database_type(),
                        &mut StandaloneContext,
                        owned,
                    ) {
                        Ok(value) => Some(self.add_binding(value)),
                        Err(error) => {
                            self.error = Some(error.context(
                                "A SQLPage function used by the database could not be evaluated before the query",
                            ));
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(error) => {
                    self.error = Some(error);
                    None
                }
            },
            SqlExpr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } if self.database.database_type() == SupportedDatabase::Mssql => {
                let left = std::mem::replace(left.as_mut(), SqlExpr::value(Value::Null));
                let right = std::mem::replace(right.as_mut(), SqlExpr::value(Value::Null));
                Some(make_concat(left, right))
            }
            SqlExpr::Cast {
                kind: kind @ CastKind::DoubleColon,
                ..
            } if !matches!(
                self.database.database_type(),
                SupportedDatabase::Postgres
                    | SupportedDatabase::Duckdb
                    | SupportedDatabase::Snowflake
                    | SupportedDatabase::Generic
            ) =>
            {
                *kind = CastKind::Cast;
                None
            }
            _ => None,
        };
        if let Some(replacement) = replacement {
            *expression = replacement;
        }
        ControlFlow::Continue(())
    }
}

struct PositionalBindingFinalizer {
    binding_count: usize,
    order: Vec<usize>,
    error: Option<anyhow::Error>,
}

impl VisitorMut for PositionalBindingFinalizer {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &mut SqlExpr) -> ControlFlow<Self::Break> {
        let SqlExpr::Value(ValueWithSpan {
            value: Value::Placeholder(name),
            span,
        }) = expression
        else {
            return ControlFlow::Continue(());
        };
        if *span != Span::empty() {
            return ControlFlow::Continue(());
        }
        let Some(index) = name
            .strip_prefix('$')
            .and_then(|number| number.parse::<usize>().ok())
            .and_then(|number| number.checked_sub(1))
        else {
            return ControlFlow::Continue(());
        };
        if index >= self.binding_count {
            self.error = Some(anyhow!("Invalid generated binding placeholder {name}"));
            return ControlFlow::Break(());
        }
        self.order.push(index);
        "?".clone_into(name);
        ControlFlow::Continue(())
    }
}

fn make_concat(left: SqlExpr, right: SqlExpr) -> SqlExpr {
    SqlExpr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("CONCAT"))]),
        args: FunctionArguments::List(FunctionArgumentList {
            args: vec![
                FunctionArg::Unnamed(left.into()),
                FunctionArg::Unnamed(right.into()),
            ],
            duplicate_treatment: None,
            clauses: Vec::new(),
        }),
        parameters: FunctionArguments::None,
        over: None,
        filter: None,
        null_treatment: None,
        within_group: Vec::new(),
        uses_odbc_syntax: false,
    })
}

/// Casts generated placeholders where the backend cannot infer string typing.
fn cast_placeholder(placeholder: String, database: DbInfo) -> SqlExpr {
    let data_type = match database {
        DbInfo::Native(SupportedDatabase::Sqlite)
        | DbInfo::Odbc(SupportedDatabase::Sqlite | SupportedDatabase::Postgres) => DataType::Text,
        DbInfo::Odbc(SupportedDatabase::Oracle) => {
            DataType::Varchar(Some(CharacterLength::IntegerLength {
                length: 4000,
                unit: None,
            }))
        }
        DbInfo::Odbc(SupportedDatabase::Snowflake | SupportedDatabase::Generic) => {
            DataType::Varchar(None)
        }
        _ => return SqlExpr::value(Value::Placeholder(placeholder)),
    };
    SqlExpr::Cast {
        expr: Box::new(SqlExpr::value(Value::Placeholder(placeholder))),
        data_type,
        format: None,
        kind: CastKind::Cast,
    }
}
