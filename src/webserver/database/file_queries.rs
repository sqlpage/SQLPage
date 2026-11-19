use super::Database;
use chrono::{DateTime, Utc};

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
    ) -> Result<(), sqlx::Error> {
        self.connection.execute_with(sql, params).await
    }

    pub(crate) async fn exists(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<bool, sqlx::Error> {
        self.connection.exists(sql, params).await
    }

    pub(crate) async fn fetch_bytes(
        &self,
        sql: &str,
        params: &[Parameter<'_>],
    ) -> Result<Option<Vec<u8>>, sqlx::Error> {
        self.connection.fetch_bytes(sql, params).await
    }
}
