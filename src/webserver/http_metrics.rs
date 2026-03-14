use std::future::{ready, Ready};
use std::time::Instant;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
};
use futures_util::future::LocalBoxFuture;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions::attribute as otel;
use tracing_actix_web::root_span_macro::private::{http_method_str, http_scheme};

use crate::AppState;

pub struct HttpMetrics;

impl<S, B> Transform<S, ServiceRequest> for HttpMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = HttpMetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HttpMetricsMiddleware { service }))
    }
}

pub struct HttpMetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for HttpMetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = http_method_str(req.method()).to_string();
        let connection_info = req.connection_info();
        let scheme = http_scheme(connection_info.scheme()).to_string();
        let host = connection_info.host().to_string();
        drop(connection_info);
        
        // We get the route pattern. In Actix, req.match_pattern() returns the matched route
        let route = req.match_pattern().unwrap_or_else(|| req.path().to_string());
        
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed().as_secs_f64();
            let status = res.status().as_u16();

            let mut attributes = vec![
                KeyValue::new(otel::HTTP_REQUEST_METHOD, method),
                KeyValue::new(otel::HTTP_RESPONSE_STATUS_CODE, status.to_string()),
                KeyValue::new(otel::HTTP_ROUTE, route),
                KeyValue::new(otel::URL_SCHEME, scheme),
                KeyValue::new(otel::SERVER_ADDRESS, host),
            ];

            if status >= 500 {
                attributes.push(KeyValue::new(otel::ERROR_TYPE, status.to_string()));
            }

            if let Some(app_state) = res.request().app_data::<web::Data<AppState>>() {
                app_state
                    .telemetry_metrics
                    .http_request_duration
                    .record(duration, &attributes);
            }

            Ok(res)
        })
    }
}
