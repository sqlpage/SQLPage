use std::{fmt, future::poll_fn, net::SocketAddr};

use actix_service::{IntoServiceFactory, Service, ServiceFactory};
use actix_web::{body::MessageBody, dev::AppConfig, web::Bytes};
use http_body_util::Full;
use lambda_http::{Request, RequestExt, Response, request::RequestContext, service_fn};
use tokio::sync::{mpsc, oneshot};

type LambdaResponse = Response<Full<Bytes>>;
type LambdaResult = Result<LambdaResponse, lambda_http::Error>;
type LambdaRequest = (Request, oneshot::Sender<LambdaResult>);

pub(super) fn is_running_on_lambda() -> bool {
    std::env::var_os("AWS_LAMBDA_RUNTIME_API").is_some()
}

pub(super) async fn run<F, I, S, B>(factory: F) -> Result<(), lambda_http::Error>
where
    F: Fn() -> I + Send + Clone + 'static,
    I: IntoServiceFactory<S, actix_http::Request>,
    S: ServiceFactory<
            actix_http::Request,
            Config = AppConfig,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        > + 'static,
    S::InitError: fmt::Debug,
    B: MessageBody + 'static,
    B::Error: fmt::Display,
{
    let service = factory()
        .into_factory()
        .new_service(AppConfig::default())
        .await
        .map_err(|error| lambda_error(format!("failed to initialize Actix service: {error:?}")))?;
    let (request_sender, mut request_receiver) = mpsc::channel::<LambdaRequest>(1);

    actix_web::rt::spawn(async move {
        while let Some((request, response_sender)) = request_receiver.recv().await {
            let result = handle_request(&service, request).await;
            let _ = response_sender.send(result);
        }
    });

    lambda_http::run(service_fn(move |request| {
        let request_sender = request_sender.clone();
        async move {
            let (response_sender, response_receiver) = oneshot::channel();
            request_sender
                .send((request, response_sender))
                .await
                .map_err(|_| lambda_error("Actix Lambda worker is unavailable"))?;
            response_receiver
                .await
                .map_err(|_| lambda_error("Actix Lambda worker dropped the response"))?
        }
    }))
    .await
}

async fn handle_request<S, B>(service: &S, request: Request) -> LambdaResult
where
    S: Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<B>,
            Error = actix_web::Error,
        >,
    B: MessageBody,
    B::Error: fmt::Display,
{
    let request = to_actix_request(request)?;
    poll_fn(|context| service.poll_ready(context))
        .await
        .map_err(lambda_error)?;
    let response = service.call(request).await.map_err(lambda_error)?;
    to_lambda_response(response).await
}

fn to_actix_request(request: Request) -> Result<actix_http::Request, lambda_http::Error> {
    let peer_addr = source_ip(&request).map(|ip| SocketAddr::new(ip, 0));
    let (parts, body) = request.into_parts();
    let mut request = actix_http::Request::new();
    let head = request.head_mut();

    head.method = actix_web::http::Method::from_bytes(parts.method.as_str().as_bytes())
        .map_err(lambda_error)?;
    let path_and_query = parts
        .uri
        .path_and_query()
        .map_or("/", lambda_http::http::uri::PathAndQuery::as_str);
    head.uri = path_and_query.parse().map_err(lambda_error)?;
    head.version = match parts.version {
        lambda_http::http::Version::HTTP_09 => actix_web::http::Version::HTTP_09,
        lambda_http::http::Version::HTTP_10 => actix_web::http::Version::HTTP_10,
        lambda_http::http::Version::HTTP_2 => actix_web::http::Version::HTTP_2,
        lambda_http::http::Version::HTTP_3 => actix_web::http::Version::HTTP_3,
        _ => actix_web::http::Version::HTTP_11,
    };
    head.peer_addr = peer_addr.or_else(|| forwarded_peer_addr(&parts.headers));

    for (name, value) in &parts.headers {
        let name = actix_web::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
            .map_err(lambda_error)?;
        let value = actix_web::http::header::HeaderValue::from_bytes(value.as_bytes())
            .map_err(lambda_error)?;
        head.headers.append(name, value);
    }
    *request.payload() = actix_http::Payload::from(Bytes::copy_from_slice(body.as_ref()));
    Ok(request)
}

async fn to_lambda_response<B>(response: actix_web::dev::ServiceResponse<B>) -> LambdaResult
where
    B: MessageBody,
    B::Error: fmt::Display,
{
    let status = lambda_http::http::StatusCode::from_u16(response.status().as_u16())
        .map_err(lambda_error)?;
    let mut result = Response::builder()
        .status(status)
        .body(Full::new(Bytes::new()))?;

    for (name, value) in response.headers() {
        let name = lambda_http::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
            .map_err(lambda_error)?;
        let value = lambda_http::http::header::HeaderValue::from_bytes(value.as_bytes())
            .map_err(lambda_error)?;
        result.headers_mut().append(name, value);
    }

    let body = actix_web::body::to_bytes(response.into_body())
        .await
        .map_err(lambda_error)?;
    *result.body_mut() = Full::new(body);
    Ok(result)
}

fn source_ip(request: &Request) -> Option<std::net::IpAddr> {
    let source_ip = match request.request_context_ref()? {
        RequestContext::ApiGatewayV1(context) => context.identity.source_ip.as_deref(),
        RequestContext::ApiGatewayV2(context) => context.http.source_ip.as_deref(),
        RequestContext::WebSocket(context) => context.identity.source_ip.as_deref(),
        _ => None,
    }?;
    source_ip.parse().ok()
}

fn forwarded_peer_addr(headers: &lambda_http::http::HeaderMap) -> Option<SocketAddr> {
    let forwarded_for = headers.get(lambda_http::http::header::HeaderName::from_static(
        "x-forwarded-for",
    ))?;
    let ip = forwarded_for
        .to_str()
        .ok()?
        .split(',')
        .next()?
        .trim()
        .parse()
        .ok()?;
    Some(SocketAddr::new(ip, 0))
}

fn lambda_error(error: impl fmt::Display) -> lambda_http::Error {
    std::io::Error::other(error.to_string()).into()
}

#[cfg(test)]
mod tests {
    use actix_web::{HttpMessage, HttpResponse};
    use futures_util::StreamExt;
    use lambda_http::http::header::{HeaderName, HeaderValue};

    use super::*;

    #[actix_web::test]
    async fn converts_lambda_request_without_losing_duplicate_headers() {
        let mut request = lambda_http::http::Request::builder()
            .method("POST")
            .uri("/path?one=1&one=2")
            .body(lambda_http::Body::Text("hello".to_owned()))
            .unwrap();
        request.headers_mut().append(
            HeaderName::from_static("x-value"),
            HeaderValue::from_static("first"),
        );
        request.headers_mut().append(
            HeaderName::from_static("x-value"),
            HeaderValue::from_static("second"),
        );

        let mut request = to_actix_request(request).unwrap();
        assert_eq!(request.method(), actix_web::http::Method::POST);
        assert_eq!(request.uri(), "/path?one=1&one=2");
        assert_eq!(
            request
                .headers()
                .get_all("x-value")
                .map(|value| value.to_str().unwrap())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        let body = request.take_payload().next().await.unwrap().unwrap();
        assert_eq!(body, Bytes::from_static(b"hello"));
    }

    #[actix_web::test]
    async fn converts_actix_response_without_losing_cookies() {
        let request = actix_web::test::TestRequest::default().to_http_request();
        let response = HttpResponse::Created()
            .append_header(("set-cookie", "one=1"))
            .append_header(("set-cookie", "two=2"))
            .content_type("text/plain")
            .body("hello");
        let response = actix_web::dev::ServiceResponse::new(request, response);

        let response = to_lambda_response(response).await.unwrap();
        assert_eq!(response.status(), lambda_http::http::StatusCode::CREATED);
        assert_eq!(response.headers().get_all("set-cookie").iter().count(), 2);
    }
}
