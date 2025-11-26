use std::{borrow::Cow, str::FromStr};

use anyhow::Context as _;

pub(super) fn as_sql(param: Option<Cow<'_, str>>) -> String {
    param.map_or_else(|| "NULL".into(), |x| format!("'{}'", x.replace('\'', "''")))
}

pub(super) trait FunctionParamType<'a>: Sized {
    type TargetType: 'a;
    fn from_args(
        arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>,
    ) -> anyhow::Result<Self::TargetType>;
}

impl<'a> FunctionParamType<'a> for Option<Cow<'a, str>> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        Ok(arg.next().flatten())
    }
}

impl<'a> FunctionParamType<'a> for Vec<Option<Cow<'a, str>>> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        Ok(arg.collect())
    }
}

impl<'a> FunctionParamType<'a> for Vec<Cow<'a, str>> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        Ok(arg.flatten().collect())
    }
}

impl<'a> FunctionParamType<'a> for Cow<'a, str> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        <Option<Cow<'a, str>>>::from_args(arg)?
            .ok_or_else(|| anyhow::anyhow!("Unexpected NULL value"))
    }
}

impl<'a> FunctionParamType<'a> for String {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        Ok(<Cow<'a, str>>::from_args(arg)?.into_owned())
    }
}

impl<'a> FunctionParamType<'a> for Option<String> {
    type TargetType = Self;
    fn from_args(arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>) -> anyhow::Result<Self> {
        <Option<Cow<'a, str>>>::from_args(arg).map(|x| x.map(Cow::into_owned))
    }
}

/// similar to `FromStr`, but borrows the input string
pub(super) trait BorrowFromStr<'a>: Sized {
    fn borrow_from_str(s: Cow<'a, str>) -> anyhow::Result<Self>;
}

impl<'a, T: FromStr> BorrowFromStr<'a> for T
where
    <T as FromStr>::Err: Sync + Send + std::error::Error + 'static,
{
    fn borrow_from_str(s: Cow<'a, str>) -> anyhow::Result<Self> {
        s.parse()
            .with_context(|| format!("Unable to parse {s:?} as {}", std::any::type_name::<T>()))
    }
}

pub(super) struct SqlPageFunctionParam<T: Sized>(pub T);

impl<'a, T: BorrowFromStr<'a> + Sized + 'a> FunctionParamType<'a> for SqlPageFunctionParam<T> {
    type TargetType = T;

    fn from_args(
        arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>,
    ) -> anyhow::Result<Self::TargetType> {
        let param = <Cow<'a, str>>::from_args(arg)?;
        T::borrow_from_str(param)
    }
}

impl<'a, T: BorrowFromStr<'a> + Sized + 'a> FunctionParamType<'a>
    for Option<SqlPageFunctionParam<T>>
{
    type TargetType = Option<T>;

    fn from_args(
        arg: &mut std::vec::IntoIter<Option<Cow<'a, str>>>,
    ) -> anyhow::Result<Self::TargetType> {
        let param = <Option<Cow<'a, str>>>::from_args(arg)?;
        let res = if let Some(param) = param {
            Some(T::borrow_from_str(param)?)
        } else {
            None
        };
        Ok(res)
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

impl<'a, T: IntoCow<'a>> IntoCow<'a> for Option<T> {
    fn into_cow(self) -> Option<Cow<'a, str>> {
        self.and_then(IntoCow::into_cow)
    }
}
