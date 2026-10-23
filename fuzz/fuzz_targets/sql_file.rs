// Copyright 2026 SQLPage contributors. SPDX-License-Identifier: MIT
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqlpage::webserver::database::{DbInfo, SqlFile, SupportedDatabase};
use sqlx::any::AnyKind;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, source)) = data.split_first() else {
        return;
    };
    let Ok(source) = std::str::from_utf8(source) else {
        return;
    };
    let database_type = [
        SupportedDatabase::Sqlite,
        SupportedDatabase::Postgres,
        SupportedDatabase::MySql,
        SupportedDatabase::Mssql,
        SupportedDatabase::Duckdb,
        SupportedDatabase::Oracle,
        SupportedDatabase::Snowflake,
        SupportedDatabase::Generic,
    ][usize::from(selector & 7)];
    let kind = match database_type {
        _ if selector & 8 != 0 => AnyKind::Odbc,
        SupportedDatabase::Sqlite => AnyKind::Sqlite,
        SupportedDatabase::Postgres => AnyKind::Postgres,
        SupportedDatabase::MySql => AnyKind::MySql,
        SupportedDatabase::Mssql => AnyKind::Mssql,
        _ => AnyKind::Odbc,
    };
    let database = DbInfo {
        dbms_name: database_type.display_name().into(),
        database_type,
        kind,
    };
    // Construction eagerly parses every statement and runs the production
    // rewrite pipeline. Syntax/rewrite errors are ordinary results; panics,
    // sanitizer failures and timeouts are bugs. No SQL is executed.
    std::hint::black_box(SqlFile::from_db_info(
        &database,
        source,
        Path::new("fuzz.sql"),
    ));
});
