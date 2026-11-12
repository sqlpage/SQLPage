use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::Value;
use sqlx::any::AnyKind;
use sqlx_next::types::Uuid;
use sqlx_next::{Column, ConnectOptions, Row, ValueRef};
use sqlx_sqlserver::{Mssql, MssqlColumn, MssqlConnectOptions, MssqlRow, MssqlType};

use super::{Pool, decode_or_default, pool_options};
use crate::app_config::AppConfig;
use crate::webserver::database::blob_to_data_url::vec_to_data_uri_value;
use crate::webserver::database::connect::retry_connecting;
use crate::webserver::database::sql_to_json::decimal_to_json;

const DBMS_NAME: &str = "Microsoft SQL Server";

pub(super) async fn connect(config: &AppConfig) -> anyhow::Result<(Pool, String)> {
    anyhow::ensure!(
        config.database_password.is_none(),
        "The unstable sqlx 0.9 SQL Server driver does not support database_password yet. \
        Put the password in database_url, or set unstable_sqlx_next to false."
    );
    let options: MssqlConnectOptions = config.database_url.parse()?;
    drop(retry_connecting(config, || options.connect()).await?);
    let pool = pool_options::<Mssql>(config, AnyKind::Mssql)
        .connect_with(options)
        .await
        .with_context(|| format!("Unable to open connection pool to {}", config.database_url))?;
    Ok((Pool::SqlServer(pool), DBMS_NAME.to_owned()))
}

pub(super) fn decode_row(row: &MssqlRow) -> Vec<(String, Value)> {
    row.columns()
        .iter()
        .map(|column| (column.name().to_owned(), column_to_json(row, column)))
        .collect()
}

fn column_to_json(row: &MssqlRow, column: &MssqlColumn) -> Value {
    let value = match row.try_get_raw(column.ordinal()) {
        Ok(value) if value.is_null() => return Value::Null,
        Ok(value) => value,
        Err(error) => {
            log::warn!("Unable to extract value from row: {error:?}");
            return Value::Null;
        }
    };
    let kind = value.type_info().kind().clone();
    match kind {
        MssqlType::Null => Value::Null,
        MssqlType::Real | MssqlType::Float | MssqlType::Money => {
            decode_or_default::<Mssql, f64>(value).into()
        }
        MssqlType::Decimal => decimal_to_json(&decode_or_default::<Mssql, BigDecimal>(value)),
        MssqlType::BigInt => decode_or_default::<Mssql, i64>(value).into(),
        MssqlType::Int => decode_or_default::<Mssql, i32>(value).into(),
        MssqlType::SmallInt => decode_or_default::<Mssql, i16>(value).into(),
        MssqlType::TinyInt => decode_or_default::<Mssql, u8>(value).into(),
        MssqlType::Bit => decode_or_default::<Mssql, bool>(value).into(),
        MssqlType::Date => decode_or_default::<Mssql, NaiveDate>(value)
            .to_string()
            .into(),
        MssqlType::Time => decode_or_default::<Mssql, NaiveTime>(value)
            .to_string()
            .into(),
        MssqlType::DateTimeOffset => decode_or_default::<Mssql, DateTime<FixedOffset>>(value)
            .to_rfc3339()
            .into(),
        MssqlType::DateTime | MssqlType::DateTime2 => {
            decode_or_default::<Mssql, NaiveDateTime>(value)
                .format("%FT%T%.f")
                .to_string()
                .into()
        }
        MssqlType::UniqueIdentifier => decode_or_default::<Mssql, Uuid>(value).to_string().into(),
        MssqlType::VarBinary => vec_to_data_uri_value(&decode_or_default::<Mssql, Vec<u8>>(value)),
        MssqlType::NVarChar | MssqlType::VarChar | MssqlType::Other(_) => {
            decode_or_default::<Mssql, String>(value).into()
        }
    }
}
