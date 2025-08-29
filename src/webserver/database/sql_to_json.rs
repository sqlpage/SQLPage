use crate::utils::add_value_to_map;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
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
            log::trace!("Decoded value: {decoded:?}");
            decoded
        }
        Ok(_null) => Value::Null,
        Err(e) => {
            log::warn!("Unable to extract value from row: {e:?}");
            Value::Null
        }
    }
}

fn decode_raw<'a, T: Decode<'a, sqlx::any::Any> + Default>(
    raw_value: sqlx::any::AnyValueRef<'a>,
) -> T {
    match T::decode(raw_value) {
        Ok(v) => v,
        Err(e) => {
            let type_name = std::any::type_name::<T>();
            log::error!("Failed to decode {type_name} value: {e}");
            T::default()
        }
    }
}

pub fn sql_nonnull_to_json<'r>(mut get_ref: impl FnMut() -> sqlx::any::AnyValueRef<'r>) -> Value {
    use AnyTypeInfoKind::{Mssql, MySql};
    let raw_value = get_ref();
    let type_info = raw_value.type_info();
    let type_name = type_info.name();
    log::trace!("Decoding a value of type {type_name:?} (type info: {type_info:?})");
    let AnyTypeInfo(ref db_type) = *type_info;
    match type_name {
        "REAL" | "FLOAT" | "FLOAT4" | "FLOAT8" | "DOUBLE" | "NUMERIC" | "DECIMAL" => {
            decode_raw::<f64>(raw_value).into()
        }
        "INT8" | "BIGINT" | "SERIAL8" | "BIGSERIAL" | "IDENTITY" | "INT64" | "INTEGER8"
        | "BIGINT SIGNED" => decode_raw::<i64>(raw_value).into(),
        "INT" | "INT4" | "INTEGER" | "MEDIUMINT" | "YEAR" => decode_raw::<i32>(raw_value).into(),
        "INT2" | "SMALLINT" | "TINYINT" => decode_raw::<i16>(raw_value).into(),
        "BIGINT UNSIGNED" => decode_raw::<u64>(raw_value).into(),
        "INT UNSIGNED" | "MEDIUMINT UNSIGNED" | "SMALLINT UNSIGNED" | "TINYINT UNSIGNED" => {
            decode_raw::<u32>(raw_value).into()
        }
        "BOOL" | "BOOLEAN" => decode_raw::<bool>(raw_value).into(),
        "BIT" if matches!(db_type, Mssql(_)) => decode_raw::<bool>(raw_value).into(),
        "BIT" if matches!(db_type, MySql(mysql_type) if mysql_type.max_size() == Some(1)) => {
            decode_raw::<bool>(raw_value).into()
        }
        "BIT" if matches!(db_type, MySql(_)) => decode_raw::<u64>(raw_value).into(),
        "DATE" => decode_raw::<chrono::NaiveDate>(raw_value)
            .to_string()
            .into(),
        "TIME" | "TIMETZ" => decode_raw::<chrono::NaiveTime>(raw_value)
            .to_string()
            .into(),
        "DATETIMEOFFSET" | "TIMESTAMP" | "TIMESTAMPTZ" => {
            decode_raw::<DateTime<FixedOffset>>(raw_value)
                .to_rfc3339()
                .into()
        }
        "DATETIME" | "DATETIME2" => decode_raw::<NaiveDateTime>(raw_value)
            .format("%FT%T%.f")
            .to_string()
            .into(),
        "MONEY" | "SMALLMONEY" if matches!(db_type, Mssql(_)) => {
            decode_raw::<f64>(raw_value).into()
        }
        "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => decode_raw::<Value>(raw_value),
        "BLOB" | "BYTEA" | "FILESTREAM" | "VARBINARY" | "BIGVARBINARY" | "BINARY" | "IMAGE" => {
            vec_to_data_uri_value(&decode_raw::<Vec<u8>>(raw_value))
        }
        // Deserialize as a string by default
        _ => decode_raw::<String>(raw_value).into(),
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

/// Converts binary data to a data URL string.
/// This function is used by both SQL type conversion and file reading functions.
pub fn vec_to_data_uri(bytes: &[u8]) -> String {
    vec_to_data_uri_with_mime(bytes, "application/octet-stream")
}

/// Converts binary data to a data URL string with a specific MIME type.
/// This function is used by both SQL type conversion and file reading functions.
pub fn vec_to_data_uri_with_mime(bytes: &[u8], mime_type: &str) -> String {
    let mut data_url = format!("data:{mime_type};base64,");
    base64::Engine::encode_string(
        &base64::engine::general_purpose::STANDARD,
        bytes,
        &mut data_url,
    );
    data_url
}

/// Converts binary data to a data URL JSON value.
/// This is a convenience function for SQL type conversion.
pub fn vec_to_data_uri_value(bytes: &[u8]) -> Value {
    Value::String(vec_to_data_uri(bytes))
}

#[cfg(test)]
mod tests {
    use crate::app_config::tests::test_database_url;

    use super::*;
    use sqlx::Connection;

    fn setup_logging() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init();
    }

    fn db_specific_test(db_type: &str) -> Option<String> {
        setup_logging();
        let db_url = test_database_url();
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
        let db_url = test_database_url();
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
                42.25::FLOAT4 as float4,
                42.25::FLOAT8 as float8,
                TRUE as boolean,
                '2024-03-14'::DATE as date,
                '13:14:15'::TIME as time,
                '2024-03-14 13:14:15'::TIMESTAMP as timestamp,
                '2024-03-14 13:14:15+02:00'::TIMESTAMPTZ as timestamptz,
                INTERVAL '1 year 2 months 3 days' as complex_interval,
                INTERVAL '4 hours' as hour_interval,
                INTERVAL '1.5 days' as fractional_interval,
                '{\"key\": \"value\"}'::JSON as json,
                '{\"key\": \"value\"}'::JSONB as jsonb,
                age('2024-03-14'::timestamp, '2024-01-01'::timestamp) as age_interval,
                justify_interval(interval '1 year 2 months 3 days') as justified_interval,
                1234.56::MONEY as money_val,
                '\\x68656c6c6f20776f726c64'::BYTEA as blob_data",
        )
        .fetch_one(&mut c)
        .await?;

        expect_json_object_equal(
            &row_to_json(&row),
            &serde_json::json!({
                "small_int": 42,
                "integer": 42,
                "big_int": 42,
                "float4": 42.25,
                "float8": 42.25,
                "boolean": true,
                "date": "2024-03-14",
                "time": "13:14:15",
                "timestamp": "2024-03-14T13:14:15+00:00",
                "timestamptz": "2024-03-14T11:14:15+00:00",
                "complex_interval": "1 year 2 mons 3 days",
                "hour_interval": "04:00:00",
                "fractional_interval": "1 day 12:00:00",
                "json": {"key": "value"},
                "jsonb": {"key": "value"},
                "age_interval": "2 mons 13 days",
                "justified_interval": "1 year 2 mons 3 days",
                "money_val": "$1,234.56",
                "blob_data": "data:application/octet-stream;base64,aGVsbG8gd29ybGQ="
            }),
        );
        Ok(())
    }

    /// Postgres encodes values differently in prepared statements and in "simple" queries
    /// <https://www.postgresql.org/docs/9.1/protocol-flow.html#PROTOCOL-FLOW-EXT-QUERY>
    #[actix_web::test]
    async fn test_postgres_prepared_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("postgres") else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;
        let row = sqlx::query(
            "SELECT
                '2024-03-14'::DATE as date,
                '13:14:15'::TIME as time,
                '2024-03-14 13:14:15+02:00'::TIMESTAMPTZ as timestamptz,
                INTERVAL '-01:02:03' as time_interval,
                '{\"key\": \"value\"}'::JSON as json,
                1234.56::MONEY as money_val,
                '\\x74657374'::BYTEA as blob_data
            where $1",
        )
        .bind(true)
        .fetch_one(&mut c)
        .await?;

        expect_json_object_equal(
            &row_to_json(&row),
            &serde_json::json!({
                "date": "2024-03-14",
                "time": "13:14:15",
                "timestamptz": "2024-03-14T11:14:15+00:00",
                "time_interval": "-01:02:03",
                "json": {"key": "value"},
                "money_val": "", // TODO: fix this bug: https://github.com/sqlpage/SQLPage/issues/983
                "blob_data": "data:application/octet-stream;base64,dGVzdA=="
            }),
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_mysql_types() -> anyhow::Result<()> {
        let db_url = db_specific_test("mysql").or_else(|| db_specific_test("mariadb"));
        let Some(db_url) = db_url else {
            return Ok(());
        };
        let mut c = sqlx::AnyConnection::connect(&db_url).await?;

        sqlx::query(
            "CREATE TEMPORARY TABLE _sqlp_t (
                tiny_int TINYINT,
                small_int SMALLINT,
                medium_int MEDIUMINT,
                signed_int INTEGER,
                big_int BIGINT,
                unsigned_int INTEGER UNSIGNED,
                tiny_int_unsigned TINYINT UNSIGNED,
                small_int_unsigned SMALLINT UNSIGNED,
                medium_int_unsigned MEDIUMINT UNSIGNED,
                big_int_unsigned BIGINT UNSIGNED,
                decimal_num DECIMAL(10,2),
                float_num FLOAT,
                double_num DOUBLE,
                bit_val BIT(1),
                date_val DATE,
                time_val TIME,
                datetime_val DATETIME,
                timestamp_val TIMESTAMP,
                year_val YEAR,
                char_val CHAR(10),
                varchar_val VARCHAR(50),
                text_val TEXT,
                blob_val BLOB
            ) AS
            SELECT
                127 as tiny_int,
                32767 as small_int,
                8388607 as medium_int,
                -1000000 as signed_int,
                9223372036854775807 as big_int,
                1000000 as unsigned_int,
                255 as tiny_int_unsigned,
                65535 as small_int_unsigned,
                16777215 as medium_int_unsigned,
                18446744073709551615 as big_int_unsigned,
                123.45 as decimal_num,
                42.25 as float_num,
                42.25 as double_num,
                1 as bit_val,
                '2024-03-14' as date_val,
                '13:14:15' as time_val,
                '2024-03-14 13:14:15' as datetime_val,
                '2024-03-14 13:14:15' as timestamp_val,
                2024 as year_val,
                'CHAR' as char_val,
                'VARCHAR' as varchar_val,
                'TEXT' as text_val,
                x'626c6f62' as blob_val",
        )
        .execute(&mut c)
        .await?;

        let row = sqlx::query("SELECT * FROM _sqlp_t")
            .fetch_one(&mut c)
            .await?;

        expect_json_object_equal(
            &row_to_json(&row),
            &serde_json::json!({
                "tiny_int": 127,
                "small_int": 32767,
                "medium_int": 8_388_607,
                "signed_int": -1_000_000,
                "big_int": 9_223_372_036_854_775_807_u64,
                "unsigned_int": 1_000_000,
                "tiny_int_unsigned": 255,
                "small_int_unsigned": 65_535,
                "medium_int_unsigned": 16_777_215,
                "big_int_unsigned": 18_446_744_073_709_551_615_u64,
                "decimal_num": 123.45,
                "float_num": 42.25,
                "double_num": 42.25,
                "bit_val": true,
                "date_val": "2024-03-14",
                "time_val": "13:14:15",
                "datetime_val": "2024-03-14T13:14:15",
                "timestamp_val": "2024-03-14T13:14:15+00:00",
                "year_val": 2024,
                "char_val": "CHAR",
                "varchar_val": "VARCHAR",
                "text_val": "TEXT",
                "blob_val": "data:application/octet-stream;base64,YmxvYg=="
            }),
        );

        sqlx::query("DROP TABLE _sqlp_t").execute(&mut c).await?;

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
                42.25 as real,
                'xxx' as string,
                x'68656c6c6f20776f726c64' as blob",
        )
        .fetch_one(&mut c)
        .await?;

        expect_json_object_equal(
            &row_to_json(&row),
            &serde_json::json!({
                "integer": 42,
                "real": 42.25,
                "string": "xxx",
                "blob": "data:application/octet-stream;base64,aGVsbG8gd29ybGQ=",
            }),
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
                CAST(255 AS TINYINT) as tiny_int,
                CAST(42 AS SMALLINT) as small_int,
                CAST(42 AS INT) as integer,
                CAST(42 AS BIGINT) as big_int,
                CAST(42.25 AS REAL) as real,
                CAST(42.25 AS FLOAT) as float,
                CAST(42.25 AS DECIMAL(10,2)) as decimal,
                CAST('2024-03-14' AS DATE) as date,
                CAST('13:14:15' AS TIME) as time,
                CAST('2024-03-14 13:14:15' AS DATETIME) as datetime,
                CAST('2024-03-14 13:14:15' AS DATETIME2) as datetime2,
                CAST('2024-03-14 13:14:15 +02:00' AS DATETIMEOFFSET) as datetimeoffset,
                N'Unicode String' as nvarchar,
                'ASCII String' as varchar,
                CAST(1234.56 AS MONEY) as money_val,
                CAST(12.34 AS SMALLMONEY) as small_money_val,
                CAST(0x6D7373716C AS VARBINARY(10)) as blob_data",
        )
        .fetch_one(&mut c)
        .await?;

        expect_json_object_equal(
            &row_to_json(&row),
            &serde_json::json!({
                "true_bit": true,
                "false_bit": false,
                "null_bit": null,
                "tiny_int": 255,
                "small_int": 42,
                "integer": 42,
                "big_int": 42,
                "real": 42.25,
                "float": 42.25,
                "decimal": 42.25,
                "date": "2024-03-14",
                "time": "13:14:15",
                "datetime": "2024-03-14T13:14:15",
                "datetime2": "2024-03-14T13:14:15",
                "datetimeoffset": "2024-03-14T13:14:15+02:00",
                "nvarchar": "Unicode String",
                "varchar": "ASCII String",
                "money_val": 1234.56,
                "small_money_val": 12.34,
                "blob_data": "data:application/octet-stream;base64,bXNzcWw="
            }),
        );
        Ok(())
    }

    #[test]
    fn test_vec_to_data_uri() {
        // Test with empty bytes
        let result = vec_to_data_uri(&[]);
        assert_eq!(result, "data:application/octet-stream;base64,");

        // Test with simple text
        let result = vec_to_data_uri(b"Hello World");
        assert_eq!(
            result,
            "data:application/octet-stream;base64,SGVsbG8gV29ybGQ="
        );

        // Test with binary data
        let binary_data = [0, 1, 2, 255, 254, 253];
        let result = vec_to_data_uri(&binary_data);
        assert_eq!(result, "data:application/octet-stream;base64,AAEC//79");
    }

    #[test]
    fn test_vec_to_data_uri_with_mime() {
        // Test with custom MIME type
        let result = vec_to_data_uri_with_mime(b"Hello", "text/plain");
        assert_eq!(result, "data:text/plain;base64,SGVsbG8=");

        // Test with image MIME type
        let result = vec_to_data_uri_with_mime(&[255, 216, 255], "image/jpeg");
        assert_eq!(result, "data:image/jpeg;base64,/9j/");

        // Test with empty bytes and custom MIME
        let result = vec_to_data_uri_with_mime(&[], "application/json");
        assert_eq!(result, "data:application/json;base64,");
    }

    #[test]
    fn test_vec_to_data_uri_value() {
        // Test that it returns a JSON string value
        let result = vec_to_data_uri_value(b"test");
        match result {
            Value::String(s) => assert_eq!(s, "data:application/octet-stream;base64,dGVzdA=="),
            _ => panic!("Expected String value"),
        }
    }

    fn expect_json_object_equal(actual: &Value, expected: &Value) {
        use std::fmt::Write;

        if actual == expected {
            return;
        }
        let actual = actual.as_object().unwrap();
        let expected = expected.as_object().unwrap();

        let all_keys: std::collections::BTreeSet<_> =
            actual.keys().chain(expected.keys()).collect();
        let max_key_len = all_keys.iter().map(|k| k.len()).max().unwrap_or(0);

        let mut comparison_string = String::new();
        for key in all_keys {
            let actual_value = actual.get(key).unwrap_or(&Value::Null);
            let expected_value = expected.get(key).unwrap_or(&Value::Null);
            if actual_value == expected_value {
                continue;
            }
            writeln!(
                &mut comparison_string,
                "{key:<max_key_len$}  actual  : {actual_value:?}\n{key:max_key_len$}  expected: {expected_value:?}\n"
            )
            .unwrap();
        }
        assert_eq!(
            actual, expected,
            "JSON objects are not equal:\n\n{comparison_string}"
        );
    }
}
