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
                use $crate::webserver::database::sqlpage_functions::function_traits::*;
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
                                $($param_name.into()),*
                            ).await;
                            result.into_cow_result()
                        }
                    )*
                }
            }
        }
    }
}

pub use sqlpage_functions;
