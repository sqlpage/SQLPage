use crate::render::{HeaderContext, PageContext, RenderContext};
use crate::webserver::database::{stream_query_results, DbItem};
use crate::webserver::ErrorWithStatus;
use crate::{AppState, Config, ParsedSqlFile};
use actix_web::dev::{fn_service, ServiceFactory, ServiceRequest};
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::http::header::{ContentType, Header, HttpDate, IfModifiedSince, LastModified};
use actix_web::http::{header, StatusCode};
use actix_web::web::Form;
use actix_web::{
    dev::ServiceResponse, middleware, middleware::Logger, web, web::Bytes, App, FromRequest,
    HttpResponse, HttpServer,
};

use actix_web::body::{BoxBody, MessageBody};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use anyhow::Context;
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::mem;
use std::net::IpAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

use super::static_content;

/// If the sending queue exceeds this number of outgoing messages, an error will be thrown
/// This prevents a single request from using up all available memory
const MAX_PENDING_MESSAGES: usize = 128;

#[derive(Clone)]
pub struct ResponseWriter {
    buffer: Vec<u8>,
    response_bytes: mpsc::Sender<actix_web::Result<Bytes>>,
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
        self.response_bytes
            .send(Ok(mem::take(&mut self.buffer).into()))
            .await
            .map_err(|err| {
                use std::io::{Error, ErrorKind};
                Error::new(
                    ErrorKind::BrokenPipe,
                    format!("The HTTP response writer with a capacity of {MAX_PENDING_MESSAGES} has already been closed: {err}"),
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
        self.response_bytes
            .try_send(Ok(mem::take(&mut self.buffer).into()))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::WouldBlock, e.to_string()))
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
) -> anyhow::Result<ResponseWithWriter<S>> {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_MESSAGES);
    let writer = ResponseWriter::new(sender);
    let mut head_context = HeaderContext::new(app_state, writer);
    let mut stream = Box::pin(database_entries);
    while let Some(item) = stream.next().await {
        match item {
            DbItem::Row(data) => match head_context.handle_row(data).await? {
                PageContext::Header(h) => {
                    head_context = h;
                }
                PageContext::Body {
                    mut http_response,
                    renderer,
                } => {
                    let body_stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
                    let http_response = http_response.streaming(body_stream);
                    return Ok(ResponseWithWriter {
                        http_response,
                        renderer,
                        database_entries_stream: stream,
                    });
                }
                PageContext::Close(h) => {
                    head_context = h;
                    break;
                }
            },
            DbItem::FinishedQuery => log::debug!("finished query"),
            DbItem::Error(source_err) => return Err(source_err),
        }
    }
    log::debug!("No SQL statements left to execute for the body of the response");
    let (renderer, http_response) = head_context.close().await?;
    Ok(ResponseWithWriter {
        http_response,
        renderer,
        database_entries_stream: stream,
    })
}

struct ResponseWithWriter<S> {
    http_response: HttpResponse,
    renderer: RenderContext<ResponseWriter>,
    database_entries_stream: Pin<Box<S>>,
}

async fn render_sql(
    srv_req: &mut ServiceRequest,
    sql_file: Arc<ParsedSqlFile>,
) -> actix_web::Result<HttpResponse> {
    let req_param = extract_request_info(srv_req).await;
    log::debug!("Received a request with the following parameters: {req_param:?}");
    let app_state = srv_req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorInternalServerError("no state"))?
        .clone()
        .into_inner(); // Cheap reference count increase

    let (resp_send, resp_recv) = tokio::sync::oneshot::channel::<HttpResponse>();
    actix_web::rt::spawn(async move {
        let database_entries_stream =
            stream_query_results(&app_state.db, &sql_file, &req_param).await;
        let response_with_writer =
            build_response_header_and_stream(Arc::clone(&app_state), database_entries_stream).await;
        match response_with_writer {
            Ok(ResponseWithWriter {
                http_response,
                renderer,
                database_entries_stream,
            }) => {
                resp_send
                    .send(http_response)
                    .unwrap_or_else(|e| log::error!("could not send headers {e:?}"));
                stream_response(database_entries_stream, renderer).await;
            }
            Err(err) => {
                send_anyhow_error(&err, resp_send);
            }
        }
    });
    resp_recv.await.map_err(ErrorInternalServerError)
}

fn send_anyhow_error(e: &anyhow::Error, resp_send: tokio::sync::oneshot::Sender<HttpResponse>) {
    log::error!("An error occurred before starting to send the response body: {e:#}");
    let mut resp = HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR).set_body(BoxBody::new(
        format!("Sorry, but we were not able to process your request. \n\nError:\n\n {e:?}"),
    ));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain"),
    );
    if let Some(&ErrorWithStatus { status }) = e.downcast_ref() {
        *resp.status_mut() = status;
        if status == StatusCode::UNAUTHORIZED {
            resp.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                header::HeaderValue::from_static(
                    "Basic realm=\"Authentication required\", charset=\"UTF-8\"",
                ),
            );
            resp = resp.set_body(BoxBody::new(
                "Sorry, but you are not authorized to access this page.",
            ));
        }
    };
    if let Some(sqlx::Error::PoolTimedOut) = e.downcast_ref() {
        // People are HTTP connections faster than we can open SQL connections. Ask them to slow down politely.
        use rand::Rng;
        *resp.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
        resp.headers_mut().insert(
            header::RETRY_AFTER,
            header::HeaderValue::from(rand::thread_rng().gen_range(1..=15)),
        );
    }
    resp_send
        .send(resp)
        .unwrap_or_else(|_| log::error!("could not send headers"));
}

type ParamMap = HashMap<String, SingleOrVec>;

#[derive(Debug)]
pub enum SingleOrVec {
    Single(String),
    Vec(Vec<String>),
}

impl SingleOrVec {
    fn merge(&mut self, other: Self) {
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

    pub fn as_json_str(&self) -> Cow<'_, str> {
        match self {
            SingleOrVec::Single(x) => Cow::Borrowed(x),
            SingleOrVec::Vec(v) => Cow::Owned(serde_json::to_string(v).unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct RequestInfo {
    pub get_variables: ParamMap,
    pub post_variables: ParamMap,
    pub headers: ParamMap,
    pub client_ip: Option<IpAddr>,
    pub cookies: ParamMap,
    pub basic_auth: Option<Basic>,
}

fn param_map<PAIRS: IntoIterator<Item = (String, String)>>(values: PAIRS) -> ParamMap {
    values
        .into_iter()
        .fold(HashMap::new(), |mut map, (mut k, v)| {
            let entry = if k.ends_with("[]") {
                k.replace_range(k.len() - 2.., "");
                SingleOrVec::Vec(vec![v])
            } else {
                SingleOrVec::Single(v)
            };
            match map.entry(k) {
                Entry::Occupied(mut s) => {
                    SingleOrVec::merge(s.get_mut(), entry);
                }
                Entry::Vacant(v) => {
                    v.insert(entry);
                }
            }
            map
        })
}

async fn extract_request_info(req: &mut ServiceRequest) -> RequestInfo {
    let (http_req, payload) = req.parts_mut();
    let post_variables = Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(Form::into_inner)
        .unwrap_or_default();

    let headers = req.headers().iter().map(|(name, value)| {
        (
            name.to_string(),
            String::from_utf8_lossy(value.as_bytes()).to_string(),
        )
    });
    let get_variables = web::Query::<Vec<(String, String)>>::from_query(req.query_string())
        .map(web::Query::into_inner)
        .unwrap_or_default();
    let client_ip = req.peer_addr().map(|addr| addr.ip());

    let raw_cookies = req.cookies();
    let cookies = raw_cookies
        .iter()
        .flat_map(|c| c.iter())
        .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()));

    let basic_auth = Authorization::<Basic>::parse(req)
        .ok()
        .map(Authorization::into_scheme);

    RequestInfo {
        headers: param_map(headers),
        get_variables: param_map(get_variables),
        post_variables: param_map(post_variables),
        client_ip,
        cookies: param_map(cookies),
        basic_auth,
    }
}

/// Resolves the path in a query to the path to a local SQL file if there is one that matches
fn path_to_sql_file(path: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(path.strip_prefix('/').unwrap_or(path));
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
        .get(app_state, &sql_path)
        .await
        .map_err(|e| {
            log::error!("Error while trying to get SQL file: {:#}", e);
            if e.downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
            {
                ErrorNotFound("The requested file was not found.")
            } else {
                ErrorInternalServerError(format!(
                    "An error occurred while trying to handle your request: {e:#}"
                ))
            }
        })?;
    let response = render_sql(&mut req, sql_file).await?;
    Ok(req.into_response(response))
}

async fn serve_file(
    path: &str,
    state: &AppState,
    if_modified_since: Option<IfModifiedSince>,
) -> actix_web::Result<HttpResponse> {
    let path = path.strip_prefix('/').unwrap_or(path);
    if let Some(IfModifiedSince(date)) = if_modified_since {
        let since = DateTime::<Utc>::from(SystemTime::from(date));
        let modified = state
            .file_system
            .modified_since(state, path.as_ref(), since)
            .await
            .map_err(actix_web::error::ErrorBadRequest)?;
        if !modified {
            return Ok(HttpResponse::NotModified().finish());
        }
    }
    state
        .file_system
        .read_file(state, path.as_ref())
        .await
        .map_err(actix_web::error::ErrorBadRequest)
        .map(|b| {
            HttpResponse::Ok()
                .insert_header(
                    mime_guess::from_path(path)
                        .first()
                        .map(ContentType)
                        .unwrap_or(ContentType::octet_stream()),
                )
                .insert_header(LastModified(HttpDate::from(SystemTime::now())))
                .body(b)
        })
}

async fn main_handler(mut service_request: ServiceRequest) -> actix_web::Result<ServiceResponse> {
    let path = req_path(&service_request);
    let sql_file_path = path_to_sql_file(&path);
    if let Some(sql_path) = sql_file_path {
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
    percent_encoding::percent_decode_str(encoded_path).decode_utf8_lossy()
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
    App::new()
        .service(static_content::js())
        .service(static_content::css())
        .default_service(fn_service(main_handler))
        .wrap(Logger::default())
        .wrap(
            middleware::DefaultHeaders::new()
                .add((
                    "Server",
                    format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
                ))
                .add((
                    "Content-Security-Policy",
                    "script-src 'self' https://cdn.jsdelivr.net",
                )),
        )
        .wrap(middleware::Compress::default())
        .app_data(app_state)
}

pub async fn run_server(config: Config, state: AppState) -> anyhow::Result<()> {
    let listen_on = config.listen_on;
    let state = web::Data::new(state);
    let factory = move || create_app(web::Data::clone(&state));

    #[cfg(feature = "lambda-web")]
    if lambda_web::is_running_on_lambda() {
        lambda_web::run_actix_on_lambda(factory)
            .await
            .map_err(|e| anyhow::anyhow!("Unable to start the lambda: {e}"))?;
        return Ok(());
    }
    HttpServer::new(factory)
        .bind(listen_on)
        .with_context(|| "Unable to listen to the specified port")?
        .run()
        .await
        .with_context(|| "Unable to start the application")?;
    Ok(())
}
