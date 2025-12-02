use std::collections::HashSet;
use std::future::ready;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use crate::webserver::http_client::get_http_client_from_appdata;
use crate::{app_config::AppConfig, AppState};
use actix_web::http::header;
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
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken,
    CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    core::CoreAuthenticationFlow, url::Url, AsyncHttpClient, Audience, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, EndSessionUrl, IssuerUrl, LogoutRequest, Nonce, OAuth2TokenResponse,
    PostLogoutRedirectUrl, ProviderMetadataWithLogout, RedirectUrl, Scope, TokenResponse,
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
const SQLPAGE_LOGOUT_URI: &str = "/sqlpage/oidc_logout";
const SQLPAGE_NONCE_COOKIE_NAME: &str = "sqlpage_oidc_nonce";
const SQLPAGE_TMP_LOGIN_STATE_COOKIE_PREFIX: &str = "sqlpage_oidc_state_";
const SQLPAGE_LOGOUT_STATE_COOKIE_PREFIX: &str = "sqlpage_logout_state_";
const OIDC_CLIENT_MAX_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);
const OIDC_CLIENT_MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const AUTH_COOKIE_EXPIRATION: awc::cookie::time::Duration =
    actix_web::cookie::time::Duration::days(7);
const LOGIN_FLOW_STATE_COOKIE_EXPIRATION: awc::cookie::time::Duration =
    actix_web::cookie::time::Duration::minutes(10);

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
    end_session_endpoint: Option<EndSessionUrl>,
    last_update: Instant,
}

pub struct OidcState {
    pub config: OidcConfig,
    client: RwLock<ClientWithTime>,
}

impl OidcState {
    pub async fn new(oidc_cfg: OidcConfig, app_config: AppConfig) -> anyhow::Result<Self> {
        let http_client = make_http_client(&app_config)?;
        let (client, end_session_endpoint) =
            build_oidc_client(&oidc_cfg, &http_client).await?;

        Ok(Self {
            config: oidc_cfg,
            client: RwLock::new(ClientWithTime {
                client,
                end_session_endpoint,
                last_update: Instant::now(),
            }),
        })
    }

    async fn refresh(&self, service_request: &ServiceRequest) {
        let mut write_guard = self.client.write().await;
        match build_oidc_client_from_appdata(&self.config, service_request).await {
            Ok((http_client, end_session_endpoint)) => {
                *write_guard = ClientWithTime {
                    client: http_client,
                    end_session_endpoint,
                    last_update: Instant::now(),
                }
            }
            Err(e) => log::error!("Failed to refresh OIDC client: {e:#}"),
        }
    }

    /// Refreshes the OIDC client from the provider metadata URL if it has expired.
    /// Most providers update their signing keys periodically.
    pub async fn refresh_if_expired(&self, service_request: &ServiceRequest) {
        if self.client.read().await.last_update.elapsed() > OIDC_CLIENT_MAX_REFRESH_INTERVAL {
            self.refresh(service_request).await;
        }
    }

    /// When an authentication error is encountered, refresh the OIDC client info faster
    pub async fn refresh_on_error(&self, service_request: &ServiceRequest) {
        if self.client.read().await.last_update.elapsed() > OIDC_CLIENT_MIN_REFRESH_INTERVAL {
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

    pub async fn get_end_session_endpoint(&self) -> Option<EndSessionUrl> {
        self.client.read().await.end_session_endpoint.clone()
    }

    /// Validate and decode the claims of an OIDC token, without refreshing the client.
    async fn get_token_claims(
        &self,
        id_token: OidcToken,
        expected_nonce: &Nonce,
    ) -> anyhow::Result<OidcClaims> {
        let client = &self.get_client().await;
        let verifier = self.config.create_id_token_verifier(client);
        let nonce_verifier = |nonce: Option<&Nonce>| check_nonce(nonce, expected_nonce);
        let claims: OidcClaims = id_token
            .into_claims(&verifier, nonce_verifier)
            .map_err(|e| anyhow::anyhow!("Could not verify the ID token: {e}"))?;
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
) -> anyhow::Result<(OidcClient, Option<EndSessionUrl>)> {
    let http_client = get_http_client_from_appdata(req)?;
    build_oidc_client(cfg, http_client).await
}

async fn build_oidc_client(
    oidc_cfg: &OidcConfig,
    http_client: &Client,
) -> anyhow::Result<(OidcClient, Option<EndSessionUrl>)> {
    let issuer_url = oidc_cfg.issuer_url.clone();
    let provider_metadata = discover_provider_metadata(http_client, issuer_url.clone()).await?;
    let end_session_endpoint = provider_metadata
        .additional_metadata()
        .end_session_endpoint
        .clone();
    let client = make_oidc_client(oidc_cfg, provider_metadata)?;
    Ok((client, end_session_endpoint))
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
) -> anyhow::Result<ProviderMetadataWithLogout> {
    log::debug!("Discovering provider metadata for {issuer_url}");
    let provider_metadata = ProviderMetadataWithLogout::discover_async(
        issuer_url,
        &AwcHttpClient::from_client(http_client),
    )
    .await
    .with_context(|| "Failed to discover OIDC provider metadata".to_string())?;
    log::debug!("Provider metadata discovered: {provider_metadata:?}");
    log::debug!(
        "end_session_endpoint: {:?}",
        provider_metadata.additional_metadata().end_session_endpoint
    );
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

async fn handle_request(oidc_state: &OidcState, request: ServiceRequest) -> MiddlewareResponse {
    log::trace!("Started OIDC middleware request handling");
    oidc_state.refresh_if_expired(&request).await;

    if request.path() == SQLPAGE_REDIRECT_URI {
        let response = handle_oidc_callback(oidc_state, request).await;
        return MiddlewareResponse::Respond(response);
    }

    if request.path() == SQLPAGE_LOGOUT_URI {
        let response = handle_oidc_logout(oidc_state, request).await;
        return MiddlewareResponse::Respond(response);
    }

    match get_authenticated_user_info(oidc_state, &request).await {
        Ok(Some(claims)) => {
            log::trace!("Storing authenticated user info in request extensions: {claims:?}");
            request.extensions_mut().insert(claims);
            MiddlewareResponse::Forward(request)
        }
        Ok(None) => {
            log::trace!("No authenticated user found");
            handle_unauthenticated_request(oidc_state, request).await
        }
        Err(e) => {
            log::debug!("An auth cookie is present but could not be verified. Redirecting to OIDC provider to re-authenticate. {e:?}");
            oidc_state.refresh_on_error(&request).await;
            handle_unauthenticated_request(oidc_state, request).await
        }
    }
}

async fn handle_unauthenticated_request(
    oidc_state: &OidcState,
    request: ServiceRequest,
) -> MiddlewareResponse {
    log::debug!("Handling unauthenticated request to {}", request.path());

    if oidc_state.config.is_public_path(request.path()) {
        return MiddlewareResponse::Forward(request);
    }

    log::debug!("Redirecting to OIDC provider");

    let initial_url = request.uri().to_string();
    let response = build_auth_provider_redirect_response(oidc_state, &initial_url).await;
    MiddlewareResponse::Respond(request.into_response(response))
}

async fn handle_oidc_callback(oidc_state: &OidcState, request: ServiceRequest) -> ServiceResponse {
    match process_oidc_callback(oidc_state, &request).await {
        Ok(response) => request.into_response(response),
        Err(e) => {
            log::error!("Failed to process OIDC callback. Refreshing oidc provider metadata, then redirecting to home page: {e:#}");
            oidc_state.refresh_on_error(&request).await;
            let resp = build_auth_provider_redirect_response(oidc_state, "/").await;
            request.into_response(resp)
        }
    }
}

async fn handle_oidc_logout(oidc_state: &OidcState, request: ServiceRequest) -> ServiceResponse {
    match process_oidc_logout(oidc_state, &request).await {
        Ok(response) => request.into_response(response),
        Err(e) => {
            log::error!("Failed to process OIDC logout: {e:#}");
            request.into_response(
                HttpResponse::BadRequest()
                    .content_type("text/plain")
                    .body(format!("Logout failed: {e}")),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
struct LogoutParams {
    state: CsrfToken,
}

async fn process_oidc_logout(
    oidc_state: &OidcState,
    request: &ServiceRequest,
) -> anyhow::Result<HttpResponse> {
    let params = Query::<LogoutParams>::from_query(request.query_string())
        .with_context(|| format!("{SQLPAGE_LOGOUT_URI}: missing or invalid state parameter"))?
        .into_inner();

    let state_cookie = get_logout_state_cookie(request, &params.state)?;
    let LogoutState { redirect_uri } = parse_logout_state(&state_cookie)?;

    let id_token_cookie = request.cookie(SQLPAGE_AUTH_COOKIE_NAME);
    let id_token = id_token_cookie
        .as_ref()
        .map(|c| OidcToken::from_str(c.value()))
        .transpose()
        .ok()
        .flatten();

    let mut response =
        if let Some(end_session_endpoint) = oidc_state.get_end_session_endpoint().await {
            let post_logout_redirect_uri = PostLogoutRedirectUrl::new(redirect_uri.to_string())
                .with_context(|| format!("Invalid post_logout_redirect_uri: {redirect_uri}"))?;

            let mut logout_request = LogoutRequest::from(end_session_endpoint)
                .set_post_logout_redirect_uri(post_logout_redirect_uri);

            if let Some(ref token) = id_token {
                logout_request = logout_request.set_id_token_hint(token);
            }

            let logout_url = logout_request.http_get_url();
            log::info!("Redirecting to OIDC logout URL: {logout_url}");
            build_redirect_response(logout_url.to_string())
        } else {
            log::info!("No end_session_endpoint, redirecting to {redirect_uri}");
            build_redirect_response(redirect_uri.to_string())
        };

    let auth_cookie = Cookie::build(SQLPAGE_AUTH_COOKIE_NAME, "")
        .secure(true)
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .path("/")
        .finish();
    response.add_removal_cookie(&auth_cookie)?;

    let nonce_cookie = Cookie::build(SQLPAGE_NONCE_COOKIE_NAME, "")
        .secure(true)
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .path("/")
        .finish();
    response.add_removal_cookie(&nonce_cookie)?;

    let mut logout_state_cookie = state_cookie;
    logout_state_cookie.set_path("/");
    response.add_removal_cookie(&logout_state_cookie)?;

    log::debug!("User logged out successfully");
    Ok(response)
}

#[derive(Debug, Serialize, Deserialize)]
struct LogoutState<'a> {
    #[serde(rename = "r")]
    redirect_uri: &'a str,
}

fn get_logout_state_cookie(
    request: &ServiceRequest,
    csrf_token: &CsrfToken,
) -> anyhow::Result<Cookie<'static>> {
    let cookie_name = SQLPAGE_LOGOUT_STATE_COOKIE_PREFIX.to_owned() + csrf_token.secret();
    request
        .cookie(&cookie_name)
        .with_context(|| format!("Invalid or expired logout state. Cookie {cookie_name} not found."))
}

fn parse_logout_state<'a>(cookie: &'a Cookie<'_>) -> anyhow::Result<LogoutState<'a>> {
    serde_json::from_str(cookie.value())
        .with_context(|| format!("Invalid logout state cookie: {}", cookie.value()))
}

pub fn create_logout_url_with_state(redirect_uri: &str, site_prefix: &str) -> (String, Cookie<'_>) {
    let csrf_token = CsrfToken::new_random();
    let cookie_name = SQLPAGE_LOGOUT_STATE_COOKIE_PREFIX.to_owned() + csrf_token.secret();
    let cookie_value = serde_json::to_string(&LogoutState { redirect_uri })
        .expect("logout state is always serializable");

    let cookie = Cookie::build(cookie_name, cookie_value)
        .secure(true)
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .path("/")
        .max_age(LOGIN_FLOW_STATE_COOKIE_EXPIRATION)
        .finish();

    let logout_url = format!(
        "{}{}?state={}",
        site_prefix.trim_end_matches('/'),
        SQLPAGE_LOGOUT_URI,
        percent_encoding::percent_encode(
            csrf_token.secret().as_bytes(),
            percent_encoding::NON_ALPHANUMERIC
        )
    );

    (logout_url, cookie)
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
        let srv = Rc::clone(&self.service);
        let oidc_state = Arc::clone(&self.oidc_state);
        Box::pin(async move {
            match handle_request(&oidc_state, request).await {
                MiddlewareResponse::Respond(response) => Ok(response),
                MiddlewareResponse::Forward(request) => srv.call(request).await,
            }
        })
    }
}

async fn process_oidc_callback(
    oidc_state: &OidcState,
    request: &ServiceRequest,
) -> anyhow::Result<HttpResponse> {
    let params = Query::<OidcCallbackParams>::from_query(request.query_string())
        .with_context(|| format!("{SQLPAGE_REDIRECT_URI}: invalid url parameters"))?
        .into_inner();
    log::debug!("Processing OIDC callback with params: {params:?}. Requesting token...");
    let mut tmp_login_flow_state_cookie = get_tmp_login_flow_state_cookie(request, &params.state)?;
    let client = oidc_state.get_client().await;
    let http_client = get_http_client_from_appdata(request)?;
    let id_token = exchange_code_for_token(&client, http_client, params.clone()).await?;
    log::debug!("Received token response: {id_token:?}");
    let LoginFlowState {
        nonce,
        redirect_target,
    } = parse_login_flow_state(&tmp_login_flow_state_cookie)?;
    let redirect_target = validate_redirect_url(redirect_target.to_string());

    log::info!("Redirecting to {redirect_target} after a successful login");
    let mut response = build_redirect_response(redirect_target);
    set_auth_cookie(&mut response, &id_token);
    let claims = oidc_state
        .get_token_claims(id_token, &nonce)
        .await
        .context("The identity provider returned an invalid ID token")?;
    log::debug!("{} successfully logged in", claims.subject().as_str());
    let nonce_cookie = create_final_nonce_cookie(&nonce);
    response.add_cookie(&nonce_cookie)?;
    tmp_login_flow_state_cookie.set_path("/"); // Required to clean up the cookie
    response.add_removal_cookie(&tmp_login_flow_state_cookie)?;
    Ok(response)
}

async fn exchange_code_for_token(
    oidc_client: &OidcClient,
    http_client: &awc::Client,
    oidc_callback_params: OidcCallbackParams,
) -> anyhow::Result<OidcToken> {
    let token_response = oidc_client
        .exchange_code(openidconnect::AuthorizationCode::new(
            oidc_callback_params.code,
        ))?
        .request_async(&AwcHttpClient::from_client(http_client))
        .await
        .context("Failed to exchange code for token")?;
    let access_token = token_response.access_token();
    log::trace!("Received access token: {}", access_token.secret());
    let id_token = token_response
        .id_token()
        .context("No ID token found in the token response. You may have specified an oauth2 provider that does not support OIDC.")?;
    Ok(id_token.clone())
}

fn set_auth_cookie(response: &mut HttpResponse, id_token: &OidcToken) {
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
        .max_age(AUTH_COOKIE_EXPIRATION)
        .same_site(actix_web::cookie::SameSite::Lax)
        .path("/")
        .finish();

    response.add_cookie(&cookie).unwrap();
}

async fn build_auth_provider_redirect_response(
    oidc_state: &OidcState,
    initial_url: &str,
) -> HttpResponse {
    let AuthUrl { url, params } = build_auth_url(oidc_state).await;
    let tmp_login_flow_state_cookie = create_tmp_login_flow_state_cookie(&params, initial_url);
    HttpResponse::SeeOther()
        .append_header((header::LOCATION, url.to_string()))
        .cookie(tmp_login_flow_state_cookie)
        .body("Redirecting...")
}

fn build_redirect_response(target_url: String) -> HttpResponse {
    HttpResponse::SeeOther()
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

    let nonce = get_final_nonce_from_cookie(request)?;
    log::debug!("Verifying id token: {id_token:?}");
    let claims = oidc_state.get_token_claims(id_token, &nonce).await?;
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

#[derive(Debug, Deserialize, Clone)]
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

fn check_nonce(id_token_nonce: Option<&Nonce>, expected_nonce: &Nonce) -> Result<(), String> {
    match id_token_nonce {
        Some(id_token_nonce) => nonce_matches(id_token_nonce, expected_nonce),
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

fn create_final_nonce_cookie(nonce: &Nonce) -> Cookie<'_> {
    Cookie::build(SQLPAGE_NONCE_COOKIE_NAME, nonce.secret())
        .secure(true)
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .max_age(AUTH_COOKIE_EXPIRATION)
        .path("/")
        .finish()
}

fn create_tmp_login_flow_state_cookie<'a>(
    params: &'a AuthUrlParams,
    initial_url: &'a str,
) -> Cookie<'a> {
    let csrf_token = &params.csrf_token;
    let cookie_name = SQLPAGE_TMP_LOGIN_STATE_COOKIE_PREFIX.to_owned() + csrf_token.secret();
    let cookie_value = serde_json::to_string(&LoginFlowState {
        nonce: params.nonce.clone(),
        redirect_target: initial_url,
    })
    .expect("login flow state is always serializable");
    Cookie::build(cookie_name, cookie_value)
        .secure(true)
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .path("/")
        .max_age(LOGIN_FLOW_STATE_COOKIE_EXPIRATION)
        .finish()
}

fn get_final_nonce_from_cookie(request: &ServiceRequest) -> anyhow::Result<Nonce> {
    let cookie = request
        .cookie(SQLPAGE_NONCE_COOKIE_NAME)
        .with_context(|| format!("No {SQLPAGE_NONCE_COOKIE_NAME} cookie found"))?;
    Ok(Nonce::new(cookie.value().to_string()))
}

fn get_tmp_login_flow_state_cookie(
    request: &ServiceRequest,
    csrf_token: &CsrfToken,
) -> anyhow::Result<Cookie<'static>> {
    let cookie_name = SQLPAGE_TMP_LOGIN_STATE_COOKIE_PREFIX.to_owned() + csrf_token.secret();
    request
        .cookie(&cookie_name)
        .with_context(|| format!("No {cookie_name} cookie found"))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoginFlowState<'a> {
    #[serde(rename = "n")]
    nonce: Nonce,
    #[serde(rename = "r")]
    redirect_target: &'a str,
}

fn parse_login_flow_state<'a>(cookie: &'a Cookie<'_>) -> anyhow::Result<LoginFlowState<'a>> {
    serde_json::from_str(cookie.value())
        .with_context(|| format!("Invalid login flow state cookie: {}", cookie.value()))
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
    if url.starts_with('/') && !url.starts_with("//") && !url.starts_with(SQLPAGE_REDIRECT_URI) {
        return url;
    }
    log::warn!("Refusing to redirect to {url}");
    '/'.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn login_redirects_use_see_other() {
        let response = build_redirect_response("/foo".to_string());
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("missing location header")
            .to_str()
            .expect("invalid location header");
        assert_eq!(location, "/foo");
    }

    #[test]
    fn parse_auth0_rfc3339_updated_at() {
        let claims_json = r#"{
            "sub": "auth0|123456",
            "iss": "https://example.auth0.com/",
            "aud": "test-client-id",
            "iat": 1700000000,
            "exp": 1700086400,
            "updated_at": "2023-11-14T12:00:00.000Z"
        }"#;
        let claims: OidcClaims = serde_json::from_str(claims_json)
            .expect("Auth0 returns updated_at as RFC3339 string, not unix timestamp");
        assert!(claims.updated_at().is_some());
    }
}
