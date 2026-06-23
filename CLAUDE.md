# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SQLPage is a Rust web server that turns `.sql` files into dynamic web applications. Users write SQL queries; SQLPage executes them, maps results to handlebars UI components, and streams HTML to the client.

## Build & Test Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Tests with in-memory SQLite (default)
cargo fmt --all                # Format Rust code
cargo clippy --all-targets --all-features -- -D warnings  # Lint Rust code
npm run format                 # Format frontend (CSS/JS) with Biome
npx @biomejs/biome check .     # Check frontend formatting
```

### Testing against other databases

```bash
docker compose up -d mssql     # or postgres, mysql, mariadb
DATABASE_URL='mssql://root:Password123!@localhost/sqlpage' cargo test
```

All database backends use the same credentials: `root:Password123!@localhost/sqlpage`.

### End-to-end tests (Playwright)

```bash
# Terminal 1: start SQLPage with official site
cd examples/official-site && cargo run

# Terminal 2: run E2E tests
cd tests/end-to-end && npm install && npx playwright install chromium && npm run test
```

## Architecture

### Request flow

HTTP request → actix-web routing → `.sql` file lookup → SQL parsing (cached) → query execution → results streamed through handlebars templates → HTML streamed to client.

### Key source modules

- `src/main.rs` — entry point, logging init
- `src/lib.rs` — `AppState` definition, module declarations
- `src/app_config.rs` — configuration loading (env vars, config file)
- `src/render.rs` — component rendering pipeline
- `src/webserver/http.rs` — HTTP server setup
- `src/webserver/routing.rs` — file-based routing
- `src/webserver/database/sql.rs` — SQL parsing and execution
- `src/webserver/database/execute_queries.rs` — query result streaming
- `src/webserver/database/sqlpage_functions/` — built-in `sqlpage.*` SQL functions
- `src/webserver/oidc.rs` — OpenID Connect authentication

### Component system

- Templates live in `sqlpage/templates/*.handlebars`
- Each SQL result set's first row selects a component and sets top-level properties
- Subsequent rows provide row-level data
- Template helpers in `src/template_helpers.rs`

### Built-in functions

Registered with the `make_function!` macro in `src/webserver/database/sqlpage_functions/functions.rs`.

### Documentation

Components and functions are documented as SQL migrations in `examples/official-site/sqlpage/migrations/`. These migrations are re-run from scratch on each deployment of the official site, so existing migrations can be edited directly.

Key documentation tables:
- `component(name, description, icon, introduced_in_version)`
- `parameter(top_level, name, component, description, description_md, type, optional)` — types: BOOLEAN, COLOR, HTML, ICON, INTEGER, JSON, REAL, TEXT, TIMESTAMP, URL
- `example(component, description, properties)` — properties is JSON
- `sqlpage_functions(name, introduced_in_version, icon, description_md)`
- `sqlpage_function_parameters(function, index, name, description_md, type)`

### Configuration

All options documented in `configuration.md`. Environment variables prefixed with `SQLPAGE_`. Core config struct in `src/app_config.rs`.

### Frontend assets

- `sqlpage/sqlpage.js` and `sqlpage/sqlpage.css`
- External deps (ApexCharts, Tom Select, Tabler) downloaded and embedded at build time by `build.rs`

## Development Rules

- **Never reformat or touch files unrelated to your task.**
- **Always run tests, lints, and format before finishing when you changed code.**
- Follow patterns from similar existing modules before introducing new abstractions.
- Supported databases: SQLite, PostgreSQL, MySQL/MariaDB, MSSQL, ODBC (Oracle, DuckDB, Snowflake, etc.).

---

## Components Reference

All components are invoked via SELECT statements:
```sql
SELECT 'component_name' AS component, 'value' AS top_level_param;
SELECT column AS row_level_param FROM table;
```

Parameter types: BOOLEAN, COLOR, HTML, ICON, INTEGER, JSON, REAL, TEXT, TIMESTAMP, URL.

Header components (must be first, before visual output): authentication, redirect, http_header, cookie, json, download, status_code, log, shell.

### shell
Layout wrapper. Generates full HTML document with nav bar, content, footer.
- **Top-level:** title(TEXT), layout(TEXT), description(TEXT), link(URL), css(URL), javascript(URL), javascript_module(URL), rss(URL), image(URL), social_image(URL), icon(ICON), menu_item(TEXT), fixed_top_menu(BOOL), search_target(TEXT), search_value(TEXT), search_placeholder(TEXT), search_button(TEXT), norobot(BOOL), font(TEXT), font_size(INT), language(TEXT), rtl(BOOL), refresh(INT), sidebar(BOOL), sidebar_theme(BOOL), theme(TEXT), footer(TEXT), preview_image(URL), navbar_title(TEXT), target(TEXT), favicon(URL), manifest(URL)
- Use `'shell-empty'` to strip HTML wrapper for raw content (XML, plain text).

### hero
Large title/description with optional image. Good for home pages.
- **Top-level:** title(TEXT), description(TEXT), description_md(TEXT), image(URL), video(URL), link(URL), link_text(TEXT), poster(URL), nocontrols(BOOL), muted(BOOL), autoplay(BOOL), loop(BOOL), reverse(BOOL), id, class
- **Row-level:** title(TEXT), description(TEXT), description_md(TEXT), icon(ICON), link(TEXT)

### text
Paragraph of text. Each row is a span inside the paragraph.
- **Top-level:** title(TEXT), center(BOOL), width(INT), html(TEXT), contents(TEXT), contents_md(TEXT), article(BOOL), unsafe_contents_md(TEXT), id, class
- **Row-level:** contents(TEXT), contents_md(TEXT), unsafe_contents_md(TEXT), link(URL), color(COLOR), underline(BOOL), bold(BOOL), code(BOOL), italics(BOOL), break(BOOL), size(INT)

### title
HTML headings h1-h6.
- **Top-level:** contents(TEXT, required), center(BOOL), level(INT), id, class

### card
Grid of small cards.
- **Top-level:** title(TEXT), description(TEXT), description_md(TEXT), columns(INT)
- **Row-level:** title(TEXT, required), description(TEXT), description_md(TEXT), top_image(URL), footer(TEXT), footer_md(TEXT), link(URL), footer_link(URL), style(TEXT), icon(ICON), color(COLOR), background_color(COLOR), active(BOOL), width(INT), embed(TEXT), embed_mode(TEXT), class

### list
Vertical list of items.
- **Top-level:** title(TEXT), empty_title(TEXT), empty_description(TEXT), empty_description_md(TEXT), empty_link(URL), compact(BOOL), wrap(BOOL), id, class
- **Row-level:** title(TEXT, required), description(TEXT), description_md(TEXT), link(URL), icon(ICON), image_url(URL), color(COLOR), active(BOOL), view_link(URL), edit_link(URL), delete_link(URL), id, class

### table
Table with optional filtering/sorting. Any column returned becomes a table column.
- **Top-level:** sort(BOOL), search(BOOL), initial_search_value(TEXT), search_placeholder(TEXT), markdown(TEXT), icon(TEXT), align_right(TEXT), align_center(TEXT), monospace(TEXT), striped_rows(BOOL), striped_columns(BOOL), hover(BOOL), border(BOOL), overflow(BOOL), small(BOOL), description(TEXT), empty_description(TEXT), freeze_columns(BOOL), freeze_headers(BOOL), freeze_footers(BOOL), raw_numbers(TEXT), money(TEXT), currency(TEXT), number_format_digits(INT), edit_url(TEXT), delete_url(TEXT), custom_actions(JSON), id, class
- **Row-level (special):** _sqlpage_css_class(TEXT), _sqlpage_color(COLOR), _sqlpage_footer(BOOL), _sqlpage_id(TEXT), _sqlpage_actions(JSON)

### form
Input fields for data submission.
- **Top-level:** method(TEXT), action(TEXT), title(TEXT), validate(TEXT), validate_color(COLOR), validate_outline(COLOR), reset(TEXT), id, auto_submit(BOOL), validate_icon(ICON), reset_icon(ICON), reset_color(COLOR), enctype(TEXT), class
- **Row-level:** type(TEXT), name(TEXT, required), label(TEXT), placeholder(TEXT), value(TEXT), options(JSON), required(BOOL), min(REAL), max(REAL), checked(BOOL), multiple(BOOL), empty_option(TEXT), searchable(BOOL), dropdown(BOOL), create_new(BOOL), step(REAL), description(TEXT), description_md(TEXT), pattern(TEXT), autofocus(BOOL), width(INT), autocomplete(BOOL), minlength(INT), maxlength(INT), formaction(TEXT), formenctype(TEXT), class, prefix_icon(ICON), prefix(TEXT), suffix(TEXT), readonly(BOOL), rows(INT), disabled(BOOL), id

### login
Authentication form (POST).
- **Top-level:** title(TEXT), enctype(TEXT), action(TEXT), error_message(TEXT), error_message_md(TEXT), username(TEXT, required), password(TEXT, required), username_icon(ICON), password_icon(ICON), image(URL), forgot_password_text(TEXT), forgot_password_link(TEXT), remember_me_text(TEXT), footer(TEXT), footer_md(TEXT), validate(TEXT), validate_color(COLOR), validate_shape(TEXT), validate_outline(COLOR), validate_size(TEXT), id, class

### chart
Line, area, bar, pie charts using ApexCharts.
- **Top-level:** title(TEXT), type(TEXT, required), time(BOOL), ymin(REAL), ymax(REAL), xtitle(TEXT), ytitle(TEXT), ztitle(TEXT), xticks(INT), ystep(REAL), marker(REAL), labels(BOOL), color(COLOR), stacked(BOOL), toolbar(BOOL), logarithmic(BOOL), horizontal(BOOL), height(INT), id, class
- **Row-level:** x(REAL, required), y(REAL, required), label(REAL), value(REAL), series(TEXT)

### alert
Notification/message box.
- **Top-level:** title(TEXT, required), icon(TEXT), color(COLOR), description(TEXT), description_md(TEXT), dismissible(BOOL), important(BOOL), link(URL), link_text(TEXT), id, class
- **Row-level:** link(URL), title(TEXT), color(COLOR)

### button
One or multiple button links.
- **Top-level:** justify(TEXT), size(TEXT), shape(TEXT), class
- **Row-level:** link(URL), color(COLOR), title(TEXT), tooltip(TEXT), disabled(BOOL), outline(COLOR), space_after(BOOL), icon_after(ICON), icon(ICON), image(TEXT), narrow(BOOL), form(TEXT), rel(TEXT), target(TEXT), download(TEXT), id, modal(TEXT)

### datagrid
Small pieces of info (key-value pairs).
- **Top-level:** title(TEXT), description(TEXT), description_md(TEXT), icon(ICON), image_url(URL), id, class
- **Row-level:** title(TEXT, required), description(TEXT), footer(TEXT), image_url(URL), link(URL), icon(ICON), color(COLOR), active(BOOL), tooltip(TEXT)

### big_number
Dashboard metrics with progress bars.
- **Top-level:** columns(INT), id, class
- **Row-level:** title(TEXT), title_link(TEXT), title_link_new_tab(BOOL), value_link(TEXT), value_link_new_tab(BOOL), value(TEXT, required), unit(TEXT), description(TEXT), change_percent(INT), progress_percent(INT), progress_color(TEXT), dropdown_item(JSON), color(COLOR)

### steps
Multi-stage process indicator.
- **Top-level:** color(COLOR), counter(TEXT), title(TEXT), description(TEXT), id
- **Row-level:** title(TEXT), description(TEXT), link(URL), icon(ICON), active(BOOL)

### tab
Tabbed interface.
- **Top-level:** center(BOOL)
- **Row-level:** title(TEXT, required), link(TEXT), active(BOOL), icon(TEXT), color(TEXT), description(TEXT), id, class

### timeline
Vertical event list.
- **Top-level:** simple(BOOL), id, class
- **Row-level:** title(TEXT, required), date(TEXT, required), icon(TEXT), color(TEXT), description(TEXT), description_md(TEXT), link(TEXT), id, class

### breadcrumb
Secondary navigation path.
- **Top-level:** id, class
- **Row-level:** title(TEXT, required), link(TEXT), active(TEXT), description(TEXT)

### divider
Content separator.
- **Top-level:** contents(TEXT), position(TEXT), color(COLOR), size(INT), bold(BOOL), italics(BOOL), underline(BOOL), link(URL), class

### map
Interactive map with markers.
- **Top-level:** latitude(REAL), longitude(REAL), zoom(REAL), max_zoom(INT), tile_source(URL), attribution(HTML), height(INT), id, class
- **Row-level:** latitude(REAL, required), longitude(REAL, required), title(TEXT), link(TEXT), description(TEXT), description_md(TEXT), icon(ICON), color(COLOR), geojson(JSON), size(INT)

### carousel
Image slideshow.
- **Top-level:** title(TEXT), indicators(TEXT), vertical(BOOL), controls(BOOL), width(INT), auto(BOOL), center(BOOL), fade(BOOL), delay(INT), id, class
- **Row-level:** image(URL, required), title(TEXT), description(TEXT), description_md(TEXT), width(INT), height(INT)

### columns
Card layout for features/pricing.
- **Row-level:** title(TEXT), value(TEXT), description(TEXT), description_md(TEXT), item(JSON), link(TEXT), button_text(TEXT), button_color(TEXT), target(TEXT), value_color(TEXT), small_text(TEXT), icon(ICON), icon_color(TEXT), size(INT)

### foldable
Expandable accordion list.
- **Top-level:** id, class
- **Row-level:** id, class, title(TEXT), description(TEXT), description_md(TEXT), expanded(BOOL)

### modal
Popup dialog. Opened via button with `modal` param.
- **Top-level:** title(TEXT, required), id(TEXT, required), close(TEXT), scrollable(BOOL), class, large(BOOL), small(BOOL), embed(TEXT), embed_mode(TEXT), height(INT), allow(TEXT), sandbox(TEXT), style(TEXT)
- **Row-level:** contents(TEXT), contents_md(TEXT)

### empty_state
Large placeholder message.
- **Top-level:** title(TEXT, required), header(TEXT), icon(ICON), image(URL), description(TEXT), link_text(TEXT, required), link_icon(ICON, required), link(URL, required), class, id

### tracking
Activity/monitoring visualization.
- **Top-level:** title(TEXT, required), information(TEXT), description(TEXT), description_md(TEXT), width(INT), placement(TEXT), center(BOOL), id, class
- **Row-level:** color(TEXT), title(TEXT, required)

### pagination
Page navigation links.
- **Top-level:** first_link(URL), previous_link(URL), next_link(URL), last_link(URL), first_title(TEXT), previous_title(TEXT), next_title(TEXT), last_title(TEXT), first_disabled(BOOL), previous_disabled(BOOL), next_disabled(BOOL), last_disabled(BOOL), outline(BOOL), circle(BOOL), id, class
- **Row-level:** contents(INT, required), link(URL), offset(BOOL), active(BOOL)

### code
Code blocks with syntax highlighting.
- **Top-level:** id, class
- **Row-level:** title(TEXT), contents(TEXT, required), description(TEXT), description_md(TEXT), language(TEXT)

### csv
Download data as CSV.
- **Top-level:** separator(TEXT), title(TEXT, required), filename(TEXT), icon(ICON), color(COLOR), size(TEXT), bom(BOOL), id, class
- **Row-level:** Any columns become CSV columns.

### rss
RSS/podcast feed.
- **Top-level:** title(TEXT, required), link(URL, required), description(TEXT, required), language(TEXT), category(TEXT), explicit(BOOL), image_url(URL), author(TEXT), copyright(TEXT), self_link(URL), funding_url(URL), type(TEXT), complete(BOOL), locked(BOOL), guid(TEXT)
- **Row-level:** title(TEXT, required), link(URL, required), description(TEXT, required), date(TEXT), enclosure_url(URL), enclosure_length(INT), enclosure_type(TEXT), guid(TEXT), episode(INT), season(INT), episode_type(TEXT), block(BOOL), explicit(BOOL), image_url(URL), duration(INT), transcript_url(URL), transcript_type(TEXT)

### json
SQL results to JSON. Must be at top of file.
- **Top-level:** contents(TEXT), type(TEXT: "array"|"jsonlines"|"sse")

### dynamic
Render components from JSON properties.
- **Top-level:** properties(JSON)

### debug
Display values as JSON. No fixed params — shows everything passed.

### authentication
Password-restricted access.
- **Top-level:** link(TEXT), password(TEXT), password_hash(TEXT)

### redirect
Redirect and stop execution. Must be at top of file.
- **Top-level:** link(TEXT, required)

### http_header
Set arbitrary HTTP headers. Must be first component.
- **Top-level:** Any valid HTTP header name as parameter.

### cookie
Set browser cookie.
- **Top-level:** name(TEXT, required), value(TEXT), path(TEXT), domain(TEXT), secure(BOOL), http_only(BOOL), remove(BOOL), max_age(INT), expires(TIMESTAMP), same_site(TEXT)

### status_code
Set HTTP response code.
- **Top-level:** status(INT, required)

### html
Raw HTML output. XSS risk with user content.
- **Top-level:** html(TEXT)
- **Row-level:** html(TEXT), text(TEXT), post_html(TEXT)

### download
Send file as response. Must be at top of page.
- **Top-level:** data_url(TEXT, required), filename(TEXT)

### log
Write to server logs.
- **Top-level:** message(TEXT, required), level(TEXT: trace/debug/info/warn/error)

---

## Built-in Functions Reference

All functions are called as `sqlpage.function_name(args)` in SQL queries.

### Authentication & Security

**sqlpage.basic_auth_username()** — Returns username from HTTP Basic Auth header. Triggers 401 if absent.

**sqlpage.basic_auth_password()** — Returns password from HTTP Basic Auth header. Triggers 401 if absent.

**sqlpage.hash_password(password TEXT) → TEXT** — Hashes password with Argon2id in PHC format. Use only for creating/resetting passwords.
```sql
INSERT INTO users (name, password_hash) VALUES (:username, sqlpage.hash_password(:password));
```

**sqlpage.random_string(length INTEGER) → TEXT** — Cryptographically secure random alphanumeric string.

**sqlpage.hmac(data TEXT, key TEXT, algorithm? TEXT) → TEXT** — HMAC signature. Algorithms: sha256, sha256-base64, sha512, sha512-base64. Default: sha256.

### OIDC / SSO

**sqlpage.user_info(claim TEXT) → TEXT** — Returns specific OIDC ID token claim. Common claims: name, email, picture, sub, preferred_username, given_name, family_name.

**sqlpage.user_info_token() → JSON** — Returns entire OIDC ID token as JSON.

**sqlpage.oidc_logout_url(redirect_uri? TEXT) → TEXT** — Generates CSRF-protected OIDC logout URL. Token expires after 10min. Returns NULL if OIDC not configured.

### Request Information

**sqlpage.cookie(name TEXT) → TEXT** — Read cookie value from request. Returns NULL if absent.

**sqlpage.header(name TEXT) → TEXT** — Read HTTP request header (case-insensitive). Returns NULL if absent.

**sqlpage.headers() → JSON** — All request headers as JSON object.

**sqlpage.path() → TEXT** — URL-encoded request path of current page.

**sqlpage.protocol() → TEXT** — Returns "http" or "https". Checks Forwarded, X-Forwarded-Proto headers.

**sqlpage.client_ip() → TEXT** — Client IP address. Returns null through Unix socket. Behind proxy, use `header('x-forwarded-for')`.

**sqlpage.request_method() → TEXT** — HTTP method: GET, POST, PUT, DELETE, PATCH, etc.

**sqlpage.request_body() → TEXT** — Raw request body as text. NULL for form-encoded/multipart (use `variables('post')`).

**sqlpage.request_body_base64() → TEXT** — Raw request body as base64. For binary data.

**sqlpage.variables(method? TEXT) → JSON** — Request variables as JSON. Filter: `'get'` (URL params), `'post'` (form data), `'set'` (user-defined). Without arg: all variables (SET > POST > GET).
```sql
-- SQLite
SELECT key, value FROM json_each(sqlpage.variables('post'));
-- PostgreSQL
SELECT key, value FROM json_each_text(sqlpage.variables('post')::json);
```

### File Upload

**sqlpage.uploaded_file_path(name TEXT, allowed_mime_type? TEXT) → TEXT** — Path to temp file with uploaded content. Returns NULL if field empty or MIME mismatch.

**sqlpage.uploaded_file_name(name TEXT) → TEXT** — Original filename from upload.

**sqlpage.uploaded_file_mime_type(name TEXT) → TEXT** — MIME type of uploaded file.

**sqlpage.persist_uploaded_file(field TEXT, folder? TEXT, allowed_extensions? TEXT) → TEXT** — Save uploaded file permanently. Returns path. Default folder: `uploads`. Default extensions: jpg,jpeg,png,gif,bmp,webp,pdf,txt,doc,docx,xls,xlsx,csv,mp3,mp4,wav,avi,mov.

### File I/O

**sqlpage.read_file_as_text(path TEXT) → TEXT** — File contents as UTF-8 text. Path relative to web root.

**sqlpage.read_file_as_data_url(path TEXT) → TEXT** — File contents as data URL. Path relative to web root.

### HTTP Client

**sqlpage.fetch(url TEXT|JSON) → TEXT** — HTTP request returning body as text. Simple GET: pass URL string. Complex: pass JSON with `url`, `method`, `headers`, `body`, `timeout_ms`, `username`, `password`, `response_encoding`.
```sql
SET result = sqlpage.fetch('https://api.example.com/data');
-- Or complex:
SET result = sqlpage.fetch(json_object(
    'method', 'POST',
    'url', 'https://api.example.com/data',
    'headers', json_object('Content-Type', 'application/json'),
    'body', json_object('key', 'value')
));
```

**sqlpage.fetch_with_meta(url TEXT|JSON) → JSON** — Like fetch but returns JSON with `status`, `headers`, `body`, `error`. Does not throw on errors.

### URL Helpers

**sqlpage.url_encode(string TEXT) → TEXT** — Percent-encode string for URL safety.

**sqlpage.link(file TEXT, parameters? JSON, fragment? TEXT) → URL** — Generate properly encoded URL to a SQLPage file with parameters.
```sql
sqlpage.link('product.sql', json_object('id', 42, 'name', 'Widget'))
-- → product.sql?id=42&name=Widget
```

**sqlpage.set_variable(name TEXT, value? TEXT) → URL** — Returns current page URL with one variable changed. NULL value removes the variable.

### Execution

**sqlpage.run_sql(file TEXT, parameters? JSON) → JSON** — Execute another SQL file, return results as JSON array. Max recursion depth: 10.
```sql
SELECT 'dynamic' AS component, sqlpage.run_sql('header.sql') AS properties;
```

**sqlpage.exec(program TEXT, args... TEXT) → TEXT** — Execute shell command. Requires `"allow_exec": true` in config.

### Server Info

**sqlpage.version() → TEXT** — Current SQLPage version string.

**sqlpage.current_working_directory() → TEXT** — Server's working directory.

**sqlpage.web_root() → TEXT** — Directory where .sql files are served from.

**sqlpage.configuration_directory() → TEXT** — Directory for sqlpage.json, templates, migrations.

**sqlpage.environment_variable(name TEXT) → TEXT** — Read environment variable. Name must be literal string.
