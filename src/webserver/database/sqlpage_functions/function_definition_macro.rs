/// Defines all sqlpage functions
#[macro_export]
macro_rules! sqlpage_functions {
    ($($func_name:ident($(($request:ty)$(,)?)? $($param_name:ident : $param_type:ty),*);)*) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum SqlPageFunctionName {
            $( #[allow(non_camel_case_types)] $func_name ),*
        }

        impl ::std::str::FromStr for SqlPageFunctionName {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> anyhow::Result<Self> {
                match s {
                    $(stringify!($func_name) => Ok(SqlPageFunctionName::$func_name),)*
                    unknown_name => anyhow::bail!("Unknown function {unknown_name:?}"),
                }
            }
        }

        impl std::fmt::Display for SqlPageFunctionName {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(SqlPageFunctionName::$func_name => {
                        write!(f, "sqlpage.{}", stringify!($func_name))?;
                        if f.alternate() {
                            write!(f, "(")?;
                            let mut _first = true;
                            $(
                                if !_first {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}", stringify!($param_name))?;
                                _first = false;
                            )*
                            write!(f, ")")?;
                        }
                        Ok(())
                    }),*
                }
            }
        }
        impl SqlPageFunctionName {
            pub(crate) async fn evaluate<'a>(
                &self,
                #[allow(unused_variables)]
                request: &'a RequestInfo,
                params: Vec<Option<Cow<'a, str>>>
            ) -> anyhow::Result<Option<Cow<'a, str>>> {
                use $crate::webserver::database::sqlpage_functions::function_definition_macro::*;
                match self {
                    $(
                        SqlPageFunctionName::$func_name => {
                            let mut iter_params = params.into_iter();
                            $(
                                let $param_name = <$param_type as FunctionParamType<'_>>::from_args(&mut iter_params)
                                    .map_err(|e| anyhow!("Parameter {}: {e}", stringify!($param_name)))?;
                            )*
                            if let Some(extraneous_param) = iter_params.next() {
                                anyhow::bail!("Too many arguments. Remove extra argument {}", as_sql(extraneous_param));
                            }
                            let result = $func_name(
                                $(<$request>::from(request),)*
                                $($param_name.into_arg()),*
                            ).await;
                            result.into_cow_result()
                        }
                    )*
                }
            }
        }
    }
}

use std::{borrow::Cow, str::FromStr};

use anyhow::Context as _;
pub use sqlpage_functions;

pub(super) fn as_sql(param: Option<Cow<'_, str>>) -> String {
    param.map_or_else(|| "NULL".into(), |x| format!("'{}'", x.replace('\'', "''")))
}

pub(super) trait FunctionParamType<'a>: Sized {
    type TargetType: 'a;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self>;
    fn into_arg(self) -> Self::TargetType;
}

impl<'a> FunctionParamType<'a> for Option<Cow<'a, str>> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        arg.next().ok_or_else(|| anyhow::anyhow!("Missing"))
    }
    fn into_arg(self) -> Self::TargetType {
        self
    }
}

impl<'a> FunctionParamType<'a> for Cow<'a, str> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        <Option<Cow<'a, str>>>::from_args(arg)?
            .ok_or_else(|| anyhow::anyhow!("Unexpected NULL value"))
    }
    fn into_arg(self) -> Self::TargetType {
        self
    }
}

impl<'a> FunctionParamType<'a> for String {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        Ok(<Cow<'a, str>>::from_args(arg)?.into_owned())
    }
    fn into_arg(self) -> Self::TargetType {
        self
    }
}

pub(super) struct SqlPageFunctionParam<T>(T);

impl<'a, T: FromStr + Sized + 'a> FunctionParamType<'a> for SqlPageFunctionParam<T>
where
    <T as FromStr>::Err: Sync + Send + std::error::Error + 'static,
{
    type TargetType = T;

    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        let param = <Cow<'a, str>>::from_args(arg)?;
        param
            .parse()
            .with_context(|| {
                format!(
                    "Unable to parse {param:?} as {}",
                    std::any::type_name::<T>()
                )
            })
            .map(SqlPageFunctionParam)
    }
    fn into_arg(self) -> Self::TargetType {
        self.0
    }
}
pub(super) trait FunctionResultType<'a> {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>>;
}

impl<'a, T: IntoCow<'a>> FunctionResultType<'a> for anyhow::Result<T> {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>> {
        self.map(IntoCow::into_cow)
    }
}

impl<'a, T: IntoCow<'a>> FunctionResultType<'a> for T {
    fn into_cow_result(self) -> anyhow::Result<Option<Cow<'a, str>>> {
        Ok(self.into_cow())
    }
}

trait IntoCow<'a> {
    fn into_cow(self) -> Option<Cow<'a, str>>;
}

impl<'a> IntoCow<'a> for Option<Cow<'a, str>> {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        self
    }
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

impl<'a> IntoCow<'a> for &'a str {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}
