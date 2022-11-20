use crate::render::{HeaderContext, PageContext, RenderContext};
use crate::webserver::database::{stream_query_results, DbItem};
use crate::{AppState, Config, ParsedSqlFile, CONFIG_DIR};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::web::Form;
use actix_web::{
    dev::Service, dev::ServiceResponse, middleware::Logger, web, web::Bytes, App, FromRequest,
    HttpResponse, HttpServer, Responder,
};

use crate::utils::log_error;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::mem;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

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
        log_error(&renderer.writer.async_flush().await);
    }
    log_error(&renderer.close().await.async_flush().await);
    log::debug!("Successfully finished rendering the page");
}

async fn build_response_header_and_stream<S: Stream<Item = DbItem>>(
    app_state: Arc<AppState>,
    database_entries: S,
) -> actix_web::Result<ResponseWithWriter<S>> {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_MESSAGES);
    let writer = ResponseWriter::new(sender);
    let mut head_context = HeaderContext::new(app_state, writer);
    let mut stream = Box::pin(database_entries);
    while let Some(item) = stream.next().await {
        match item {
            DbItem::Row(data) => {
                match head_context
                    .handle_row(data)
                    .await
                    .map_err(ErrorInternalServerError)?
                {
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
                }
            }
            DbItem::FinishedQuery => {
                log::debug!("finished query");
            }
            DbItem::Error(err) => {
                log::error!("An error occurred while preparing HTTP headers: {err}");
                return Err(ErrorInternalServerError(err));
            }
        }
    }
    Err(ErrorInternalServerError("no SQL statements to execute"))
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
                    .unwrap_or_else(|_| log::error!("could not send headers"));
                stream_response(database_entries_stream, renderer).await;
            }
            Err(e) => {
                let http_response = ErrorInternalServerError(e).into();
                resp_send
                    .send(http_response)
                    .unwrap_or_else(|_| log::error!("could not send headers"));
            }
        }
    });
    resp_recv.await.map_err(ErrorInternalServerError)
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
}

#[derive(Debug)]
pub struct RequestInfo {
    pub get_variables: ParamMap,
    pub post_variables: ParamMap,
    pub headers: ParamMap,
    pub client_ip: Option<IpAddr>,
}

fn param_map(values: Vec<(String, String)>) -> ParamMap {
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
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();
    let get_variables = web::Query::<Vec<(String, String)>>::from_query(req.query_string())
        .map(web::Query::into_inner)
        .unwrap_or_default();
    let client_ip = req.peer_addr().map(|addr| addr.ip());
    let (http_req, payload) = req.parts_mut();
    let post_variables = Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(Form::into_inner)
        .unwrap_or_default();
    RequestInfo {
        headers: param_map(headers),
        get_variables: param_map(get_variables),
        post_variables: param_map(post_variables),
        client_ip,
    }
}

/// Resolves the path in a query to the path to a local SQL file if there is one that matches
fn path_to_sql_file(root: &Path, path: &str) -> Option<PathBuf> {
    let mut path_buf: PathBuf = PathBuf::from(root);
    path_buf.push(path.strip_prefix('/')?);
    let request_path = PathBuf::from(path);
    let extension = request_path.extension();
    if !matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("sql")) {
        path_buf.push("index.sql");
        if !path_buf.is_file() {
            return None;
        }
    }
    let final_path = path_buf.canonicalize().ok()?;
    if final_path.starts_with(root) {
        Some(final_path)
    } else {
        None
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
        .map_err(ErrorInternalServerError)?;
    let response = render_sql(&mut req, sql_file).await?;
    Ok(req.into_response(response))
}

#[allow(clippy::unused_async)]
async fn handle_static_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript;charset=UTF-8")
        .append_header(CacheControl(vec![CacheDirective::MaxAge(3600u32)]))
        .body(&include_bytes!("../../sqlpage/sqlpage.js")[..])
}

pub async fn run_server(config: Config, state: AppState) -> anyhow::Result<()> {
    let listen_on = config.listen_on;
    let app_state = web::Data::new(state);

    let factory = move || {
        App::new()
            .app_data(app_state.clone())
            .route("sqlpage.js", actix_web::web::get().to(handle_static_js))
            .wrap_fn(|req, srv| {
                let app_state: web::Data<AppState> = web::Data::clone(req.app_data().expect("app_state"));
                let sql_file_path = path_to_sql_file(&app_state.web_root, req.path());
                let sql_file = if let Some(sql_path) = sql_file_path {
                    Ok((req, sql_path))
                } else {
                    Err(srv.call(req))
                };
                async move {
                    match sql_file {
                        Ok((req, sql_path)) => process_sql_request(req, sql_path).await,
                        Err(fallback) => fallback.await
                    }
                }
            })
            .default_service(
                actix_files::Files::new("/", &app_state.web_root)
                    .index_file("index.sql")
                    .path_filter(|path, _|
                        !matches!(path.components().next(), Some(Component::Normal(x)) if x == CONFIG_DIR))
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .wrap(Logger::default())
    };

    #[cfg(feature = "lambda-web")]
    if lambda_web::is_running_on_lambda() {
        lambda_web::run_actix_on_lambda(factory).await?;
        return Ok(());
    }
    HttpServer::new(factory).bind(listen_on)?.run().await?;
    Ok(())
}
