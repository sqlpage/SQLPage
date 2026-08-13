# SQLPage architecture

SQLPage is an SQL-only web application builder and web server. An application is primarily a set of `.sql`
files: SQLPage routes an HTTP request to a file, executes its statements against a database, interprets rows
whose `component` column names a UI component, and streams the resulting HTML (or another response) to the
client. It is intended for fast, data-centric applications while still allowing custom HTML, CSS, and
JavaScript where needed.

## Features and repository layout

- **Application entry point and configuration** (`src/main.rs`, `src/lib.rs`, `src/app_config.rs`, `src/cli/`).
  The executable starts the server; application state, configuration, environment variables, and command-line
  handling are defined here. `configuration.md` documents the user-facing settings.
- **SQL semantics and execution** (`src/webserver/database/`). SQLPage uses the database's SQL for selects, joins, aggregation, inserts, updates,
  deletes, transactions, JSON processing, and database-specific features. It parses SQL, recognizes SQLPage
  extensions, binds request values safely, and sends ordinary SQL to the selected database. SQL files contain
  sequential statements; result sets become component invocations in response order. `SET` assigns a value
  to a mutable SQLPage variable and is useful for reusing query results or controlling later statements.
- **Request variables** (`src/webserver/request_variables.rs`, `src/webserver/http_request_info.rs`,
  `src/webserver/database/syntax_tree.rs`). `?name` refers to a URL/GET parameter, `:name` explicitly refers to a form/POST
  value, and `$name` is the compatibility shorthand that uses a POST value when present and otherwise a GET
  value (a SET variable takes precedence where applicable). Values are passed as parameters, not interpolated
  into SQL. GET and POST variables are request inputs; SET variables are mutable during request execution.
  `sqlpage.variables()` exposes them as JSON, with SET > POST > GET precedence.
- **SQLPage functions** (`src/webserver/database/sqlpage_functions/`). Calls such as `sqlpage.fetch`, `sqlpage.run_sql`, `sqlpage.set_variable`, file
  readers, hashing/HMAC helpers, request metadata, uploads, headers/cookies, URL helpers, OIDC user info,
  and HTTP fetch are registered in `src/webserver/database/sqlpage_functions/functions.rs`. Functions can
  return values, alter response/request state, include another SQL file, or raise an error. `sqlpage.exec`
  is deliberately disabled by default because it runs server processes.
- **Database support and pooling** (`src/webserver/database/connect.rs`, `execute_queries.rs`, `migrations.rs`).
  Native drivers support SQLite, PostgreSQL, MySQL, and Microsoft SQL
  Server; the ODBC driver provides access to other ODBC-compatible databases. SQLPage uses `sqlx` and a
  reusable connection pool, with configurable maximum connections, idle/lifetime timeouts, acquire timeout,
  retries, and optional `on_connect.sql`/`on_reset.sql` hooks. Database-specific SQL should be isolated or
  covered by the relevant database tests.
- **Rendering and components** (`src/render.rs`, `src/templates.rs`, `src/dynamic_component.rs`,
  `src/template_helpers.rs`, `sqlpage/templates/`, `sqlpage/sqlpage.css`, `sqlpage/sqlpage.js`). Built-in components live in `sqlpage/templates/*.handlebars` and cover shells, text,
  tables, lists, cards, charts, forms, navigation, modals, downloads, maps, and more. Query columns map to
  component properties; nested/dynamic components and `sqlpage.run_sql` support composition and lazy loading.
  Custom Handlebars components can be placed in the configured `sqlpage/templates` directory. Raw HTML and
  custom assets are possible through the HTML/shell components. Rendering is streamed so the response can
  start while later query results are still being processed.
- **Control flow and errors** (`src/webserver/error.rs`, `error_with_status.rs`, `routing.rs`,
  `src/default_404.sql`). SQL remains declarative: use predicates, `CASE`, `SET`, component rows, and
  the `redirect` component to conditionally continue, redirect, or implement guards/error pages. There is no
  general SQLPage `IF` statement. Parse, database, function, component, and response errors are converted to
  contextual HTTP errors; `default_404.sql` handles missing routes. Do not hide errors by changing unrelated
  error handling or tests.
- **HTTP server and client** (`src/webserver/http.rs`, `http_client.rs`, `response_writer.rs`, `static_content.rs`,
  `https.rs`, `content_security_policy.rs`, `server_timing.rs`). The server is built on Actix Web, supports normal HTTP request/response
  handling, streaming, uploads, static assets, HTTP/2, HTTPS, and optional Unix sockets/serverless adapters.
  The shared outbound client is used by HTTP-fetch and OIDC integrations and honors configured/native TLS
  certificates and timeouts. Content-security-policy and response/header helpers are part of the request
  pipeline.
- **OIDC** (`src/webserver/oidc.rs`, `src/webserver/database/sqlpage_functions/functions/user_info.rs`).
  Optional OpenID Connect middleware protects configured path prefixes, performs provider discovery,
  login/callback/logout, validates tokens, maintains an authenticated cookie, and exposes identity claims to
  SQLPage functions. Configuration is in `AppConfig`/`configuration.md`; public paths can be excluded.
- **Caching and files** (`src/filesystem.rs`, `src/file_cache.rs`, `src/telemetry*.rs`). Parsed SQL files are cached. Files may come from the web root/filesystem or from the
  database-backed `sqlpage_files` store, and templates/migrations are loaded from the configuration directory.
  Telemetry, request timing, and debug logging help diagnose query, pool, and rendering performance.

- **Examples, tests, and project operations** (`examples/`, `tests/`, `configuration.md`, `CONTRIBUTING.md`,
  `README.md`, `.github/workflows/ci.yml`). Examples include the official documentation site and its migrations;
  tests cover SQL fixtures, database variants, uploads, OIDC, and server timing. The contribution guide and CI
  workflow define the development and validation conventions.
- **Deployment and local infrastructure** (`Dockerfile`, `lambda.Dockerfile`, `docker-compose.yml`,
  `sqlpage.service`). These provide container, serverless, local database-testing, and service deployment support.

## Documentation and release notes

The official documentation site is itself an SQLPage application in `examples/official-site/`. Its database
schema and documentation content are created by the SQL migrations in
`examples/official-site/sqlpage/migrations/`; the site is recreated from scratch during deployment. Existing
official-site migrations are editable source files: update the migration that already documents a component,
function, configuration option, or feature in place. Do not create a new migration merely to update existing
documentation. Add a new migration only for genuinely new documentation content when that is the established
pattern for the relevant area.

- Add or update a component's row, parameters, and examples in the component documentation migrations when
  changing `sqlpage/templates/` or component behavior.
- Add or update a function's description, parameters, examples, and caveats in the function documentation
  migrations when changing `sqlpage_functions` or any `sqlpage.*` function behavior.
- Update the relevant configuration documentation when adding or changing `AppConfig` settings, environment
  variables, defaults, authentication behavior, HTTP/TLS behavior, database settings, or custom components.
- Document OIDC changes in the authentication/OIDC migrations, including configuration requirements, exposed
  claims/functions, login/logout behavior, and security implications.
- Document other user-visible behavior—SQL syntax extensions, variables, control flow, errors, uploads,
  rendering, HTTP endpoints, performance, or deployment—in the corresponding official-site SQL page or
  migration. Follow nearby migrations and keep examples executable and database-portable where possible.
- Update `CHANGELOG.md` for user-visible changes, bug fixes, breaking changes, deprecations, and noteworthy
  internal changes. Keep the entry concise and use the existing version/section conventions.

## Validation

### When working on rust code
Mandatory formatting (rust): `cargo fmt --all`
Mandatory linting: `cargo clippy --all-targets --all-features -- -D warnings`

### When working on css or js
Frontend formatting: `npm run format`
Mandatory frontend validation: `npm test`

More about testing: see [github actions](./.github/workflows/ci.yml).
Contributor setup and validation: see [CONTRIBUTING.md](./CONTRIBUTING.md). Module overview: see [src/lib.rs](./src/lib.rs); architecture diagram: [docs/architecture-detailed.png](./docs/architecture-detailed.png).

NEVER reformat/lint/touch files unrelated to your task. Always run tests/lints/format before stopping when you changed code.

### Testing

```
cargo test # tests with in-memory SQLite by default
```

For other databases, see [docker testing setup](./docker-compose.yml)

```
docker compose up --wait mssql # or postgres, mysql, mariadb, oracle
DATABASE_URL='mssql://root:Password123!@localhost/sqlpage' cargo test
```

ODBC tests require the database-specific ODBC driver on the host; starting the container is not sufficient. See the PostgreSQL ODBC and Oracle matrix entries in [CI](./.github/workflows/ci.yml) for driver setup and connection strings. On Linux and macOS, `cargo test --features odbc-static` matches CI's static unixODBC linking.

For dynamic frontend changes, run the Playwright tests under `tests/end-to-end/` as described in [CONTRIBUTING.md](./CONTRIBUTING.md). For examples containing `test.hurl`, run `scripts/test-examples-hurl.sh <example-path>`.

### Documentation

Components and functions are documented in [official-site migrations](./examples/official-site/sqlpage/migrations/). Edit the existing migration for an existing entity; add an appropriately ordered migration for a new entity. The official-site database is recreated from migrations on each deployment.

official documentation website sql tables:
  - `parameter_type(type)`
  - `component(name,description,icon,introduced_in_version)` -- icon name from tabler icon
  - `parameter(top_level BOOLEAN, name, component REFERENCES component(name), description, description_md, type, optional BOOLEAN)` parameter types: BOOLEAN, COLOR, HTML, ICON, INTEGER, JSON, REAL, TEXT, TIMESTAMP, URL
  - `example(component REFERENCES component(name), description, properties JSON)`
  - `sqlpage_functions(name,icon,description_md,return_type,introduced_in_version)`
  - `sqlpage_function_parameters(function,index,name,description_md,type)`

#### Project Conventions

- Built-in UI component templates: `sqlpage/templates/*.handlebars`; header/control components: `src/render.rs`.
- SQLPage functions: one `async fn` module under `src/webserver/database/sqlpage_functions/functions/`, declared with `mod` and registered with `sqlpage_functions!` in `functions.rs`. See its [README](./src/webserver/database/sqlpage_functions/README.md).
- [Configuration](./configuration.md): see [AppConfig](./src/app_config.rs)
- Routing: file-based in `src/webserver/routing.rs`. Missing paths use the nearest ancestor `404.sql`; without one, HTML uses `src/default_404.sql` and other formats receive a plain-text 404.
- Follow patterns from similar modules before introducing new abstractions.
- frontend: see [css](./sqlpage/sqlpage.css) and [js](./sqlpage/sqlpage.js)
