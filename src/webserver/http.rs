//! This module handles HTTP requests and responses for the web server,
//! including rendering SQL files, serving static content, and managing
//! request contexts and response headers.

use crate::render::{AnyRenderBodyContext, HeaderContext, PageContext};
use crate::webserver::ErrorWithStatus;
use crate::webserver::content_security_policy::ContentSecurityPolicy;
use crate::webserver::database::execute_queries::stop_at_first_error;
use crate::webserver::database::{DbItem, execute_queries::stream_query_results_with_conn};
use crate::webserver::http_request_info::extract_request_info;
use crate::webserver::server_timing::ServerTiming;
use crate::{AppConfig, AppState, DEFAULT_404_FILE, ParsedSqlFile};
use actix_web::dev::{ServiceFactory, ServiceRequest, fn_service};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::http::header::Accept;
use actix_web::http::header::{ContentType, Header, HttpDate, IfModifiedSince, LastModified};
use actix_web::http::{StatusCode, header};
use actix_web::web::PayloadConfig;
use actix_web::{App, Error, HttpResponse, HttpServer, dev::ServiceResponse, middleware, web};
use opentelemetry_semantic_conventions::attribute as otel;
use tracing::{Instrument, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};

use super::error::{anyhow_err_to_actix, bind_error, send_anyhow_error};
use super::http_client::make_http_client;
use super::https::make_auto_rustls_config;
use super::oidc::OidcMiddleware;
use super::response_writer::ResponseWriter;
use super::static_content;
use crate::webserver::routing::RoutingAction::{
    CustomNotFound, Execute, NotFound, Redirect, Serve,
};
use crate::webserver::routing::{AppFileStore, calculate_route};
use actix_web::body::MessageBody;
use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use futures_util::stream::Stream;
use std::borrow::Cow;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseFormat {
    #[default]
    Html,
    Json,
    JsonLines,
}

#[derive(Clone)]
pub struct RequestContext {
    pub is_embedded: bool,
    pub source_path: PathBuf,
    pub content_security_policy: ContentSecurityPolicy,
    pub server_timing: Arc<ServerTiming>,
    pub response_format: ResponseFormat,
}

impl ResponseFormat {
    #[must_use]
    pub fn from_accept_header(accept: &Accept) -> Self {
        for quality_item in accept.iter() {
            let mime = &quality_item.item;
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();

            match (type_, subtype) {
                ("application", "json") => return Self::Json,
                ("application", "x-ndjson" | "jsonlines" | "x-jsonlines") => {
                    return Self::JsonLines;
                }
                ("text", "x-ndjson" | "jsonlines" | "x-jsonlines") => return Self::JsonLines,
                ("text", "html") | ("*", "*") => return Self::Html,
                _ => {}
            }
        }
        Self::Html
    }

    #[must_use]
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Html => "text/html; charset=utf-8",
            Self::Json => "application/json",
            Self::JsonLines => "application/x-ndjson",
        }
    }
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
            DbItem::Row(data) => {
                head_context.request_context.server_timing.record("row");
                head_context.handle_row(data).await?
            }
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
                return Err(source_err);
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
                return Ok(ResponseWithWriter::FinishedResponse { http_response });
            }
        }
    }
    log::debug!("No SQL statements left to execute for the body of the response");
    let http_response = head_context.close()?;
    Ok(ResponseWithWriter::FinishedResponse { http_response })
}

#[allow(clippy::large_enum_variant)]
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
    server_timing: ServerTiming,
) -> actix_web::Result<HttpResponse> {
    let app_state = srv_req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorInternalServerError("no state"))?
        .clone()
        .into_inner();

    let response_format = Accept::parse(srv_req)
        .map(|accept| ResponseFormat::from_accept_header(&accept))
        .unwrap_or_default();

    let exec_ctx = {
        let content_type = srv_req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let content_length = srv_req
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok());
        let url_query = srv_req.query_string();
        let url_query = if url_query.is_empty() {
            None
        } else {
            Some(url_query)
        };
        let parse_span = tracing::info_span!(
            "http.parse_request",
            http.request.method = %srv_req.method(),
            http.request.header.content_type = content_type,
            http.request.body.size = content_length,
            url.query = url_query,
        );
        extract_request_info(srv_req, Arc::clone(&app_state), server_timing)
            .instrument(parse_span)
            .await
            .map_err(|e| anyhow_err_to_actix(e, &app_state))?
    };
    log::debug!("Received a request with the following parameters: {exec_ctx:?}");

    exec_ctx.request().server_timing.record("parse_req");

    let (resp_send, resp_recv) = tokio::sync::oneshot::channel::<HttpResponse>();
    let source_path: PathBuf = sql_file.source_path.clone();
    let exec_span = tracing::info_span!(
        "sqlpage.file",
        otel.name = %sql_execution_span_name(&source_path),
        { otel::CODE_FILE_PATH } = %source_path.display(),
    );
    actix_web::rt::spawn(tracing::Instrument::instrument(
        async move {
            let request_info = exec_ctx.request();
            let request_context = RequestContext {
                is_embedded: request_info.url_params.contains_key("_sqlpage_embed"),
                source_path,
                content_security_policy: ContentSecurityPolicy::with_random_nonce(),
                server_timing: Arc::clone(&request_info.server_timing),
                response_format,
            };
            let mut conn = None;
            let database_entries_stream =
                stream_query_results_with_conn(&sql_file, &exec_ctx, &mut conn);
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
                    tracing::Instrument::instrument(
                        stream_response(database_entries_stream, renderer),
                        tracing::info_span!("render"),
                    )
                    .await;
                }
                Ok(ResponseWithWriter::FinishedResponse { http_response }) => {
                    resp_send
                        .send(http_response)
                        .unwrap_or_else(|e| log::error!("could not send headers {e:?}"));
                }
                Err(err) => {
                    send_anyhow_error(&err, resp_send, &app_state);
                }
            }
        },
        exec_span,
    ));
    resp_recv.await.map_err(ErrorInternalServerError)
}

fn request_span_route(request: &ServiceRequest) -> Cow<'_, str> {
    request
        .match_pattern()
        .map_or_else(|| request.path().to_owned().into(), Cow::from)
}

fn request_span_name(request: &ServiceRequest) -> String {
    format!("{} {}", request.method(), request_span_route(request))
}

fn sql_execution_span_name(source_path: &std::path::Path) -> String {
    format!("SQL {}", source_path.display())
}

struct SqlPageRootSpanBuilder;

impl RootSpanBuilder for SqlPageRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let user_agent = request
            .headers()
            .get("User-Agent")
            .map_or("", |h| h.to_str().unwrap_or(""));
        let http_route = request_span_route(request);
        let http_method =
            tracing_actix_web::root_span_macro::private::http_method_str(request.method());
        let otel_name = request_span_name(request);
        let connection_info = request.connection_info();
        let request_id = tracing_actix_web::root_span_macro::private::get_request_id(request);

        let span = tracing::span!(
            tracing::Level::INFO,
            "HTTP request",
            { otel::HTTP_REQUEST_METHOD } = %http_method,
            { otel::HTTP_ROUTE } = %http_route,
            { otel::NETWORK_PROTOCOL_NAME } = "http",
            { otel::NETWORK_PROTOCOL_VERSION } = %tracing_actix_web::root_span_macro::private::http_flavor(request.version()),
            { otel::URL_SCHEME } = %tracing_actix_web::root_span_macro::private::http_scheme(connection_info.scheme()),
            { otel::SERVER_ADDRESS } = %connection_info.host(),
            { otel::CLIENT_ADDRESS } = %request.connection_info().realip_remote_addr().unwrap_or(""),
            { otel::USER_AGENT_ORIGINAL } = %user_agent,
            { otel::URL_PATH } = %request.path(),
            { otel::URL_QUERY } = %request.query_string(),
            { otel::HTTP_RESPONSE_STATUS_CODE } = tracing::field::Empty,
            "otel.name" = %otel_name,
            "otel.kind" = "server",
            { otel::OTEL_STATUS_CODE } = tracing::field::Empty,
            request_id = %request_id,
            { otel::EXCEPTION_MESSAGE } = tracing::field::Empty,
            "exception.details" = tracing::field::Empty,
        );
        std::mem::drop(connection_info);
        tracing_actix_web::root_span_macro::private::set_otel_parent(request, &span);
        span
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        let span_ref = span.clone();
        DefaultRootSpanBuilder::on_request_end(span, outcome);

        // Emit a single log event per completed request so it appears in logs.
        let _enter = span_ref.enter();
        if let Ok(response) = outcome {
            let status = response.response().status();
            let level = if status.is_server_error() {
                log::Level::Error
            } else if status.is_client_error() {
                log::Level::Warn
            } else {
                log::Level::Info
            };
            log::log!(level, "{status}");
        }
    }
}

async fn process_sql_request(
    req: &mut ServiceRequest,
    sql_path: PathBuf,
) -> actix_web::Result<HttpResponse> {
    let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
    let server_timing = ServerTiming::for_env(app_state.config.environment);

    let sql_file = {
        let span = tracing::info_span!(
            "sqlpage.file.load",
            { otel::CODE_FILE_PATH } = %sql_path.display(),
        );
        app_state
            .sql_file_cache
            .get_with_privilege(app_state, &sql_path, false)
            .instrument(span)
            .await
            .with_context(|| format!("Unable to read SQL file \"{}\"", sql_path.display()))
            .map_err(|e| anyhow_err_to_actix(e, app_state))?
    };
    server_timing.record("sql_file");

    render_sql(req, sql_file, server_timing).await
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
            .map_err(|e| anyhow_err_to_actix(e, state))?;
        if !modified {
            return Ok(HttpResponse::NotModified().finish());
        }
    }
    state
        .file_system
        .read_file(state, path.as_ref(), false)
        .await
        .with_context(|| format!("Unable to read file {path:?}"))
        .map_err(|e| anyhow_err_to_actix(e, state))
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

pub async fn main_handler(
    mut service_request: ServiceRequest,
) -> actix_web::Result<ServiceResponse> {
    let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
    let store = AppFileStore::new(&app_state.sql_file_cache, &app_state.file_system, app_state);
    let path_and_query = service_request
        .uri()
        .path_and_query()
        .ok_or_else(|| ErrorBadRequest("expected valid path with query from request"))?;
    let routing_action = match calculate_route(path_and_query, &store, &app_state.config).await {
        Ok(action) => action,
        Err(e) => {
            let e = e.context(format!(
                "The server was unable to fulfill your request. \n\
                The following page is not accessible: {path_and_query:?}"
            ));
            return Err(anyhow_err_to_actix(e, app_state));
        }
    };
    match routing_action {
        NotFound => {
            let accept_header =
                header::Accept::parse(&service_request).unwrap_or(header::Accept::star());
            let prefers_html = accept_header.iter().any(|h| h.item.subtype() == "html");

            if prefers_html {
                let mut response =
                    process_sql_request(&mut service_request, PathBuf::from(DEFAULT_404_FILE))
                        .await?;
                *response.status_mut() = StatusCode::NOT_FOUND;
                Ok(response)
            } else {
                Ok(HttpResponse::NotFound()
                    .content_type(ContentType::plaintext())
                    .body("404 Not Found\n"))
            }
        }
        Execute(path) => process_sql_request(&mut service_request, path).await,
        CustomNotFound(path) => {
            // Currently, we do not set a 404 status when the user provides a fallback 404.sql file.
            process_sql_request(&mut service_request, path).await
        }
        Redirect(redirect_target) => Ok(HttpResponse::MovedPermanently()
            .insert_header((header::LOCATION, redirect_target))
            .finish()),
        Serve(path) => {
            let if_modified_since = IfModifiedSince::parse(&service_request).ok();
            let app_state: &web::Data<AppState> = service_request.app_data().expect("app_state");
            let path = path
                .as_os_str()
                .to_str()
                .ok_or_else(|| ErrorBadRequest("requested file path must be valid unicode"))?;
            serve_file(path, app_state, if_modified_since).await
        }
    }
    .map(|response| service_request.into_response(response))
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
                .service(static_content::favicon())
                .default_service(fn_service(main_handler)),
        )
        // when receiving a request outside of the prefix, redirect to the prefix
        .default_service(fn_service(default_prefix_redirect))
        .wrap(OidcMiddleware::new(&app_state))
        .wrap(super::http_metrics::HttpMetrics)
        .wrap(TracingLogger::<SqlPageRootSpanBuilder>::new())
        .wrap(default_headers())
        .wrap(middleware::Condition::new(
            app_state.config.compress_responses,
            middleware::Compress::default(),
        ))
        .wrap(middleware::NormalizePath::new(
            middleware::TrailingSlash::MergeOnly,
        ))
        .app_data(payload_config(&app_state))
        .app_data(make_http_client(&app_state.config))
        .app_data(form_config(&app_state))
        .app_data(app_state)
}

#[must_use]
pub fn form_config(app_state: &web::Data<AppState>) -> web::FormConfig {
    web::FormConfig::default()
        .limit(app_state.config.max_uploaded_file_size)
        .error_handler(super::error::handle_form_error)
}

#[must_use]
pub fn payload_config(app_state: &web::Data<AppState>) -> PayloadConfig {
    PayloadConfig::default().limit(app_state.config.max_uploaded_file_size * 2)
}

fn default_headers() -> middleware::DefaultHeaders {
    let server_header = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    middleware::DefaultHeaders::new().add(("Server", server_header))
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
    #[cfg_attr(
        not(target_family = "unix"),
        expect(
            clippy::redundant_else,
            reason = "Conditional compilation produces redundant else when not on unix targets."
        )
    )]
    if let Some(unix_socket) = &config.unix_socket {
        log::info!(
            "Will start HTTP server on UNIX socket: \"{}\"",
            unix_socket.display()
        );
        #[cfg(target_family = "unix")]
        {
            server = server
                .bind_uds(unix_socket)
                .map_err(|e| bind_unix_socket_err(e, unix_socket))?;
        }
        #[cfg(not(target_family = "unix"))]
        anyhow::bail!(
            "Unix sockets are not supported on your operating system. Use listen_on instead of unix_socket."
        );
    } else {
        if let Some(domain) = &config.https_domain {
            let mut listen_on_https = listen_on;
            listen_on_https.set_port(443);
            log::debug!("Will start HTTPS server on {listen_on_https}");
            let config = make_auto_rustls_config(domain, config);
            server = server
                .bind_rustls_0_23(listen_on_https, config)
                .map_err(|e| bind_error(e, listen_on_https))?;
        } else if listen_on.port() == 443 {
            bail!(
                "Please specify a value for https_domain in the configuration file. This is required when using HTTPS (port 443)"
            );
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
        format!("unix socket \"{}\"", unix_socket.display())
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
        } else if ip.is_ipv6() {
            format!("http://[{ip}]:{port}")
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

    log::info!(
        "\n{sparkle} SQLPage v{version} started successfully! {sparkle}\n\n\
        View your website at:\n{link} {address_message}\n\n\
        Create your pages with SQL files in:\n{computer} {web_root}\n\n\
        Happy coding! {rocket}"
    );
}

#[cfg(target_family = "unix")]
fn bind_unix_socket_err(e: std::io::Error, unix_socket: &std::path::Path) -> anyhow::Error {
    let ctx = if e.kind() == std::io::ErrorKind::PermissionDenied {
        format!(
            "You do not have permission to bind to the UNIX socket \"{}\". \
            You can change the socket path in the configuration file or check the permissions.",
            unix_socket.display()
        )
    } else {
        format!(
            "Unable to bind to UNIX socket \"{}\" {e:?}",
            unix_socket.display()
        )
    };
    anyhow::anyhow!(e).context(ctx)
}

#[cfg(test)]
mod tests {
    use super::{request_span_name, sql_execution_span_name};
    use actix_web::test::TestRequest;
    use std::path::Path;

    #[test]
    fn request_span_name_uses_request_path_when_no_matched_route_exists() {
        let request = TestRequest::with_uri("/todos/42?filter=open").to_srv_request();
        assert_eq!(request_span_name(&request), "GET /todos/42");
    }

    #[test]
    fn sql_execution_span_name_uses_sql_file_path() {
        assert_eq!(
            sql_execution_span_name(Path::new("website/todos.sql")),
            "SQL website/todos.sql"
        );
    }
}
