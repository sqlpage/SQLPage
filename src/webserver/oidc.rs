use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    sync::Arc,
};

use crate::app_config::AppConfig;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    Error,
};

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
