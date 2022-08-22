use crate::render::RenderContext;
use crate::{AppState, CONFIG_DIR, WEB_ROOT};
use actix_web::dev::Payload;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::Method;
use actix_web::web::Form;
use actix_web::{
    body::BodyStream, dev::Service, dev::ServiceResponse, http::header::CONTENT_TYPE,
    middleware::Logger, web, web::Bytes, App, FromRequest, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::{bail, Context};
use futures_util::StreamExt;
use futures_util::TryFutureExt;
use sqlx::any::AnyArguments;
use sqlx::{Arguments, Column, Database, Decode, Row};
use std::io::Write;
use std::mem;
use std::path::Component;
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_json::json;
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
    fn close_with_error(&mut self, msg: String) {
        if !self.response_bytes.is_closed() {
            let _ = self.flush();
            let _ = self
                .response_bytes
                .try_send(Err(ErrorInternalServerError(msg)));
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
    req: HttpRequest,
    payload: Payload,
    sql_bytes: Bytes,
    response_bytes: ResponseWriter,
) -> anyhow::Result<()> {
    let app_state: &web::Data<AppState> = req.app_data().context("no app data in render")?;
    let sql = std::str::from_utf8(&sql_bytes)?;
    let mut arguments = AnyArguments::default();
    arguments.add(request_argument_json(&req, payload).await);
    let mut stream = sqlx::query_with(sql, arguments).fetch_many(&app_state.db);

    let mut renderer = RenderContext::new(app_state, response_bytes);
    while let Some(item) = stream.next().await {
        let render_result = match item {
            Ok(sqlx::Either::Left(result)) => renderer.finish_query(result).await,
            Ok(sqlx::Either::Right(row)) => renderer.handle_row(json!(SerializeRow(row))).await,
            Err(e) => renderer.handle_error(&e),
        };
        if let Err(e) = render_result {
            if let Err(nested_err) = renderer.handle_error(&e.root_cause()) {
                renderer.close().close_with_error(nested_err.to_string());
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


struct SerializeRow<R: Row>(R);

impl<'r, R: Row> Serialize for &'r SerializeRow<R>
    where
        usize: sqlx::ColumnIndex<R>,
        &'r str: sqlx::Decode<'r, <R as Row>::Database>,
        f64: sqlx::Decode<'r, <R as Row>::Database>,
        i64: sqlx::Decode<'r, <R as Row>::Database>,
        bool: sqlx::Decode<'r, <R as Row>::Database>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        use sqlx::{TypeInfo, ValueRef};
        let columns = self.0.columns();
        let mut map = serializer.serialize_map(Some(columns.len()))?;
        for col in columns {
            let key = col.name();
            match self.0.try_get_raw(col.ordinal()) {
                Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                    "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                        map_serialize::<_, _, f64>(&mut map, key, raw_value)
                    }
                    "INT" | "INTEGER" | "INT8" | "INT2" | "INT4" | "TINYINT" | "SMALLINT"
                    | "BIGINT" => map_serialize::<_, _, i64>(&mut map, key, raw_value),
                    "BOOL" | "BOOLEAN" => map_serialize::<_, _, bool>(&mut map, key, raw_value),
                    // Deserialize as a string by default
                    _ => map_serialize::<_, _, &str>(&mut map, key, raw_value),
                },
                _ => map.serialize_entry(key, &()), // Serialize null
            }?
        }
        map.end()
    }
}

fn map_serialize<'r, M: SerializeMap, DB: Database, T: Decode<'r, DB> + Serialize>(
    map: &mut M,
    key: &str,
    raw_value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
) -> Result<(), M::Error> {
    let val = T::decode(raw_value).map_err(serde::ser::Error::custom)?;
    map.serialize_entry(key, &val)
}

async fn request_argument_json(req: &HttpRequest, mut payload: Payload) -> String {
    let headers: serde_json::Map<String, serde_json::Value> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                serde_json::Value::String(String::from_utf8_lossy(value.as_bytes()).to_string()),
            )
        })
        .collect();
    let query = web::Query::<serde_json::Value>::from_query(req.query_string())
        .map(|q| q.into_inner())
        .unwrap_or_default();
    let client_ip = req.peer_addr().map(|addr| addr.ip());
    let form = Form::<serde_json::Value>::from_request(req, &mut payload)
        .await
        .map(|form| form.into_inner())
        .unwrap_or_default();
    serde_json::json!({
        "headers": headers,
        "client_ip": client_ip,
        "query": query,
        "form": form
    })
        .to_string()
}

async fn render_sql(
    req: HttpRequest,
    payload: Payload,
    sql_bytes: Bytes,
) -> actix_web::Result<HttpResponse> {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_MESSAGES);
    let writer = ResponseWriter::new(sender);
    actix_web::rt::spawn(async {
        if let Err(err) = stream_response(req, payload, sql_bytes, writer).await {
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
    let new_resp = if ctype.map(|ct| ct == "application/x-sql").unwrap_or(false) {
        let sql = actix_web::body::to_bytes(old_resp.into_body()).await?;
        render_sql(req.clone(), payload, sql).await?
    } else {
        old_resp
    };
    Ok(ServiceResponse::new(req, new_resp))
}

pub async fn run_server(state: AppState) -> std::io::Result<()> {
    let listen_on = state.listen_on;
    let app_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap_fn(|mut req, srv| {
                // Remove the payload from the request so that it can be used later by our sql service
                let payload = Payload::take(req.parts_mut().1);
                // Make all requests GET so that they can be served by the file server
                req.head_mut().method = Method::GET;
                srv.call(req).and_then(|resp| postprocess_response(resp, payload))
            })
            .default_service(
                actix_files::Files::new("/", WEB_ROOT)
                    .index_file("index.sql")
                    .path_filter(|path, _|
                        !matches!(path.components().next(), Some(Component::Normal(x)) if x == CONFIG_DIR))
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .wrap(Logger::default())
    })
        .bind(listen_on)?
        .run()
        .await
}
