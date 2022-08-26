use futures_util::stream::{self, Stream};
use futures_util::StreamExt;
use std::future::ready;
use serde_json::{Map, Value};

use sqlx::any::{AnyArguments, AnyQueryResult, AnyRow};
use sqlx::{Arguments, Column, Decode, Either, Row};

pub fn stream_query_results<'a>(
    db: &'a sqlx::AnyPool,
    sql_source: &'a [u8],
    argument: &'a str,
) -> impl Stream<Item=DbItem> + 'a {
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


pub fn row_to_json(row: AnyRow) -> Value {
    use sqlx::{TypeInfo, ValueRef};
    use Value::{Null, Object};

    let columns = row.columns();
    let mut map = Map::new();
    for col in columns {
        let key = col.name().to_string();
        let value: Value = match row.try_get_raw(col.ordinal()) {
            Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                    <f64 as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or(f64::NAN).into()
                }
                "INT" | "INTEGER" | "INT8" | "INT2" | "INT4" | "TINYINT" | "SMALLINT" | "BIGINT" => {
                    <i64 as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or_default().into()
                }
                "BOOL" | "BOOLEAN" => <bool as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or_default().into(),
                "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => <&[u8] as Decode<sqlx::any::Any>>::decode(raw_value)
                    .and_then(|rv| serde_json::from_slice::<Value>(rv).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>))
                    .unwrap_or_default(),
                // Deserialize as a string by default
                _ => <String as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or_default().into(),
            },
            Ok(_null) => Null,
            Err(e) => {
                log::warn!("Unable to extract value from row: {:?}", e);
                Null
            }
        };
        map = add_value_to_map(map, (key, value));
    }
    Object(map)
}

pub fn add_value_to_map(mut map: Map<String, Value>, (key, value): (String, Value)) -> Map<String, Value> {
    use Value::{Array};
    use serde_json::map::Entry::*;
    match map.entry(key) {
        Vacant(vacant) => { vacant.insert(value); }
        Occupied(mut old_entry) => match old_entry.get_mut() {
            Array(old_array) => { old_array.push(value) }
            old_scalar => {
                *old_scalar = Array(vec![old_scalar.take(), value])
            }
        }
    }
    map
}

#[actix_web::test]
async fn test_row_to_json() -> anyhow::Result<()> {
    use sqlx::Connection;
    let mut c = sqlx::AnyConnection::connect("sqlite://:memory:").await?;
    let row = sqlx::query("SELECT \
        3.14159 as one_value, \
        1 as two_values, \
        2 as two_values, \
        'x' as three_values, \
        'y' as three_values, \
        'z' as three_values \
    ").fetch_one(&mut c).await?;
    assert_eq!(row_to_json(row), serde_json::json!({
        "one_value": 3.14159,
        "two_values": [1,2],
        "three_values": ["x","y","z"],
    }));
    Ok(())
}