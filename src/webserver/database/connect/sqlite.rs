use std::ffi::c_int;

use futures_util::future::BoxFuture;
use libsqlite3_sys as ffi;
use sqlx::{SqliteConnection, sqlite::SqliteConnectOptions};

use crate::app_config::AppConfig;

pub(super) fn options(config: &AppConfig) -> Result<SqliteConnectOptions, sqlx::Error> {
    let mut options = config
        .database_url
        .parse::<SqliteConnectOptions>()?
        .collation("NOCASE", |a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    for extension in &config.sqlite_extensions {
        log::info!("Loading SQLite extension: {extension}");
        // Only the server operator can configure which native libraries to load.
        options = unsafe { options.extension(extension.clone()) };
    }
    Ok(options)
}

pub(super) fn register_functions(
    conn: &mut SqliteConnection,
) -> BoxFuture<'_, Result<(), sqlx::Error>> {
    Box::pin(async move {
        let mut handle = conn.lock_handle().await?;
        for (name, callback) in [
            (c"upper", change_case::<true> as _),
            (c"lower", change_case::<false> as _),
        ] {
            // SQLx's worker cannot access the handle while it is locked.
            let result = unsafe {
                ffi::sqlite3_create_function_v2(
                    handle.as_raw_handle().as_ptr(),
                    name.as_ptr(),
                    1,
                    ffi::SQLITE_UTF8 | ffi::SQLITE_DETERMINISTIC,
                    std::ptr::null_mut(),
                    Some(callback),
                    None,
                    None,
                    None,
                )
            };
            if result != ffi::SQLITE_OK {
                return Err(sqlx::Error::Protocol(format!(
                    "Unable to register SQLite function {name:?}: error {result}"
                )));
            }
        }
        Ok(())
    })
}

unsafe extern "C" fn change_case<const UPPER: bool>(
    ctx: *mut ffi::sqlite3_context,
    _: c_int,
    args: *mut *mut ffi::sqlite3_value,
) {
    // Both functions are registered with one argument. SQLite owns it and the
    // context until this callback returns; SQLITE_TRANSIENT copies our result.
    unsafe {
        let value = *args;
        if ffi::sqlite3_value_type(value) == ffi::SQLITE_NULL {
            return;
        }
        let data = ffi::sqlite3_value_text(value);
        if data.is_null() {
            ffi::sqlite3_result_error_nomem(ctx);
            return;
        }
        let len = usize::try_from(ffi::sqlite3_value_bytes(value)).unwrap_or_default();
        let bytes = std::slice::from_raw_parts(data, len);
        let Ok(text) = std::str::from_utf8(bytes) else {
            ffi::sqlite3_result_error(ctx, c"Invalid UTF-8 in SQLite text".as_ptr(), -1);
            return;
        };
        let result = if UPPER {
            text.to_uppercase()
        } else {
            text.to_lowercase()
        };
        ffi::sqlite3_result_text64(
            ctx,
            result.as_ptr().cast(),
            result.len() as u64,
            ffi::SQLITE_TRANSIENT(),
            u8::try_from(ffi::SQLITE_UTF8).unwrap(),
        );
    }
}
