// Copyright 2026 SQLPage contributors. SPDX-License-Identifier: MIT
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqlpage::webserver::database::{DbInfo, SqlFile, SupportedDatabase::*};
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, source)) = data.split_first() else {
        return;
    };
    let Ok(source) = std::str::from_utf8(source) else {
        return;
    };
    let mut database = [
        DbInfo::Native(Sqlite),
        DbInfo::Native(Postgres),
        DbInfo::Native(MySql),
        DbInfo::Native(Mssql),
        DbInfo::Odbc(Duckdb),
        DbInfo::Odbc(Oracle),
        DbInfo::Odbc(Snowflake),
        DbInfo::Odbc(Generic),
    ][usize::from(selector & 7)];
    if selector & 8 != 0 {
        database = DbInfo::Odbc(database.database_type());
    }
    // Construction eagerly parses every statement and runs the production
    // rewrite pipeline. Syntax/rewrite errors are ordinary results; panics,
    // sanitizer failures and timeouts are bugs. No SQL is executed.
    std::hint::black_box(SqlFile::from_db_info(
        database,
        source,
        Path::new("fuzz.sql"),
    ));
});
