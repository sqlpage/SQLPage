use crate::utils::add_value_to_map;
use chrono::{DateTime, Utc};
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

pub fn sql_nonnull_to_json<'r>(mut get_ref: impl FnMut() -> sqlx::any::AnyValueRef<'r>) -> Value {
    let raw_value = get_ref();
    let type_info = raw_value.type_info();
    let type_name = type_info.name();
    log::trace!("Decoding a value of type {:?}", type_name);
    match type_name {
        "REAL" | "FLOAT" | "FLOAT4" | "FLOAT8" | "DOUBLE" | "NUMERIC" | "DECIMAL" => {
            <f64 as Decode<sqlx::any::Any>>::decode(raw_value)
                .unwrap_or(f64::NAN)
                .into()
        }
        "INT8" | "BIGINT" | "SERIAL8" | "BIGSERIAL" | "IDENTITY" | "INT64" | "INTEGER8"
        | "BIGINT UNSIGNED" | "BIGINT SIGNED" => <i64 as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
        "INT" | "INT4" | "INTEGER" => <i32 as Decode<sqlx::any::Any>>::decode(raw_value)
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
        "TIME" | "TIMETZ" => <chrono::NaiveTime as Decode<sqlx::any::Any>>::decode(raw_value)
            .as_ref()
            .map_or_else(ToString::to_string, ToString::to_string)
            .into(),
        "DATETIME" | "DATETIME2" | "DATETIMEOFFSET" | "TIMESTAMP" | "TIMESTAMPTZ" => {
            let mut date_time = <DateTime<Utc> as Decode<sqlx::any::Any>>::decode(get_ref());
            if date_time.is_err() {
                date_time = <chrono::NaiveDateTime as Decode<sqlx::any::Any>>::decode(raw_value)
                    .map(|d| d.and_utc());
            }
            Value::String(
                date_time
                    .as_ref()
                    .map_or_else(ToString::to_string, DateTime::to_rfc3339),
            )
        }
        "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => {
            <Value as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or_default()
        }
        // Deserialize as a string by default
        _ => <String as Decode<sqlx::any::Any>>::decode(raw_value)
            .unwrap_or_default()
            .into(),
    }
}

/// Takes the first column of a row and converts it to a string.
pub fn row_to_string(row: &AnyRow) -> Option<String> {
    let col = row.columns().first()?;
    match sql_to_json(row, col) {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Null => None,
        other => Some(other.to_string()),
    }
}

#[actix_web::test]
async fn test_row_to_json() -> anyhow::Result<()> {
    use sqlx::Connection;
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://:memory:".to_string());
    let mut c = sqlx::AnyConnection::connect(&db_url).await?;
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
