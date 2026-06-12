//! Dispatch machinery for the built-in `sqlpage.*` SQL functions.
//!
//! Each function is a plain `async fn` in its own module under [`functions/`](super::functions).
//! [`sqlpage_functions!`] turns the module list in [`functions`](super::functions) into the
//! [`SqlPageFunctionName`](super::functions::SqlPageFunctionName) enum the engine dispatches on.
//! Adapting each signature to the uniform
//! `(request, db, args) -> Option<string>` convention is done generically by [`Extract`] (per
//! argument type), [`Handler`] (per argument count, the trick `axum` uses) and [`IntoCowResult`]
//! (per return type); the macro itself carries no type-level glue.

use std::borrow::Cow;
use std::future::Future;

use anyhow::Context as _;

use super::http_fetch_request::HttpFetchRequest;
use crate::webserver::database::execute_queries::DbConn;
use crate::webserver::http_request_info::{ExecutionContext, RequestInfo};

/// Renders a SQL argument as it would appear in a query, for error messages.
fn as_sql(param: Option<Cow<'_, str>>) -> String {
    param.map_or_else(|| "NULL".into(), |x| format!("'{}'", x.replace('\'', "''")))
}

/// The request, optional database connection, and evaluated SQL arguments a function call works on.
pub(crate) struct FunctionContext<'a, 'c> {
    request: &'a ExecutionContext,
    db: Option<&'c mut DbConn>,
    arguments: std::vec::IntoIter<Option<Cow<'a, str>>>,
}

impl<'a, 'c> FunctionContext<'a, 'c> {
    pub(crate) fn new(
        request: &'a ExecutionContext,
        db_connection: &'c mut DbConn,
        arguments: Vec<Option<Cow<'a, str>>>,
    ) -> Self {
        Self {
            request,
            db: Some(db_connection),
            arguments: arguments.into_iter(),
        }
    }

    /// The next argument, treating both a missing and an explicit `NULL` argument as `None`.
    fn next_arg(&mut self) -> Option<Cow<'a, str>> {
        self.arguments.next().flatten()
    }

    fn next_required(&mut self) -> anyhow::Result<Cow<'a, str>> {
        self.next_arg()
            .ok_or_else(|| anyhow::anyhow!("Unexpected NULL value"))
    }

    fn expect_no_extra_args(&mut self) -> anyhow::Result<()> {
        match self.arguments.next() {
            None => Ok(()),
            Some(extra) => anyhow::bail!(
                "Too many arguments. Remove extra argument {}",
                as_sql(extra)
            ),
        }
    }
}

/// Obtains one function argument from the context. Implemented per concrete argument type, so the
/// context is only borrowed briefly and the extracted values can coexist while the function runs.
pub(crate) trait Extract<'a, 'c>: Sized {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self>;
}

impl<'a, 'c> Extract<'a, 'c> for &'a RequestInfo {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        Ok(ctx.request.into())
    }
}

impl<'a, 'c> Extract<'a, 'c> for &'a ExecutionContext {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        Ok(ctx.request)
    }
}

impl<'a, 'c> Extract<'a, 'c> for &'c mut DbConn {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        ctx.db
            .take()
            .context("This function cannot be called in this context (no database connection)")
    }
}

impl<'a, 'c> Extract<'a, 'c> for Cow<'a, str> {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        ctx.next_required()
    }
}

impl<'a, 'c> Extract<'a, 'c> for Option<Cow<'a, str>> {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        Ok(ctx.next_arg())
    }
}

impl<'a, 'c> Extract<'a, 'c> for Option<String> {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        Ok(ctx.next_arg().map(Cow::into_owned))
    }
}

/// Collects the remaining arguments (dropping `NULL`s) for variadic functions.
impl<'a, 'c> Extract<'a, 'c> for Vec<Cow<'a, str>> {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        Ok(ctx.arguments.by_ref().flatten().collect())
    }
}

impl<'a, 'c> Extract<'a, 'c> for usize {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        let arg = ctx.next_required()?;
        arg.parse()
            .with_context(|| format!("Unable to parse {arg:?} as a positive integer"))
    }
}

impl<'a, 'c> Extract<'a, 'c> for Option<HttpFetchRequest<'a>> {
    fn extract(ctx: &mut FunctionContext<'a, 'c>) -> anyhow::Result<Self> {
        ctx.next_arg()
            .map(HttpFetchRequest::borrow_from_str)
            .transpose()
    }
}

/// Like [`FromStr`](std::str::FromStr) but able to borrow from the input (see [`HttpFetchRequest`]).
pub(crate) trait BorrowFromStr<'a>: Sized {
    fn borrow_from_str(s: Cow<'a, str>) -> anyhow::Result<Self>;
}

/// Implemented for every `async fn` whose arguments all [`Extract`] and whose output
/// [`IntoCowResult`]. One `impl_handler!` line per argument count; adding an argument is one more.
pub(crate) trait Handler<'a, 'c, Args> {
    fn call(
        self,
        ctx: FunctionContext<'a, 'c>,
    ) -> impl Future<Output = anyhow::Result<Option<Cow<'a, str>>>>;
}

macro_rules! impl_handler {
    ($($arg:ident),*) => {
        impl<'a, 'c, Func, Fut, Ret $(, $arg)*> Handler<'a, 'c, ($($arg,)*)> for Func
        where
            'a: 'c,
            Func: Fn($($arg),*) -> Fut,
            Fut: Future<Output = Ret>,
            Ret: IntoCowResult<'a>,
            $($arg: Extract<'a, 'c>,)*
        {
            #[allow(non_snake_case, unused_mut)]
            async fn call(self, mut ctx: FunctionContext<'a, 'c>) -> anyhow::Result<Option<Cow<'a, str>>> {
                $(let $arg = $arg::extract(&mut ctx)?;)*
                ctx.expect_no_extra_args()?;
                self($($arg),*).await.into_cow_result()
            }
        }
    };
}

impl_handler!();
impl_handler!(A0);
impl_handler!(A0, A1);
impl_handler!(A0, A1, A2);
impl_handler!(A0, A1, A2, A3);
impl_handler!(A0, A1, A2, A3, A4);

/// Normalises a function's return value into what the SQL engine consumes.
pub(crate) trait IntoCowResult<'a> {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>>;
}

impl<'a, T: IntoCow<'a>> IntoCowResult<'a> for anyhow::Result<T> {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>> {
        self.map(IntoCow::into_cow)
    }
}

impl<'a, T: IntoCow<'a>> IntoCowResult<'a> for T {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>> {
        Ok(self.into_cow())
    }
}

trait IntoCow<'a> {
    fn into_cow(self) -> Option<Cow<'a, str>>;
}

impl<'a> IntoCow<'a> for Cow<'a, str> {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        Some(self)
    }
}

impl<'a> IntoCow<'a> for String {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        Some(Cow::Owned(self))
    }
}

impl<'a, 'b: 'a> IntoCow<'a> for &'b str {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl<'a, T: IntoCow<'a>> IntoCow<'a> for Option<T> {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        self.and_then(IntoCow::into_cow)
    }
}

/// Declares the listed function modules and builds the [`SqlPageFunctionName`] dispatch enum from
/// them.
macro_rules! sqlpage_functions {
    ($($func:ident),* $(,)?) => {
        $(
            mod $func;
        )*

        /// One variant per built-in `sqlpage.*` function.
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum SqlPageFunctionName {
            $($func),*
        }

        impl SqlPageFunctionName {
            const ALL: &'static [Self] = &[$(Self::$func),*];

            fn name(self) -> &'static str {
                match self {
                    $(Self::$func => stringify!($func)),*
                }
            }

            pub(crate) async fn evaluate<'a, 'c>(
                self,
                request: &'a $crate::webserver::http_request_info::ExecutionContext,
                db_connection: &'c mut $crate::webserver::database::execute_queries::DbConn,
                arguments: Vec<Option<::std::borrow::Cow<'a, str>>>,
            ) -> anyhow::Result<Option<::std::borrow::Cow<'a, str>>>
            where
                'a: 'c,
            {
                use $crate::webserver::database::sqlpage_functions::function_traits::{
                    FunctionContext, Handler,
                };
                let ctx = FunctionContext::new(request, db_connection, arguments);
                match self {
                    $(SqlPageFunctionName::$func => Handler::call($func::$func, ctx).await),*
                }
            }
        }
    };
}

pub(crate) use sqlpage_functions;
