//! This module handles HTTP requests and responses for the web server,
//! including rendering SQL files, serving static content, and managing
//! request contexts and response headers.

use crate::render::{AnyRenderBodyContext, HeaderContext, PageContext};
use crate::webserver::content_security_policy::ContentSecurityPolicy;
use crate::webserver::database::execute_queries::stop_at_first_error;
use crate::webserver::database::{execute_queries::stream_query_results_with_conn, DbItem};
use crate::webserver::http_request_info::extract_request_info;
use crate::webserver::ErrorWithStatus;
use crate::{app_config, AppConfig, AppState, ParsedSqlFile};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{ContentType, Header, HttpDate, IfModifiedSince, LastModified};
use actix_web::http::{header, StatusCode, Uri};
use actix_web::web::PayloadConfig;
use actix_web::{
    dev::ServiceResponse, middleware, middleware::Logger, web, App, HttpResponse, HttpServer,
};
use actix_web::{HttpResponseBuilder, ResponseError};

use super::https::make_auto_rustls_config;
use super::response_writer::ResponseWriter;
use super::static_content;
use actix_web::body::MessageBody;
use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;
use crate::webserver::routing::{calculate_route, AppFileStore};
use crate::webserver::routing::RoutingAction::{NotFound, Execute, CustomNotFound, Redirect, Serve};

#[derive(Clone)]
pub struct RequestContext {
    pub is_embedded: bool,
    pub content_security_policy: ContentSecurityPolicy,
}

async fn stream_response(stream: impl Stream<Item = DbItem>, mut renderer: AnyRenderBodyContext) {
    let mut stream = Box::pin(stream);

    if let Err(e) = &renderer.flush().await {
        log::error!("Unable to flush initial data to client: {e}");
        return;
    }

    while let Some(item) = stream.next().await {
        log::trace!("Received item from database: {item:?}");
        let render_result = match item {
            DbItem::FinishedQuery => renderer.finish_query().await,
            DbItem::Row(row) => renderer.handle_row(&row).await,
            DbItem::Error(e) => renderer.handle_error(&e).await,
        };
        if let Err(e) = render_result {
            if let Err(nested_err) = renderer.handle_error(&e).await {
                renderer
                    .close()
                    .await
                    .close_with_error(nested_err.to_string())
                    .await;
                log::error!(
                    "An error occurred while trying to display an other error. \
                    \nRoot error: {e}\n
                    \nNested error: {nested_err}"
                );
                return;
            }
        }
        if let Err(e) = &renderer.flush().await {
            log::error!(
                "Stopping rendering early because we were unable to flush data to client. \
                The user has probably closed the connection before we finished rendering the page: {e:#}"
            );
            // If we cannot write to the client anymore, there is nothing we can do, so we just stop rendering
            return;
        }
    }
    if let Err(e) = &renderer.close().await.async_flush().await {
        log::error!("Unable to flush data to client after rendering the page end: {e}");
        return;
    }
    log::debug!("Successfully finished rendering the page");
}

async fn build_response_header_and_stream<S: Stream<Item = DbItem>>(
    app_state: Arc<AppState>,
    database_entries: S,
    request_context: RequestContext,
) -> anyhow::Result<ResponseWithWriter<S>> {
    let chan_size = app_state.config.max_pending_rows;
    let (sender, receiver) = mpsc::channel(chan_size);
    let writer = ResponseWriter::new(sender);
    let mut head_context = HeaderContext::new(app_state, request_context, writer);
    let mut stream = Box::pin(database_entries);
    while let Some(item) = stream.next().await {
        let page_context = match item {
            DbItem::Row(data) => head_context.handle_row(data).await?,
            DbItem::FinishedQuery => {
                log::debug!("finished query");
                continue;
            }
            DbItem::Error(source_err)
                if matches!(
                    source_err.downcast_ref(),
                    Some(&ErrorWithStatus { status: _ })
                ) =>
            {
                return Err(source_err)
            }
            DbItem::Error(source_err) => head_context.handle_error(source_err).await?,
        };
        match page_context {
            PageContext::Header(h) => {
                head_context = h;
            }
            PageContext::Body {
                mut http_response,
                renderer,
            } => {
                let body_stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
                let result_stream = body_stream.map(Ok::<_, actix_web::Error>);
                let http_response = http_response.streaming(result_stream);
                return Ok(ResponseWithWriter::RenderStream {
                    http_response,
                    renderer,
                    database_entries_stream: stream,
                });
            }
            PageContext::Close(http_response) => {
                return Ok(ResponseWithWriter::FinishedResponse { http_response })
            }
        }
    }
    log::debug!("No SQL statements left to execute for the body of the response");
    let http_response = head_context.close();
    Ok(ResponseWithWriter::FinishedResponse { http_response })
}

enum ResponseWithWriter<S> {
    RenderStream {
        http_response: HttpResponse,
        renderer: AnyRenderBodyContext,
        database_entries_stream: Pin<Box<S>>,
    },
    FinishedResponse {
        http_response: HttpResponse,
    },
}

async fn render_sql(
    srv_req: &mut ServiceRequest,
    sql_file: Arc<ParsedSqlFile>,
) -> actix_web::Result<HttpResponse> {
    let app_state = srv_req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorInternalServerError("no state"))?
        .clone() // Cheap reference count increase
        .into_inner();

    let mut req_param = extract_request_info(srv_req, Arc::clone(&app_state))
        .await
        .map_err(|e| anyhow_err_to_actix(e, app_state.config.environment))?;
    log::debug!("Received a request with the following parameters: {req_param:?}");

    let (resp_send, resp_recv) = tokio::sync::oneshot::channel::<HttpResponse>();
    actix_web::rt::spawn(async move {
        let request_context = RequestContext {
            is_embedded: req_param.get_variables.contains_key("_sqlpage_embed"),
            content_security_policy: ContentSecurityPolicy::default(),
        };
        let mut conn = None;
        let database_entries_stream =
            stream_query_results_with_conn(&sql_file, &mut req_param, &mut conn);
        let database_entries_stream = stop_at_first_error(database_entries_stream);
        let response_with_writer = build_response_header_and_stream(
            Arc::clone(&app_state),
            database_entries_stream,
            request_context,
        )
        .await;
        match response_with_writer {
            Ok(ResponseWithWriter::RenderStream {
                http_response,
                renderer,
                database_entries_stream,
            }) => {
                resp_send
                    .send(http_response)
                    .unwrap_or_else(|e| log::error!("could not send headers {e:?}"));
                stream_response(database_entries_stream, renderer).await;
            }
            Ok(ResponseWithWriter::FinishedResponse { http_response }) => {
                resp_send
                    .send(http_response)
                    .unwrap_or_else(|e| log::error!("could not send headers {e:?}"));
            }
            Err(err) => {
                send_anyhow_error(&err, resp_send, app_state.config.environment);
            }
        }
    });
    resp_recv.await.map_err(ErrorInternalServerError)
}

fn anyhow_err_to_actix_resp(e: &anyhow::Error, env: app_config::DevOrProd) -> HttpResponse {
    let mut resp = HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR);
    let mut body = "Sorry, but we were not able to process your request. \n\n\
        Below are detailed debugging information which may contain sensitive data. \
        Set environment to \"prod\" in the configuration file to hide this information. \n\n"
        .to_owned();
    if env.is_prod() {
        body.push_str("Contact the administrator for more information. A detailed error message has been logged.");
    } else {
        use std::fmt::Write;
        write!(body, "{e:?}").unwrap();
    }
    resp.insert_header((
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain"),
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
                header::HeaderValue::from(rand::thread_rng().gen_range(1..=15)),
            ))
            .body("The database is currently too busy to handle your request. Please try again later.\n\n".to_owned() + &body)
    } else {
        resp.body(body)
    }
}

fn send_anyhow_error(
    e: &anyhow::Error,
    resp_send: tokio::sync::oneshot::Sender<HttpResponse>,
    env: app_config::DevOrProd,
) {
    log::error!("An error occurred before starting to send the response body: {e:#}");
    resp_send
        .send(anyhow_err_to_actix_resp(e, env))
        .unwrap_or_else(|_| log::error!("could not send headers"));
}

fn anyhow_err_to_actix(e: anyhow::Error, env: app_config::DevOrProd) -> actix_web::Error {
    log::error!("{e:#}");
    let resp = anyhow_err_to_actix_resp(&e, env);
    actix_web::error::InternalError::from_response(e, resp).into()
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum SingleOrVec {
    Single(String),
    Vec(Vec<String>),
}

impl SingleOrVec {
    pub(crate) fn merge(&mut self, other: Self) {
        match (self, other) {
            (Self::Single(old), Self::Single(new)) => *old = new,
            (old, mut new) => {
                let mut v = old.take_vec();
                v.extend_from_slice(&new.take_vec());
                *old = Self::Vec(v);
            }
        }
    }
    fn take_vec(&mut self) -> Vec<String> {
        match self {
            SingleOrVec::Single(x) => vec![mem::take(x)],
            SingleOrVec::Vec(v) => mem::take(v),
        }
    }

    #[must_use]
    pub fn as_json_str(&self) -> Cow<'_, str> {
        match self {
            SingleOrVec::Single(x) => Cow::Borrowed(x),
            SingleOrVec::Vec(v) => Cow::Owned(serde_json::to_string(v).unwrap()),
        }
    }
}

/// Resolves the path in a query to the path to a local SQL file if there is one that matches
fn path_to_sql_file(path: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(path);
    match path.extension() {
        None => {
            path.push("index.sql");
            Some(path)
        }
        Some(ext) if ext == "sql" => Some(path),
        Some(_other) => None,
    }
}

async fn process_sql_request(
    req: &mut ServiceRequest,
    sql_path: PathBuf,
) -> actix_web::Result<HttpResponse> {
    let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
    let sql_file = app_state
        .sql_file_cache
        .get_with_privilege(app_state, &sql_path, false)
        .await
        .with_context(|| format!("Unable to get SQL file {sql_path:?}"))
        .map_err(|e| anyhow_err_to_actix(e, app_state.config.environment))?;
    render_sql(req, sql_file).await
}

async fn serve_file(
    path: &str,
    state: &AppState,
    if_modified_since: Option<IfModifiedSince>,
) -> actix_web::Result<HttpResponse> {
    let path = strip_site_prefix(path, state);
    if let Some(IfModifiedSince(date)) = if_modified_since {
        let since = DateTime::<Utc>::from(SystemTime::from(date));
        let modified = state
            .file_system
            .modified_since(state, path.as_ref(), since, false)
            .await
            .with_context(|| format!("Unable to get modification time of file {path:?}"))
            .map_err(|e| anyhow_err_to_actix(e, state.config.environment))?;
        if !modified {
            return Ok(HttpResponse::NotModified().finish());
        }
    }
    state
        .file_system
        .read_file(state, path.as_ref(), false)
        .await
        .with_context(|| format!("Unable to read file {path:?}"))
        .map_err(|e| anyhow_err_to_actix(e, state.config.environment))
        .map(|b| {
            HttpResponse::Ok()
                .insert_header(
                    mime_guess::from_path(path)
                        .first()
                        .map_or_else(ContentType::octet_stream, ContentType),
                )
                .insert_header(LastModified(HttpDate::from(SystemTime::now())))
                .body(b)
        })
}

/// Strips the site prefix from a path
fn strip_site_prefix<'a>(path: &'a str, state: &AppState) -> &'a str {
    path.strip_prefix(&state.config.site_prefix).unwrap_or(path)
}

/// Fallback handler for when a file could not be served
///
/// Recursively traverses upwards in the request's path, looking for a `404.sql` to call as the
/// fallback handler. Fails if none is found, or if there is an issue while running the `404.sql`.
async fn serve_fallback(
    service_request: &mut ServiceRequest,
    original_err: actix_web::Error,
) -> actix_web::Result<HttpResponse> {
    let catch_all = "404.sql";

    let req_path = req_path(service_request);
    let mut fallback_path_candidate = req_path.clone().into_owned();
    log::debug!("Trying to find a {catch_all:?} handler for {fallback_path_candidate:?}");

    let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");

    // Find the indices of each char which follows a directroy separator (`/`). Also consider the 0
    // index, as we also have to try to check the empty path, for a root dir `404.sql`.
    for idx in req_path
        .rmatch_indices('/')
        .map(|(idx, _)| idx + 1)
        .take(128) // Limit the number of iterations because lookups can be expensive
        .chain(std::iter::once(0))
    {
        // Remove the trailing substring behind the current `/`, and append `404.sql`.
        fallback_path_candidate.truncate(idx);
        fallback_path_candidate.push_str(catch_all);

        // Check if `maybe_fallback_path` actually exists, if not, skip to the next round (which
        // will check `maybe_fallback_path`s parent directory for fallback handler).
        let mabye_sql_path = PathBuf::from(&fallback_path_candidate);
        match app_state
            .sql_file_cache
            .get_with_privilege(app_state, &mabye_sql_path, false)
            .await
        {
            // `maybe_fallback_path` does seem to exist, lets try to run it!
            Ok(sql_file) => {
                log::debug!("Processing SQL request via fallback: {:?}", mabye_sql_path);
                return render_sql(service_request, sql_file).await;
            }
            Err(e) => {
                let actix_web_err = anyhow_err_to_actix(e, app_state.config.environment);

                // `maybe_fallback_path` does not exist, continue search in parent dir.
                if actix_web_err.as_response_error().status_code() == StatusCode::NOT_FOUND {
                    log::trace!("The 404 handler {mabye_sql_path:?} does not exist");
                    continue;
                }

                // Another error occured, bubble it up!
                return Err(actix_web_err);
            }
        }
    }

    log::debug!("There is no {catch_all:?} handler, this response is terminally failed");
    Err(original_err)
}

pub async fn main_handler(
    mut service_request: ServiceRequest,
) -> actix_web::Result<ServiceResponse> {
    let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
    let store = AppFileStore::new(&app_state.sql_file_cache, &app_state.file_system);
    let path_and_query = service_request.uri().path_and_query().expect("expected valid path with query from request");
    return match calculate_route(path_and_query, &store, &app_state.config).await {
        NotFound => { serve_fallback(&mut service_request, ErrorWithStatus { status: StatusCode::NOT_FOUND }.into()).await }
        Execute(path) => { process_sql_request(&mut service_request, path).await }
        CustomNotFound(path) => { process_sql_request(&mut service_request, path).await }
        Redirect(uri) => { Ok(HttpResponse::MovedPermanently()
            .insert_header((header::LOCATION, uri.to_string()))
            .finish()) }
        Serve(path) => {
            let if_modified_since = IfModifiedSince::parse(&service_request).ok();
            let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
            serve_file(path.as_os_str().to_str().unwrap(), app_state, if_modified_since).await
        }
    }.map(|response| service_request.into_response(response));

    // if let Some(redirect) = redirect_missing_prefix(&service_request) {
    //     return Ok(service_request.into_response(redirect));
    // }
    //
    // let path = req_path(&service_request);
    // let sql_file_path = path_to_sql_file(&path);
    // let maybe_response = if let Some(sql_path) = sql_file_path {
    //     if let Some(redirect) = redirect_missing_trailing_slash(service_request.uri()) {
    //         Ok(redirect)
    //     } else {
    //         log::debug!("Processing SQL request: {:?}", sql_path);
    //         process_sql_request(&mut service_request, sql_path).await
    //     }
    // } else {
    //     log::debug!("Serving file: {:?}", path);
    //     let path = req_path(&service_request);
    //     let if_modified_since = IfModifiedSince::parse(&service_request).ok();
    //     let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
    //     serve_file(&path, app_state, if_modified_since).await
    // };
    //
    // // On 404/NOT_FOUND error, fall back to `404.sql` handler if it exists
    // let response = match maybe_response {
    //     // File could not be served due to a 404 error. Try to find a user provide 404 handler in
    //     // the form of a `404.sql` in the current directory. If there is none, look in the parent
    //     // directeory, and its parent directory, ...
    //     Err(e) if e.as_response_error().status_code() == StatusCode::NOT_FOUND => {
    //         serve_fallback(&mut service_request, e).await?
    //     }
    //
    //     // Either a valid response, or an unrelated error that shall be bubbled up.
    //     e => e?,
    // };
    //
    // Ok(service_request.into_response(response))
}

fn redirect_missing_prefix(service_request: &ServiceRequest) -> Option<HttpResponse> {
    let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
    if !service_request
        .path()
        .starts_with(&app_state.config.site_prefix)
    {
        let header = (header::LOCATION, app_state.config.site_prefix.clone());
        return Some(
            HttpResponse::PermanentRedirect()
                .insert_header(header)
                .finish(),
        );
    }
    None
}

/// Extracts the path from a request and percent-decodes it
fn req_path(req: &ServiceRequest) -> Cow<'_, str> {
    let encoded_path = req.path();
    let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
    let encoded_path = strip_site_prefix(encoded_path, app_state);
    percent_encoding::percent_decode_str(encoded_path).decode_utf8_lossy()
}

fn redirect_missing_trailing_slash(uri: &Uri) -> Option<HttpResponse> {
    let path = uri.path();
    if !path.ends_with('/')
        && path
            .rsplit_once('.')
            .map(|(_, ext)| ext.eq_ignore_ascii_case("sql"))
            != Some(true)
    {
        let mut redirect_path = path.to_owned();
        redirect_path.push('/');
        if let Some(query) = uri.query() {
            redirect_path.push('?');
            redirect_path.push_str(query);
        }
        Some(
            HttpResponse::MovedPermanently()
                .insert_header((header::LOCATION, redirect_path))
                .finish(),
        )
    } else {
        None
    }
}

/// called when a request is made to a path outside of the sub-path we are serving the site from
async fn default_prefix_redirect(
    service_request: ServiceRequest,
) -> actix_web::Result<ServiceResponse> {
    let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
    let original_path = service_request.path();
    let site_prefix = &app_state.config.site_prefix;
    let redirect_path = site_prefix.trim_end_matches('/').to_string() + original_path;
    log::info!(
        "Received request to {original_path} (outside of site prefix {site_prefix}), redirecting to {redirect_path}"
    );
    Ok(service_request.into_response(
        HttpResponse::PermanentRedirect()
            .insert_header((header::LOCATION, redirect_path))
            .finish(),
    ))
}

pub fn create_app(
    app_state: web::Data<AppState>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<
            impl MessageBody<Error = impl std::fmt::Display + std::fmt::Debug>,
        >,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let encoded_scope: &str = app_state.config.site_prefix.trim_end_matches('/');
    let decoded_scope = percent_encoding::percent_decode_str(encoded_scope).decode_utf8_lossy();
    App::new()
        .service(
            web::scope(&decoded_scope)
                .service(static_content::js())
                .service(static_content::apexcharts_js())
                .service(static_content::tomselect_js())
                .service(static_content::css())
                .service(static_content::icons())
                .service(static_content::favicon())
                .default_service(fn_service(main_handler)),
        )
        // when receiving a request outside of the prefix, redirect to the prefix
        .default_service(fn_service(default_prefix_redirect))
        .wrap(Logger::default())
        .wrap(default_headers(&app_state))
        .wrap(middleware::Condition::new(
            app_state.config.compress_responses,
            middleware::Compress::default(),
        ))
        .wrap(middleware::NormalizePath::new(
            middleware::TrailingSlash::MergeOnly,
        ))
        .app_data(payload_config(&app_state))
        .app_data(form_config(&app_state))
        .app_data(app_state)
}

#[must_use]
pub fn form_config(app_state: &web::Data<AppState>) -> web::FormConfig {
    web::FormConfig::default()
        .limit(app_state.config.max_uploaded_file_size)
        .error_handler(|decode_err, _req| {
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
        })
}

#[must_use]
pub fn payload_config(app_state: &web::Data<AppState>) -> PayloadConfig {
    PayloadConfig::default().limit(app_state.config.max_uploaded_file_size * 2)
}

fn default_headers(app_state: &web::Data<AppState>) -> middleware::DefaultHeaders {
    let server_header = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let mut headers = middleware::DefaultHeaders::new().add(("Server", server_header));
    if let Some(csp) = &app_state.config.content_security_policy {
        headers = headers.add(("Content-Security-Policy", csp.as_str()));
    }
    headers
}

pub async fn run_server(config: &AppConfig, state: AppState) -> anyhow::Result<()> {
    let listen_on = config.listen_on();
    let state = web::Data::new(state);
    let final_state = web::Data::clone(&state);
    let factory = move || create_app(web::Data::clone(&state));

    #[cfg(feature = "lambda-web")]
    if lambda_web::is_running_on_lambda() {
        lambda_web::run_actix_on_lambda(factory)
            .await
            .map_err(|e| anyhow::anyhow!("Unable to start the lambda: {e}"))?;
        return Ok(());
    }
    let mut server = HttpServer::new(factory);
    if let Some(unix_socket) = &config.unix_socket {
        log::info!("Will start HTTP server on UNIX socket: {:?}", unix_socket);
        #[cfg(target_family = "unix")]
        {
            server = server
                .bind_uds(unix_socket)
                .map_err(|e| bind_unix_socket_err(e, unix_socket))?;
        }
        #[cfg(not(target_family = "unix"))]
        anyhow::bail!("Unix sockets are not supported on your operating system. Use listen_on instead of unix_socket.");
    } else {
        if let Some(domain) = &config.https_domain {
            let mut listen_on_https = listen_on;
            listen_on_https.set_port(443);
            log::debug!("Will start HTTPS server on {listen_on_https}");
            let config = make_auto_rustls_config(domain, config);
            server = server
                .bind_rustls_0_22(listen_on_https, config)
                .map_err(|e| bind_error(e, listen_on_https))?;
        } else if listen_on.port() == 443 {
            bail!("Please specify a value for https_domain in the configuration file. This is required when using HTTPS (port 443)");
        }
        if listen_on.port() != 443 {
            log::debug!("Will start HTTP server on {listen_on}");
            server = server
                .bind(listen_on)
                .map_err(|e| bind_error(e, listen_on))?;
        }
    }

    log_welcome_message(config);
    server
        .run()
        .await
        .with_context(|| "Unable to start the application")?;

    // We are done, we can close the database connection
    final_state.db.close().await?;
    Ok(())
}

fn log_welcome_message(config: &AppConfig) {
    let address_message = if let Some(unix_socket) = &config.unix_socket {
        format!("unix socket {unix_socket:?}")
    } else if let Some(domain) = &config.https_domain {
        format!("https://{domain}")
    } else {
        let listen_on = config.listen_on();
        let port = listen_on.port();
        let ip = listen_on.ip();
        if ip.is_unspecified() {
            format!(
                "http://localhost:{port}\n\
            (also accessible from other devices using your IP address)"
            )
        } else {
            format!("http://{ip}:{port}")
        }
    };

    let (sparkle, link, computer, rocket) = if cfg!(target_os = "windows") {
        ("", "", "", "")
    } else {
        ("✨", "🔗", "💻", "🚀")
    };
    let version = env!("CARGO_PKG_VERSION");
    let web_root = config.web_root.display();

    eprintln!(
        "{sparkle} SQLPage v{version} started successfully! {sparkle}\n\n\
        View your website at:\n{link} {address_message}\n\n\
        Create your pages with SQL files in:\n{computer} {web_root}\n\n\
        Happy coding! {rocket}"
    );
}

fn bind_error(e: std::io::Error, listen_on: std::net::SocketAddr) -> anyhow::Error {
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

#[cfg(target_family = "unix")]
fn bind_unix_socket_err(e: std::io::Error, unix_socket: &PathBuf) -> anyhow::Error {
    let ctx = if e.kind() == std::io::ErrorKind::PermissionDenied {
        format!(
            "You do not have permission to bind to the UNIX socket {unix_socket:?}. \
            You can change the socket path in the configuration file or check the permissions.",
        )
    } else {
        format!("Unable to bind to UNIX socket {unix_socket:?} {e:?}")
    };
    anyhow::anyhow!(e).context(ctx)
}
