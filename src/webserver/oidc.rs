use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use crate::{app_config::AppConfig, AppState};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    web, Error, HttpResponse,
};
use anyhow::{anyhow, Context};
use awc::Client;
use openidconnect::{
    core::{CoreAuthDisplay, CoreAuthenticationFlow},
    AsyncHttpClient, CsrfToken, EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, RedirectUrl, Scope,
};

use super::http_client::make_http_client;

const SQLPAGE_AUTH_COOKIE_NAME: &str = "sqlpage_auth";
const SQLPAGE_REDIRECT_URI: &str = "/sqlpage/oidc_callback";

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer_url: IssuerUrl,
    pub client_id: String,
    pub client_secret: String,
    pub app_host: String,
    pub scopes: Vec<Scope>,
}

impl TryFrom<&AppConfig> for OidcConfig {
    type Error = Option<&'static str>;

    fn try_from(config: &AppConfig) -> Result<Self, Self::Error> {
        let issuer_url = config.oidc_issuer_url.as_ref().ok_or(None)?;
        let client_secret = config.oidc_client_secret.as_ref().ok_or(Some(
            "The \"oidc_client_secret\" setting is required to authenticate with the OIDC provider",
        ))?;

        let app_host = config
            .host
            .as_ref()
            .or_else(|| config.https_domain.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                let host = config.listen_on().to_string();
                log::warn!(
                    "No host or https_domain provided in the configuration, using \"{}\" as the app host to build the redirect URL. This will only work locally. Disable this warning by providing a value for the \"host\" setting.",
                    host
                );
                host
            });

        Ok(Self {
            issuer_url: issuer_url.clone(),
            client_id: config.oidc_client_id.clone(),
            client_secret: client_secret.clone(),
            scopes: config
                .oidc_scopes
                .split_whitespace()
                .map(|s| Scope::new(s.to_string()))
                .collect(),
            app_host: app_host.clone(),
        })
    }
}

pub struct OidcMiddleware {
    pub config: Option<Arc<OidcConfig>>,
    app_state: web::Data<AppState>,
}

impl OidcMiddleware {
    pub fn new(app_state: &web::Data<AppState>) -> Condition<Self> {
        let config = OidcConfig::try_from(&app_state.config);
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
        Condition::new(
            config.is_some(),
            Self {
                config,
                app_state: web::Data::clone(app_state),
            },
        )
    }
}

async fn discover_provider_metadata(
    app_config: &AppConfig,
    issuer_url: IssuerUrl,
) -> anyhow::Result<openidconnect::core::CoreProviderMetadata> {
    let http_client = AwcHttpClient::new(app_config)?;
    let provider_metadata =
        openidconnect::core::CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;
    Ok(provider_metadata)
}

impl<S> Transform<S, ServiceRequest> for OidcMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = OidcService<S>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = self.config.clone();
        let app_state = web::Data::clone(&self.app_state);
        Box::pin(async move {
            match config {
                Some(config) => Ok(OidcService::new(service, &app_state, Arc::clone(&config))
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
    app_state: web::Data<AppState>,
    config: Arc<OidcConfig>,
    client: OidcClient,
}

impl<S> OidcService<S> {
    pub async fn new(
        service: S,
        app_state: &web::Data<AppState>,
        config: Arc<OidcConfig>,
    ) -> anyhow::Result<Self> {
        let issuer_url = config.issuer_url.clone();
        let provider_metadata = discover_provider_metadata(&app_state.config, issuer_url).await?;
        let client: OidcClient = make_oidc_client(&config, provider_metadata)?;
        Ok(Self {
            service,
            app_state: web::Data::clone(app_state),
            config,
            client,
        })
    }

    fn build_auth_url(&self, request: &ServiceRequest) -> String {
        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            // Set the desired scopes.
            .add_scopes(self.config.scopes.iter().cloned())
            .url();
        auth_url.to_string()
    }
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;
use actix_web::body::BoxBody;

impl<S> Service<ServiceRequest> for OidcService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        log::debug!("Started OIDC middleware with config: {:?}", self.config);
        match get_sqlpage_auth_cookie(&request) {
            Some(cookie) => {
                log::trace!("Found SQLPage auth cookie: {cookie}");
            }
            None => {
                log::trace!("No SQLPage auth cookie found, redirecting to login");
                let auth_url = self.build_auth_url(&request);

                return Box::pin(async move {
                    Ok(request.into_response(build_redirect_response(auth_url)))
                });
            }
        }
        let future = self.service.call(request);
        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}

fn build_redirect_response(auth_url: String) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", auth_url))
        .body("Redirecting to the login page.")
}

fn get_sqlpage_auth_cookie(request: &ServiceRequest) -> Option<String> {
    let cookie = request.cookie(SQLPAGE_AUTH_COOKIE_NAME)?;
    log::error!("TODO: actually check the validity of the cookie");
    Some(cookie.value().to_string())
}

pub struct AwcHttpClient {
    client: Client,
}

impl AwcHttpClient {
    pub fn new(app_config: &AppConfig) -> anyhow::Result<Self> {
        Ok(Self {
            client: make_http_client(app_config)?,
        })
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
    log::debug!("Executing OIDC request: {awc_method} {awc_uri}");
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
type OidcClient = openidconnect::core::CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;
impl std::error::Error for StringError {}

fn make_oidc_client(
    config: &Arc<OidcConfig>,
    provider_metadata: openidconnect::core::CoreProviderMetadata,
) -> anyhow::Result<OidcClient> {
    let client_id = openidconnect::ClientId::new(config.client_id.clone());
    let client_secret = openidconnect::ClientSecret::new(config.client_secret.clone());

    let mut redirect_url = RedirectUrl::new(format!(
        "https://{}{}",
        config.app_host, SQLPAGE_REDIRECT_URI,
    ))
    .with_context(|| {
        format!(
            "Failed to build the redirect URL; invalid app host \"{}\"",
            config.app_host
        )
    })?;
    let needs_http = match redirect_url.url().host() {
        Some(openidconnect::url::Host::Domain(domain)) => domain == "localhost",
        Some(openidconnect::url::Host::Ipv4(_)) => true,
        Some(openidconnect::url::Host::Ipv6(_)) => true,
        None => false,
    };
    if needs_http {
        log::debug!("App host seems to be local, changing redirect URL to HTTP");
        redirect_url = RedirectUrl::new(format!(
            "http://{}{}",
            config.app_host, SQLPAGE_REDIRECT_URI,
        ))?;
    }
    log::info!("OIDC redirect URL for {}: {redirect_url}", config.client_id);
    let client = openidconnect::core::CoreClient::from_provider_metadata(
        provider_metadata,
        client_id,
        Some(client_secret),
    )
    .set_redirect_uri(redirect_url);

    Ok(client)
}
