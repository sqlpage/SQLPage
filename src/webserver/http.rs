use crate::render::RenderContext;
use crate::webserver::database::{stream_query_results, DbItem};
use crate::{AppState, Config, CONFIG_DIR, WEB_ROOT};
use actix_web::dev::{Payload, ServiceRequest};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::Method;
use actix_web::web::{Data, Form};
use actix_web::{
    body::BodyStream, dev::Service, dev::ServiceResponse, http::header::CONTENT_TYPE,
    middleware::Logger, web, web::Bytes, App, FromRequest, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::{bail, Context};
use futures_util::StreamExt;
use futures_util::TryFutureExt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::mem;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
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
    async fn close_with_error(&mut self, msg: String) {
        if !self.response_bytes.is_closed() {
            let _ = self.async_flush().await;
            let _ = self
                .response_bytes
                .send(Err(ErrorInternalServerError(msg)))
                .await;
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
                use std::io::*;
                Error::new(
                    ErrorKind::BrokenPipe,
                    format!("The HTTP response writer with a capacity of {} has already been closed: {err}", MAX_PENDING_MESSAGES),
                )
            })
    }
}

impl Write for ResponseWriter {
    #[inline(always)]
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
    app_state: &AppState,
    req_param: RequestInfo,
    sql_bytes: Bytes,
    response_bytes: ResponseWriter,
) -> anyhow::Result<()> {
    log::debug!(
        "Received a request with the following parameters: {:?}",
        req_param
    );
    let mut stream = stream_query_results(&app_state.db, &sql_bytes, &req_param).await;

    let mut renderer = RenderContext::new(app_state, response_bytes);
    while let Some(item) = stream.next().await {
        let render_result = match item {
            DbItem::FinishedQuery => renderer.finish_query().await,
            DbItem::Row(row) => renderer.handle_row(&row),
            DbItem::Error(e) => renderer.handle_anyhow_error(&e),
        };
        if let Err(e) = render_result {
            if let Err(nested_err) = renderer.handle_anyhow_error(&e) {
                renderer
                    .close()
                    .close_with_error(nested_err.to_string())
                    .await;
                bail!(
                    "An error occurred while trying to display an other error. \
                    \nRoot error: {e}\n
                    \nNested error: {nested_err}"
                );
            }
        }
        renderer.writer.async_flush().await?;
    }
    renderer.close().async_flush().await?;
    log::debug!("Successfully finished rendering the page");
    Ok(())
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
                *old = Self::Vec(v)
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
            let entry = if !k.ends_with("[]") {
                SingleOrVec::Single(v)
            } else {
                k.replace_range(k.len() - 2.., "");
                SingleOrVec::Vec(vec![v])
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
        .map(|q| q.into_inner())
        .unwrap_or_default();
    let client_ip = req.peer_addr().map(|addr| addr.ip());
    let (http_req, payload) = req.parts_mut();
    let post_variables = Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(|form| form.into_inner())
        .unwrap_or_default();
    RequestInfo {
        headers: param_map(headers),
        get_variables: param_map(get_variables),
        post_variables: param_map(post_variables),
        client_ip,
    }
}

async fn render_sql(
    srv_req: &mut ServiceRequest,
    sql_bytes: Bytes,
) -> actix_web::Result<HttpResponse> {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_MESSAGES);
    let writer = ResponseWriter::new(sender);
    let req_param = extract_request_info(srv_req).await;
    let app_state = srv_req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorInternalServerError("no state"))?
        .clone(); // Cheap reference count increase
    actix_web::rt::spawn(async move {
        if let Err(err) = stream_response(&app_state, req_param, sql_bytes, writer).await {
            log::error!("Unable to serve page: {}", err);
        }
    });
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(BodyStream::new(
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )))
}

async fn postprocess_response(
    serv_resp: ServiceResponse,
    payload: Payload,
) -> actix_web::Result<ServiceResponse> {
    let (req, old_resp) = serv_resp.into_parts();
    let ctype = old_resp.headers().get(CONTENT_TYPE);
    if ctype.map(|ct| ct == "application/x-sql").unwrap_or(false) {
        let sql = actix_web::body::to_bytes(old_resp.into_body()).await?;
        let mut srv_req = ServiceRequest::from_parts(req, payload);
        let new_resp = render_sql(&mut srv_req, sql).await?;
        let old_req = srv_req.into_parts().0;
        Ok(ServiceResponse::new(old_req, new_resp))
    } else {
        Ok(ServiceResponse::new(req, old_resp))
    }
}

/// Resolves the path in a query to the path to a local SQL file if there is one that matches
fn path_to_sql_file(root: &Path, path: &str) -> Option<PathBuf> {
    let mut path_buf: PathBuf = PathBuf::from(root);
    path_buf.push(path.strip_prefix('/')?);
    if !path.ends_with(".sql") {
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

pub async fn run_server(config: Config, state: AppState) -> anyhow::Result<()> {
    let listen_on = config.listen_on;
    let app_state = web::Data::new(state);

    let factory = move || {
        App::new()
            .app_data(app_state.clone())
            .wrap_fn(|mut req, srv| {
                let app_state: &web::Data<AppState> = req.app_data().expect("app_state");
                if let Some(sql_path) = path_to_sql_file(&app_state.web_root, req.path()) {
                    let file = app_state.sql_file_cache.get(&app_state, &sql_path);
                }
                // Remove the payload from the request so that it can be used later by our sql service
                let payload = Payload::take(req.parts_mut().1);
                // Make all requests GET so that they can be served by the file server
                req.head_mut().method = Method::GET;
                srv.call(req).and_then(|resp| postprocess_response(resp, payload))
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
