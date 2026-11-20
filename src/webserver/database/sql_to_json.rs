//! Decodes physical driver values and separates generated inputs from public columns.

use super::execute_queries::QueryRow;
use crate::utils::add_value_to_map;
use crate::webserver::database::blob_to_data_url::vec_to_data_uri_value;
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::{Map, Value};
use sqlx::mysql::{MySqlRow, MySqlTypeInfo, MySqlValueRef};
use sqlx::postgres::types::{PgInterval, PgRange, PgTimeTz};
use sqlx::postgres::{PgRow, PgTypeInfo, PgValueFormat, PgValueRef};
use sqlx::sqlite::{SqliteRow, SqliteTypeInfo, SqliteValueRef};
use sqlx::types::{Json, Uuid};
use sqlx::{Column, ColumnIndex, Database, Decode, Postgres, Row, Type, TypeInfo, ValueRef};
use sqlx_odbc::{Odbc, OdbcRow, OdbcTypeInfo};
use sqlx_sqlserver::{MssqlRow, MssqlType, MssqlTypeInfo, MssqlValueRef};

pub(super) trait DecodeRow: Row {
    fn decode_value(
        value: <Self::Database as Database>::ValueRef<'_>,
        declared_type: &<Self::Database as Database>::TypeInfo,
    ) -> Value;

    fn column_name(name: &str) -> String {
        name.to_owned()
    }

    /// Private inputs occupy the last columns, even when a public column has the same name.
    fn query_row(&self, json_inputs: &[bool]) -> Result<QueryRow, sqlx::Error>
    where
        usize: ColumnIndex<Self>,
    {
        let public_column_count = self
            .len()
            .checked_sub(json_inputs.len())
            .ok_or_else(|| sqlx::Error::Protocol("missing internal columns".into()))?;
        let mut columns = self.columns().iter().map(|column| {
            let value = match self.try_get_raw(column.ordinal()) {
                Ok(raw) if !raw.is_null() => Self::decode_value(raw, column.type_info()),
                Ok(_) => Value::Null,
                Err(err) => {
                    log::warn!("Unable to extract value from row: {err:?}");
                    Value::Null
                }
            };
            log::trace!(
                "Decoded {:?} column {:?}: {value:?}",
                column.type_info(),
                column.name()
            );
            (column.name(), value)
        });
        let public = columns
            .by_ref()
            .take(public_column_count)
            .fold(Map::new(), |map, (name, value)| {
                add_value_to_map(map, (Self::column_name(name), value))
            });
        let inputs = columns
            .zip(json_inputs)
            .map(|((_, value), &is_json)| {
                if is_json
                    && let Value::String(text) = &value
                    && let Ok(json) = serde_json::from_str(text)
                {
                    return json;
                }
                value
            })
            .collect();
        Ok(QueryRow {
            value: Value::Object(public),
            inputs,
            public_column_count,
        })
    }
}

trait DecodeValue<'r>: ValueRef<'r> {
    fn decode_as<T: Decode<'r, Self::Database> + Default>(self) -> T
    where
        Self::Database: Database<ValueRef<'r> = Self>,
    {
        T::decode(self).unwrap_or_else(|err| {
            log::error!(
                "Failed to decode {} value: {err}",
                std::any::type_name::<T>()
            );
            T::default()
        })
    }
}

impl<'r, V: ValueRef<'r>> DecodeValue<'r> for V {}

fn decimal_to_json(decimal: &BigDecimal) -> Value {
    Value::Number(serde_json::Number::from_string_unchecked(
        decimal.normalized().to_plain_string(),
    ))
}

fn naive_datetime(value: NaiveDateTime) -> Value {
    value.format("%FT%T%.f").to_string().into()
}

fn decode_pg_range<T>(value: PgValueRef<'_>) -> Value
where
    T: std::fmt::Display + Type<Postgres> + for<'r> Decode<'r, Postgres>,
{
    match PgRange::<T>::decode(value) {
        Ok(range) => range.to_string().into(),
        Err(err) => {
            log::error!("Failed to decode postgres range value: {err}");
            Value::Null
        }
    }
}

fn postgres_interval(value: PgValueRef<'_>) -> Value {
    if value.format() == PgValueFormat::Text {
        return value.decode_as::<String>().into();
    }
    let Ok(interval) = PgInterval::decode(value) else {
        return Value::Null;
    };
    let years = interval.months / 12;
    let months = interval.months % 12;
    let mut parts = Vec::new();
    for (count, singular, plural) in [
        (years, "year", "years"),
        (months, "mon", "mons"),
        (interval.days, "day", "days"),
    ] {
        if count != 0 {
            let unit = if count.unsigned_abs() == 1 {
                singular
            } else {
                plural
            };
            parts.push(format!("{count} {unit}"));
        }
    }
    if interval.microseconds != 0 || parts.is_empty() {
        let sign = if interval.microseconds < 0 { "-" } else { "" };
        let micros = interval.microseconds.unsigned_abs();
        let seconds = micros / 1_000_000;
        let mut time = format!(
            "{sign}{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        );
        if micros % 1_000_000 != 0 {
            let fraction = format!(".{:06}", micros % 1_000_000);
            time.push_str(fraction.trim_end_matches('0'));
        }
        parts.push(time);
    }
    parts.join(" ").into()
}

impl DecodeRow for PgRow {
    fn decode_value(value: PgValueRef<'_>, declared_type: &PgTypeInfo) -> Value {
        match declared_type.name() {
            "INT2" => value.decode_as::<i16>().into(),
            "INT4" => value.decode_as::<i32>().into(),
            "INT8" => value.decode_as::<i64>().into(),
            "FLOAT4" => f64::from(value.decode_as::<f32>()).into(),
            "FLOAT8" => value.decode_as::<f64>().into(),
            "NUMERIC" => decimal_to_json(&value.decode_as::<BigDecimal>()),
            "BOOL" => value.decode_as::<bool>().into(),
            "DATE" => value.decode_as::<NaiveDate>().to_string().into(),
            "TIME" => value.decode_as::<NaiveTime>().to_string().into(),
            "TIMETZ" => match PgTimeTz::<NaiveTime, FixedOffset>::decode(value) {
                Ok(time) => time.time.to_string().into(),
                Err(err) => {
                    log::error!("Failed to decode postgres time with time zone: {err}");
                    NaiveTime::default().to_string().into()
                }
            },
            "TIMESTAMP" => value
                .decode_as::<NaiveDateTime>()
                .and_utc()
                .to_rfc3339()
                .into(),
            "TIMESTAMPTZ" => value.decode_as::<DateTime<Utc>>().to_rfc3339().into(),
            "INTERVAL" => postgres_interval(value),
            "UUID" => value.decode_as::<Uuid>().to_string().into(),
            "JSON" | "JSONB" => value.decode_as::<Json<Value>>().0,
            "JSON[]" | "JSONB[]" => Value::Array(
                value
                    .decode_as::<Vec<Json<Value>>>()
                    .into_iter()
                    .map(|json| json.0)
                    .collect(),
            ),
            "BYTEA" => vec_to_data_uri_value(&value.decode_as::<Vec<u8>>()),
            "INT4RANGE" => decode_pg_range::<i32>(value),
            "INT8RANGE" => decode_pg_range::<i64>(value),
            "NUMRANGE" => decode_pg_range::<BigDecimal>(value),
            "DATERANGE" => decode_pg_range::<NaiveDate>(value),
            "TSRANGE" => decode_pg_range::<NaiveDateTime>(value),
            "TSTZRANGE" => decode_pg_range::<DateTime<FixedOffset>>(value),
            _ => value.decode_as::<String>().into(),
        }
    }
}

impl DecodeRow for MySqlRow {
    fn decode_value(value: MySqlValueRef<'_>, declared_type: &MySqlTypeInfo) -> Value {
        match declared_type.name() {
            "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" | "YEAR" => {
                value.decode_as::<i64>().into()
            }
            "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "MEDIUMINT UNSIGNED" | "INT UNSIGNED"
            | "BIGINT UNSIGNED" => value.decode_as::<u64>().into(),
            "FLOAT" => f64::from(value.decode_as::<f32>()).into(),
            "DOUBLE" => value.decode_as::<f64>().into(),
            "DECIMAL" => decimal_to_json(&value.decode_as::<BigDecimal>()),
            "BOOLEAN" => value.decode_as::<bool>().into(),
            // SQLx 0.9 does not expose BIT(M)'s declared width. The wire value
            // includes leading zero bytes for widths above 8.
            "BIT" => match value.clone().decode_as::<&[u8]>() {
                [0] => false.into(),
                [1] => true.into(),
                _ => value.decode_as::<u64>().into(),
            },
            "DATE" => value.decode_as::<NaiveDate>().to_string().into(),
            "TIME" => value.decode_as::<NaiveTime>().to_string().into(),
            "DATETIME" => naive_datetime(value.decode_as::<NaiveDateTime>()),
            "TIMESTAMP" => value.decode_as::<DateTime<Utc>>().to_rfc3339().into(),
            "JSON" => value.decode_as::<Json<Value>>().0,
            "BINARY" | "VARBINARY" | "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" => {
                vec_to_data_uri_value(&value.decode_as::<Vec<u8>>())
            }
            _ => value.decode_as::<String>().into(),
        }
    }
}

impl DecodeRow for SqliteRow {
    fn decode_value(value: SqliteValueRef<'_>, _: &SqliteTypeInfo) -> Value {
        let info = value.type_info().into_owned();
        match info.name() {
            "INTEGER" => value.decode_as::<i64>().into(),
            "REAL" => value.decode_as::<f64>().into(),
            "BLOB" => vec_to_data_uri_value(&value.decode_as::<Vec<u8>>()),
            "BOOLEAN" => value.decode_as::<bool>().into(),
            "DATE" => value.decode_as::<NaiveDate>().to_string().into(),
            "TIME" => value.decode_as::<NaiveTime>().to_string().into(),
            "DATETIME" => naive_datetime(value.decode_as::<NaiveDateTime>()),
            _ => value.decode_as::<String>().into(),
        }
    }
}

impl DecodeRow for MssqlRow {
    fn decode_value(value: MssqlValueRef<'_>, _: &MssqlTypeInfo) -> Value {
        match value.type_info().kind() {
            MssqlType::Null => Value::Null,
            MssqlType::Real => f64::from(value.decode_as::<f32>()).into(),
            MssqlType::Float | MssqlType::Money => value.decode_as::<f64>().into(),
            MssqlType::Decimal => decimal_to_json(&value.decode_as::<BigDecimal>()),
            MssqlType::BigInt => value.decode_as::<i64>().into(),
            MssqlType::Int => value.decode_as::<i32>().into(),
            MssqlType::SmallInt => value.decode_as::<i16>().into(),
            MssqlType::TinyInt => value.decode_as::<u8>().into(),
            MssqlType::Bit => value.decode_as::<bool>().into(),
            MssqlType::Date => value.decode_as::<NaiveDate>().to_string().into(),
            MssqlType::Time => value.decode_as::<NaiveTime>().to_string().into(),
            MssqlType::DateTimeOffset => value
                .decode_as::<DateTime<FixedOffset>>()
                .to_rfc3339()
                .into(),
            MssqlType::DateTime | MssqlType::DateTime2 => {
                naive_datetime(value.decode_as::<NaiveDateTime>())
            }
            MssqlType::UniqueIdentifier => value.decode_as::<Uuid>().to_string().into(),
            MssqlType::VarBinary => vec_to_data_uri_value(&value.decode_as::<Vec<u8>>()),
            _ => value.decode_as::<String>().into(),
        }
    }
}

impl DecodeRow for OdbcRow {
    fn decode_value(
        value: <Odbc as Database>::ValueRef<'_>,
        declared_type: &OdbcTypeInfo,
    ) -> Value {
        // The driver transports decimal values as text. Use the declared SQL type.
        match declared_type.name() {
            "REAL" | "FLOAT" | "DOUBLE" => value.decode_as::<f64>().into(),
            "NUMERIC" | "DECIMAL" => decimal_to_json(&value.decode_as::<BigDecimal>()),
            "BIGINT" => value.decode_as::<i64>().into(),
            "INTEGER" => value.decode_as::<i32>().into(),
            "SMALLINT" | "TINYINT" => value.decode_as::<i16>().into(),
            "BIT" => value.decode_as::<bool>().into(),
            "DATE" => value.decode_as::<NaiveDate>().to_string().into(),
            "TIME" => value.decode_as::<NaiveTime>().to_string().into(),
            "TIMESTAMP" => value
                .decode_as::<DateTime<FixedOffset>>()
                .to_rfc3339()
                .into(),
            "VARBINARY" | "BINARY" | "LONGVARBINARY" => {
                vec_to_data_uri_value(&value.decode_as::<Vec<u8>>())
            }
            _ => value.decode_as::<String>().into(),
        }
    }

    fn column_name(name: &str) -> String {
        // Some ODBC drivers fold unquoted identifiers to uppercase.
        if name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            name.to_ascii_lowercase()
        } else {
            name.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app_config::tests::test_database_url;

    use super::*;
    use crate::webserver::database::execute_queries::QueryEvent;
    use crate::webserver::database::{Database as AppDatabase, DbInfo, SupportedDatabase};
    use futures_util::{Stream, TryStreamExt};

    async fn connect(url: String) -> anyhow::Result<AppDatabase> {
        let mut config = crate::app_config::tests::test_config();
        config.database_url = url;
        config.configuration_directory = tempfile::TempDir::new()?.path().to_owned();
        AppDatabase::init(&config).await
    }

    async fn first_row(
        mut stream: impl Stream<Item = Result<QueryEvent, sqlx::Error>> + Unpin,
    ) -> anyhow::Result<QueryRow> {
        while let Some(event) = stream.try_next().await? {
            if let QueryEvent::Row(row) = event {
                return Ok(row);
            }
        }
        Err(sqlx::Error::RowNotFound.into())
    }

    fn setup_logging() {
        crate::telemetry::init_test_logging();
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
        let db_url = test_database_url();
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
            "SELECT
                123.456 as one_value,
                1 as two_values,
                2 as two_values,
                'x' as three_values,
                'y' as three_values,
                'z' as three_values
        ",
            &[],
            &[],
        ))
        .await?;
        expect_json_object_equal(
            &row.value,
            &serde_json::json!({
                "one_value": 123.456,
                "two_values": [1,2],
                "three_values": ["x","y","z"],
            }),
        );
        Ok(())
    }

    #[actix_web::test]
    async fn private_inputs_are_split_from_the_trailing_suffix() -> anyhow::Result<()> {
        let db_url = test_database_url();
        let db = connect(db_url).await?;
        let mut connection = db.connection.acquire().await?;
        let row = first_row(connection.fetch_many(
            "SELECT 'public' AS \"__sqlpage_input_0\", 'private' AS \"__sqlpage_input_0\",
                    '[1]' AS json_input, '[2]' AS text_input",
            &[],
            &[false, true, false],
        ))
        .await?;
        expect_json_object_equal(
            &row.value,
            &serde_json::json!({ "__sqlpage_input_0": "public" }),
        );
        assert_eq!(row.public_column_count, 1);
        assert_eq!(
            row.inputs,
            [
                serde_json::json!("private"),
                serde_json::json!([1]),
                serde_json::json!("[2]")
            ]
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_postgres_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("postgres") else {
            return Ok(());
        };
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
            "SELECT
                42::INT2 as small_int,
                42::INT4 as integer,
                42::INT8 as big_int,
                42.25::FLOAT4 as float4,
                42.25::FLOAT8 as float8,
                123456789123456789123456789::NUMERIC as numeric,
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
                '\\x68656c6c6f20776f726c64'::BYTEA as blob_data,
                '550e8400-e29b-41d4-a716-446655440000'::UUID as uuid,
                '[1,5)'::INT4RANGE as int4range,
                '[1,5]'::INT8RANGE as int8range,
                '[1.5,4.5)'::NUMRANGE as numrange,
                -- '[2024-11-12 01:02:03,2024-11-12 23:00:00)'::TSRANGE as tsrange,
                -- '[2024-11-12 01:02:03+01:00,2024-11-12 23:00:00+00:00)'::TSTZRANGE as tstzrange,
                '[2024-11-12,2024-11-13)'::DATERANGE as daterange
            ",
            &[],
            &[],
        ))
        .await?;

        expect_json_object_equal(
            &row.value,
            &serde_json::json!({
                "small_int": 42,
                "integer": 42,
                "big_int": 42,
                "float4": 42.25,
                "float8": 42.25,
                "numeric": 123_456_789_123_456_789_123_456_789_u128,
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
                "blob_data": "data:application/octet-stream;base64,aGVsbG8gd29ybGQ=",
                "uuid": "550e8400-e29b-41d4-a716-446655440000",
                "int4range": "[1,5)",
                "int8range": "[1,6)",
                "numrange": "[1.5,4.5)",
                //"tsrange": "[2024-11-12 01:02:03,2024-11-12 23:00:00)", // todo: bug in sqlx datetime range parsing
                //"tstzrange": "[\"2024-11-12 02:00:00 +01:00\",\"2024-11-12 23:00:00 +00:00\")", // todo: tz info is lost in sqlx
                "daterange": "[2024-11-12,2024-11-13)"
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
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
            "SELECT
                '2024-03-14'::DATE as date,
                '13:14:15'::TIME as time,
                '2024-03-14 13:14:15+02:00'::TIMESTAMPTZ as timestamptz,
                INTERVAL '-01:02:03' as time_interval,
                '{\"key\": \"value\"}'::JSON as json,
                1234.56::MONEY as money_val,
                '\\x74657374'::BYTEA as blob_data,
                '550e8400-e29b-41d4-a716-446655440000'::UUID as uuid
            where CAST($1 AS BOOLEAN)",
            &[Some("true".into())],
            &[],
        ))
        .await?;

        expect_json_object_equal(
            &row.value,
            &serde_json::json!({
                "date": "2024-03-14",
                "time": "13:14:15",
                "timestamptz": "2024-03-14T11:14:15+00:00",
                "time_interval": "-01:02:03",
                "json": {"key": "value"},
                "money_val": "", // TODO: fix this bug: https://github.com/sqlpage/SQLPage/issues/983
                "blob_data": "data:application/octet-stream;base64,dGVzdA==",
                "uuid": "550e8400-e29b-41d4-a716-446655440000",
            }),
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_postgres_prepared_range_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("postgres") else {
            return Ok(());
        };
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
            "SELECT
                '[1,5)'::INT4RANGE as int4range,
                '[2024-11-12 01:02:03,2024-11-12 23:00:00)'::TSRANGE as tsrange,
                '[2024-11-12 01:02:03+01:00,2024-11-12 23:00:00+00:00)'::TSTZRANGE as tstzrange,
                '[2024-11-12,2024-11-13)'::DATERANGE as daterange
            where CAST($1 AS BOOLEAN)",
            &[Some("true".into())],
            &[],
        ))
        .await?;

        expect_json_object_equal(
            &row.value,
            &serde_json::json!({
                "int4range": "[1,5)",
                "tsrange": "[2024-11-12 01:02:03,2024-11-12 23:00:00)",
                "tstzrange": "[2024-11-12 00:02:03 +00:00,2024-11-12 23:00:00 +00:00)", // todo: tz info is lost in sqlx
                "daterange": "[2024-11-12,2024-11-13)"
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
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;

        c.execute(
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
                bit_false BIT(1),
                bit_9 BIT(9),
                bit_64 BIT(64),
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
                0 as bit_false,
                1 as bit_9,
                0 as bit_64,
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
        .await?;

        let row = first_row(c.fetch_many("SELECT * FROM _sqlp_t", &[], &[])).await?;

        expect_json_object_equal(
            &row.value,
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
                "bit_false": false,
                "bit_9": 1,
                "bit_64": 0,
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

        c.execute("DROP TABLE _sqlp_t").await?;

        Ok(())
    }

    #[actix_web::test]
    async fn test_sqlite_types() -> anyhow::Result<()> {
        let Some(db_url) = db_specific_test("sqlite") else {
            return Ok(());
        };
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
            "SELECT
                42 as integer,
                42.25 as real,
                'xxx' as string,
                x'68656c6c6f20776f726c64' as blob",
            &[],
            &[],
        ))
        .await?;

        expect_json_object_equal(
            &row.value,
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
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;
        let row = first_row(c.fetch_many(
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
                CAST(0x6D7373716C AS VARBINARY(10)) as blob_data,
                CONVERT(UNIQUEIDENTIFIER, '6F9619FF-8B86-D011-B42D-00C04FC964FF') as unique_identifier
            ", &[], &[]))
        .await?;

        expect_json_object_equal(
            &row.value,
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
                "blob_data": "data:application/octet-stream;base64,bXNzcWw=",
                "unique_identifier": "6f9619ff-8b86-d011-b42d-00c04fc964ff"
            }),
        );
        Ok(())
    }

    fn expect_json_object_equal(actual: &Value, expected: &Value) {
        use std::fmt::Write;

        if json_values_equal(actual, expected) {
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
            if json_values_equal(actual_value, expected_value) {
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

    #[actix_web::test]
    async fn test_canonical_col_name_variations() -> anyhow::Result<()> {
        let db_url = test_database_url();
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;

        // Test various column name formats to ensure canonical_col_name works correctly
        let row = first_row(c.fetch_many(
            r#"SELECT
                42 as "UPPERCASE_COL",
                42 as "lowercase_col",
                42 as "Mixed_Case_Col",
                42 as "COL_WITH_123_NUMBERS",
                42 as "col-with-dashes",
                42 as "col with spaces",
                42 as "_UNDERSCORE_PREFIX",
                42 as "123_NUMBER_PREFIX"
        "#,
            &[],
            &[],
        ))
        .await?;

        let json_result = row.value;

        // For ODBC databases, uppercase columns should be converted to lowercase
        // For other databases, names should remain as-is
        let expected_json = if matches!(db.info, DbInfo::Odbc(_)) {
            // ODBC database - uppercase should be converted to lowercase
            serde_json::json!({
                "uppercase_col": 42,
                "lowercase_col": 42,
                "Mixed_Case_Col": 42,
                "COL_WITH_123_NUMBERS": 42,
                "col-with-dashes": 42,
                "col with spaces": 42,
                "_underscore_prefix": 42,
                "123_NUMBER_PREFIX": 42
            })
        } else {
            // Non-ODBC database - names remain as-is
            serde_json::json!({
                "UPPERCASE_COL": 42,
                "lowercase_col": 42,
                "Mixed_Case_Col": 42,
                "COL_WITH_123_NUMBERS": 42,
                "col-with-dashes": 42,
                "col with spaces": 42,
                "_UNDERSCORE_PREFIX": 42,
                "123_NUMBER_PREFIX": 42
            })
        };

        expect_json_object_equal(&json_result, &expected_json);

        Ok(())
    }

    #[actix_web::test]
    async fn test_row_to_json_edge_cases() -> anyhow::Result<()> {
        let db_url = test_database_url();
        let db = connect(db_url).await?;
        let mut c = db.connection.acquire().await?;

        // Test edge cases for row_to_json
        let row = first_row(c.fetch_many(
            "SELECT
                NULL as null_col,
                '' as empty_string,
                0 as zero_value,
                -42 as negative_int,
                1.23456 as my_float,
                'special_chars_!@#$%^&*()' as special_chars,
                'line1
line2' as multiline_string
        ",
            &[],
            &[],
        ))
        .await?;

        let json_result = row.value;

        // For Oracle databases, empty string is treated as NULL.
        let empty_str_is_null = db.info.database_type() == SupportedDatabase::Oracle;

        let expected_json = serde_json::json!({
            "null_col": null,
            "empty_string": if empty_str_is_null { Value::Null } else { Value::String(String::new()) },
            "zero_value": 0,
            "negative_int": -42,
            "my_float": 1.23456,
            "special_chars": "special_chars_!@#$%^&*()",
            "multiline_string": "line1\nline2"
        });

        expect_json_object_equal(&json_result, &expected_json);

        Ok(())
    }

    /// Compare JSON values, treating integers and floats that are numerically equal as equal
    fn json_values_equal(a: &Value, b: &Value) -> bool {
        use Value::*;

        match (a, b) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Number(a), Number(b)) => {
                // Treat integers and floats as equal if they represent the same numerical value
                a.as_f64() == b.as_f64()
            }
            (String(a), String(b)) => a == b,
            (Array(a), Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| json_values_equal(a, b))
            }
            (Object(a), Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().all(|(key, value)| {
                    b.get(key)
                        .is_some_and(|expected_value| json_values_equal(value, expected_value))
                })
            }
            _ => false,
        }
    }
}
