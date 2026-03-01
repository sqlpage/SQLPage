use super::super::{DbInfo, SupportedDatabase};
use super::{is_sqlpage_func, sqlpage_func_name};
use crate::webserver::database::sqlpage_functions::func_call_to_param;
use crate::webserver::database::syntax_tree::StmtParam;
use sqlparser::ast::{
    BinaryOperator, CastKind, CharacterLength, DataType, Expr, Function, FunctionArg,
    FunctionArgExpr, FunctionArgumentList, FunctionArguments, Ident, ObjectName, ObjectNamePart,
    Spanned, Statement, Value, ValueWithSpan, Visit, VisitMut, Visitor, VisitorMut,
};
use sqlx::any::AnyKind;
use std::ops::ControlFlow;

pub(super) struct ParameterExtractor {
    pub(super) db_info: DbInfo,
    pub(super) parameters: Vec<StmtParam>,
    pub(super) extract_error: Option<anyhow::Error>,
}

#[derive(Debug)]
pub(crate) enum DbPlaceHolder {
    PrefixedNumber { prefix: &'static str },
    Positional { placeholder: &'static str },
}

pub(crate) const DB_PLACEHOLDERS: [(AnyKind, DbPlaceHolder); 5] = [
    (
        AnyKind::Sqlite,
        DbPlaceHolder::PrefixedNumber { prefix: "?" },
    ),
    (
        AnyKind::Postgres,
        DbPlaceHolder::PrefixedNumber { prefix: "$" },
    ),
    (
        AnyKind::MySql,
        DbPlaceHolder::Positional { placeholder: "?" },
    ),
    (
        AnyKind::Mssql,
        DbPlaceHolder::PrefixedNumber { prefix: "@p" },
    ),
    (
        AnyKind::Odbc,
        DbPlaceHolder::Positional { placeholder: "?" },
    ),
];

/// For positional parameters, we use a temporary placeholder during parameter extraction,
/// And then replace it with the actual placeholder during statement rewriting.
pub(crate) const TEMP_PLACEHOLDER_PREFIX: &str = "@SQLPAGE_TEMP";

fn get_placeholder_prefix(kind: AnyKind) -> &'static str {
    if let Some((_, DbPlaceHolder::PrefixedNumber { prefix })) = DB_PLACEHOLDERS
        .iter()
        .find(|(placeholder_kind, _prefix)| *placeholder_kind == kind)
    {
        prefix
    } else {
        TEMP_PLACEHOLDER_PREFIX
    }
}

impl ParameterExtractor {
    pub(super) fn extract_parameters(
        sql_ast: &mut Statement,
        db_info: DbInfo,
    ) -> anyhow::Result<Vec<StmtParam>> {
        let mut this = Self {
            db_info,
            parameters: vec![],
            extract_error: None,
        };
        let _ = sql_ast.visit(&mut this);
        if let Some(e) = this.extract_error {
            return Err(e);
        }
        Ok(this.parameters)
    }

    fn replace_with_placeholder(&mut self, value: &mut Expr, param: StmtParam) {
        let placeholder =
            if let Some(existing_idx) = self.parameters.iter().position(|p| *p == param) {
                // Parameter already exists, use its index
                self.make_placeholder_for_index(existing_idx + 1)
            } else {
                // New parameter, add it to the list
                let placeholder = self.make_placeholder();
                log::trace!("Replacing {param} with {placeholder}");
                self.parameters.push(param);
                placeholder
            };
        *value = placeholder;
    }

    fn make_placeholder_for_index(&self, index: usize) -> Expr {
        let name = make_tmp_placeholder(self.db_info.kind, index);
        let data_type = match self.db_info.database_type {
            SupportedDatabase::MySql => DataType::Char(None),
            SupportedDatabase::Mssql => DataType::Varchar(Some(CharacterLength::Max)),
            SupportedDatabase::Postgres | SupportedDatabase::Sqlite => DataType::Text,
            SupportedDatabase::Oracle => DataType::Varchar(Some(CharacterLength::IntegerLength {
                length: 4000,
                unit: None,
            })),
            _ => DataType::Varchar(None),
        };
        let value = Expr::value(Value::Placeholder(name));
        Expr::Cast {
            expr: Box::new(value),
            data_type,
            format: None,
            kind: CastKind::Cast,
        }
    }

    fn make_placeholder(&self) -> Expr {
        self.make_placeholder_for_index(self.parameters.len() + 1)
    }

    pub(super) fn is_own_placeholder(&self, param: &str) -> bool {
        let prefix = get_placeholder_prefix(self.db_info.kind);
        if let Some(param) = param.strip_prefix(prefix) {
            if let Ok(index) = param.parse::<usize>() {
                return index <= self.parameters.len() + 1;
            }
        }
        false
    }
}

struct InvalidFunctionFinder;
impl Visitor for InvalidFunctionFinder {
    type Break = (String, Vec<FunctionArg>);
    fn pre_visit_expr(&mut self, value: &Expr) -> ControlFlow<Self::Break> {
        match value {
            Expr::Function(Function {
                name: ObjectName(func_name_parts),
                args:
                    FunctionArguments::List(FunctionArgumentList {
                        args,
                        duplicate_treatment: None,
                        ..
                    }),
                ..
            }) if is_sqlpage_func(func_name_parts) => {
                let func_name = sqlpage_func_name(func_name_parts);
                let arguments = args.clone();
                return ControlFlow::Break((func_name.to_string(), arguments));
            }
            _ => (),
        }
        ControlFlow::Continue(())
    }
}

pub(super) fn validate_function_calls(stmt: &Statement) -> anyhow::Result<()> {
    let mut finder = InvalidFunctionFinder;
    if let ControlFlow::Break((func_name, mut args)) = stmt.visit(&mut finder) {
        let ctx = ParamExtractContext {
            parent_func: Some(func_name.clone()),
        };
        function_args_to_stmt_params(&mut args, &ctx)?;

        let args_str = FormatArguments(&args);
        let error_msg = format!(
            "Invalid SQLPage function call: sqlpage.{func_name}({args_str})\n\n\
            Arbitrary SQL expressions as function arguments are not supported.\n\n\
            SQLPage functions can either:\n\
            1. Run BEFORE the query (to provide input values)\n\
            2. Run AFTER the query (to process the results)\n\
            But they can't run DURING the query - the database doesn't know how to call them!\n\n\
            To fix this, you can either:\n\
            1. Store the function argument in a variable first:\n\
               SET {func_name}_arg = ...;\n\
               SET {func_name}_result = sqlpage.{func_name}(${func_name}_arg);\n\
               SELECT * FROM example WHERE xxx = ${func_name}_result;\n\n\
            2. Or move the function to the top level to process results:\n\
               SELECT sqlpage.{func_name}(...) FROM example;"
        );
        Err(anyhow::anyhow!(error_msg))
    } else {
        Ok(())
    }
}

/** This is a helper struct to format a list of arguments for an error message. */
struct FormatArguments<'a>(&'a [FunctionArg]);
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

#[derive(Clone, Default)]
pub(crate) struct ParamExtractContext {
    pub parent_func: Option<String>,
}

impl ParamExtractContext {
    fn with_parent(parent: &str) -> Self {
        Self {
            parent_func: Some(parent.to_string()),
        }
    }

    fn build_error(&self, e: &ExprToParamError, arguments: &[FunctionArg]) -> SqlPageFunctionError {
        let line = e.line.unwrap_or(0);
        let func_name = self.parent_func.as_deref().unwrap_or("unknown").to_string();
        let arguments_str = FormatArguments(arguments).to_string();

        let reason = match &e.kind {
            ExprToParamErrorKind::UnsupportedExpr { summary } => {
                format!("\"{summary}\" is an sql expression, which cannot be passed as a nested sqlpage function argument.")
            }
            ExprToParamErrorKind::UnemulatedFunction { name } => {
                format!("\"{name}\" is not a supported sqlpage function. Only a few basic sql functions like concat or json_object can be used inside sqlpage functions.")
            }
            ExprToParamErrorKind::NamedArgs => "Named function arguments are not supported.\n\
                Please use positional arguments only."
                .to_string(),
        };

        SqlPageFunctionError {
            line,
            func_name,
            arguments_str,
            reason,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SqlPageFunctionError {
    pub line: u64,
    pub func_name: String,
    pub arguments_str: String,
    pub reason: String,
}

impl std::fmt::Display for SqlPageFunctionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unsupported sqlpage function argument:\n\
sqlpage.{func}({args_str})\n\n\
{reason}\n\n\
SQLPage functions can either:\n\
1. Run BEFORE the query (to provide input values)\n\
2. Run AFTER the query (to process the results)\n\
But they can't run DURING the query - the database doesn't know how to call them!\n\n\
To fix this, you can either:\n\
1. Store the function argument in a variable first:\n\
SET {func}_arg = ...;\n\
SET {func}_result = sqlpage.{func}(${func}_arg);\n\
SELECT * FROM example WHERE xxx = ${func}_result;\n\n\
2. Or move the function to the top level to process results:\n\
SELECT sqlpage.{func}(...) FROM example;",
            func = self.func_name,
            args_str = self.arguments_str,
            reason = self.reason
        )
    }
}
impl std::error::Error for SqlPageFunctionError {}

#[derive(Debug)]
struct ExprToParamError {
    line: Option<u64>,
    kind: ExprToParamErrorKind,
}

#[derive(Debug)]
enum ExprToParamErrorKind {
    UnsupportedExpr { summary: String },
    UnemulatedFunction { name: String },
    NamedArgs,
}

fn expr_summary(expr: &Expr) -> String {
    match expr {
        Expr::CompoundIdentifier(idents) => {
            let s = idents
                .iter()
                .map(|i| i.value.as_str())
                .collect::<Vec<_>>()
                .join(".");
            format!("column/table reference '{s}'")
        }
        _ => format!("{expr}"),
    }
}

fn function_arg_to_stmt_param(
    arg: &mut FunctionArg,
    ctx: &ParamExtractContext,
) -> Result<StmtParam, ExprToParamError> {
    let expr = function_arg_expr(arg).ok_or(ExprToParamError {
        line: None,
        kind: ExprToParamErrorKind::NamedArgs,
    })?;
    expr_to_stmt_param(expr, ctx)
}

pub(crate) fn function_args_to_stmt_params(
    arguments: &mut [FunctionArg],
    ctx: &ParamExtractContext,
) -> anyhow::Result<Vec<StmtParam>> {
    let mut params = Vec::with_capacity(arguments.len());
    // We iterate manually so we can pass the entire `arguments` slice to into_error on failure
    for arg in arguments.iter_mut() {
        match function_arg_to_stmt_param(arg, ctx) {
            Ok(p) => params.push(p),
            Err(e) => {
                let func_err = ctx.build_error(&e, arguments);
                return Err(anyhow::Error::new(func_err));
            }
        }
    }
    Ok(params)
}

fn emulated_func_args_to_param(
    func_name: &str,
    args: &mut [FunctionArg],
    line: u64,
) -> Result<StmtParam, ExprToParamError> {
    let inner = ParamExtractContext::with_parent(func_name);
    if func_name.eq_ignore_ascii_case("concat") {
        let mut concat_args = Vec::with_capacity(args.len());
        for a in args {
            concat_args.push(function_arg_to_stmt_param(a, &inner)?);
        }
        Ok(StmtParam::Concat(concat_args))
    } else if func_name.eq_ignore_ascii_case("json_object")
        || func_name.eq_ignore_ascii_case("jsonb_object")
        || func_name.eq_ignore_ascii_case("json_build_object")
        || func_name.eq_ignore_ascii_case("jsonb_build_object")
    {
        let mut json_obj_args = Vec::with_capacity(args.len());
        for a in args {
            json_obj_args.push(function_arg_to_stmt_param(a, &inner)?);
        }
        Ok(StmtParam::JsonObject(json_obj_args))
    } else if func_name.eq_ignore_ascii_case("json_array")
        || func_name.eq_ignore_ascii_case("jsonb_array")
        || func_name.eq_ignore_ascii_case("json_build_array")
        || func_name.eq_ignore_ascii_case("jsonb_build_array")
    {
        let mut json_obj_args = Vec::with_capacity(args.len());
        for a in args {
            json_obj_args.push(function_arg_to_stmt_param(a, &inner)?);
        }
        Ok(StmtParam::JsonArray(json_obj_args))
    } else if func_name.eq_ignore_ascii_case("coalesce") {
        let mut coalesce_args = Vec::with_capacity(args.len());
        for a in args {
            coalesce_args.push(function_arg_to_stmt_param(a, &inner)?);
        }
        Ok(StmtParam::Coalesce(coalesce_args))
    } else {
        Err(ExprToParamError {
            line: Some(line),
            kind: ExprToParamErrorKind::UnemulatedFunction {
                name: func_name.to_string(),
            },
        })
    }
}

fn expr_to_stmt_param(
    arg: &mut Expr,
    ctx: &ParamExtractContext,
) -> Result<StmtParam, ExprToParamError> {
    let line = arg.span().start.line;
    match arg {
        Expr::Value(ValueWithSpan {
            value: Value::Placeholder(placeholder),
            ..
        }) => Ok(map_param(std::mem::take(placeholder))),
        Expr::Identifier(ident) => extract_ident_param(ident).ok_or_else(|| ExprToParamError {
            line: Some(line),
            kind: ExprToParamErrorKind::UnsupportedExpr {
                summary: expr_summary(arg),
            },
        }),
        Expr::Function(Function {
            name: ObjectName(func_name_parts),
            args:
                FunctionArguments::List(FunctionArgumentList {
                    args,
                    duplicate_treatment: None,
                    ..
                }),
            ..
        }) if is_sqlpage_func(func_name_parts) => Ok(func_call_to_param(
            sqlpage_func_name(func_name_parts),
            args.as_mut_slice(),
            ctx,
        )),
        Expr::Value(ValueWithSpan {
            value: Value::SingleQuotedString(param_value),
            ..
        }) => Ok(StmtParam::Literal(std::mem::take(param_value))),
        Expr::Value(ValueWithSpan {
            value: Value::Number(param_value, _is_long),
            ..
        }) => Ok(StmtParam::Literal(param_value.clone())),
        Expr::Value(ValueWithSpan {
            value: Value::Null, ..
        }) => Ok(StmtParam::Null),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => {
            let left = expr_to_stmt_param(left, ctx)?;
            let right = expr_to_stmt_param(right, ctx)?;
            Ok(StmtParam::Concat(vec![left, right]))
        }
        Expr::Function(Function {
            name: ObjectName(func_name_parts),
            args:
                FunctionArguments::List(FunctionArgumentList {
                    args,
                    duplicate_treatment: None,
                    ..
                }),
            ..
        }) if func_name_parts.len() == 1 => {
            let func_name = func_name_parts[0]
                .as_ident()
                .map(|ident| ident.value.as_str())
                .unwrap_or_default();
            emulated_func_args_to_param(func_name, args.as_mut_slice(), line)
        }
        _ => Err(ExprToParamError {
            line: Some(line),
            kind: ExprToParamErrorKind::UnsupportedExpr {
                summary: expr_summary(arg),
            },
        }),
    }
}

fn function_arg_expr(arg: &mut FunctionArg) -> Option<&mut Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
        _ => None,
    }
}

#[inline]
#[must_use]
pub(super) fn make_tmp_placeholder(kind: AnyKind, arg_number: usize) -> String {
    let prefix = if let Some((_, DbPlaceHolder::PrefixedNumber { prefix })) =
        DB_PLACEHOLDERS.iter().find(|(db_typ, _)| *db_typ == kind)
    {
        prefix
    } else {
        TEMP_PLACEHOLDER_PREFIX
    };
    format!("{prefix}{arg_number}")
}

pub(super) fn extract_ident_param(Ident { value, .. }: &mut Ident) -> Option<StmtParam> {
    if value.starts_with('$') || value.starts_with(':') {
        let name = std::mem::take(value);
        Some(map_param(name))
    } else {
        None
    }
}

fn map_param(mut name: String) -> StmtParam {
    if name.is_empty() {
        return StmtParam::PostOrGet(name);
    }
    let prefix = name.remove(0);
    match prefix {
        '$' => StmtParam::PostOrGet(name),
        ':' => StmtParam::Post(name),
        _ => StmtParam::Get(name),
    }
}

impl VisitorMut for ParameterExtractor {
    type Break = ();
    fn pre_visit_expr(&mut self, value: &mut Expr) -> ControlFlow<Self::Break> {
        match value {
            Expr::Identifier(ident) => {
                if let Some(param) = extract_ident_param(ident) {
                    self.replace_with_placeholder(value, param);
                }
            }
            Expr::Value(ValueWithSpan {
                value: Value::Placeholder(param),
                ..
            }) if !self.is_own_placeholder(param) =>
            // this check is to avoid recursively replacing placeholders in the form of '?', or '$1', '$2', which we emit ourselves
            {
                let name = std::mem::take(param);
                self.replace_with_placeholder(value, map_param(name));
            }
            Expr::Function(Function {
                name: ObjectName(func_name_parts),
                args:
                    FunctionArguments::List(FunctionArgumentList {
                        args,
                        duplicate_treatment: None,
                        ..
                    }),
                filter: None,
                null_treatment: None,
                over: None,
                ..
            }) if is_sqlpage_func(func_name_parts) => {
                let func_name = sqlpage_func_name(func_name_parts);
                log::trace!("Handling builtin function: {func_name}");
                let arguments = std::mem::take(args);
                let ctx = ParamExtractContext {
                    parent_func: Some(func_name.to_string()),
                };
                let mut arguments_clone = arguments.clone();
                let param = func_call_to_param(func_name, &mut arguments_clone, &ctx);
                if let StmtParam::Error(msg) = &param {
                    log::trace!("Skipping extraction of {func_name} due to: {msg}");
                    *args = arguments;
                    return ControlFlow::Continue(());
                }
                self.replace_with_placeholder(value, param);
            }
            // Replace 'str1' || 'str2' with CONCAT('str1', 'str2') for MSSQL
            Expr::BinaryOp {
                left,
                op: BinaryOperator::StringConcat,
                right,
            } if self.db_info.database_type == SupportedDatabase::Mssql => {
                let left = std::mem::replace(left.as_mut(), Expr::value(Value::Null));
                let right = std::mem::replace(right.as_mut(), Expr::value(Value::Null));
                *value = Expr::Function(Function {
                    name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("CONCAT"))]),
                    args: FunctionArguments::List(FunctionArgumentList {
                        args: vec![
                            FunctionArg::Unnamed(FunctionArgExpr::Expr(left)),
                            FunctionArg::Unnamed(FunctionArgExpr::Expr(right)),
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
                });
            }
            Expr::Cast {
                kind: kind @ CastKind::DoubleColon,
                ..
            } if ![
                SupportedDatabase::Postgres,
                SupportedDatabase::Snowflake,
                SupportedDatabase::Generic,
            ]
            .contains(&self.db_info.database_type) =>
            {
                log::warn!("Casting with '::' is not supported on your database. \
                For backwards compatibility with older SQLPage versions, we will transform it to CAST(... AS ...).");
                *kind = CastKind::Cast;
            }
            _ => (),
        }
        ControlFlow::<()>::Continue(())
    }
}
