//! HTTP error handling

use std::path::PathBuf;

use crate::render::get_backtrace_as_strings;
use crate::webserver::ErrorWithStatus;
use crate::AppState;
use actix_web::error::UrlencodedError;
use actix_web::http::{header, StatusCode};
use actix_web::HttpResponseBuilder;
use actix_web::{HttpRequest, HttpResponse};
use handlebars::{Renderable, StringOutput};
use serde_json::json;

fn error_to_html_string(app_state: &AppState, err: &anyhow::Error) -> anyhow::Result<String> {
    let mut out = StringOutput::new();
    let shell_template = app_state.all_templates.get_static_template("shell")?;
    let error_template = app_state.all_templates.get_static_template("error")?;
    let registry = &app_state.all_templates.handlebars;
    let shell_ctx = handlebars::Context::null();
    let data = if app_state.config.environment.is_prod() {
        json!(null)
    } else {
        json!({
            "description": err.to_string(),
            "backtrace": get_backtrace_as_strings(err),
            "note": "You can hide error messages like this one from your users by setting the 'environment' configuration option to 'production'.",
        })
    };
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

fn anyhow_err_to_actix_resp(e: &anyhow::Error, state: &AppState) -> HttpResponse {
    let mut resp = HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR);
    resp.insert_header((header::CONTENT_TYPE, header::ContentType::plaintext()));

    if let Some(&ErrorWithStatus { status }) = e.downcast_ref() {
        resp.status(status);
        if status == StatusCode::UNAUTHORIZED {
            resp.append_header((
                header::WWW_AUTHENTICATE,
                "Basic realm=\"Authentication required\", charset=\"UTF-8\"",
            ));
        }
    } else if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        use rand::Rng;
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
            actix_web::error::ErrorPayloadTooLarge(
                format!(
                    "The submitted form data size ({size} bytes) exceeds the maximum allowed upload size ({limit} bytes). \
                    You can increase this limit by setting max_uploaded_file_size in the configuration file.",
                ),
            )
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
            executable_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sqlpage.bin")).display(),
        ),
        std::io::ErrorKind::AddrNotAvailable => format!(
            "The IP address {ip} does not exist on this computer. \
            You can change the value of listen_on in the configuration file.",
        ),
        _ => format!("Unable to bind to {ip} on port {port}"),
    };
    anyhow::anyhow!(e).context(ctx)
}
