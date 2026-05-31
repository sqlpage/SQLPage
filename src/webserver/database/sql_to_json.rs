use crate::utils::add_value_to_map;
use crate::webserver::database::blob_to_data_url;
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Utc};
use serde_json::{Map, Value};
use sqlx::postgres::types::PgRange;
use sqlx::{Column, ColumnIndex, Row, TypeInfo, ValueRef};

pub trait SqlPageRow {
    fn to_json(&self) -> Value;
    fn first_value_to_string(&self) -> Option<String>;
}

pub fn row_to_json(row: &impl SqlPageRow) -> Value {
    row.to_json()
}

pub fn row_to_string(row: &impl SqlPageRow) -> Option<String> {
    row.first_value_to_string()
}

macro_rules! impl_sqlpage_row {
    ($row:ty, $db:ty, $canonical_odbc_names:expr) => {
        impl SqlPageRow for $row {
            fn to_json(&self) -> Value {
                let mut map = Map::new();
                for col in self.columns() {
                    let key = canonical_col_name(col.name(), $canonical_odbc_names);
                    let value = sql_to_json::<$db, _>(self, col.ordinal());
                    map = add_value_to_map(map, (key, value));
                }
                Value::Object(map)
            }

            fn first_value_to_string(&self) -> Option<String> {
                let col = self.columns().first()?;
                match sql_to_json::<$db, _>(self, col.ordinal()) {
                    Value::String(s) => Some(s),
                    Value::Null => None,
                    other => Some(other.to_string()),
                }
            }
        }
    };
}

impl_sqlpage_row!(sqlx::mysql::MySqlRow, sqlx::MySql, false);
impl_sqlpage_row!(sqlx::sqlite::SqliteRow, sqlx::Sqlite, false);
impl_sqlpage_row!(sqlx_odbc::OdbcRow, sqlx_odbc::Odbc, true);

impl SqlPageRow for sqlx_sqlserver::MssqlRow {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        for col in self.columns() {
            let key = canonical_col_name(col.name(), false);
            let value = mssql_to_json(self, col.ordinal());
            map = add_value_to_map(map, (key, value));
        }
        Value::Object(map)
    }

    fn first_value_to_string(&self) -> Option<String> {
        let col = self.columns().first()?;
        match mssql_to_json(self, col.ordinal()) {
            Value::String(s) => Some(s),
            Value::Null => None,
            other => Some(other.to_string()),
        }
    }
}

impl SqlPageRow for sqlx::postgres::PgRow {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        for col in self.columns() {
            let key = canonical_col_name(col.name(), false);
            let value = pg_to_json(self, col.ordinal());
            map = add_value_to_map(map, (key, value));
        }
        Value::Object(map)
    }

    fn first_value_to_string(&self) -> Option<String> {
        let col = self.columns().first()?;
        match pg_to_json(self, col.ordinal()) {
            Value::String(s) => Some(s),
            Value::Null => None,
            other => Some(other.to_string()),
        }
    }
}

fn canonical_col_name(name: &str, canonicalize_uppercase: bool) -> String {
    if canonicalize_uppercase
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    }
}

fn sql_to_json<DB, R>(row: &R, ordinal: usize) -> Value
where
    DB: sqlx::Database,
    R: Row<Database = DB>,
    for<'r> bool: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> i16: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> i32: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> i64: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> f32: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> f64: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> String: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    for<'r> Vec<u8>: sqlx::Decode<'r, DB> + sqlx::Type<DB>,
    usize: ColumnIndex<R>,
{
    let raw_value = match row.try_get_raw(ordinal) {
        Ok(raw_value) if raw_value.is_null() => return Value::Null,
        Ok(raw_value) => raw_value,
        Err(e) => {
            log::warn!("Unable to extract value from row: {e:?}");
            return Value::Null;
        }
    };

    let type_info = raw_value.type_info();
    let type_name = type_info.name().to_ascii_uppercase();
    log::trace!("Decoding a value of type {type_name:?} (type info: {type_info:?})");

    match type_name.as_str() {
        "BOOL" | "BOOLEAN" | "BIT" => decode::<DB, R, bool>(row, ordinal).into(),
        "INT2" | "SMALLINT" | "TINYINT" => decode::<DB, R, i16>(row, ordinal).into(),
        "INT" | "INT4" | "INTEGER" | "MEDIUMINT" | "YEAR" => {
            decode::<DB, R, i32>(row, ordinal).into()
        }
        "INT8" | "BIGINT" | "SERIAL8" | "BIGSERIAL" | "IDENTITY" | "INT64" | "INTEGER8" => {
            decode::<DB, R, i64>(row, ordinal).into()
        }
        "REAL" | "FLOAT4" => decode::<DB, R, f32>(row, ordinal).into(),
        "FLOAT" | "FLOAT8" | "DOUBLE" => decode::<DB, R, f64>(row, ordinal).into(),
        "BLOB" | "BYTEA" | "FILESTREAM" | "VARBINARY" | "BIGVARBINARY" | "BINARY" | "IMAGE" => {
            blob_to_data_url::vec_to_data_uri_value(&decode::<DB, R, Vec<u8>>(row, ordinal))
        }
        _ => decode::<DB, R, String>(row, ordinal).into(),
    }
}

fn pg_to_json(row: &sqlx::postgres::PgRow, ordinal: usize) -> Value {
    let raw_value = match row.try_get_raw(ordinal) {
        Ok(raw_value) if raw_value.is_null() => return Value::Null,
        Ok(raw_value) => raw_value,
        Err(e) => {
            log::warn!("Unable to extract value from row: {e:?}");
            return Value::Null;
        }
    };
    let type_info = raw_value.type_info();
    let type_name = type_info.name().to_ascii_uppercase();
    log::trace!("Decoding a PostgreSQL value of type {type_name:?} (type info: {type_info:?})");

    match type_name.as_str() {
        "BOOL" | "BOOLEAN" => decode::<sqlx::Postgres, _, bool>(row, ordinal).into(),
        "INT2" | "SMALLINT" => decode::<sqlx::Postgres, _, i16>(row, ordinal).into(),
        "INT" | "INT4" | "INTEGER" => decode::<sqlx::Postgres, _, i32>(row, ordinal).into(),
        "INT8" | "BIGINT" | "SERIAL8" | "BIGSERIAL" => {
            decode::<sqlx::Postgres, _, i64>(row, ordinal).into()
        }
        "REAL" | "FLOAT4" => decode::<sqlx::Postgres, _, f32>(row, ordinal).into(),
        "FLOAT" | "FLOAT8" | "DOUBLE" => decode::<sqlx::Postgres, _, f64>(row, ordinal).into(),
        "NUMERIC" | "DECIMAL" => {
            decimal_to_json(&decode::<sqlx::Postgres, _, BigDecimal>(row, ordinal))
        }
        "DATE" => decode::<sqlx::Postgres, _, NaiveDate>(row, ordinal)
            .to_string()
            .into(),
        "TIME" | "TIMETZ" => decode::<sqlx::Postgres, _, chrono::NaiveTime>(row, ordinal)
            .to_string()
            .into(),
        "TIMESTAMP" => decode::<sqlx::Postgres, _, NaiveDateTime>(row, ordinal)
            .format("%FT%T%.f")
            .to_string()
            .into(),
        "TIMESTAMPTZ" => decode::<sqlx::Postgres, _, DateTime<Utc>>(row, ordinal)
            .to_rfc3339()
            .into(),
        "JSON" | "JSON[]" | "JSONB" | "JSONB[]" => decode::<sqlx::Postgres, _, Value>(row, ordinal),
        "BYTEA" => blob_to_data_url::vec_to_data_uri_value(&decode::<sqlx::Postgres, _, Vec<u8>>(
            row, ordinal,
        )),
        "UUID" => decode::<sqlx::Postgres, _, sqlx::types::uuid::Uuid>(row, ordinal)
            .to_string()
            .into(),
        "INT4RANGE" => decode_pg_range::<i32>(row, ordinal),
        "INT8RANGE" => decode_pg_range::<i64>(row, ordinal),
        "NUMRANGE" => decode_pg_range::<BigDecimal>(row, ordinal),
        "DATERANGE" => decode_pg_range::<NaiveDate>(row, ordinal),
        "TSRANGE" => decode_pg_range::<NaiveDateTime>(row, ordinal),
        "TSTZRANGE" => decode_pg_range::<DateTime<Utc>>(row, ordinal),
        _ => decode::<sqlx::Postgres, _, String>(row, ordinal).into(),
    }
}

fn mssql_to_json(row: &sqlx_sqlserver::MssqlRow, ordinal: usize) -> Value {
    let raw_value = match row.try_get_raw(ordinal) {
        Ok(raw_value) if raw_value.is_null() => return Value::Null,
        Ok(raw_value) => raw_value,
        Err(e) => {
            log::warn!("Unable to extract value from row: {e:?}");
            return Value::Null;
        }
    };
    let type_info = raw_value.type_info();
    let type_name = type_info.name().to_ascii_uppercase();
    log::trace!("Decoding a SQL Server value of type {type_name:?} (type info: {type_info:?})");

    match type_name.as_str() {
        "BIT" => decode::<sqlx_sqlserver::Mssql, _, bool>(row, ordinal).into(),
        "SMALLINT" | "TINYINT" => decode::<sqlx_sqlserver::Mssql, _, i16>(row, ordinal).into(),
        "INT" => decode::<sqlx_sqlserver::Mssql, _, i32>(row, ordinal).into(),
        "BIGINT" => decode::<sqlx_sqlserver::Mssql, _, i64>(row, ordinal).into(),
        "REAL" => decode::<sqlx_sqlserver::Mssql, _, f32>(row, ordinal).into(),
        "FLOAT" => decode::<sqlx_sqlserver::Mssql, _, f64>(row, ordinal).into(),
        "DECIMAL" | "NUMERIC" | "MONEY" | "SMALLMONEY" => {
            decimal_to_json(&decode::<sqlx_sqlserver::Mssql, _, BigDecimal>(
                row, ordinal,
            ))
        }
        "DATE" => decode::<sqlx_sqlserver::Mssql, _, NaiveDate>(row, ordinal)
            .to_string()
            .into(),
        "TIME" => decode::<sqlx_sqlserver::Mssql, _, chrono::NaiveTime>(row, ordinal)
            .to_string()
            .into(),
        "DATETIME2" => decode::<sqlx_sqlserver::Mssql, _, NaiveDateTime>(row, ordinal)
            .format("%FT%T%.f")
            .to_string()
            .into(),
        "DATETIME" | "SMALLDATETIME" => {
            decode::<sqlx_sqlserver::Mssql, _, DateTime<FixedOffset>>(row, ordinal)
                .naive_local()
                .format("%FT%T%.f")
                .to_string()
                .into()
        }
        "DATETIMEOFFSET" => decode::<sqlx_sqlserver::Mssql, _, DateTime<FixedOffset>>(row, ordinal)
            .to_rfc3339()
            .into(),
        "UNIQUEIDENTIFIER" => {
            decode::<sqlx_sqlserver::Mssql, _, sqlx::types::uuid::Uuid>(row, ordinal)
                .to_string()
                .into()
        }
        "FILESTREAM" | "VARBINARY" | "BIGVARBINARY" | "BINARY" | "IMAGE" => {
            blob_to_data_url::vec_to_data_uri_value(&decode::<sqlx_sqlserver::Mssql, _, Vec<u8>>(
                row, ordinal,
            ))
        }
        _ => decode::<sqlx_sqlserver::Mssql, _, String>(row, ordinal).into(),
    }
}

fn decimal_to_json(decimal: &BigDecimal) -> Value {
    Value::Number(serde_json::Number::from_string_unchecked(
        decimal.normalized().to_plain_string(),
    ))
}

fn decode_pg_range<T>(row: &sqlx::postgres::PgRow, ordinal: usize) -> Value
where
    T: std::fmt::Display + sqlx::Type<sqlx::Postgres>,
    for<'r> T: sqlx::Decode<'r, sqlx::Postgres>,
    PgRange<T>: sqlx::Type<sqlx::Postgres>,
    for<'r> PgRange<T>: sqlx::Decode<'r, sqlx::Postgres>,
{
    match row.try_get::<PgRange<T>, _>(ordinal) {
        Ok(pg_range) => pg_range.to_string().into(),
        Err(e) => {
            log::error!("Failed to decode postgres range value: {e}");
            Value::Null
        }
    }
}

fn decode<DB, R, T>(row: &R, ordinal: usize) -> T
where
    DB: sqlx::Database,
    R: Row<Database = DB>,
    for<'r> T: sqlx::Decode<'r, DB> + sqlx::Type<DB> + Default,
    usize: ColumnIndex<R>,
{
    match row.try_get::<T, _>(ordinal) {
        Ok(v) => v,
        Err(e) => {
            let type_name = std::any::type_name::<T>();
            log::error!("Failed to decode {type_name} value: {e}");
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;

    #[actix_web::test]
    async fn test_sqlite_row_to_json() -> anyhow::Result<()> {
        let mut c = sqlx::SqliteConnection::connect(":memory:").await?;
        let row = sqlx::query(
            "SELECT
                42 as integer,
                42.25 as real,
                'xxx' as string,
                x'68656c6c6f20776f726c64' as blob",
        )
        .fetch_one(&mut c)
        .await?;

        assert_eq!(
            row_to_json(&row),
            serde_json::json!({
                "integer": 42,
                "real": 42.25,
                "string": "xxx",
                "blob": "data:application/octet-stream;base64,aGVsbG8gd29ybGQ=",
            }),
        );
        Ok(())
    }
}
