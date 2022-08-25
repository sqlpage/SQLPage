use futures_util::stream::{self, Stream};
use futures_util::StreamExt;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::future::ready;

use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow};
use sqlx::{Arguments, Column, Database, Decode, Either, Row};

pub fn stream_query_results<'a>(
    db: &'a sqlx::AnyPool,
    sql_source: &'a [u8],
    argument: &'a str,
) -> impl Stream<Item = DbItem> + 'a {
    let mut arguments = AnyArguments::default();
    arguments.add(argument);
    match std::str::from_utf8(sql_source) {
        Ok(sql) => sqlx::query_with(sql, arguments).fetch_many(db),
        Err(e) => {
            let error = sqlx::Error::Decode(Box::new(e));
            stream::once(ready(Err(error))).boxed()
        }
    }
    .map(|res| match res {
        Ok(Either::Right(r)) => DbItem::Row(r),
        Ok(Either::Left(r)) => DbItem::FinishedQuery(r),
        Err(e) => DbItem::Error(e),
    })
}

pub enum DbItem {
    Row(AnyRow),
    FinishedQuery(AnyQueryResult),
    Error(sqlx::Error),
}

pub struct SerializeRow<R: Row>(pub R);

impl<'r, R: Row> Serialize for &'r SerializeRow<R>
where
    usize: sqlx::ColumnIndex<R>,
    &'r str: sqlx::Decode<'r, <R as Row>::Database>,
    f64: sqlx::Decode<'r, <R as Row>::Database>,
    i64: sqlx::Decode<'r, <R as Row>::Database>,
    bool: sqlx::Decode<'r, <R as Row>::Database>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use sqlx::{TypeInfo, ValueRef};
        let columns = self.0.columns();
        let mut map = serializer.serialize_map(Some(columns.len()))?;
        for col in columns {
            let key = col.name();
            match self.0.try_get_raw(col.ordinal()) {
                Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                    "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                        map_serialize::<_, _, f64>(&mut map, key, raw_value)
                    }
                    "INT" | "INTEGER" | "INT8" | "INT2" | "INT4" | "TINYINT" | "SMALLINT"
                    | "BIGINT" => map_serialize::<_, _, i64>(&mut map, key, raw_value),
                    "BOOL" | "BOOLEAN" => map_serialize::<_, _, bool>(&mut map, key, raw_value),
                    // Deserialize as a string by default
                    _ => map_serialize::<_, _, &str>(&mut map, key, raw_value),
                },
                _ => map.serialize_entry(key, &()), // Serialize null
            }?
        }
        map.end()
    }
}

fn map_serialize<'r, M: SerializeMap, DB: Database, T: Decode<'r, DB> + Serialize>(
    map: &mut M,
    key: &str,
    raw_value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
) -> Result<(), M::Error> {
    let val = T::decode(raw_value).map_err(serde::ser::Error::custom)?;
    map.serialize_entry(key, &val)
}
