use crate::render::RenderContext;
use crate::AppState;
use actix_web::{
    body::BodyStream, dev::Service, dev::ServiceResponse, http::header::CONTENT_TYPE,
    middleware::Logger, web, web::Bytes, App, HttpRequest, HttpResponse, HttpServer,
};
use futures_util::StreamExt;
use futures_util::TryFutureExt;

const WEB_ROOT: &str = ".";

pub struct ResponseWriter {
    response_bytes: tokio::sync::mpsc::UnboundedSender<actix_web::Result<Bytes>>,
}

impl std::io::Write for &ResponseWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.response_bytes
            .send(Ok(Bytes::copy_from_slice(buf)))
            .map(|_| buf.len())
            .map_err(|_err| {
                use std::io::*;
                Error::new(ErrorKind::BrokenPipe, "The HTTP response writer has already been closed")
            })
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

async fn stream_response(req: HttpRequest, sql_bytes: Bytes, response_bytes: ResponseWriter) -> std::io::Result<()> {
    let app_state: &web::Data<AppState> = req.app_data().expect("no app data in render");
    let sql = std::str::from_utf8(&sql_bytes).unwrap();
    let mut stream = sqlx::query(sql).fetch_many(&app_state.db);

    let mut renderer = RenderContext::new(app_state, response_bytes);
    while let Some(item) = stream.next().await {
        renderer.handle_result(&item)?;
        let res = match item {
            Ok(sqlx::Either::Left(result)) => renderer.finish_query(result).await,
            Ok(sqlx::Either::Right(row)) => renderer.handle_row(row).await,
            Err(_) => Ok(()),
        };
        renderer.handle_result(&res)?;
    }
    Ok(())
}

async fn render_sql(req: HttpRequest, sql_bytes: Bytes) -> actix_web::Result<HttpResponse> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let writer = ResponseWriter {
        response_bytes: sender,
    };
    actix_web::rt::spawn(stream_response(req, sql_bytes, writer));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(BodyStream::new(
            tokio_stream::wrappers::UnboundedReceiverStream::new(receiver),
        )))
}

async fn postprocess_response(serv_resp: ServiceResponse) -> actix_web::Result<ServiceResponse> {
    let (req, old_resp) = serv_resp.into_parts();
    let ctype = old_resp.headers().get(CONTENT_TYPE);
    let new_resp = if ctype.map(|ct| ct == "application/x-sql").unwrap_or(false) {
        let sql = actix_web::body::to_bytes(old_resp.into_body()).await?;
        render_sql(req.clone(), sql).await?
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
            .wrap_fn(|req, srv| srv.call(req).and_then(postprocess_response))
            .default_service(
                actix_files::Files::new("/", WEB_ROOT)
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .wrap(Logger::default())
    })
        .bind(listen_on)?
        .run()
        .await
}
