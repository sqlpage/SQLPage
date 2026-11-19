use chrono::{DateTime, Utc};
use sqlx::{
    any::{Any, AnyArguments},
    query::{Query, query},
    row::Row as _,
};

use super::Database;

#[derive(Clone, Copy)]
pub enum Parameter<'a> {
    Text(Option<&'a str>),
    Bytes(&'a [u8]),
    Timestamp(DateTime<Utc>),
}

impl Database {
    pub async fn execute_with(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<(), sqlx::error::Error> {
        file_query(sql, params)
            .execute(&self.connection)
            .await
            .map(drop)
    }

    pub(crate) async fn exists(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<bool, sqlx::error::Error> {
        file_query(sql, params)
            .fetch_optional(&self.connection)
            .await
            .map(|row| row.is_some())
    }

    pub(crate) async fn fetch_bytes(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<Option<Vec<u8>>, sqlx::error::Error> {
        file_query(sql, params)
            .fetch_optional(&self.connection)
            .await?
            .map(|row| row.try_get(0))
            .transpose()
    }
}

fn file_query<'a>(sql: &'a str, params: &[Parameter<'a>]) -> Query<'a, Any, AnyArguments<'a>> {
    params.iter().fold(query(sql), |query, param| match *param {
        Parameter::Text(text) => query.bind(text),
        Parameter::Bytes(bytes) => query.bind(bytes),
        Parameter::Timestamp(timestamp) => query.bind(timestamp),
    })
}
