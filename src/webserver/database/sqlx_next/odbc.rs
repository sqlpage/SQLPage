use std::time::Duration;

use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use serde_json::Value;
use sqlx::any::AnyKind;
use sqlx_next::{Column, ConnectOptions, Row, TypeInfo, ValueRef};
use sqlx_odbc::{Odbc, OdbcColumn, OdbcConnectOptions, OdbcRow};

use super::{Pool, decode_or_default, pool_options};
use crate::app_config::AppConfig;
use crate::webserver::database::blob_to_data_url::vec_to_data_uri_value;
use crate::webserver::database::connect::retry_connecting;
use crate::webserver::database::sql_to_json::{decimal_to_json, odbc_column_name};

pub(super) async fn connect(config: &AppConfig) -> anyhow::Result<(Pool, String)> {
    let mut options: OdbcConnectOptions = config.database_url.parse()?;
    options
        .batch_size(config.max_pending_rows.clamp(1, 1024))
        .max_column_size(None)
        .log_statements(log::LevelFilter::Trace)
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(250));
    if config.database_password.is_some() {
        log::warn!(
            "Setting a password for an ODBC connection is not supported via separate config; include credentials in the DSN or connection string"
        );
    }
    let dbms_name = retry_connecting(config, || options.connect())
        .await?
        .dbms_name()?;
    let pool = pool_options::<Odbc>(config, AnyKind::Odbc)
        .connect_with(options)
        .await
        .with_context(|| format!("Unable to open connection pool to {}", config.database_url))?;
    Ok((Pool::Odbc(pool), dbms_name))
}

pub(super) fn decode_row(row: &OdbcRow) -> Vec<(String, Value)> {
    row.columns()
        .iter()
        .map(|column| (odbc_column_name(column.name()), column_to_json(row, column)))
        .collect()
}

fn column_to_json(row: &OdbcRow, column: &OdbcColumn) -> Value {
    let value = match row.try_get_raw(column.ordinal()) {
        Ok(value) if value.is_null() => return Value::Null,
        Ok(value) => value,
        Err(error) => {
            log::warn!("Unable to extract value from row: {error:?}");
            return Value::Null;
        }
    };
    match column.type_info().name() {
        "REAL" | "FLOAT" | "DOUBLE" => decode_or_default::<Odbc, f64>(value).into(),
        "NUMERIC" | "DECIMAL" => decimal_to_json(&decode_or_default::<Odbc, BigDecimal>(value)),
        "BIGINT" => decode_or_default::<Odbc, i64>(value).into(),
        "INTEGER" => decode_or_default::<Odbc, i32>(value).into(),
        "SMALLINT" | "TINYINT" => decode_or_default::<Odbc, i16>(value).into(),
        "BIT" => decode_or_default::<Odbc, bool>(value).into(),
        "DATE" => decode_or_default::<Odbc, NaiveDate>(value)
            .to_string()
            .into(),
        "TIME" => decode_or_default::<Odbc, NaiveTime>(value)
            .to_string()
            .into(),
        "TIMESTAMP" => decode_or_default::<Odbc, DateTime<FixedOffset>>(value)
            .to_rfc3339()
            .into(),
        "VARBINARY" | "BINARY" => vec_to_data_uri_value(&decode_or_default::<Odbc, Vec<u8>>(value)),
        _ => decode_or_default::<Odbc, String>(value).into(),
    }
}
