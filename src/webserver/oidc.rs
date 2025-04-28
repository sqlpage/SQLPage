use std::{
    future::Future,
    hash::{DefaultHasher, Hash, Hasher},
    pin::Pin,
    str::FromStr,
    sync::Arc,
};

use crate::{app_config::AppConfig, AppState};
use actix_web::{
    cookie::Cookie,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware::Condition,
    web::{self, Query},
    Error, HttpResponse,
};
use anyhow::{anyhow, Context};
use awc::Client;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreGenderClaim, CoreIdToken},
    url::Url,
    AsyncHttpClient, CsrfToken, EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IdTokenClaims, IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, Scope,
    TokenResponse,
};
use password_hash::{rand_core::OsRng, SaltString};
use serde::{Deserialize, Serialize};

use super::http_client::make_http_client;

const SQLPAGE_AUTH_COOKIE_NAME: &str = "sqlpage_auth";
const SQLPAGE_REDIRECT_URI: &str = "/sqlpage/oidc_callback";
const SQLPAGE_STATE_COOKIE_NAME: &str = "sqlpage_oidc_state";

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

        let app_host = get_app_host(config);

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
         using \"{}\" as the app host to build the redirect URL. \
         This will only work locally. \
         Disable this warning by providing a value for the \"host\" setting.",
        host
    );
    host
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
    http_client: &AwcHttpClient,
    issuer_url: IssuerUrl,
) -> anyhow::Result<openidconnect::core::CoreProviderMetadata> {
    log::debug!("Discovering provider metadata for {}", issuer_url);
    let provider_metadata =
        openidconnect::core::CoreProviderMetadata::discover_async(issuer_url, http_client)
            .await
            .with_context(|| format!("Failed to discover OIDC provider metadata"))?;
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

#[derive(Clone)]
pub struct OidcService<S> {
    service: S,
    app_state: web::Data<AppState>,
    config: Arc<OidcConfig>,
    oidc_client: Arc<OidcClient>,
    http_client: Arc<AwcHttpClient>,
}

impl<S> OidcService<S> {
    pub async fn new(
        service: S,
        app_state: &web::Data<AppState>,
        config: Arc<OidcConfig>,
    ) -> anyhow::Result<Self> {
        let issuer_url = config.issuer_url.clone();
        let http_client = AwcHttpClient::new(&app_state.config)?;
        let provider_metadata = discover_provider_metadata(&http_client, issuer_url).await?;
        let client: OidcClient = make_oidc_client(&config, provider_metadata)?;
        Ok(Self {
            service,
            app_state: web::Data::clone(app_state),
            config,
            oidc_client: Arc::new(client),
            http_client: Arc::new(http_client),
        })
    }

    fn build_auth_url(&self, request: &ServiceRequest) -> String {
        let (auth_url, csrf_token, nonce) = self
            .oidc_client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scopes(self.config.scopes.iter().cloned())
            .url();
        auth_url.to_string()
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

        log::debug!("Redirecting to OIDC provider");

        let response =
            build_auth_provider_redirect_response(&self.oidc_client, &self.config, &request);
        Box::pin(async move { Ok(request.into_response(response)) })
    }

    fn handle_oidc_callback(
        &self,
        request: ServiceRequest,
    ) -> LocalBoxFuture<Result<ServiceResponse<BoxBody>, Error>> {
        let oidc_client = Arc::clone(&self.oidc_client);
        let http_client = Arc::clone(&self.http_client);
        let oidc_config = Arc::clone(&self.config);

        Box::pin(async move {
            let query_string = request.query_string();
            match process_oidc_callback(&oidc_client, &http_client, query_string, &request).await {
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
        let oidc_client = Arc::clone(&self.oidc_client);
        match get_sqlpage_auth_cookie(&oidc_client, &request) {
            Ok(Some(cookie)) => {
                log::trace!("Found SQLPage auth cookie: {cookie}");
            }
            Ok(None) => {
                log::trace!("No SQLPage auth cookie found");
                return self.handle_unauthenticated_request(request);
            }
            Err(e) => {
                log::error!("Found an invalid SQLPage auth cookie: {e}");
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
    oidc_client: &Arc<OidcClient>,
    http_client: &Arc<AwcHttpClient>,
    query_string: &str,
    request: &ServiceRequest,
) -> anyhow::Result<HttpResponse> {
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
    set_auth_cookie(&mut response, &token_response)?;
    Ok(response)
}

async fn exchange_code_for_token(
    oidc_client: &OidcClient,
    http_client: &AwcHttpClient,
    oidc_callback_params: OidcCallbackParams,
) -> anyhow::Result<openidconnect::core::CoreTokenResponse> {
    // TODO: Verify the state matches the expected CSRF token
    let token_response = oidc_client
        .exchange_code(openidconnect::AuthorizationCode::new(
            oidc_callback_params.code,
        ))?
        .request_async(http_client)
        .await?;
    Ok(token_response)
}

fn set_auth_cookie(
    response: &mut HttpResponse,
    token_response: &openidconnect::core::CoreTokenResponse,
) -> anyhow::Result<()> {
    let access_token = token_response.access_token();
    log::trace!("Received access token: {}", access_token.secret());
    let id_token = token_response
        .id_token()
        .context("No ID token found in the token response. You may have specified an oauth2 provider that does not support OIDC.")?;

    let id_token_str = id_token.to_string();
    log::trace!("Setting auth cookie: {SQLPAGE_AUTH_COOKIE_NAME}=\"{id_token_str}\"");
    let cookie = Cookie::build(SQLPAGE_AUTH_COOKIE_NAME, id_token_str)
        .secure(true)
        .http_only(true)
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

fn get_sqlpage_auth_cookie(
    oidc_client: &OidcClient,
    request: &ServiceRequest,
) -> anyhow::Result<Option<String>> {
    let Some(cookie) = request.cookie(SQLPAGE_AUTH_COOKIE_NAME) else {
        return Ok(None);
    };
    let cookie_value = cookie.value().to_string();

    let verifier = oidc_client.id_token_verifier();
    let id_token = CoreIdToken::from_str(&cookie_value)
        .with_context(|| anyhow!("Invalid SQLPage auth cookie"))?;

    let nonce_verifier = |_: Option<&Nonce>| Ok(());
    let claims: &IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim> = id_token
        .claims(&verifier, nonce_verifier)
        .with_context(|| anyhow!("Invalid SQLPage auth cookie"))?;
    log::debug!("The current user is: {claims:?}");
    Ok(Some(cookie_value))
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
    // low-cost parameters
    let params = argon2::Params::new(8, 1, 1, None).expect("bug: invalid Argon2 parameters");
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = argon2
        .hash_password(nonce.secret().as_bytes(), &salt)
        .expect("bug: failed to hash nonce");
    hash.to_string()
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
        .with_context(|| format!("Failed to parse OIDC state from cookie"))
}
