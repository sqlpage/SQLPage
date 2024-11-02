use crate::utils::add_value_to_map;
use chrono::{DateTime, Utc};
use serde_json::{self, Map, Value};
use sqlx::any::{AnyRow, AnyTypeInfo, AnyTypeInfoKind};
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
        "BIT" if matches!(*type_info, AnyTypeInfo(AnyTypeInfoKind::Mssql(_))) => {
            <bool as Decode<sqlx::any::Any>>::decode(raw_value)
                .unwrap_or_default()
                .into()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;

    fn db_specific_test(db_type: &str) -> Option<String> {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://:memory:".to_string());
        if db_url.starts_with(db_type) {
            Some(db_url)
        } else {
            log::warn!("Skipping test because DATABASE_URL is not set to a {db_type} database");
            None
        }
    }

    #[actix_web::test]
    async fn test_row_to_json() -> anyhow::Result<()> {
        use sqlx::Connection;
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://:memory:".to_string());
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

    #[actix_web::test]
    async fn test_mssql_bit_to_json() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("mssql") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query("SELECT CAST(1 AS BIT) as true_bit, CAST(0 AS BIT) as false_bit, CAST(NULL AS BIT) as null_bit")
            .fetch_one(&mut c)
            .await?;
        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "true_bit": true,
                "false_bit": false,
                "null_bit": null,
            })
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_postgres_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("postgres") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query(
            "SELECT 
                42::INT2 as small_int,
                42::INT4 as integer,
                42::INT8 as big_int,
                42.42::FLOAT4 as float4,
                42.42::FLOAT8 as float8,
                TRUE as boolean,
                '2024-03-14'::DATE as date,
                '13:14:15'::TIME as time,
                '2024-03-14 13:14:15'::TIMESTAMP as timestamp,
                '2024-03-14 13:14:15+00'::TIMESTAMPTZ as timestamptz,
                '{\"key\": \"value\"}'::JSON as json,
                '{\"key\": \"value\"}'::JSONB as jsonb",
        )
        .fetch_one(&mut c)
        .await?;

        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "small_int": 42,
                "integer": 42,
                "big_int": 42,
                "float4": 42.42,
                "float8": 42.42,
                "boolean": true,
                "date": "2024-03-14",
                "time": "13:14:15",
                "timestamp": "2024-03-14T13:14:15+00:00",
                "timestamptz": "2024-03-14T13:14:15+00:00",
                "json": {"key": "value"},
                "jsonb": {"key": "value"},
            })
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_mysql_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("mysql") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query(
            "SELECT 
                CAST(42 AS SIGNED) as signed_int,
                CAST(42 AS UNSIGNED) as unsigned_int,
                42.42 as decimal_number,
                TRUE as boolean,
                CAST('2024-03-14' AS DATE) as date,
                CAST('13:14:15' AS TIME) as time,
                CAST('2024-03-14 13:14:15' AS DATETIME) as datetime,
                CAST('{\"key\": \"value\"}' AS JSON) as json",
        )
        .fetch_one(&mut c)
        .await?;

        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "signed_int": 42,
                "unsigned_int": 42,
                "decimal_number": 42.42,
                "boolean": true,
                "date": "2024-03-14",
                "time": "13:14:15",
                "datetime": "2024-03-14T13:14:15+00:00",
                "json": {"key": "value"},
            })
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_sqlite_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("sqlite") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query(
            "SELECT 
                42 as integer,
                42.42 as real,
                'xxx' as string,
                x'68656c6c6f20776f726c64' as blob",
        )
        .fetch_one(&mut c)
        .await?;

        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "integer": 42,
                "real": 42.42,
                "string": "xxx",
                "blob": "hello world",
            })
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_mssql_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("mssql") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query(
            "SELECT 
                CAST(1 AS BIT) as true_bit,
                CAST(0 AS BIT) as false_bit,
                CAST(NULL AS BIT) as null_bit,
                CAST(42 AS SMALLINT) as small_int,
                CAST(42 AS INT) as integer,
                CAST(42 AS BIGINT) as big_int,
                CAST(42.42 AS REAL) as real,
                CAST(42.42 AS FLOAT) as float,
                CAST(42.42 AS DECIMAL(10,2)) as decimal,
                CAST('2024-03-14' AS DATE) as date,
                CAST('13:14:15' AS TIME) as time,
                CAST('2024-03-14 13:14:15' AS DATETIME) as datetime,
                CAST('2024-03-14 13:14:15' AS DATETIME2) as datetime2,
                CAST('2024-03-14 13:14:15 +00:00' AS DATETIMEOFFSET) as datetimeoffset,
                N'Unicode String' as nvarchar,
                'ASCII String' as varchar",
        )
        .fetch_one(&mut c)
        .await?;

        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "true_bit": true,
                "false_bit": false,
                "null_bit": null,
                "small_int": 42,
                "integer": 42,
                "big_int": 42,
                "real": 42.42,
                "float": 42.42,
                "decimal": 42.42,
                "date": "2024-03-14",
                "time": "13:14:15",
                "datetime": "2024-03-14T13:14:15+00:00",
                "datetime2": "2024-03-14T13:14:15+00:00",
                "datetimeoffset": "2024-03-14T13:14:15+00:00",
                "nvarchar": "Unicode String",
                "varchar": "ASCII String",
            })
        );
        Ok(())
    }
}
