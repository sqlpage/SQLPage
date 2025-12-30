use actix_web::{
    cookie::Cookie,
    http::{header, StatusCode},
    test,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use base64::Engine;
use openidconnect::url::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlpage::webserver::http::create_app;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::{CancellationToken, DropGuard};

fn base64url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn make_jwt(claims: &serde_json::Value, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let header = json!({
        "alg": "HS256",
        "typ": "JWT",
        "kid": "test"
    });

    let header_b64 = base64url_encode(header.to_string().as_bytes());
    let payload_b64 = base64url_encode(claims.to_string().as_bytes());

    let message = format!("{}.{}", header_b64, payload_b64);

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(message.as_bytes());
    let signature = mac.finalize().into_bytes();
    let signature_b64 = base64url_encode(&signature);

    format!("{}.{}.{}", header_b64, payload_b64, signature_b64)
}

type JwtCustomizer<'a> = dyn Fn(serde_json::Value, &str) -> String + Send + Sync + 'a;

struct ProviderState<'a> {
    secret: String,
    issuer_url: String,
    client_id: String,
    auth_codes: HashMap<String, String>, // code -> nonce
    jwt_customizer: Option<Box<JwtCustomizer<'a>>>,
}

type ProviderStateWithLifetime<'a> = ProviderState<'a>;
type SharedProviderState = Arc<Mutex<ProviderStateWithLifetime<'static>>>;

#[derive(Serialize, Deserialize)]
struct DiscoveryResponse {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
}

#[derive(Serialize)]
struct JwksResponse {
    keys: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    id_token: String,
    expires_in: i64,
}

async fn discovery_endpoint(state: Data<SharedProviderState>) -> impl Responder {
    let state = state.lock().unwrap();
    let discovery = DiscoveryResponse {
        issuer: state.issuer_url.clone(),
        authorization_endpoint: format!("{}/auth", state.issuer_url),
        token_endpoint: format!("{}/token", state.issuer_url),
        jwks_uri: format!("{}/jwks", state.issuer_url),
        response_types_supported: vec!["code".to_string()],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["HS256".to_string()],
    };
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .json(discovery)
}

async fn jwks_endpoint(state: Data<SharedProviderState>) -> impl Responder {
    let state = state.lock().unwrap();
    let jwks = JwksResponse {
        keys: vec![json!({
            "kty": "oct",
            "kid": "test",
            "use": "sig",
            "alg": "HS256",
            "k": base64url_encode(state.secret.as_bytes())
        })],
    };
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .json(jwks)
}

async fn token_endpoint(
    state: Data<SharedProviderState>,
    req: web::Form<HashMap<String, String>>,
) -> impl Responder {
    let mut state = state.lock().unwrap();
    let Some(code) = req.get("code") else {
        return HttpResponse::BadRequest().body("Missing code");
    };
    let nonce = state.auth_codes.get(code).cloned().unwrap_or_default();
    if nonce.is_empty() {
        return HttpResponse::BadRequest().body("Unknown code");
    }

    let now = chrono::Utc::now().timestamp();
    let claims = json!({
        "iss": state.issuer_url.as_str(),
        "sub": "test_user",
        "aud": state.client_id.as_str(),
        "exp": now + 3600,
        "iat": now,
        "nonce": nonce,
    });

    let id_token = state
        .jwt_customizer
        .take()
        .map(|customizer| customizer(claims.clone(), &state.secret))
        .unwrap_or_else(|| make_jwt(&claims, &state.secret));

    let response = TokenResponse {
        access_token: "test_access_token".to_string(),
        token_type: "Bearer".to_string(),
        id_token,
        expires_in: 3600,
    };

    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .json(response)
}

pub struct FakeOidcProvider {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    state: SharedProviderState,
    _stop_on_drop: DropGuard,
}

fn extract_set_cookies(headers: &header::HeaderMap) -> Vec<Cookie<'static>> {
    headers
        .get_all(header::SET_COOKIE)
        .filter_map(|h| h.to_str().ok())
        .filter_map(|s| Cookie::parse(s).ok())
        .map(Cookie::into_owned)
        .collect()
}

impl FakeOidcProvider {
    pub async fn new() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let issuer_url = format!("http://127.0.0.1:{}", port);
        let client_id = "test_client".to_string();
        let client_secret = "test_secret".to_string();

        let state: SharedProviderState = Arc::new(Mutex::new(ProviderState {
            secret: client_secret.clone(),
            issuer_url: issuer_url.clone(),
            client_id: client_id.clone(),
            auth_codes: HashMap::new(),
            jwt_customizer: None,
        }));

        let state_for_server = Arc::clone(&state);

        let server_stop = CancellationToken::new();
        let stop_on_drop = server_stop.clone().drop_guard();

        let server = HttpServer::new(move || {
            let state = Data::new(Arc::clone(&state_for_server));
            App::new()
                .app_data(state.clone())
                .route(
                    "/.well-known/openid-configuration",
                    web::get().to(discovery_endpoint),
                )
                .route("/jwks", web::get().to(jwks_endpoint))
                .route("/token", web::post().to(token_endpoint))
        })
        .workers(1)
        .listen(listener)
        .unwrap()
        .shutdown_timeout(1)
        .shutdown_signal(server_stop.cancelled_owned())
        .run();

        tokio::spawn(server);

        Self {
            issuer_url,
            client_id,
            client_secret,
            state,
            _stop_on_drop: stop_on_drop,
        }
    }

    fn with_state_mut<R>(&self, f: impl FnOnce(&mut ProviderState) -> R) -> R {
        let mut state = self.state.lock().unwrap();
        f(&mut state)
    }

    pub fn store_auth_code(&self, code: String, nonce: String) {
        self.with_state_mut(|s| {
            s.auth_codes.insert(code, nonce);
        });
    }
}

fn get_query_param(url: &Url, name: &str) -> String {
    url.query_pairs()
        .find(|(k, _)| k == name)
        .unwrap()
        .1
        .to_string()
}

macro_rules! request_with_cookies {
    ($app:expr, $req:expr, $cookies:expr) => {{
        let mut req = $req;
        for cookie in $cookies.iter() {
            req = req.cookie(cookie.clone());
        }
        let resp = test::call_service(&$app, req.to_request()).await;
        for new_cookie in extract_set_cookies(resp.headers()) {
            $cookies.retain(|c: &Cookie| c.name() != new_cookie.name());
            if !new_cookie.value().is_empty() {
                $cookies.push(new_cookie);
            }
        }
        resp
    }};
}

async fn setup_oidc_test(
    provider_mutator: impl FnOnce(&mut ProviderState),
) -> (
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    FakeOidcProvider,
) {
    use sqlpage::{
        app_config::{test_database_url, AppConfig},
        AppState,
    };
    crate::common::init_log();
    let provider = FakeOidcProvider::new().await;
    provider.with_state_mut(provider_mutator);

    let db_url = test_database_url();
    let config_json = format!(
        r#"{{
        "database_url": "{db_url}",
        "max_database_pool_connections": 1,
        "database_connection_retries": 3,
        "database_connection_acquire_timeout_seconds": 15,
        "allow_exec": true,
        "max_uploaded_file_size": 123456,
        "listen_on": "127.0.0.1:0",
        "system_root_ca_certificates": false,
        "oidc_issuer_url": "{}",
        "oidc_client_id": "{}",
        "oidc_client_secret": "{}",
        "oidc_protected_paths": ["/"],
        "host": "localhost:1"
    }}"#,
        provider.issuer_url, provider.client_id, provider.client_secret
    );

    let config: AppConfig = serde_json::from_str(&config_json).unwrap();
    let app_state = AppState::init(&config).await.unwrap();
    let app = test::init_service(create_app(Data::new(app_state))).await;
    (app, provider)
}

#[actix_web::test]
async fn test_oidc_happy_path() {
    let (app, provider) = setup_oidc_test(|_| {}).await;
    let mut cookies: Vec<Cookie<'static>> = Vec::new();

    let resp = request_with_cookies!(app, test::TestRequest::get().uri("/"), cookies);
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let auth_url = Url::parse(resp.headers().get("location").unwrap().to_str().unwrap()).unwrap();

    let state = get_query_param(&auth_url, "state");
    let nonce = get_query_param(&auth_url, "nonce");
    let redirect_uri = get_query_param(&auth_url, "redirect_uri");
    provider.store_auth_code("test_auth_code".to_string(), nonce);

    let callback_uri = format!(
        "{}?code=test_auth_code&state={}",
        Url::parse(&redirect_uri).unwrap().path(),
        state
    );
    let callback_resp =
        request_with_cookies!(app, test::TestRequest::get().uri(&callback_uri), cookies);
    assert_eq!(callback_resp.status(), StatusCode::SEE_OTHER);

    let final_resp = request_with_cookies!(app, test::TestRequest::get().uri("/"), cookies);
    assert_eq!(final_resp.status(), StatusCode::OK);
}

async fn assert_oidc_login_fails(
    provider_mutator: impl FnOnce(&mut ProviderState),
    state_override: Option<String>,
) {
    let (app, provider) = setup_oidc_test(provider_mutator).await;
    let mut cookies: Vec<Cookie<'static>> = Vec::new();

    let resp = request_with_cookies!(app, test::TestRequest::get().uri("/"), cookies);
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let auth_url = Url::parse(resp.headers().get("location").unwrap().to_str().unwrap()).unwrap();

    let state = get_query_param(&auth_url, "state");
    let nonce = get_query_param(&auth_url, "nonce");
    let redirect_uri = get_query_param(&auth_url, "redirect_uri");
    provider.store_auth_code("test_auth_code".to_string(), nonce);

    let callback_state = state_override.unwrap_or(state);
    let callback_uri = format!(
        "{}?code=test_auth_code&state={}",
        Url::parse(&redirect_uri).unwrap().path(),
        callback_state
    );
    let callback_resp =
        request_with_cookies!(app, test::TestRequest::get().uri(&callback_uri), cookies);

    assert_eq!(callback_resp.status(), StatusCode::SEE_OTHER);
    let location = callback_resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        location.starts_with(&provider.issuer_url),
        "Expected redirect to OIDC provider, but got {location}"
    );

    let auth_cookie_present = cookies.iter().any(|c| c.name() == "sqlpage_auth");
    assert!(
        !auth_cookie_present,
        "Authentication cookie should not be set on failure"
    );
}

async fn assert_oidc_callback_fails_with_bad_jwt(
    mutate_jwt_claims: impl FnMut(&mut serde_json::Value) + Send + Sync + 'static,
) {
    let mutate_jwt_claims = Mutex::new(mutate_jwt_claims);
    assert_oidc_login_fails(
        |state| {
            state.jwt_customizer = Some(Box::new(move |mut claims, secret| {
                mutate_jwt_claims.lock().unwrap()(&mut claims);
                make_jwt(&claims, secret)
            }));
        },
        None,
    )
    .await;
}

#[actix_web::test]
async fn test_oidc_csrf_state_mismatch_is_rejected() {
    assert_oidc_login_fails(|_| {}, Some("wrong_state".to_string())).await;
}

#[actix_web::test]
async fn test_oidc_nonce_mismatch_is_rejected() {
    assert_oidc_callback_fails_with_bad_jwt(|claims| {
        claims["nonce"] = json!("wrong_nonce");
    })
    .await;
}

#[actix_web::test]
async fn test_oidc_bad_signature_is_rejected() {
    assert_oidc_login_fails(
        |state| {
            state.jwt_customizer = Some(Box::new(|claims, _| make_jwt(&claims, "wrong_secret")));
        },
        None,
    )
    .await;
}

#[actix_web::test]
async fn test_oidc_wrong_audience_is_rejected() {
    assert_oidc_callback_fails_with_bad_jwt(|claims| {
        claims["aud"] = json!("wrong_client");
    })
    .await;
}

#[actix_web::test]
async fn test_oidc_wrong_issuer_is_rejected() {
    assert_oidc_callback_fails_with_bad_jwt(|claims| {
        claims["iss"] = json!("https://wrong-issuer.com");
    })
    .await;
}

#[actix_web::test]
async fn test_oidc_expired_token_is_rejected() {
    assert_oidc_callback_fails_with_bad_jwt(|claims| {
        let current_exp = claims["exp"].as_i64().unwrap();
        claims["exp"] = json!(current_exp - 7200);
    })
    .await;
}
