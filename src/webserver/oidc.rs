use std::future::ready;
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
use chrono::Utc;
use openidconnect::{
    core::CoreAuthenticationFlow, url::Url, AsyncHttpClient, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, Scope,
    TokenResponse,
};
use serde::{Deserialize, Serialize};

use super::http_client::make_http_client;

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

const SQLPAGE_AUTH_COOKIE_NAME: &str = "sqlpage_auth";
const SQLPAGE_REDIRECT_URI: &str = "/sqlpage/oidc_callback";
const SQLPAGE_STATE_COOKIE_NAME: &str = "sqlpage_oidc_state";

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
        })
    }
}

impl OidcConfig {
    #[must_use]
    pub fn is_public_path(&self, path: &str) -> bool {
        !self.protected_paths.iter().any(|p| path.starts_with(p))
            || self.public_paths.iter().any(|p| path.starts_with(p))
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

pub struct OidcState {
    pub config: Arc<OidcConfig>,
    pub client: Arc<OidcClient>,
}

pub async fn initialize_oidc_state(
    app_config: &AppConfig,
) -> anyhow::Result<Option<Arc<OidcState>>> {
    let oidc_cfg = match OidcConfig::try_from(app_config) {
        Ok(c) => Arc::new(c),
        Err(None) => return Ok(None), // OIDC not configured
        Err(Some(e)) => return Err(anyhow::anyhow!(e)),
    };

    let http_client = make_http_client(app_config)?;
    let provider_metadata =
        discover_provider_metadata(&http_client, oidc_cfg.issuer_url.clone()).await?;
    let client = make_oidc_client(&oidc_cfg, provider_metadata)?;

    Ok(Some(Arc::new(OidcState {
        config: oidc_cfg,
        client: Arc::new(client),
    })))
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
    service: S,
    oidc_state: Arc<OidcState>,
}

impl<S> OidcService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    pub fn new(service: S, oidc_state: Arc<OidcState>) -> Self {
        Self {
            service,
            oidc_state,
        }
    }

    fn handle_unauthenticated_request(
        &self,
        request: ServiceRequest,
    ) -> LocalBoxFuture<Result<ServiceResponse<BoxBody>, Error>> {
        log::debug!("Handling unauthenticated request to {}", request.path());
        if request.path() == SQLPAGE_REDIRECT_URI {
            log::debug!("The request is the OIDC callback");
            return self.handle_oidc_callback(request);
        }

        if self.oidc_state.config.is_public_path(request.path()) {
            log::debug!(
                "The request path {} is not in a public path, skipping OIDC authentication",
                request.path()
            );
            return Box::pin(self.service.call(request));
        }

        log::debug!("Redirecting to OIDC provider");

        let response = build_auth_provider_redirect_response(
            &self.oidc_state.client,
            &self.oidc_state.config,
            &request,
        );
        Box::pin(async move { Ok(request.into_response(response)) })
    }

    fn handle_oidc_callback(
        &self,
        request: ServiceRequest,
    ) -> LocalBoxFuture<Result<ServiceResponse<BoxBody>, Error>> {
        let oidc_client = Arc::clone(&self.oidc_state.client);
        let oidc_config = Arc::clone(&self.oidc_state.config);

        Box::pin(async move {
            let query_string = request.query_string();
            match process_oidc_callback(&oidc_client, query_string, &request).await {
                Ok(response) => Ok(request.into_response(response)),
                Err(e) => {
                    log::error!("Failed to process OIDC callback with params {query_string}: {e}");
                    let resp =
                        build_auth_provider_redirect_response(&oidc_client, &oidc_config, &request);
                    Ok(request.into_response(resp))
                }
            }
        })
    }

    fn handle_authenticated_oidc_callback(
        &self,
        request: ServiceRequest,
    ) -> LocalBoxFuture<Result<ServiceResponse<BoxBody>, Error>> {
        Box::pin(async move {
            log::debug!("Handling OIDC callback for already authenticated user");
            
            // Try to get the initial URL from the state cookie
            let redirect_url = match get_state_from_cookie(&request) {
                Ok(state) => {
                    log::debug!("Found initial URL in state: {}", state.initial_url);
                    state.initial_url
                }
                Err(e) => {
                    log::debug!("Could not get state from cookie (user might have been redirected from elsewhere): {e}. Redirecting to /");
                    "/".to_string()
                }
            };
            
            let response = build_redirect_response(redirect_url);
            Ok(request.into_response(response))
        })
    }
}

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
        log::trace!("Started OIDC middleware request handling");

        // Handle OIDC callback URL even for authenticated users
        if request.path() == SQLPAGE_REDIRECT_URI {
            log::debug!("The request is the OIDC callback for an authenticated user");
            return self.handle_authenticated_oidc_callback(request);
        }

        let oidc_client = Arc::clone(&self.oidc_state.client);
        match get_authenticated_user_info(&oidc_client, &request) {
            Ok(Some(claims)) => {
                log::trace!("Storing authenticated user info in request extensions: {claims:?}");
                request.extensions_mut().insert(claims);
            }
            Ok(None) => {
                log::trace!("No authenticated user found");
                return self.handle_unauthenticated_request(request);
            }
            Err(e) => {
                log::debug!(
                    "{:?}",
                    e.context(
                        "An auth cookie is present but could not be verified. \
                     Redirecting to OIDC provider to re-authenticate."
                    )
                );
                return self.handle_unauthenticated_request(request);
            }
        }
        let future = self.service.call(request);
        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}

async fn process_oidc_callback(
    oidc_client: &OidcClient,
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

    log::debug!("Processing OIDC callback with params: {params:?}. Requesting token...");
    let token_response = exchange_code_for_token(oidc_client, http_client, params).await?;
    log::debug!("Received token response: {token_response:?}");

    let mut response = build_redirect_response(state.initial_url);
    set_auth_cookie(&mut response, &token_response, oidc_client)?;
    Ok(response)
}

async fn exchange_code_for_token(
    oidc_client: &OidcClient,
    http_client: &awc::Client,
    oidc_callback_params: OidcCallbackParams,
) -> anyhow::Result<openidconnect::core::CoreTokenResponse> {
    let token_response = oidc_client
        .exchange_code(openidconnect::AuthorizationCode::new(
            oidc_callback_params.code,
        ))?
        .request_async(&AwcHttpClient::from_client(http_client))
        .await?;
    Ok(token_response)
}

fn set_auth_cookie(
    response: &mut HttpResponse,
    token_response: &openidconnect::core::CoreTokenResponse,
    oidc_client: &OidcClient,
) -> anyhow::Result<()> {
    let access_token = token_response.access_token();
    log::trace!("Received access token: {}", access_token.secret());
    let id_token = token_response
        .id_token()
        .context("No ID token found in the token response. You may have specified an oauth2 provider that does not support OIDC.")?;

    let id_token_verifier = oidc_client.id_token_verifier();
    let nonce_verifier = |_nonce: Option<&Nonce>| Ok(()); // The nonce will be verified in request handling
    let claims = id_token.claims(&id_token_verifier, nonce_verifier)?;
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

fn build_auth_provider_redirect_response(
    oidc_client: &OidcClient,
    oidc_config: &Arc<OidcConfig>,
    request: &ServiceRequest,
) -> HttpResponse {
    let AuthUrl { url, params } = build_auth_url(oidc_client, &oidc_config.scopes);
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
fn get_authenticated_user_info(
    oidc_client: &OidcClient,
    request: &ServiceRequest,
) -> anyhow::Result<Option<OidcClaims>> {
    let Some(cookie) = request.cookie(SQLPAGE_AUTH_COOKIE_NAME) else {
        return Ok(None);
    };
    let cookie_value = cookie.value().to_string();

    let state = get_state_from_cookie(request)?;
    let verifier: openidconnect::IdTokenVerifier<'_, openidconnect::core::CoreJsonWebKey> =
        oidc_client.id_token_verifier();
    let id_token = OidcToken::from_str(&cookie_value)
        .with_context(|| format!("Invalid SQLPage auth cookie: {cookie_value:?}"))?;

    let nonce_verifier = |nonce: Option<&Nonce>| check_nonce(nonce, &state.nonce);
    let claims: OidcClaims = id_token
        .claims(&verifier, nonce_verifier)
        .with_context(|| format!("Could not verify the ID token: {cookie_value:?}"))?
        .clone();
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
type OidcClient = openidconnect::core::CoreClient<
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
    let client = openidconnect::core::CoreClient::from_provider_metadata(
        provider_metadata,
        client_id,
        Some(client_secret),
    )
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

fn build_auth_url(oidc_client: &OidcClient, scopes: &[Scope]) -> AuthUrl {
    let nonce_source = Nonce::new_random();
    let hashed_nonce = Nonce::new(hash_nonce(&nonce_source));
    let (url, csrf_token, _nonce) = oidc_client
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

fn check_nonce(id_token_nonce: Option<&Nonce>, state_nonce: &Nonce) -> Result<(), String> {
    match id_token_nonce {
        Some(id_token_nonce) => nonce_matches(id_token_nonce, state_nonce),
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
            initial_url: request.path().to_string(),
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
