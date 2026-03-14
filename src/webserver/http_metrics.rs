use std::future::{ready, Ready};
use std::time::Instant;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::Histogram;
use tracing_actix_web::root_span_macro::private::{http_method_str, http_scheme};

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
        let histogram = global::meter("sqlpage")
            .f64_histogram("http.server.request.duration")
            .with_unit("s")
            .with_description("Duration of HTTP requests processed by the server.")
            .build();
            
        ready(Ok(HttpMetricsMiddleware {
            service,
            histogram,
        }))
    }
}

pub struct HttpMetricsMiddleware<S> {
    service: S,
    histogram: Histogram<f64>,
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
        
        let histogram = self.histogram.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed().as_secs_f64();
            let status = res.status().as_u16();

            let mut attributes = vec![
                KeyValue::new("http.request.method", method),
                KeyValue::new("http.response.status_code", status.to_string()),
                KeyValue::new("http.route", route),
                KeyValue::new("url.scheme", scheme),
                KeyValue::new("server.address", host),
            ];

            if status >= 500 {
                attributes.push(KeyValue::new("error.type", status.to_string()));
            }

            histogram.record(duration, &attributes);

            Ok(res)
        })
    }
}
