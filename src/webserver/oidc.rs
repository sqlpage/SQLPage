use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    str::FromStr,
    sync::Arc,
};

use crate::app_config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    Error,
};
use awc::Client;
use openidconnect::AsyncHttpClient;
use std::error::Error as StdError;

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: String,
}

impl TryFrom<&AppConfig> for OidcConfig {
    type Error = Option<&'static str>;

    fn try_from(config: &AppConfig) -> Result<Self, Self::Error> {
        let issuer_url = config.oidc_issuer_url.as_ref().ok_or(None)?;
        let client_secret = config
            .oidc_client_secret
            .as_ref()
            .ok_or(Some("Missing oidc_client_secret"))?;

        Ok(Self {
            issuer_url: issuer_url.clone(),
            client_id: config.oidc_client_id.clone(),
            client_secret: client_secret.clone(),
            scopes: config.oidc_scopes.clone(),
        })
    }
}

pub struct OidcMiddleware {
    pub config: Option<Arc<OidcConfig>>,
}

impl OidcMiddleware {
    pub fn new(config: &AppConfig) -> Condition<Self> {
        let config = OidcConfig::try_from(config);
        match &config {
            Ok(config) => {
                log::info!("Setting up OIDC with config: {config:?}");
            }
            Err(Some(err)) => {
                log::error!("Invalid OIDC configuration: {err}");
            }
            Err(None) => {
                log::debug!("No OIDC configuration provided, skipping middleware.");
            }
        }
        let config = config.ok().map(Arc::new);
        Condition::new(config.is_some(), Self { config })
    }
}

impl<S, B> Transform<S, ServiceRequest> for OidcMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OidcService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(
            self.config
                .as_ref()
                .map(|config| OidcService {
                    service,
                    config: Arc::clone(config),
                })
                .ok_or(()),
        )
    }
}

pub struct OidcService<S> {
    service: S,
    config: Arc<OidcConfig>,
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

impl<S, B> Service<ServiceRequest> for OidcService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        log::info!("OIDC config: {:?}", self.config);
        let future = self.service.call(request);

        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}

pub struct AwcHttpClient {
    client: Client,
}

impl AwcHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::default(),
        }
    }
}

impl<'c> AsyncHttpClient<'c> for AwcHttpClient {
    type Error = awc::error::SendRequestError;
    type Future = Pin<
        Box<dyn Future<Output = Result<openidconnect::http::Response<Vec<u8>>, Self::Error>> + 'c>,
    >;

    fn call(&'c self, request: openidconnect::http::Request<Vec<u8>>) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            let awc_method = awc::http::Method::from_bytes(request.method().as_str().as_bytes())
                .map_err(to_awc_error)?;
            let awc_uri =
                awc::http::Uri::from_str(&request.uri().to_string()).map_err(to_awc_error)?;
            let mut req = client.request(awc_method, awc_uri);
            for (name, value) in request.headers() {
                req = req.insert_header((name.as_str(), value.to_str().map_err(to_awc_error)?));
            }
            let mut response = req.send_body(request.into_body()).await?;
            let head = response.headers();
            let mut resp_builder =
                openidconnect::http::Response::builder().status(response.status().as_u16());
            for (name, value) in head {
                resp_builder =
                    resp_builder.header(name.as_str(), value.to_str().map_err(to_awc_error)?);
            }
            let body = response.body().await.map_err(to_awc_error)?.to_vec();
            let resp = resp_builder.body(body).map_err(to_awc_error)?;
            Ok(resp)
        })
    }
}

fn to_awc_error<T: StdError + 'static>(err: T) -> awc::error::SendRequestError {
    let err_str = err.to_string();
    awc::error::SendRequestError::Custom(Box::new(err), Box::new(err_str))
}
