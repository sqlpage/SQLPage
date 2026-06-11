//! HTTP error handling
//!
//! This module owns the single boundary where an internal [`anyhow::Error`]
//! becomes the user-facing error representation. The [`ClientError`] type, built
//! by [`ClientError::new`], is the *only* place that consults the environment
//! (development vs production) to decide how much detail an error may expose.
//! Every output format ([`crate::render`]) renders a `ClientError` and never
//! inspects the environment itself, so no renderer can leak the SQL statement,
//! the source path, the raw database error, environment values, or
//! configuration. The full error is always logged server-side, independently of
//! what the client receives.

use std::path::PathBuf;

use crate::AppState;
use crate::app_config::DevOrProd;
use crate::webserver::ErrorWithStatus;
use actix_web::HttpResponseBuilder;
use actix_web::error::UrlencodedError;
use actix_web::http::{StatusCode, header};
use actix_web::{HttpRequest, HttpResponse};
use handlebars::{Renderable, StringOutput};
use serde_json::{Value, json};

/// Generic message shown to end users in production instead of the detailed
/// error, which would leak the source file path, the SQL statement, and the
/// raw database error text.
const PRODUCTION_ERROR_MESSAGE: &str =
    "Please contact the administrator for more information. The error has been logged.";

const DEV_ERROR_NOTE: &str = "You can hide error messages like this one from your users by setting the 'environment' configuration option to 'production'.";

/// An internal error reduced to the form that is safe to send to a client.
///
/// This is the single boundary where a raw [`anyhow::Error`] becomes a
/// user-facing error. It is built once, from the error and the environment, by
/// [`ClientError::new`] (the only place that consults the environment).
/// Renderers (JSON, NDJSON, SSE, CSV, HTML) only ever receive a `ClientError`,
/// never the raw error, so no output format can leak the source path, the SQL
/// statement, the raw database error, environment values, or configuration: in
/// production every `ClientError` holds nothing but the generic message. The
/// full error is always logged server-side, independently of what the client
/// receives.
///
/// Adding a new output format is therefore safe by construction: it can only
/// render the fields of `ClientError`, all of which are already
/// production-safe.
#[derive(Debug, Clone)]
pub struct ClientError {
    /// One-line, client-safe message. Generic in production, detailed in
    /// development. Suitable for machine formats (JSON/NDJSON/SSE/CSV).
    message: String,
    /// Causes chain, for the human-facing HTML backtrace. Empty in production.
    backtrace: Vec<String>,
    /// The query that failed, shown to humans in development. `None` in
    /// production and when not applicable.
    query_number: Option<usize>,
    /// Hint shown to humans in development on how to hide errors. `None` in
    /// production.
    note: Option<&'static str>,
    /// `true` when this error was reduced to the generic production message.
    /// This is the only environment-derived flag callers may branch on, so the
    /// environment itself is never consulted outside [`ClientError::new`].
    is_generic: bool,
}

impl ClientError {
    /// Reduces a raw error to the client-safe form for the given environment.
    /// In production, only the generic message survives; in development, the
    /// full detail (message, backtrace, hint) is kept.
    ///
    /// This is the single location in the whole codebase that consults the
    /// environment (the only call to `DevOrProd::is_prod`) to decide how much
    /// of an error reaches a client. Renderers never make this decision; they
    /// only forward the configured environment here and then format the result.
    #[must_use]
    pub fn new(error: &anyhow::Error, environment: DevOrProd, query_number: Option<usize>) -> Self {
        if environment.is_prod() {
            Self {
                message: PRODUCTION_ERROR_MESSAGE.to_owned(),
                backtrace: Vec::new(),
                query_number: None,
                note: None,
                is_generic: true,
            }
        } else {
            Self {
                message: error.to_string(),
                backtrace: get_backtrace_as_strings(error),
                query_number,
                note: Some(DEV_ERROR_NOTE),
                is_generic: false,
            }
        }
    }

    /// The single-line, client-safe message used by machine-readable formats
    /// (JSON, NDJSON, SSE) and CSV.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether this error carries only the generic production message (i.e. it
    /// was built in production). Lets the header path decide between rendering
    /// the error inline and bubbling up to a top-level error response, without
    /// itself looking at the environment.
    #[must_use]
    pub fn is_generic(&self) -> bool {
        self.is_generic
    }

    /// The data passed to the `error` HTML component. Every field is already
    /// client-safe (empty/`None` in production).
    #[must_use]
    pub fn to_html_data(&self) -> Value {
        json!({
            "query_number": self.query_number,
            "description": self.message,
            "backtrace": self.backtrace,
            "note": self.note,
        })
    }
}

/// Collects the chain of error causes as strings, for the human-facing HTML
/// backtrace shown in development.
#[must_use]
pub(crate) fn get_backtrace_as_strings(error: &anyhow::Error) -> Vec<String> {
    let mut backtrace = vec![];
    let mut source = error.source();
    while let Some(s) = source {
        backtrace.push(format!("{s}"));
        source = s.source();
    }
    backtrace
}

fn error_to_html_string(app_state: &AppState, err: &anyhow::Error) -> anyhow::Result<String> {
    let mut out = StringOutput::new();
    let shell_template = app_state.all_templates.get_static_template("shell")?;
    let error_template = app_state.all_templates.get_static_template("error")?;
    let registry = &app_state.all_templates.handlebars;
    let shell_ctx = handlebars::Context::null();
    // Reduce the error to its client-safe form before rendering. In production
    // this hides the message, backtrace, and source detail behind the generic
    // text shown by the `error` template.
    let data = ClientError::new(err, app_state.config.environment, None).to_html_data();
    let err_ctx = handlebars::Context::wraps(data)?;
    let rc = &mut handlebars::RenderContext::new(None);

    // Open the shell component
    shell_template
        .before_list
        .render(registry, &shell_ctx, rc, &mut out)?;

    // Open the error component
    error_template
        .before_list
        .render(registry, &err_ctx, rc, &mut out)?;
    // Close the error component
    error_template
        .after_list
        .render(registry, &err_ctx, rc, &mut out)?;

    // Close the shell component
    shell_template
        .after_list
        .render(registry, &shell_ctx, rc, &mut out)?;

    Ok(out.into_string()?)
}

pub(super) fn anyhow_err_to_actix_resp(e: &anyhow::Error, state: &AppState) -> HttpResponse {
    let mut resp = HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR);
    resp.insert_header((header::CONTENT_TYPE, header::ContentType::plaintext()));

    if let Some(status) = anyhow_error_status(e) {
        resp.status(status);
        if status == StatusCode::UNAUTHORIZED {
            resp.append_header((
                header::WWW_AUTHENTICATE,
                "Basic realm=\"Authentication required\", charset=\"UTF-8\"",
            ));
        }
    } else if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        use rand::RngExt;
        resp.status(StatusCode::TOO_MANY_REQUESTS).insert_header((
            header::RETRY_AFTER,
            header::HeaderValue::from(rand::rng().random_range(1..=15)),
        ));
    }
    match error_to_html_string(state, e) {
        Ok(body) => {
            resp.insert_header((header::CONTENT_TYPE, header::ContentType::html()));
            resp.body(body)
        }
        Err(second_err) => {
            log::error!("Unable to render error: {e:#}");
            resp.body(format!(
                "A second error occurred while rendering the error page: \n\n\
                Initial error: \n\
                {e:#}\n\n\
                Second error: \n\
                {second_err:#}"
            ))
        }
    }
}

fn anyhow_error_status(e: &anyhow::Error) -> Option<StatusCode> {
    if let Some(&ErrorWithStatus { status }) = e.downcast_ref() {
        Some(status)
    } else if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        Some(StatusCode::TOO_MANY_REQUESTS)
    } else {
        None
    }
}

pub(super) fn send_anyhow_error(
    e: &anyhow::Error,
    resp_send: tokio::sync::oneshot::Sender<HttpResponse>,
    state: &AppState,
) {
    log::error!("An error occurred before starting to send the response body: {e:#}");
    resp_send
        .send(anyhow_err_to_actix_resp(e, state))
        .unwrap_or_else(|_| log::error!("could not send headers"));
}

pub(super) fn anyhow_err_to_actix(e: anyhow::Error, state: &AppState) -> actix_web::Error {
    log::error!("{e:#}");
    let resp = anyhow_err_to_actix_resp(&e, state);
    actix_web::error::InternalError::from_response(e, resp).into()
}

pub(super) fn handle_form_error(
    decode_err: UrlencodedError,
    _req: &HttpRequest,
) -> actix_web::Error {
    match decode_err {
        actix_web::error::UrlencodedError::Overflow { size, limit } => {
            actix_web::error::ErrorPayloadTooLarge(format!(
                "The submitted form data size ({size} bytes) exceeds the maximum allowed upload size ({limit} bytes). \
                    You can increase this limit by setting max_uploaded_file_size in the configuration file.",
            ))
        }
        _ => actix_web::Error::from(decode_err),
    }
}

pub(super) fn bind_error(e: std::io::Error, listen_on: std::net::SocketAddr) -> anyhow::Error {
    let (ip, port) = (listen_on.ip(), listen_on.port());
    // Let's try to give a more helpful error message in common cases
    let ctx = match e.kind() {
        std::io::ErrorKind::AddrInUse => format!(
            "Another program is already using port {port} (maybe {} ?). \
            You can either stop that program or change the port in the configuration file.",
            if port == 80 || port == 443 {
                "Apache or Nginx"
            } else {
                "another instance of SQLPage"
            },
        ),
        std::io::ErrorKind::PermissionDenied => format!(
            "You do not have permission to bind to {ip} on port {port}. \
            You can either run SQLPage as root with sudo, give it the permission to bind to low ports with `sudo setcap cap_net_bind_service=+ep {executable_path}`, \
            or change the port in the configuration file.",
            executable_path = std::env::current_exe()
                .unwrap_or_else(|_| PathBuf::from("sqlpage.bin"))
                .display(),
        ),
        std::io::ErrorKind::AddrNotAvailable => format!(
            "The IP address {ip} does not exist on this computer. \
            You can change the value of listen_on in the configuration file.",
        ),
        _ => format!("Unable to bind to {ip} on port {port}"),
    };
    anyhow::anyhow!(e).context(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Structural guard for the production-leak invariant.
    ///
    /// Every output format renders an error solely from the fields of
    /// [`ClientError`] (via [`ClientError::message`] for machine formats and
    /// [`ClientError::to_html_data`] for HTML). If, in production, neither of
    /// those exposes any sensitive substring of the original error, then no
    /// renderer can leak, including formats added in the future. This test
    /// asserts exactly that at the single boundary, so it does not need to be
    /// repeated per format.
    #[test]
    fn test_production_client_error_hides_all_detail() {
        let sensitive = anyhow::anyhow!("DB error near 'secret_table'")
            .context("The SQL statement sent by SQLPage was: SELECT * FROM secret_table")
            .context("Error in file /srv/www/private/admin.sql");

        let client_error = ClientError::new(&sensitive, DevOrProd::Production, Some(3));

        // Everything a renderer can read about the error, serialized together.
        let exposed = format!(
            "{}\n{}",
            client_error.message(),
            client_error.to_html_data()
        );

        for needle in [
            "secret_table",
            "SELECT",
            "/srv/www/private/admin.sql",
            ".sql",
            "DB error",
        ] {
            assert!(
                !exposed.contains(needle),
                "production ClientError leaked {needle:?}: {exposed}"
            );
        }
        assert!(
            exposed.to_lowercase().contains("administrator"),
            "production ClientError should carry the generic message: {exposed}"
        );
        // The query number is detail too: it must not survive into production.
        assert!(
            !exposed.contains('3'),
            "production ClientError leaked the query number: {exposed}"
        );
        assert!(
            client_error.is_generic(),
            "a production ClientError must report itself as generic"
        );
    }

    /// In development, the full detail must be preserved so authors can debug.
    #[test]
    fn test_development_client_error_keeps_detail() {
        let error = anyhow::anyhow!("near 'secret_table': syntax error")
            .context("The SQL statement sent by SQLPage was: SELECT 1");
        let client_error = ClientError::new(&error, DevOrProd::Development, Some(2));
        assert!(client_error.message().contains("SELECT 1"));
        assert!(!client_error.is_generic());
        let html = client_error.to_html_data();
        assert_eq!(html["query_number"], json!(2));
        assert!(html["backtrace"].as_array().is_some_and(|b| !b.is_empty()));
        assert!(html["note"].is_string());
    }
}
