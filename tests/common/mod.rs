use std::fmt::Write as _;
use std::time::Duration;

use actix_web::{
    App, HttpResponse, HttpServer,
    dev::{ServiceRequest, fn_service},
    http::header,
    http::header::ContentType,
    test::TestRequest,
    web,
    web::Data,
};
use sqlpage::{
    AppState,
    app_config::{AppConfig, test_database_url},
    telemetry,
    webserver::http::{form_config, main_handler, payload_config},
};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub(crate) async fn get_request_to_with_data(
    path: &str,
    data: Data<AppState>,
) -> actix_web::Result<TestRequest> {
    Ok(TestRequest::get()
        .uri(path)
        .insert_header(ContentType::plaintext())
        .insert_header(header::Accept::html())
        .app_data(payload_config(&data))
        .app_data(form_config(&data))
        .app_data(data))
}

pub(crate) async fn get_request_to(path: &str) -> actix_web::Result<TestRequest> {
    let data = make_app_data().await;
    get_request_to_with_data(path, data).await
}

pub(crate) async fn make_app_data_from_config(config: AppConfig) -> Data<AppState> {
    let state = make_app_state_from_config(&config).await.unwrap();
    Data::new(state)
}

// Pools must be emptied while the test runtime is still running.
//
// Each request built with `TestRequest::to_srv_request()` leaks its
// `Data<AppState>`: actix-web recycles the `HttpRequestInner` allocation into
// a per-request pool on drop, and that pool is only drained by a real server,
// never on the test path. The leaked `AppState` pins its database pool (and
// its pooled connections) until process exit. The ODBC backend prepares and
// caches a native statement for every distinct parameterized query, and
// Oracle's ODBC driver deadlocks in its process destructor when connections
// still hold native statements at exit.
//
// Note that `pool.close()` alone is not sufficient: it only closes idle
// connections, so one still checked out (e.g. by a return-to-pool task
// finishing the test's last request) would keep its cached statements alive.
// Acquire every connection and clear its cache first to free those handles.
thread_local! {
    static TEST_POOLS: std::cell::RefCell<Vec<sqlx::any::AnyPool>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) struct TestSystem(actix_web::rt::SystemRunner);

impl TestSystem {
    pub(crate) fn new() -> Self {
        Self(actix_web::rt::System::new())
    }

    pub(crate) fn block_on<F: Future>(&self, future: F) -> F::Output {
        use futures_util::FutureExt as _;

        self.0.block_on(async {
            let result = std::panic::AssertUnwindSafe(future).catch_unwind().await;
            // Empty every pool created by the test while the runtime is still
            // alive, even if the test panicked. Clearing first releases cached
            // prepared statements on connections that may still be checked out
            // (which `close()` alone would leave behind); closing then
            // disconnects each pooled connection. The pool objects themselves
            // may stay alive (see above), but they are left empty.
            let pools = TEST_POOLS.with(std::cell::RefCell::take);
            for pool in pools {
                use sqlx::connection::Connection as _;

                let mut connections = Vec::new();
                for _ in 0..pool.size() {
                    let mut connection = pool.acquire().await.unwrap();
                    connection.clear_cached_statements().await.unwrap();
                    connections.push(connection);
                }
                drop(connections);
                pool.close().await;
            }
            match result {
                Ok(output) => output,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        })
    }
}

pub(crate) async fn make_app_state_from_config(config: &AppConfig) -> anyhow::Result<AppState> {
    let state = AppState::init(config).await?;
    TEST_POOLS.with(|pools| pools.borrow_mut().push(state.db.connection.clone()));
    Ok(state)
}

#[test]
fn test_system_closes_pools_on_success_and_panic() {
    for panic_in_test in [false, true] {
        let system = TestSystem::new();
        let mut pool = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            system.block_on(async {
                let mut config = test_config();
                config.database_url = "sqlite::memory:".to_owned();
                let state = make_app_state_from_config(&config).await.unwrap();
                pool = Some(state.db.connection.clone());
                assert!(!panic_in_test, "intentional test panic");
            });
        }));
        assert_eq!(result.is_err(), panic_in_test);
        assert!(pool.unwrap().is_closed());
    }
}

/// Reads a whole response body as a UTF-8 string, failing the test otherwise.
pub(crate) async fn read_body_string<B>(resp: actix_web::dev::ServiceResponse<B>) -> String
where
    B: actix_web::body::MessageBody,
{
    String::from_utf8(actix_web::test::read_body(resp).await.to_vec()).unwrap()
}

/// Whether the database engine is one of `kinds`.
/// Tests that only run on some engines return early otherwise.
pub(crate) fn supports_database(
    db: &sqlpage::webserver::database::Database,
    kinds: &[sqlpage::webserver::database::SupportedDatabase],
) -> bool {
    kinds.contains(&db.info.database_type)
}

pub(crate) async fn make_app_data() -> Data<AppState> {
    init_log();
    let config = test_config();
    make_app_data_from_config(config).await
}

/// Creates test application state running in the given environment
/// (development enables debug helpers like `Server-Timing`).
pub(crate) async fn make_app_data_with_env(
    environment: sqlpage::app_config::DevOrProd,
) -> Data<AppState> {
    init_log();
    let mut config = test_config();
    config.environment = environment;
    make_app_data_from_config(config).await
}

pub(crate) async fn req_path(
    path: impl AsRef<str>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let req = get_request_to(path.as_ref()).await?.to_srv_request();
    main_handler(req).await
}

const REQ_TIMEOUT: Duration = Duration::from_secs(8);
pub(crate) async fn req_path_with_app_data(
    path: impl AsRef<str>,
    app_data: Data<AppState>,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    req_path_with_app_data_and_accept(path, app_data, header::Accept::html()).await
}

pub(crate) async fn req_path_with_app_data_json(
    path: impl AsRef<str>,
    app_data: Data<AppState>,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    req_path_with_app_data_and_accept(path, app_data, header::Accept::json()).await
}

async fn req_path_with_app_data_and_accept(
    path: impl AsRef<str>,
    app_data: Data<AppState>,
    accept: header::Accept,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    let path = path.as_ref();
    let req = get_request_to_with_data(path, app_data)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build request for {path}: {e}"))?
        .insert_header(("cookie", "test_cook=123"))
        .insert_header(("authorization", "Basic dGVzdDp0ZXN0"))
        .insert_header(accept)
        .to_srv_request();
    let resp = tokio::time::timeout(REQ_TIMEOUT, main_handler(req))
        .await
        .map_err(|e| anyhow::anyhow!("Request to {path} timed out: {e}"))?
        .map_err(|e| {
            anyhow::anyhow!(
                "Request to {path} failed with status {}: {e:#}",
                e.as_response_error().status_code()
            )
        })?;
    Ok(resp)
}

pub(crate) fn test_config() -> AppConfig {
    let db_url = test_database_url();
    serde_json::from_str::<AppConfig>(&format!(
        r#"{{
        "database_url": "{db_url}",
        "max_database_pool_connections": 1,
        "database_connection_retries": 3,
        "database_connection_acquire_timeout_seconds": 15,
        "allow_exec": true,
        "max_uploaded_file_size": 123456,
        "listen_on": "111.111.111.111:1",
        "system_root_ca_certificates" : false
    }}"#
    ))
    .unwrap()
}

pub(crate) fn init_log() {
    telemetry::init_test_logging();
}

fn format_request_line_and_headers(req: &ServiceRequest) -> String {
    let mut out = format!("{} {}", req.method(), req.uri());
    let mut headers: Vec<_> = req.headers().iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in headers {
        if k.as_str().eq_ignore_ascii_case("date") {
            continue;
        }
        write!(out, "|{k}: {}", v.to_str().unwrap_or("?")).unwrap();
    }
    out
}

async fn format_body(req: &mut ServiceRequest) -> Vec<u8> {
    req.extract::<web::Bytes>()
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

fn build_echo_response(body: &[u8], meta: String) -> HttpResponse {
    let mut resp = meta.into_bytes();
    resp.push(b'|');
    resp.extend_from_slice(body);
    HttpResponse::Ok()
        .insert_header((header::DATE, "Mon, 24 Feb 2025 12:00:00 GMT"))
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .body(resp)
}

pub(crate) fn start_echo_server(shutdown: oneshot::Receiver<()>) -> (JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(|| {
        App::new()
            .route(
                "/json",
                web::to(|body: web::Bytes| async move {
                    HttpResponse::Ok()
                        .insert_header((header::CONTENT_TYPE, "application/json"))
                        .body(body)
                }),
            )
            .default_service(fn_service(|mut req: ServiceRequest| async move {
                let meta = format_request_line_and_headers(&req);
                let body = format_body(&mut req).await;
                let resp = build_echo_response(&body, meta);
                Ok(req.into_response(resp))
            }))
    })
    .workers(1)
    .listen(listener)
    .unwrap()
    .shutdown_timeout(1)
    .run();
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = server => {},
            _ = shutdown => {},
        }
    });
    (handle, port)
}
