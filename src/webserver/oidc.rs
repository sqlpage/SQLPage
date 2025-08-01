use std::collections::HashSet;
use std::future::ready;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use crate::webserver::http_client::get_http_client_from_appdata;
use crate::{app_config::AppConfig, AppState};
use actix_web::{
    body::BoxBody,
    cookie::Cookie,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    web::{self, Query},
    Error, HttpMessage, HttpResponse,
};
use anyhow::{anyhow, Context};
use awc::Client;
use base64::write;
use chrono::Utc;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken,
    CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    core::CoreAuthenticationFlow, url::Url, AsyncHttpClient, Audience, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, Scope,
    TokenResponse,
};
use openidconnect::{
    EmptyExtraTokenFields, IdTokenFields, IdTokenVerifier, StandardErrorResponse,
    StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};

use super::http_client::make_http_client;

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

const SQLPAGE_AUTH_COOKIE_NAME: &str = "sqlpage_auth";
const SQLPAGE_REDIRECT_URI: &str = "/sqlpage/oidc_callback";
const SQLPAGE_STATE_COOKIE_NAME: &str = "sqlpage_oidc_state";
const OIDC_CLIENT_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OidcAdditionalClaims(pub(crate) serde_json::Map<String, serde_json::Value>);

impl openidconnect::AdditionalClaims for OidcAdditionalClaims {}
type OidcToken = openidconnect::IdToken<
    OidcAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJwsSigningAlgorithm,
>;
pub type OidcClaims =
    openidconnect::IdTokenClaims<OidcAdditionalClaims, openidconnect::core::CoreGenderClaim>;

#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub issuer_url: IssuerUrl,
    pub client_id: String,
    pub client_secret: String,
    pub protected_paths: Vec<String>,
    pub public_paths: Vec<String>,
    pub app_host: String,
    pub scopes: Vec<Scope>,
    pub additional_audience_verifier: AudienceVerifier,
}

impl TryFrom<&AppConfig> for OidcConfig {
    type Error = Option<&'static str>;

    fn try_from(config: &AppConfig) -> Result<Self, Self::Error> {
        let issuer_url = config.oidc_issuer_url.as_ref().ok_or(None)?;
        let client_secret = config.oidc_client_secret.as_ref().ok_or(Some(
            "The \"oidc_client_secret\" setting is required to authenticate with the OIDC provider",
        ))?;
        let protected_paths: Vec<String> = config.oidc_protected_paths.clone();
        let public_paths: Vec<String> = config.oidc_public_paths.clone();

        let app_host = get_app_host(config);

        Ok(Self {
            issuer_url: issuer_url.clone(),
            client_id: config.oidc_client_id.clone(),
            client_secret: client_secret.clone(),
            protected_paths,
            public_paths,
            scopes: config
                .oidc_scopes
                .split_whitespace()
                .map(|s| Scope::new(s.to_string()))
                .collect(),
            app_host: app_host.clone(),
            additional_audience_verifier: AudienceVerifier::new(
                config.oidc_additional_trusted_audiences.clone(),
            ),
        })
    }
}

impl OidcConfig {
    #[must_use]
    pub fn is_public_path(&self, path: &str) -> bool {
        !self.protected_paths.iter().any(|p| path.starts_with(p))
            || self.public_paths.iter().any(|p| path.starts_with(p))
    }

    /// Creates a custom ID token verifier that supports multiple issuers
    fn create_id_token_verifier<'a>(
        &'a self,
        oidc_client: &'a OidcClient,
    ) -> IdTokenVerifier<'a, CoreJsonWebKey> {
        oidc_client
            .id_token_verifier()
            .set_other_audience_verifier_fn(self.additional_audience_verifier.as_fn())
    }
}

fn get_app_host(config: &AppConfig) -> String {
    if let Some(host) = &config.host {
        return host.clone();
    }
    if let Some(https_domain) = &config.https_domain {
        return https_domain.clone();
    }

    let socket_addr = config.listen_on();
    let ip = socket_addr.ip();
    let host = if ip.is_unspecified() || ip.is_loopback() {
        format!("localhost:{}", socket_addr.port())
    } else {
        socket_addr.to_string()
    };
    log::warn!(
        "No host or https_domain provided in the configuration, \
         using \"{host}\" as the app host to build the redirect URL. \
         This will only work locally. \
         Disable this warning by providing a value for the \"host\" setting."
    );
    host
}

pub struct ClientWithTime {
    client: OidcClient,
    last_update: Instant,
}

pub struct OidcState {
    pub config: OidcConfig,
    client: RwLock<ClientWithTime>,
}

impl OidcState {
    pub async fn new(oidc_cfg: OidcConfig, app_config: AppConfig) -> anyhow::Result<Self> {
        let http_client = make_http_client(&app_config)?;
        let client = build_oidc_client(&oidc_cfg, &http_client).await?;

        Ok(Self {
            config: oidc_cfg,
            client: RwLock::new(ClientWithTime {
                client,
                last_update: Instant::now(),
            }),
        })
    }

    async fn refresh(&self, service_request: &ServiceRequest) {
        // Obtain a write lock to prevent concurrent OIDC client refreshes.
        let mut write_guard = self.client.write().await;
        match build_oidc_client_from_appdata(&self.config, service_request).await {
            Ok(http_client) => {
                *write_guard = ClientWithTime {
                    client: http_client,
                    last_update: Instant::now(),
                }
            }
            Err(e) => log::error!("Failed to refresh OIDC client: {e}"),
        }
    }

    /// Refreshes the OIDC client from the provider metadata URL if it has expired.
    /// Most providers update their signing keys periodically.
    pub async fn refresh_if_expired(&self, service_request: &ServiceRequest) {
        if self.client.read().await.last_update.elapsed() > OIDC_CLIENT_REFRESH_INTERVAL {
            self.refresh(service_request).await;
        }
    }

    /// Gets a reference to the oidc client, potentially generating a new one if needed
    pub async fn get_client(&self) -> RwLockReadGuard<'_, OidcClient> {
        RwLockReadGuard::map(
            self.client.read().await,
            |ClientWithTime { client, .. }| client,
        )
    }

    /// Validate and decode the claims of an OIDC token, without refreshing the client.
    async fn get_token_claims(
        &self,
        id_token: &OidcToken,
        state: Option<&OidcLoginState>,
    ) -> anyhow::Result<OidcClaims> {
        let client = &self.get_client().await;
        let verifier = self.config.create_id_token_verifier(client);
        let nonce_verifier = |nonce: Option<&Nonce>| check_nonce(nonce, state);
        let claims: OidcClaims = id_token
            .claims(&verifier, nonce_verifier)
            .with_context(|| format!("Could not verify the ID token: {id_token:?}"))?
            .clone();
        Ok(claims)
    }
}

pub async fn initialize_oidc_state(
    app_config: &AppConfig,
) -> anyhow::Result<Option<Arc<OidcState>>> {
    let oidc_cfg = match OidcConfig::try_from(app_config) {
        Ok(c) => c,
        Err(None) => return Ok(None), // OIDC not configured
        Err(Some(e)) => return Err(anyhow::anyhow!(e)),
    };

    Ok(Some(Arc::new(
        OidcState::new(oidc_cfg, app_config.clone()).await?,
    )))
}

async fn build_oidc_client_from_appdata(
    cfg: &OidcConfig,
    req: &ServiceRequest,
) -> anyhow::Result<OidcClient> {
    let http_client = get_http_client_from_appdata(req)?;
    build_oidc_client(cfg, http_client).await
}

async fn build_oidc_client(
    oidc_cfg: &OidcConfig,
    http_client: &Client,
) -> anyhow::Result<OidcClient> {
    let issuer_url = oidc_cfg.issuer_url.clone();
    let provider_metadata = discover_provider_metadata(http_client, issuer_url.clone()).await?;
    let client = make_oidc_client(oidc_cfg, provider_metadata)?;
    Ok(client)
}

pub struct OidcMiddleware {
    oidc_state: Option<Arc<OidcState>>,
}

impl OidcMiddleware {
    #[must_use]
    pub fn new(app_state: &web::Data<AppState>) -> Condition<Self> {
        let oidc_state = app_state.oidc_state.clone();
        Condition::new(oidc_state.is_some(), Self { oidc_state })
    }
}

async fn discover_provider_metadata(
    http_client: &awc::Client,
    issuer_url: IssuerUrl,
) -> anyhow::Result<openidconnect::core::CoreProviderMetadata> {
    log::debug!("Discovering provider metadata for {issuer_url}");
    let provider_metadata = openidconnect::core::CoreProviderMetadata::discover_async(
        issuer_url,
        &AwcHttpClient::from_client(http_client),
    )
    .await
    .with_context(|| "Failed to discover OIDC provider metadata".to_string())?;
    log::debug!("Provider metadata discovered: {provider_metadata:?}");
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
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        match &self.oidc_state {
            Some(state) => ready(Ok(OidcService::new(service, Arc::clone(state)))),
            None => ready(Err(())),
        }
    }
}

#[derive(Clone)]
pub struct OidcService<S> {
    service: Rc<S>,
    oidc_state: Arc<OidcState>,
}

impl<S> OidcService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    pub fn new(service: S, oidc_state: Arc<OidcState>) -> Self {
        Self {
            service: Rc::new(service),
            oidc_state,
        }
    }
}

enum MiddlewareResponse {
    Forward(ServiceRequest),
    Respond(ServiceResponse),
}

async fn handle_request(
    oidc_state: &OidcState,
    request: ServiceRequest,
) -> actix_web::Result<MiddlewareResponse> {
    log::trace!("Started OIDC middleware request handling");
    oidc_state.refresh_if_expired(&request).await;
    let response = match get_authenticated_user_info(oidc_state, &request).await {
        Ok(Some(claims)) => {
            if request.path() != SQLPAGE_REDIRECT_URI {
                log::trace!("Storing authenticated user info in request extensions: {claims:?}");
                request.extensions_mut().insert(claims);
                return Ok(MiddlewareResponse::Forward(request));
            }
            handle_authenticated_oidc_callback(request).await
        }
        Ok(None) => {
            log::trace!("No authenticated user found");
            handle_unauthenticated_request(oidc_state, request).await
        }
        Err(e) => {
            log::debug!("An auth cookie is present but could not be verified. Redirecting to OIDC provider to re-authenticate. {e:?}");
            handle_unauthenticated_request(oidc_state, request).await
        }
    };
    response.map(MiddlewareResponse::Respond)
}

async fn handle_unauthenticated_request(
    oidc_state: &OidcState,
    request: ServiceRequest,
) -> Result<ServiceResponse<BoxBody>, Error> {
    log::debug!("Handling unauthenticated request to {}", request.path());
    if request.path() == SQLPAGE_REDIRECT_URI {
        log::debug!("The request is the OIDC callback");
        return handle_oidc_callback(oidc_state, request).await;
    }

    log::debug!("Redirecting to OIDC provider");

    let response = build_auth_provider_redirect_response(oidc_state, &request).await;
    Ok(request.into_response(response))
}

async fn handle_oidc_callback(
    oidc_state: &OidcState,
    request: ServiceRequest,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let query_string = request.query_string();
    match process_oidc_callback(oidc_state, query_string, &request).await {
        Ok(response) => Ok(request.into_response(response)),
        Err(e) => {
            log::error!("Failed to process OIDC callback with params {query_string}: {e}");
            let resp = build_auth_provider_redirect_response(oidc_state, &request).await;
            Ok(request.into_response(resp))
        }
    }
}

/// When an user has already authenticated (potentially in another tab), we ignore the callback and redirect to the initial URL.
fn handle_authenticated_oidc_callback(
    request: ServiceRequest,
) -> LocalBoxFuture<Result<ServiceResponse<BoxBody>, Error>> {
    let redirect_url = match get_state_from_cookie(&request) {
        Ok(state) => state.initial_url,
        Err(_) => "/".to_string(),
    };
    log::debug!("OIDC callback received for authenticated user. Redirecting to {redirect_url}");
    let response = request.into_response(build_redirect_response(redirect_url));
    Box::pin(ready(Ok(response)))
}

impl<S> Service<ServiceRequest> for OidcService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        if self.oidc_state.config.is_public_path(request.path()) {
            return Box::pin(self.service.call(request));
        }
        let srv = Rc::clone(&self.service);
        let oidc_state = Arc::clone(&self.oidc_state);
        Box::pin(async move {
            match handle_request(&oidc_state, request).await {
                Ok(MiddlewareResponse::Respond(response)) => Ok(response),
                Ok(MiddlewareResponse::Forward(request)) => srv.call(request).await,
                Err(err) => Err(err),
            }
        })
    }
}

async fn process_oidc_callback(
    oidc_state: &OidcState,
    query_string: &str,
    request: &ServiceRequest,
) -> anyhow::Result<HttpResponse> {
    let http_client = get_http_client_from_appdata(request)?;

    let state = get_state_from_cookie(request)?;

    let params = Query::<OidcCallbackParams>::from_query(query_string)
        .with_context(|| {
            format!(
                "{SQLPAGE_REDIRECT_URI}: failed to parse OIDC callback parameters from {query_string}"
            )
        })?
        .into_inner();

    if state.csrf_token.secret() != params.state.secret() {
        log::debug!("CSRF token mismatch: expected {state:?}, got {params:?}");
        return Err(anyhow!("Invalid CSRF token: {}", params.state.secret()));
    }

    let client = oidc_state.get_client().await;
    log::debug!("Processing OIDC callback with params: {params:?}. Requesting token...");
    let token_response = exchange_code_for_token(&client, http_client, params).await?;
    log::debug!("Received token response: {token_response:?}");

    let redirect_target = validate_redirect_url(state.initial_url);
    log::info!("Redirecting to {redirect_target} after a successful login");
    let mut response = build_redirect_response(redirect_target);
    set_auth_cookie(&mut response, &token_response, oidc_state).await?;
    Ok(response)
}

async fn exchange_code_for_token(
    oidc_client: &OidcClient,
    http_client: &awc::Client,
    oidc_callback_params: OidcCallbackParams,
) -> anyhow::Result<OidcTokenResponse> {
    let token_response = oidc_client
        .exchange_code(openidconnect::AuthorizationCode::new(
            oidc_callback_params.code,
        ))?
        .request_async(&AwcHttpClient::from_client(http_client))
        .await?;
    Ok(token_response)
}

async fn set_auth_cookie(
    response: &mut HttpResponse,
    token_response: &OidcTokenResponse,
    oidc_state: &OidcState,
) -> anyhow::Result<()> {
    let access_token = token_response.access_token();
    log::trace!("Received access token: {}", access_token.secret());
    let id_token = token_response
        .id_token()
        .context("No ID token found in the token response. You may have specified an oauth2 provider that does not support OIDC.")?;

    let claims = oidc_state.get_token_claims(id_token, None).await?;
    let expiration = claims.expiration();
    let max_age_seconds = expiration.signed_duration_since(Utc::now()).num_seconds();

    let id_token_str = id_token.to_string();
    log::trace!("Setting auth cookie: {SQLPAGE_AUTH_COOKIE_NAME}=\"{id_token_str}\"");
    let id_token_size_kb = id_token_str.len() / 1024;
    if id_token_size_kb > 4 {
        log::warn!(
            "The ID token cookie from the OIDC provider is {id_token_size_kb}kb. \
             Large cookies can cause performance issues and may be rejected by browsers or by reverse proxies."
        );
    }
    let cookie = Cookie::build(SQLPAGE_AUTH_COOKIE_NAME, id_token_str)
        .secure(true)
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(max_age_seconds))
        .same_site(actix_web::cookie::SameSite::Lax)
        .path("/")
        .finish();

    response.add_cookie(&cookie).unwrap();
    Ok(())
}

async fn build_auth_provider_redirect_response(
    oidc_state: &OidcState,
    request: &ServiceRequest,
) -> HttpResponse {
    let AuthUrl { url, params } = build_auth_url(oidc_state).await;
    let state_cookie = create_state_cookie(request, params);
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", url.to_string()))
        .cookie(state_cookie)
        .body("Redirecting...")
}

fn build_redirect_response(target_url: String) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", target_url))
        .body("Redirecting...")
}

/// Returns the claims from the ID token in the `SQLPage` auth cookie.
async fn get_authenticated_user_info(
    oidc_state: &OidcState,
    request: &ServiceRequest,
) -> anyhow::Result<Option<OidcClaims>> {
    let Some(cookie) = request.cookie(SQLPAGE_AUTH_COOKIE_NAME) else {
        return Ok(None);
    };
    let cookie_value = cookie.value().to_string();
    let id_token = OidcToken::from_str(&cookie_value)
        .with_context(|| format!("Invalid SQLPage auth cookie: {cookie_value:?}"))?;

    let state = get_state_from_cookie(request)?;
    let claims = oidc_state.get_token_claims(&id_token, Some(&state)).await?;
    log::debug!("The current user is: {claims:?}");
    Ok(Some(claims))
}

pub struct AwcHttpClient<'c> {
    client: &'c awc::Client,
}

impl<'c> AwcHttpClient<'c> {
    #[must_use]
    pub fn from_client(client: &'c awc::Client) -> Self {
        Self { client }
    }
}

impl<'c> AsyncHttpClient<'c> for AwcHttpClient<'c> {
    type Error = AwcWrapperError;
    type Future =
        Pin<Box<dyn Future<Output = Result<openidconnect::HttpResponse, Self::Error>> + 'c>>;

    fn call(&'c self, request: openidconnect::HttpRequest) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            execute_oidc_request_with_awc(client, request)
                .await
                .map_err(AwcWrapperError)
        })
    }
}

async fn execute_oidc_request_with_awc(
    client: Client,
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::http::Response<Vec<u8>>, anyhow::Error> {
    let awc_method = awc::http::Method::from_bytes(request.method().as_str().as_bytes())?;
    let awc_uri = awc::http::Uri::from_str(&request.uri().to_string())?;
    log::debug!("Executing OIDC request: {awc_method} {awc_uri}");
    let mut req = client.request(awc_method, awc_uri);
    for (name, value) in request.headers() {
        req = req.insert_header((name.as_str(), value.to_str()?));
    }
    let (req_head, body) = request.into_parts();
    let mut response = req.send_body(body).await.map_err(|e| {
        anyhow!(e.to_string()).context(format!(
            "Failed to send request: {} {}",
            &req_head.method, &req_head.uri
        ))
    })?;
    let head = response.headers();
    let mut resp_builder =
        openidconnect::http::Response::builder().status(response.status().as_u16());
    for (name, value) in head {
        resp_builder = resp_builder.header(name.as_str(), value.to_str()?);
    }
    let body = response
        .body()
        .await
        .with_context(|| format!("Couldnt read from {}", &req_head.uri))?;
    log::debug!(
        "Received OIDC response with status {}: {}",
        response.status(),
        String::from_utf8_lossy(&body)
    );
    let resp = resp_builder.body(body.to_vec())?;
    Ok(resp)
}

#[derive(Debug)]
pub struct AwcWrapperError(anyhow::Error);

impl std::fmt::Display for AwcWrapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

type OidcTokenResponse = StandardTokenResponse<
    IdTokenFields<
        OidcAdditionalClaims,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
    >,
    CoreTokenType,
>;

type OidcClient = openidconnect::Client<
    OidcAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    OidcTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

impl std::error::Error for AwcWrapperError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

fn make_oidc_client(
    config: &OidcConfig,
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
        Some(openidconnect::url::Host::Ipv4(_) | openidconnect::url::Host::Ipv6(_)) => true,
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
    let client =
        OidcClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(redirect_url);

    Ok(client)
}

#[derive(Debug, Deserialize)]
struct OidcCallbackParams {
    code: String,
    state: CsrfToken,
}

struct AuthUrl {
    url: Url,
    params: AuthUrlParams,
}

struct AuthUrlParams {
    csrf_token: CsrfToken,
    nonce: Nonce,
}

async fn build_auth_url(oidc_state: &OidcState) -> AuthUrl {
    let nonce_source = Nonce::new_random();
    let hashed_nonce = Nonce::new(hash_nonce(&nonce_source));
    let scopes = &oidc_state.config.scopes;
    let client_lock = oidc_state.get_client().await;
    let (url, csrf_token, _nonce) = client_lock
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            || hashed_nonce,
        )
        .add_scopes(scopes.iter().cloned())
        .url();
    AuthUrl {
        url,
        params: AuthUrlParams {
            csrf_token,
            nonce: nonce_source,
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OidcLoginState {
    /// The URL to redirect to after the login process is complete.
    #[serde(rename = "u")]
    initial_url: String,
    /// The CSRF token to use for the login process.
    #[serde(rename = "c")]
    csrf_token: CsrfToken,
    /// The source nonce to use for the login process. It must be checked against the hash
    /// stored in the ID token.
    #[serde(rename = "n")]
    nonce: Nonce,
}

fn hash_nonce(nonce: &Nonce) -> String {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut OsRng);
    // low-cost parameters: oidc tokens are short-lived and the source nonce is high-entropy
    let params = argon2::Params::new(8, 1, 1, Some(16)).expect("bug: invalid Argon2 parameters");
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = argon2
        .hash_password(nonce.secret().as_bytes(), &salt)
        .expect("bug: failed to hash nonce");
    hash.to_string()
}

fn check_nonce(
    id_token_nonce: Option<&Nonce>,
    login_state: Option<&OidcLoginState>,
) -> Result<(), String> {
    let Some(state) = login_state else {
        return Ok(()); // No login state, no nonce to check
    };
    match id_token_nonce {
        Some(id_token_nonce) => nonce_matches(id_token_nonce, &state.nonce),
        None => Err("No nonce found in the ID token".to_string()),
    }
}

fn nonce_matches(id_token_nonce: &Nonce, state_nonce: &Nonce) -> Result<(), String> {
    log::debug!(
        "Checking nonce: {} == {}",
        id_token_nonce.secret(),
        state_nonce.secret()
    );
    let hash = argon2::password_hash::PasswordHash::new(id_token_nonce.secret()).map_err(|e| {
        format!(
            "Failed to parse state nonce ({}): {e}",
            id_token_nonce.secret()
        )
    })?;
    argon2::password_hash::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        state_nonce.secret().as_bytes(),
        &hash,
    )
    .map_err(|e| format!("Failed to verify nonce ({}): {e}", state_nonce.secret()))?;
    log::debug!("Nonce successfully verified");
    Ok(())
}

impl OidcLoginState {
    fn new(request: &ServiceRequest, auth_url: AuthUrlParams) -> Self {
        Self {
            initial_url: request.uri().to_string(),
            csrf_token: auth_url.csrf_token,
            nonce: auth_url.nonce,
        }
    }
}

fn create_state_cookie(request: &ServiceRequest, auth_url: AuthUrlParams) -> Cookie {
    let state = OidcLoginState::new(request, auth_url);
    let state_json = serde_json::to_string(&state).unwrap();
    Cookie::build(SQLPAGE_STATE_COOKIE_NAME, state_json)
        .secure(true)
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .path("/")
        .finish()
}

fn get_state_from_cookie(request: &ServiceRequest) -> anyhow::Result<OidcLoginState> {
    let state_cookie = request.cookie(SQLPAGE_STATE_COOKIE_NAME).with_context(|| {
        format!("No {SQLPAGE_STATE_COOKIE_NAME} cookie found for {SQLPAGE_REDIRECT_URI}")
    })?;
    serde_json::from_str(state_cookie.value())
        .with_context(|| format!("Failed to parse OIDC state from cookie: {state_cookie}"))
}

/// Given an audience, verify if it is trusted. The `client_id` is always trusted, independently of this function.
#[derive(Clone, Debug)]
pub struct AudienceVerifier(Option<HashSet<String>>);

impl AudienceVerifier {
    /// JWT audiences (aud claim) are always required to contain the `client_id`, but they can also contain additional audiences.
    /// By default we allow any additional audience.
    /// The user can restrict the allowed additional audiences by providing a list of trusted audiences.
    fn new(additional_trusted_audiences: Option<Vec<String>>) -> Self {
        AudienceVerifier(additional_trusted_audiences.map(HashSet::from_iter))
    }

    /// Returns a function that given an audience, verifies if it is trusted.
    fn as_fn(&self) -> impl Fn(&Audience) -> bool + '_ {
        move |aud: &Audience| -> bool {
            let Some(trusted_set) = &self.0 else {
                return true;
            };
            trusted_set.contains(aud.as_str())
        }
    }
}

/// Validate that a redirect URL is safe to use (prevents open redirect attacks)
fn validate_redirect_url(url: String) -> String {
    if url.starts_with('/') && !url.starts_with("//") {
        return url;
    }
    log::warn!("Refusing to redirect to {url}");
    '/'.to_string()
}
