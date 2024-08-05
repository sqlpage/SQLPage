use crate::render::{HeaderContext, PageContext, RenderContext};
use crate::webserver::content_security_policy::ContentSecurityPolicy;
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
    dev::ServiceResponse, middleware, middleware::Logger, web, web::Bytes, App, HttpResponse,
    HttpServer,
};
use actix_web::{HttpResponseBuilder, ResponseError};

use super::https::make_auto_rustls_config;
use super::static_content;
use actix_web::body::MessageBody;
use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::io::Write;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

/// If the sending queue exceeds this number of outgoing messages, an error will be thrown
/// This prevents a single request from using up all available memory

#[derive(Clone)]
pub struct ResponseWriter {
    buffer: Vec<u8>,
    response_bytes: mpsc::Sender<actix_web::Result<Bytes>>,
}

#[derive(Clone)]
pub struct RequestContext {
    pub is_embedded: bool,
    pub content_security_policy: ContentSecurityPolicy,
}

impl ResponseWriter {
    fn new(response_bytes: mpsc::Sender<actix_web::Result<Bytes>>) -> Self {
        Self {
            response_bytes,
            buffer: Vec::new(),
        }
    }
    async fn close_with_error(&mut self, mut msg: String) {
        if !self.response_bytes.is_closed() {
            if let Err(e) = self.async_flush().await {
                msg.push_str(&format!("Unable to flush data: {e}"));
            }
            if let Err(e) = self
                .response_bytes
                .send(Err(ErrorInternalServerError(msg)))
                .await
            {
                log::error!("Unable to send error back to client: {e}");
            }
        }
    }

    async fn async_flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        log::trace!(
            "Async flushing data to client: {}",
            String::from_utf8_lossy(&self.buffer)
        );
        self.response_bytes
            .send(Ok(mem::take(&mut self.buffer).into()))
            .await
            .map_err(|err| {
                use std::io::{Error, ErrorKind};
                let capacity = self.response_bytes.capacity();
                Error::new(
                    ErrorKind::BrokenPipe,
                    format!("The HTTP response writer with a capacity of {capacity} has already been closed: {err}"),
                )
            })
    }
}

impl Write for ResponseWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        log::trace!(
            "Flushing data to client: {}",
            String::from_utf8_lossy(&self.buffer)
        );
        self.response_bytes
            .try_send(Ok(mem::take(&mut self.buffer).into()))
            .map_err(|e|
                std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    format!("{e}: Row limit exceeded. The server cannot store more than {} pending messages in memory. Try again later or increase max_pending_rows in the configuration.", self.response_bytes.max_capacity())
                )
            )
    }
}

impl Drop for ResponseWriter {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            log::error!("Could not flush data to client: {e}");
        }
    }
}

async fn stream_response(
    stream: impl Stream<Item = DbItem>,
    mut renderer: RenderContext<ResponseWriter>,
) {
    let mut stream = Box::pin(stream);

    if let Err(e) = &renderer.writer.async_flush().await {
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
        if let Err(e) = &renderer.writer.async_flush().await {
            log::error!(
                "Stopping rendering early because we were unable to flush data to client: {e:#}"
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
                let http_response = http_response.streaming(body_stream);
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
        renderer: RenderContext<ResponseWriter>,
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
        .map_err(anyhow_err_to_actix)?;
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

fn send_anyhow_error(
    e: &anyhow::Error,
    resp_send: tokio::sync::oneshot::Sender<HttpResponse>,
    env: app_config::DevOrProd,
) {
    log::error!("An error occurred before starting to send the response body: {e:#}");
    let mut resp = HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR);
    let mut body = "Sorry, but we were not able to process your request. \n\n".to_owned();
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
    let resp = if let Some(e @ &ErrorWithStatus { .. }) = e.downcast_ref() {
        e.error_response()
    } else if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        // People are HTTP connections faster than we can open SQL connections. Ask them to slow down politely.
        use rand::Rng;
        resp.status(StatusCode::SERVICE_UNAVAILABLE).insert_header((
            header::RETRY_AFTER,
            header::HeaderValue::from(rand::thread_rng().gen_range(1..=15)),
        )).body("The database is currently too busy to handle your request. Please try again later.\n\n".to_owned() + &body)
    } else {
        resp.body(body)
    };
    resp_send
        .send(resp)
        .unwrap_or_else(|_| log::error!("could not send headers"));
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
    mut req: ServiceRequest,
    sql_path: PathBuf,
) -> actix_web::Result<ServiceResponse> {
    let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
    let sql_file = app_state
        .sql_file_cache
        .get_with_privilege(app_state, &sql_path, false)
        .await
        .with_context(|| format!("Unable to get SQL file {sql_path:?}"))
        .map_err(anyhow_err_to_actix)?;
    let response = render_sql(&mut req, sql_file).await?;
    Ok(req.into_response(response))
}

fn anyhow_err_to_actix(e: anyhow::Error) -> actix_web::Error {
    log::error!("{e:#}");
    match e.downcast::<ErrorWithStatus>() {
        Ok(err) => actix_web::Error::from(err),
        Err(e) => ErrorInternalServerError(format!(
            "An error occurred while trying to handle your request: {e:#}"
        )),
    }
}

async fn serve_file(
    path: &str,
    state: &AppState,
    if_modified_since: Option<IfModifiedSince>,
) -> actix_web::Result<HttpResponse> {
    let path = path.strip_prefix(&state.config.site_prefix).unwrap_or(path);
    if let Some(IfModifiedSince(date)) = if_modified_since {
        let since = DateTime::<Utc>::from(SystemTime::from(date));
        let modified = state
            .file_system
            .modified_since(state, path.as_ref(), since, false)
            .await
            .with_context(|| format!("Unable to get modification time of file {path:?}"))
            .map_err(anyhow_err_to_actix)?;
        if !modified {
            return Ok(HttpResponse::NotModified().finish());
        }
    }
    state
        .file_system
        .read_file(state, path.as_ref(), false)
        .await
        .with_context(|| format!("Unable to read file {path:?}"))
        .map_err(anyhow_err_to_actix)
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

pub async fn main_handler(
    mut service_request: ServiceRequest,
) -> actix_web::Result<ServiceResponse> {
    let path = req_path(&service_request);
    let sql_file_path = path_to_sql_file(&path);
    if let Some(sql_path) = sql_file_path {
        if let Some(redirect) = redirect_missing_trailing_slash(service_request.uri()) {
            return Ok(service_request.into_response(redirect));
        }
        log::debug!("Processing SQL request: {:?}", sql_path);
        process_sql_request(service_request, sql_path).await
    } else {
        log::debug!("Serving file: {:?}", path);
        let app_state = service_request.extract::<web::Data<AppState>>().await?;
        let path = req_path(&service_request);
        let if_modified_since = IfModifiedSince::parse(&service_request).ok();
        let response = serve_file(&path, &app_state, if_modified_since).await?;
        Ok(service_request.into_response(response))
    }
}

/// Extracts the path from a request and percent-decodes it
fn req_path(req: &ServiceRequest) -> Cow<'_, str> {
    let encoded_path = req.path();
    let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
    let encoded_path = encoded_path
        .strip_prefix(&app_state.config.site_prefix)
        .unwrap_or(encoded_path);
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
    let redirect_path = app_state
        .config
        .site_prefix
        .trim_end_matches('/')
        .to_string()
        + service_request.path();
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
        .app_data(PayloadConfig::default().limit(app_state.config.max_uploaded_file_size * 2))
        .app_data(app_state)
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
    server.run().await.with_context(|| "Unable to start the application")
}

fn log_welcome_message(config: &AppConfig) {
    let address_message = if let Some(unix_socket) = &config.unix_socket {
        format!("unix socket {unix_socket:?}")
    } else if let Some(domain) = &config.https_domain {
        format!("https://{domain}")
    } else {
        use std::fmt::Write;
        let listen_on = config.listen_on();
        let mut msg = format!("{listen_on}");
        if listen_on.ip().is_unspecified() {
            // let the user know the service is publicly accessible
            write!(
                msg,
                ": accessible from the network, and locally on http://localhost:{}",
                listen_on.port()
            )
            .unwrap();
        }
        msg
    };

    log::info!(
        "SQLPage v{} started successfully.
    Now listening on {}
    You can write your website's code in .sql files in {}",
        env!("CARGO_PKG_VERSION"),
        address_message,
        config.web_root.display()
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
