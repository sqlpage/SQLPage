use actix_web::{http::header, test, web::Data};
use sqlpage::{
    AppState,
    app_config::{self, AppConfig},
    webserver::{
        database::migrations,
        http::{form_config, main_handler, payload_config},
    },
};
use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

struct ExampleSmoke {
    name: &'static str,
    web_root: Option<&'static str>,
    request_path: &'static str,
}

const SQLITE_SMOKE_EXAMPLES: &[ExampleSmoke] = &[
    ExampleSmoke {
        name: "CRUD - Authentication",
        web_root: Some("www"),
        request_path: "/",
    },
    ExampleSmoke {
        name: "cards-with-remote-content",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "charts, computations and custom components",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "corporate-conundrum",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "forms with a variable number of fields",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "forms-with-multiple-steps",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "handle-404",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "image gallery with user uploads",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "light-dark-toggle",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "master-detail-forms",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "modeling a many to many relationship with a form",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "multiple-choice-question",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "official-site",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "plots tables and forms",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "read-and-set-http-cookies",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "rich-text-editor",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "roundest_pokemon_rating",
        web_root: Some("src"),
        request_path: "/",
    },
    ExampleSmoke {
        name: "sending emails",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "simple-website-example",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "splitwise",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "todo application",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "using react and other custom scripts and styles",
        web_root: None,
        request_path: "/",
    },
    ExampleSmoke {
        name: "user-authentication",
        web_root: None,
        request_path: "/",
    },
];

const EXTERNAL_SERVICE_EXAMPLES: &[&str] = &[
    "PostGIS - using sqlpage with geographic data",
    "SQLPage developer user interface",
    "custom form component",
    "make a geographic data application using sqlite extensions",
    "microsoft sql server advanced forms",
    "mysql json handling",
    "nginx",
    "single sign on",
    "telemetry",
    "tiny_twitter",
    "todo application (PostgreSQL)",
    "web servers - apache",
];

#[actix_web::test]
async fn examples_folder_is_fully_accounted_for() {
    let actual = top_level_example_directories();
    let accounted_for = SQLITE_SMOKE_EXAMPLES
        .iter()
        .map(|example| example.name.to_owned())
        .chain(
            EXTERNAL_SERVICE_EXAMPLES
                .iter()
                .map(|name| (*name).to_owned()),
        )
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, accounted_for);
}

#[actix_web::test]
async fn sqlite_compatible_examples_render_their_entry_page() {
    for example in SQLITE_SMOKE_EXAMPLES {
        let response = request_example(example).await.unwrap_or_else(|err| {
            panic!(
                "failed to render entry page for example {:?}: {err:#}",
                example.name
            )
        });

        let status = response.status();
        let body = test::read_body(response).await;
        let body = String::from_utf8_lossy(&body);
        assert!(
            status.is_success()
                || status.is_redirection()
                || status == actix_web::http::StatusCode::UNAUTHORIZED,
            "example {:?} returned status {status}; body:\n{body}",
            example.name
        );
        assert!(
            !body.contains("SQLPage Error"),
            "example {:?} rendered an SQLPage error with status {status}; body:\n{body}",
            example.name
        );
    }
}

async fn request_example(
    example: &ExampleSmoke,
) -> anyhow::Result<actix_web::dev::ServiceResponse> {
    let app_data = make_example_app_data(example).await?;
    let request = test::TestRequest::get()
        .uri(example.request_path)
        .insert_header(header::Accept::html())
        .app_data(payload_config(&app_data))
        .app_data(form_config(&app_data))
        .app_data(app_data)
        .to_srv_request();
    tokio::time::timeout(Duration::from_secs(8), main_handler(request))
        .await
        .map_err(|err| anyhow::anyhow!("request timed out: {err}"))?
        .map_err(|err| anyhow::anyhow!("request failed: {err:#}"))
}

async fn make_example_app_data(example: &ExampleSmoke) -> anyhow::Result<Data<AppState>> {
    sqlpage::telemetry::init_test_logging();

    let root = Path::new("examples").join(example.name);
    let mut config = load_example_config(&root)?;
    config.database_url = "sqlite://:memory:?cache=shared".to_owned();
    config.max_database_pool_connections = Some(1);
    config.database_connection_retries = 0;
    config.database_connection_acquire_timeout_seconds = 8.0;
    config.web_root = example
        .web_root
        .map_or_else(|| root.clone(), |web_root| root.join(web_root));

    let state = Data::new(AppState::init(&config).await?);
    migrations::apply(&config, &state.db).await?;
    Ok(state)
}

fn load_example_config(root: &Path) -> anyhow::Result<AppConfig> {
    let config_dir = if root.join("sqlpage").exists() {
        root.join("sqlpage")
    } else if root.join("sqlpage_config").exists() {
        root.join("sqlpage_config")
    } else {
        root.join("sqlpage")
    };

    if config_dir.join("sqlpage.json").exists() || config_dir.join("sqlpage.yaml").exists() {
        let mut config = app_config::load_from_directory(&config_dir)?;
        config.configuration_directory = config_dir;
        Ok(config)
    } else {
        let mut config = serde_json::from_str::<AppConfig>(
            r#"{
                "database_url": "sqlite://:memory:",
                "max_database_pool_connections": 1,
                "database_connection_retries": 0,
                "database_connection_acquire_timeout_seconds": 8
            }"#,
        )?;
        config.configuration_directory = config_dir;
        Ok(config)
    }
}

fn top_level_example_directories() -> BTreeSet<String> {
    std::fs::read_dir("examples")
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect()
}
