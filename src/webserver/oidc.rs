use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use crate::app_config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    Error,
};
use anyhow::anyhow;
use awc::Client;
use openidconnect::{AsyncHttpClient, IssuerUrl};

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer_url: IssuerUrl,
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
                log::debug!("Setting up OIDC with issuer: {}", config.issuer_url);
                // contains secrets
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

async fn discover_provider_metadata(
    issuer_url: IssuerUrl,
) -> anyhow::Result<openidconnect::core::CoreProviderMetadata> {
    let http_client = AwcHttpClient::new();
    let provider_metadata =
        openidconnect::core::CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;
    Ok(provider_metadata)
}

impl<S, B> Transform<S, ServiceRequest> for OidcMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OidcService<S>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = self.config.clone();
        Box::pin(async move {
            match config {
                Some(config) => Ok(OidcService::new(service, Arc::clone(&config))
                    .await
                    .map_err(|err| {
                        log::error!(
                            "Error creating OIDC service with issuer: {}: {err:?}",
                            config.issuer_url
                        );
                    })?),
                None => Err(()),
            }
        })
    }
}

pub struct OidcService<S> {
    service: S,
    config: Arc<OidcConfig>,
    provider_metadata: openidconnect::core::CoreProviderMetadata,
}

impl<S> OidcService<S> {
    pub async fn new(service: S, config: Arc<OidcConfig>) -> anyhow::Result<Self> {
        Ok(Self {
            service,
            config: Arc::clone(&config),
            provider_metadata: discover_provider_metadata(config.issuer_url.clone()).await?,
        })
    }
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
        log::debug!("Started OIDC middleware with config: {:?}", self.config);
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
    type Error = StringError;
    type Future = Pin<
        Box<dyn Future<Output = Result<openidconnect::http::Response<Vec<u8>>, Self::Error>> + 'c>,
    >;

    fn call(&'c self, request: openidconnect::http::Request<Vec<u8>>) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            execute_oidc_request_with_awc(client, request)
                .await
                .map_err(|err| StringError(format!("Failed to execute OIDC request: {err:?}")))
        })
    }
}

async fn execute_oidc_request_with_awc(
    client: Client,
    request: openidconnect::http::Request<Vec<u8>>,
) -> Result<openidconnect::http::Response<Vec<u8>>, anyhow::Error> {
    let awc_method = awc::http::Method::from_bytes(request.method().as_str().as_bytes())?;
    let awc_uri = awc::http::Uri::from_str(&request.uri().to_string())?;
    log::debug!("Executing OIDC request: {} {}", awc_method, awc_uri);
    let mut req = client.request(awc_method, awc_uri);
    for (name, value) in request.headers() {
        req = req.insert_header((name.as_str(), value.to_str()?));
    }
    let mut response = req
        .send_body(request.into_body())
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    let head = response.headers();
    let mut resp_builder =
        openidconnect::http::Response::builder().status(response.status().as_u16());
    for (name, value) in head {
        resp_builder = resp_builder.header(name.as_str(), value.to_str()?);
    }
    let body = response.body().await?.to_vec();
    log::debug!(
        "Received OIDC response with status {}: {}",
        response.status(),
        String::from_utf8_lossy(&body)
    );
    let resp = resp_builder.body(body)?;
    Ok(resp)
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringError(String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for StringError {}
