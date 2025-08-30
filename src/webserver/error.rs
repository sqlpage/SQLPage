//! HTTP error handling

use std::path::PathBuf;

use crate::webserver::ErrorWithStatus;
use crate::AppState;
use actix_web::error::UrlencodedError;
use actix_web::http::{header, StatusCode};
use actix_web::{HttpRequest, HttpResponse};
use actix_web::{HttpResponseBuilder, ResponseError};

fn anyhow_err_to_actix_resp(e: &anyhow::Error, state: &AppState) -> HttpResponse {
    let mut resp = HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR);
    let mut body = "Sorry, but we were not able to process your request.\n\n".to_owned();
    let env = state.config.environment;
    if env.is_prod() {
        body.push_str("Contact the administrator for more information. A detailed error message has been logged.");
        log::error!("{e:#}");
    } else {
        use std::fmt::Write;
        write!(
            body,
            "Below are detailed debugging information which may contain sensitive data. \n\
        Set environment to \"production\" in the configuration file to hide this information. \n\n\
        {e:?}"
        )
        .unwrap();
    }
    resp.insert_header((
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    ));

    if let Some(status_err @ &ErrorWithStatus { .. }) = e.downcast_ref() {
        status_err
            .error_response()
            .set_body(actix_web::body::BoxBody::new(body))
    } else if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        use rand::Rng;
        resp.status(StatusCode::TOO_MANY_REQUESTS)
            .insert_header((
                header::RETRY_AFTER,
                header::HeaderValue::from(rand::rng().random_range(1..=15)),
            ))
            .body("The database is currently too busy to handle your request. Please try again later.\n\n".to_owned() + &body)
    } else {
        resp.body(body)
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
