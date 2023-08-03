pub use crate::file_cache::FileCache;
use crate::utils::add_value_to_map;
use serde_json::{self, Map, Value};
use sqlx::any::AnyRow;
use sqlx::Decode;
use sqlx::{Column, Row, TypeInfo, ValueRef};

pub fn row_to_json(row: &AnyRow) -> Value {
    use Value::Object;

    let columns = row.columns();
    let mut map = Map::new();
    for col in columns {
        let key = col.name().to_string();
        let value: Value = sql_to_json(row, col);
        map = add_value_to_map(map, (key, value));
    }
    Object(map)
}

pub fn sql_to_json(row: &AnyRow, col: &sqlx::any::AnyColumn) -> Value {
    let raw_value_result = row.try_get_raw(col.ordinal());
    match raw_value_result {
        Ok(raw_value) if !raw_value.is_null() => {
            let mut raw_value = Some(raw_value);
            log::trace!("Decoding a value of type {:?}", col.type_info().name());
            let decoded = sql_nonnull_to_json(|| {
                raw_value
                    .take()
                    .unwrap_or_else(|| row.try_get_raw(col.ordinal()).unwrap())
            });
            log::trace!("Decoded value: {:?}", decoded);
            decoded
        }
        Ok(_null) => Value::Null,
        Err(e) => {
            log::warn!("Unable to extract value from row: {:?}", e);
            Value::Null
        }
    }
}

macro_rules! try_decode_with {
    ($raw_value:expr, [$ty0:ty], $fn:expr) => {
        <$ty0 as Decode<sqlx::any::Any>>::decode($raw_value).map($fn)
    };
    ($raw_value:expr, [$ty0:ty, $($ty:ty),+], $fn:expr) => {
        match try_decode_with!($raw_value, [$ty0], $fn) {
            Ok(value) => Ok(value),
            Err(_) => try_decode_with!($raw_value, [$($ty),+], $fn),
        }
    };
}

pub fn sql_nonnull_to_json<'r>(mut get_ref: impl FnMut() -> sqlx::any::AnyValueRef<'r>) -> Value {
    let raw_value = get_ref();
    match raw_value.type_info().name() {
        "REAL" | "FLOAT" | "NUMERIC" | "DECIMAL" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
            <f64 as Decode<sqlx::any::Any>>::decode(raw_value)
                .unwrap_or(f64::NAN)
                .into()
        }
        "INT8" | "BIGINT" | "INTEGER" => <i64 as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT" | "INT4" => <i32 as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT2" | "SMALLINT" => <i16 as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "BOOL" | "BOOLEAN" => <bool as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "DATE" => <chrono::NaiveDate as Decode<sqlx::any::Any>>::decode(raw_value)
            .as_ref()
            .map_or_else(std::string::ToString::to_string, ToString::to_string)
            .into(),
        "TIME" => <chrono::NaiveTime as Decode<sqlx::any::Any>>::decode(raw_value)
            .as_ref()
            .map_or_else(ToString::to_string, ToString::to_string)
            .into(),
        "DATETIME" | "DATETIME2" | "DATETIMEOFFSET" | "TIMESTAMP" | "TIMESTAMPTZ" => {
            try_decode_with!(
                get_ref(),
                [chrono::NaiveDateTime, chrono::DateTime<chrono::Utc>],
                |v| dbg!(v).to_string()
            )
            .unwrap_or_else(|e| format!("Unable to decode date: {e:?}"))
            .into()
        }
        "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => {
            <&[u8] as Decode<sqlx::any::Any>>::decode(raw_value)
                .and_then(|rv| {
                    serde_json::from_slice::<Value>(rv)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Sync + Send>)
                })
                .unwrap_or_default()
        }
        // Deserialize as a string by default
        _ => <String as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
    }
}

#[actix_web::test]
async fn test_row_to_json() -> anyhow::Result<()> {
    use sqlx::Connection;
    let mut c = sqlx::AnyConnection::connect("sqlite://:memory:").await?;
    let row = sqlx::query(
        "SELECT \
        123.456 as one_value, \
        1 as two_values, \
        2 as two_values, \
        'x' as three_values, \
        'y' as three_values, \
        'z' as three_values \
    ",
    )
    .fetch_one(&mut c)
    .await?;
    assert_eq!(
        row_to_json(&row),
        serde_json::json!({
            "one_value": 123.456,
            "two_values": [1,2],
            "three_values": ["x","y","z"],
        })
    );
    Ok(())
}
