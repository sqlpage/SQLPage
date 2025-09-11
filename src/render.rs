//! Handles the rendering of SQL query results into HTTP responses using components.
//!
//! This module is responsible for transforming database query results into formatted HTTP responses
//! by utilizing a component-based rendering system. It supports multiple output formats including HTML,
//! JSON, and CSV.
//!
//! # Components
//!
//! Components are small user interface elements that display data in specific ways. The rendering
//! system supports two types of parameters for components:
//!
//! * **Top-level parameters**: Properties that customize the component's appearance and behavior
//! * **Row-level parameters**: The actual data to be displayed within the component
//!
//! # Page Context States
//!
//! The rendering process moves through different states represented by [`PageContext`]:
//!
//! * `Header`: Initial state for processing HTTP headers and response setup
//! * `Body`: Active rendering state where component output is generated
//! * `Close`: Final state indicating the response is complete
//!
//! # Header Components
//!
//! Some components must be processed before any response body is sent:
//!
//! * [`status_code`](https://sql-page.com/component.sql?component=status_code): Sets the HTTP response status
//! * [`http_header`](https://sql-page.com/component.sql?component=http_header): Sets custom HTTP headers
//! * [`redirect`](https://sql-page.com/component.sql?component=redirect): Performs HTTP redirects
//! * `authentication`: Handles password-protected access
//! * `cookie`: Manages browser cookies
//!
//! # Body Components
//!
//! The module supports multiple output formats through different renderers:
//!
//! * HTML: Renders templated HTML output using components
//! * JSON: Generates JSON responses for API endpoints
//! * CSV: Creates downloadable CSV files
//!
//! For more details on available components and their usage, see the
//! [SQLPage documentation](https://sql-page.com/documentation.sql).

use crate::templates::SplitTemplate;
use crate::webserver::http::RequestContext;
use crate::webserver::response_writer::{AsyncResponseWriter, ResponseWriter};
use crate::webserver::ErrorWithStatus;
use crate::AppState;
use actix_web::cookie::time::format_description::well_known::Rfc3339;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::http::{header, StatusCode};
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use anyhow::{bail, format_err, Context as AnyhowContext};
use awc::cookie::time::Duration;
use handlebars::{BlockContext, JsonValue, RenderError, Renderable};
use serde::Serialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

pub enum PageContext {
    /// Indicates that we should stay in the header context
    Header(HeaderContext),

    /// Indicates that we should start rendering the body
    Body {
        http_response: HttpResponseBuilder,
        renderer: AnyRenderBodyContext,
    },

    /// The response is ready, and should be sent as is. No further statements should be executed
    Close(HttpResponse),
}

/// Handles the first SQL statements, before the headers have been sent to
pub struct HeaderContext {
    app_state: Arc<AppState>,
    request_context: RequestContext,
    pub writer: ResponseWriter,
    response: HttpResponseBuilder,
    has_status: bool,
}

impl HeaderContext {
    #[must_use]
    pub fn new(
        app_state: Arc<AppState>,
        request_context: RequestContext,
        writer: ResponseWriter,
    ) -> Self {
        let mut response = HttpResponseBuilder::new(StatusCode::OK);
        response.content_type("text/html; charset=utf-8");
        let tpl = &app_state.config.content_security_policy;
        request_context
            .content_security_policy
            .apply_to_response(tpl, &mut response);
        Self {
            app_state,
            request_context,
            writer,
            response,
            has_status: false,
        }
    }
    pub async fn handle_row(self, data: JsonValue) -> anyhow::Result<PageContext> {
        log::debug!("Handling header row: {data}");
        let comp_opt =
            get_object_str(&data, "component").and_then(|s| HeaderComponent::try_from(s).ok());
        match comp_opt {
            Some(HeaderComponent::StatusCode) => self.status_code(&data).map(PageContext::Header),
            Some(HeaderComponent::HttpHeader) => {
                self.add_http_header(&data).map(PageContext::Header)
            }
            Some(HeaderComponent::Redirect) => self.redirect(&data).map(PageContext::Close),
            Some(HeaderComponent::Json) => self.json(&data),
            Some(HeaderComponent::Csv) => self.csv(&data).await,
            Some(HeaderComponent::Cookie) => self.add_cookie(&data).map(PageContext::Header),
            Some(HeaderComponent::Authentication) => self.authentication(data).await,
            Some(HeaderComponent::Download) => self.download(&data),
            Some(HeaderComponent::Log) => self.log(&data),
            None => self.start_body(data).await,
        }
    }

    pub async fn handle_error(self, err: anyhow::Error) -> anyhow::Result<PageContext> {
        if self.app_state.config.environment.is_prod() {
            return Err(err);
        }
        log::debug!("Handling header error: {err}");
        let data = json!({
            "component": "error",
            "description": err.to_string(),
            "backtrace": get_backtrace_as_strings(&err),
        });
        self.start_body(data).await
    }

    fn status_code(mut self, data: &JsonValue) -> anyhow::Result<Self> {
        let status_code = data
            .as_object()
            .and_then(|m| m.get("status"))
            .with_context(|| "status_code component requires a status")?
            .as_u64()
            .with_context(|| "status must be a number")?;
        let code = u16::try_from(status_code)
            .with_context(|| format!("status must be a number between 0 and {}", u16::MAX))?;
        self.response.status(StatusCode::from_u16(code)?);
        self.has_status = true;
        Ok(self)
    }

    fn add_http_header(mut self, data: &JsonValue) -> anyhow::Result<Self> {
        let obj = data.as_object().with_context(|| "expected object")?;
        for (name, value) in obj {
            if name == "component" {
                continue;
            }
            let value_str = value
                .as_str()
                .with_context(|| "http header values must be strings")?;
            if name.eq_ignore_ascii_case("location") && !self.has_status {
                self.response.status(StatusCode::FOUND);
                self.has_status = true;
            }
            self.response.insert_header((name.as_str(), value_str));
        }
        Ok(self)
    }

    fn add_cookie(mut self, data: &JsonValue) -> anyhow::Result<Self> {
        let obj = data.as_object().with_context(|| "expected object")?;
        let name = obj
            .get("name")
            .and_then(JsonValue::as_str)
            .with_context(|| "cookie name must be a string")?;
        let mut cookie = actix_web::cookie::Cookie::named(name);

        let path = obj.get("path").and_then(JsonValue::as_str);
        if let Some(path) = path {
            cookie.set_path(path);
        } else {
            cookie.set_path("/");
        }
        let domain = obj.get("domain").and_then(JsonValue::as_str);
        if let Some(domain) = domain {
            cookie.set_domain(domain);
        }

        let remove = obj.get("remove");
        if remove == Some(&json!(true)) || remove == Some(&json!(1)) {
            cookie.make_removal();
            self.response.cookie(cookie);
            log::trace!("Removing cookie {name}");
            return Ok(self);
        }

        let value = obj
            .get("value")
            .and_then(JsonValue::as_str)
            .with_context(|| "The 'value' property of the cookie component is required (unless 'remove' is set) and must be a string.")?;
        cookie.set_value(value);
        let http_only = obj.get("http_only");
        cookie.set_http_only(http_only != Some(&json!(false)) && http_only != Some(&json!(0)));
        let same_site = obj.get("same_site").and_then(Value::as_str);
        cookie.set_same_site(match same_site {
            Some("none") => actix_web::cookie::SameSite::None,
            Some("lax") => actix_web::cookie::SameSite::Lax,
            None | Some("strict") => actix_web::cookie::SameSite::Strict, // strict by default
            Some(other) => bail!("Cookie: invalid value for same_site: {}", other),
        });
        let secure = obj.get("secure");
        cookie.set_secure(secure != Some(&json!(false)) && secure != Some(&json!(0)));
        if let Some(max_age_json) = obj.get("max_age") {
            let seconds = max_age_json
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("max_age must be a number, not {max_age_json}"))?;
            cookie.set_max_age(Duration::seconds(seconds));
        }
        let expires = obj.get("expires");
        if let Some(expires) = expires {
            cookie.set_expires(actix_web::cookie::Expiration::DateTime(match expires {
                JsonValue::String(s) => OffsetDateTime::parse(s, &Rfc3339)?,
                JsonValue::Number(n) => OffsetDateTime::from_unix_timestamp(
                    n.as_i64().with_context(|| "expires must be a timestamp")?,
                )?,
                _ => bail!("expires must be a string or a number"),
            }));
        }
        log::trace!("Setting cookie {cookie}");
        self.response
            .append_header((header::SET_COOKIE, cookie.encoded().to_string()));
        Ok(self)
    }

    fn redirect(mut self, data: &JsonValue) -> anyhow::Result<HttpResponse> {
        self.response.status(StatusCode::FOUND);
        self.has_status = true;
        let link = get_object_str(data, "link")
            .with_context(|| "The redirect component requires a 'link' property")?;
        self.response.insert_header((header::LOCATION, link));
        let response = self.response.body(());
        Ok(response)
    }

    /// Answers to the HTTP request with a single json object
    fn json(mut self, data: &JsonValue) -> anyhow::Result<PageContext> {
        self.response
            .insert_header((header::CONTENT_TYPE, "application/json"));
        if let Some(contents) = data.get("contents") {
            let json_response = if let Some(s) = contents.as_str() {
                s.as_bytes().to_owned()
            } else {
                serde_json::to_vec(contents)?
            };
            Ok(PageContext::Close(self.response.body(json_response)))
        } else {
            let body_type = get_object_str(data, "type");
            let json_renderer = match body_type {
                None | Some("array") => JsonBodyRenderer::new_array(self.writer),
                Some("jsonlines") => JsonBodyRenderer::new_jsonlines(self.writer),
                Some("sse") => {
                    self.response
                        .insert_header((header::CONTENT_TYPE, "text/event-stream"));
                    JsonBodyRenderer::new_server_sent_events(self.writer)
                }
                _ => bail!(
                    "Invalid value for the 'type' property of the json component: {body_type:?}"
                ),
            };
            let renderer = AnyRenderBodyContext::Json(json_renderer);
            let http_response = self.response;
            Ok(PageContext::Body {
                http_response,
                renderer,
            })
        }
    }

    async fn csv(mut self, options: &JsonValue) -> anyhow::Result<PageContext> {
        self.response
            .insert_header((header::CONTENT_TYPE, "text/csv; charset=utf-8"));
        if let Some(filename) =
            get_object_str(options, "filename").or_else(|| get_object_str(options, "title"))
        {
            let extension = if filename.contains('.') { "" } else { ".csv" };
            self.response.insert_header((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename={filename}{extension}"),
            ));
        }
        let csv_renderer = CsvBodyRenderer::new(self.writer, options).await?;
        let renderer = AnyRenderBodyContext::Csv(csv_renderer);
        let http_response = self.response.take();
        Ok(PageContext::Body {
            renderer,
            http_response,
        })
    }

    async fn authentication(mut self, mut data: JsonValue) -> anyhow::Result<PageContext> {
        let password_hash = take_object_str(&mut data, "password_hash");
        let password = take_object_str(&mut data, "password");
        if let (Some(password), Some(password_hash)) = (password, password_hash) {
            log::debug!("Authentication with password_hash = {password_hash:?}");
            match verify_password_async(password_hash, password).await? {
                Ok(()) => return Ok(PageContext::Header(self)),
                Err(e) => log::info!("Password didn't match: {e}"),
            }
        }
        log::debug!("Authentication failed");
        // The authentication failed
        let http_response: HttpResponse = if let Some(link) = get_object_str(&data, "link") {
            self.response
                .status(StatusCode::FOUND)
                .insert_header((header::LOCATION, link))
                .body(
                    "Sorry, but you are not authorized to access this page. \
                    Redirecting to the login page...",
                )
        } else {
            ErrorWithStatus {
                status: StatusCode::UNAUTHORIZED,
            }
            .error_response()
        };
        self.has_status = true;
        Ok(PageContext::Close(http_response))
    }

    fn download(mut self, options: &JsonValue) -> anyhow::Result<PageContext> {
        if let Some(filename) = get_object_str(options, "filename") {
            self.response.insert_header((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ));
        }
        let data_url = get_object_str(options, "data_url")
            .with_context(|| "The download component requires a 'data_url' property")?;
        let rest = data_url
            .strip_prefix("data:")
            .with_context(|| "Invalid data URL: missing 'data:' prefix")?;
        let (mut content_type, data) = rest
            .split_once(',')
            .with_context(|| "Invalid data URL: missing comma")?;
        let mut body_bytes: Cow<[u8]> = percent_encoding::percent_decode(data.as_bytes()).into();
        if let Some(stripped) = content_type.strip_suffix(";base64") {
            content_type = stripped;
            body_bytes =
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &body_bytes)
                    .with_context(|| "Invalid base64 data in data URL")?
                    .into();
        }
        if !content_type.is_empty() {
            self.response
                .insert_header((header::CONTENT_TYPE, content_type));
        }
        Ok(PageContext::Close(
            self.response.body(body_bytes.into_owned()),
        ))
    }

    fn log(self, data: &JsonValue) -> anyhow::Result<PageContext> {
        handle_log_component(&self.request_context.source_path, Option::None, data)?;
        Ok(PageContext::Header(self))
    }

    async fn start_body(self, data: JsonValue) -> anyhow::Result<PageContext> {
        let html_renderer =
            HtmlRenderContext::new(self.app_state, self.request_context, self.writer, data)
                .await
                .with_context(|| "Failed to create a render context from the header context.")?;
        let renderer = AnyRenderBodyContext::Html(html_renderer);
        let http_response = self.response;
        Ok(PageContext::Body {
            renderer,
            http_response,
        })
    }

    pub fn close(mut self) -> HttpResponse {
        self.response.finish()
    }
}

async fn verify_password_async(
    password_hash: String,
    password: String,
) -> Result<Result<(), password_hash::Error>, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        let hash = password_hash::PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("invalid value for the password_hash property: {}", e))?;
        let phfs = &[&argon2::Argon2::default() as &dyn password_hash::PasswordVerifier];
        Ok(hash.verify_password(phfs, password))
    })
    .await?
}

fn get_object_str<'a>(json: &'a JsonValue, key: &str) -> Option<&'a str> {
    json.as_object()
        .and_then(|obj| obj.get(key))
        .and_then(JsonValue::as_str)
}

fn take_object_str(json: &mut JsonValue, key: &str) -> Option<String> {
    match json.get_mut(key)?.take() {
        JsonValue::String(s) => Some(s),
        _ => None,
    }
}

/**
 * Can receive rows, and write them in a given format to an `io::Write`
 */
pub enum AnyRenderBodyContext {
    Html(HtmlRenderContext<ResponseWriter>),
    Json(JsonBodyRenderer<ResponseWriter>),
    Csv(CsvBodyRenderer),
}

/**
 * Dummy impl to dispatch method calls to the underlying renderer
 */
impl AnyRenderBodyContext {
    pub async fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        log::debug!(
            "<- Rendering properties: {}",
            serde_json::to_string(&data).unwrap_or_else(|e| e.to_string())
        );
        match self {
            AnyRenderBodyContext::Html(render_context) => render_context.handle_row(data).await,
            AnyRenderBodyContext::Json(json_body_renderer) => json_body_renderer.handle_row(data),
            AnyRenderBodyContext::Csv(csv_renderer) => csv_renderer.handle_row(data).await,
        }
    }
    pub async fn handle_error(&mut self, error: &anyhow::Error) -> anyhow::Result<()> {
        log::error!("SQL error: {error:?}");
        match self {
            AnyRenderBodyContext::Html(render_context) => render_context.handle_error(error).await,
            AnyRenderBodyContext::Json(json_body_renderer) => {
                json_body_renderer.handle_error(error)
            }
            AnyRenderBodyContext::Csv(csv_renderer) => csv_renderer.handle_error(error).await,
        }
    }
    pub async fn finish_query(&mut self) -> anyhow::Result<()> {
        match self {
            AnyRenderBodyContext::Html(render_context) => render_context.finish_query().await,
            AnyRenderBodyContext::Json(_json_body_renderer) => Ok(()),
            AnyRenderBodyContext::Csv(_csv_renderer) => Ok(()),
        }
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        match self {
            AnyRenderBodyContext::Html(HtmlRenderContext { writer, .. })
            | AnyRenderBodyContext::Json(JsonBodyRenderer { writer, .. }) => {
                writer.async_flush().await?;
            }
            AnyRenderBodyContext::Csv(csv_renderer) => csv_renderer.flush().await?,
        }
        Ok(())
    }

    pub async fn close(self) -> ResponseWriter {
        match self {
            AnyRenderBodyContext::Html(render_context) => render_context.close().await,
            AnyRenderBodyContext::Json(json_body_renderer) => json_body_renderer.close(),
            AnyRenderBodyContext::Csv(csv_renderer) => csv_renderer.close().await,
        }
    }
}

pub struct JsonBodyRenderer<W: std::io::Write> {
    writer: W,
    is_first: bool,
    prefix: &'static [u8],
    suffix: &'static [u8],
    separator: &'static [u8],
}

impl<W: std::io::Write> JsonBodyRenderer<W> {
    pub fn new_array(writer: W) -> JsonBodyRenderer<W> {
        let mut renderer = Self {
            writer,
            is_first: true,
            prefix: b"[\n",
            suffix: b"\n]",
            separator: b",\n",
        };
        let _ = renderer.write_prefix();
        renderer
    }
    pub fn new_jsonlines(writer: W) -> JsonBodyRenderer<W> {
        let mut renderer = Self {
            writer,
            is_first: true,
            prefix: b"",
            suffix: b"",
            separator: b"\n",
        };
        renderer.write_prefix().unwrap();
        renderer
    }
    pub fn new_server_sent_events(writer: W) -> JsonBodyRenderer<W> {
        let mut renderer = Self {
            writer,
            is_first: true,
            prefix: b"data: ",
            suffix: b"\n\n",
            separator: b"\n\ndata: ",
        };
        renderer.write_prefix().unwrap();
        renderer
    }
    fn write_prefix(&mut self) -> anyhow::Result<()> {
        self.writer.write_all(self.prefix)?;
        Ok(())
    }
    pub fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        if self.is_first {
            self.is_first = false;
        } else {
            let _ = self.writer.write_all(self.separator);
        }
        serde_json::to_writer(&mut self.writer, data)?;
        Ok(())
    }
    pub fn handle_error(&mut self, error: &anyhow::Error) -> anyhow::Result<()> {
        self.handle_row(&json!({
            "error": error.to_string()
        }))
    }

    pub fn close(mut self) -> W {
        let _ = self.writer.write_all(self.suffix);
        self.writer
    }
}

pub struct CsvBodyRenderer {
    // The writer is a large struct, so we store it on the heap
    writer: Box<csv_async::AsyncWriter<AsyncResponseWriter>>,
    columns: Vec<String>,
}

impl CsvBodyRenderer {
    pub async fn new(
        mut writer: ResponseWriter,
        options: &JsonValue,
    ) -> anyhow::Result<CsvBodyRenderer> {
        let mut builder = csv_async::AsyncWriterBuilder::new();
        if let Some(separator) = get_object_str(options, "separator") {
            let &[separator_byte] = separator.as_bytes() else {
                bail!("Invalid csv separator: {separator:?}. It must be a single byte.");
            };
            builder.delimiter(separator_byte);
        }
        if let Some(quote) = get_object_str(options, "quote") {
            let &[quote_byte] = quote.as_bytes() else {
                bail!("Invalid csv quote: {quote:?}. It must be a single byte.");
            };
            builder.quote(quote_byte);
        }
        if let Some(escape) = get_object_str(options, "escape") {
            let &[escape_byte] = escape.as_bytes() else {
                bail!("Invalid csv escape: {escape:?}. It must be a single byte.");
            };
            builder.escape(escape_byte);
        }
        if options
            .get("bom")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        {
            let utf8_bom = b"\xEF\xBB\xBF";
            writer.write_all(utf8_bom)?;
        }
        let mut async_writer = AsyncResponseWriter::new(writer);
        tokio::io::AsyncWriteExt::flush(&mut async_writer).await?;
        let writer = builder.create_writer(async_writer);
        Ok(CsvBodyRenderer {
            writer: Box::new(writer),
            columns: vec![],
        })
    }

    pub async fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        if self.columns.is_empty() {
            if let Some(obj) = data.as_object() {
                let headers: Vec<String> = obj.keys().map(String::to_owned).collect();
                self.columns = headers;
                self.writer.write_record(&self.columns).await?;
            }
        }

        if let Some(obj) = data.as_object() {
            let col2bytes = |s| {
                let val = obj.get(s);
                let Some(val) = val else {
                    return Cow::Borrowed(&b""[..]);
                };
                if let Some(s) = val.as_str() {
                    Cow::Borrowed(s.as_bytes())
                } else {
                    Cow::Owned(val.to_string().into_bytes())
                }
            };
            let record = self.columns.iter().map(col2bytes);
            self.writer.write_record(record).await?;
        }

        Ok(())
    }

    pub async fn handle_error(&mut self, error: &anyhow::Error) -> anyhow::Result<()> {
        let err_str = error.to_string();
        self.writer
            .write_record(
                self.columns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| if i == 0 { &err_str } else { "" })
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn close(self) -> ResponseWriter {
        self.writer
            .into_inner()
            .await
            .expect("Failed to get inner writer")
            .into_inner()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct HtmlRenderContext<W: std::io::Write> {
    app_state: Arc<AppState>,
    pub writer: W,
    current_component: Option<SplitTemplateRenderer>,
    shell_renderer: SplitTemplateRenderer,
    current_statement: usize,
    request_context: RequestContext,
}

const DEFAULT_COMPONENT: &str = "table";
const PAGE_SHELL_COMPONENT: &str = "shell";
const FRAGMENT_SHELL_COMPONENT: &str = "shell-empty";

impl<W: std::io::Write> HtmlRenderContext<W> {
    pub async fn new(
        app_state: Arc<AppState>,
        request_context: RequestContext,
        mut writer: W,
        initial_row: JsonValue,
    ) -> anyhow::Result<HtmlRenderContext<W>> {
        log::debug!("Creating the shell component for the page");

        let mut initial_rows = vec![Cow::Borrowed(&initial_row)];

        if !initial_rows
            .first()
            .and_then(|c| get_object_str(c, "component"))
            .is_some_and(Self::is_shell_component)
        {
            let default_shell = if request_context.is_embedded {
                FRAGMENT_SHELL_COMPONENT
            } else {
                PAGE_SHELL_COMPONENT
            };
            let added_row = json!({"component": default_shell});
            log::trace!(
                "No shell component found in the first row. Adding the default shell: {added_row}"
            );
            initial_rows.insert(0, Cow::Owned(added_row));
        }
        let mut rows_iter = initial_rows.into_iter().map(Cow::into_owned);

        let shell_row = rows_iter
            .next()
            .expect("shell row should exist at this point");
        let mut shell_component =
            get_object_str(&shell_row, "component").expect("shell should exist");
        if request_context.is_embedded && shell_component != FRAGMENT_SHELL_COMPONENT {
            log::warn!(
                "Embedded pages cannot use a shell component! Ignoring the '{shell_component}' component and its properties: {shell_row}"
            );
            shell_component = FRAGMENT_SHELL_COMPONENT;
        }
        let mut shell_renderer = Self::create_renderer(
            shell_component,
            Arc::clone(&app_state),
            0,
            request_context.content_security_policy.nonce,
        )
        .await
        .with_context(|| "The shell component should always exist")?;
        log::debug!("Rendering the shell with properties: {shell_row}");
        shell_renderer.render_start(&mut writer, shell_row)?;

        let mut initial_context = HtmlRenderContext {
            app_state,
            writer,
            current_component: None,
            shell_renderer,
            current_statement: 1,
            request_context,
        };

        for row in rows_iter {
            initial_context.handle_row(&row).await?;
        }

        Ok(initial_context)
    }

    fn is_shell_component(component: &str) -> bool {
        component.starts_with(PAGE_SHELL_COMPONENT)
    }

    async fn handle_component(&mut self, comp_str: &str, data: &JsonValue) -> anyhow::Result<()> {
        if Self::is_shell_component(comp_str) {
            bail!("There cannot be more than a single shell per page. You are trying to open the {} component, but a shell component is already opened for the current page. You can fix this by removing the extra shell component, or by moving this component to the top of the SQL file, before any other component that displays data.", comp_str);
        }

        if comp_str == "log" {
            return handle_log_component(
                &self.request_context.source_path,
                Some(self.current_statement),
                data,
            );
        }

        match self.open_component_with_data(comp_str, &data).await {
            Ok(_) => Ok(()),
            Err(err) => match HeaderComponent::try_from(comp_str) {
                Ok(_) => bail!("The {comp_str} component cannot be used after data has already been sent to the client's browser. \n\
                                This component must be used before any other component. \n\
                                    To fix this, either move the call to the '{comp_str}' component to the top of the SQL file, \n\
                                or create a new SQL file where '{comp_str}' is the first component."),
                Err(()) => Err(err),
            },
        }
    }

    pub async fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        let new_component = get_object_str(data, "component");
        let current_component = self
            .current_component
            .as_ref()
            .map(SplitTemplateRenderer::name);
        if let Some(comp_str) = new_component {
            self.handle_component(comp_str, data).await?;
        } else if current_component.is_none() {
            self.open_component_with_data(DEFAULT_COMPONENT, &JsonValue::Null)
                .await?;
            self.render_current_template_with_data(&data).await?;
        } else {
            self.render_current_template_with_data(&data).await?;
        }
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub async fn finish_query(&mut self) -> anyhow::Result<()> {
        log::debug!("-> Query {} finished", self.current_statement);
        self.current_statement += 1;
        Ok(())
    }

    /// Handles the rendering of an error.
    /// Returns whether the error is irrecoverable and the rendering must stop
    pub async fn handle_error(&mut self, error: &anyhow::Error) -> anyhow::Result<()> {
        self.close_component()?;
        let data = if self.app_state.config.environment.is_prod() {
            json!({
                "description": format!("Please contact the administrator for more information. The error has been logged."),
            })
        } else {
            json!({
                "query_number": self.current_statement,
                "description": error.to_string(),
                "backtrace": get_backtrace_as_strings(error),
                "note": "You can hide error messages like this one from your users by setting the 'environment' configuration option to 'production'."
            })
        };
        let saved_component = self.open_component_with_data("error", &data).await?;
        self.close_component()?;
        self.current_component = saved_component;
        Ok(())
    }

    pub async fn handle_result<R>(&mut self, result: &anyhow::Result<R>) -> anyhow::Result<()> {
        if let Err(error) = result {
            self.handle_error(error).await
        } else {
            Ok(())
        }
    }

    pub async fn handle_result_and_log<R>(&mut self, result: &anyhow::Result<R>) {
        if let Err(e) = self.handle_result(result).await {
            log::error!("{e}");
        }
    }

    async fn render_current_template_with_data<T: Serialize>(
        &mut self,
        data: &T,
    ) -> anyhow::Result<()> {
        if self.current_component.is_none() {
            self.set_current_component(DEFAULT_COMPONENT).await?;
        }
        self.current_component
            .as_mut()
            .expect("just set the current component")
            .render_item(&mut self.writer, json!(data))?;
        self.shell_renderer
            .render_item(&mut self.writer, JsonValue::Null)?;
        Ok(())
    }

    async fn create_renderer(
        component: &str,
        app_state: Arc<AppState>,
        component_index: usize,
        nonce: u64,
    ) -> anyhow::Result<SplitTemplateRenderer> {
        let split_template = app_state
            .all_templates
            .get_template(&app_state, component)
            .await?;
        Ok(SplitTemplateRenderer::new(
            split_template,
            app_state,
            component_index,
            nonce,
        ))
    }

    /// Set a new current component and return the old one
    async fn set_current_component(
        &mut self,
        component: &str,
    ) -> anyhow::Result<Option<SplitTemplateRenderer>> {
        let current_component_index = self
            .current_component
            .as_ref()
            .map_or(1, |c| c.component_index);
        let new_component = Self::create_renderer(
            component,
            Arc::clone(&self.app_state),
            current_component_index + 1,
            self.request_context.content_security_policy.nonce,
        )
        .await?;
        Ok(self.current_component.replace(new_component))
    }

    async fn open_component_with_data<T: Serialize>(
        &mut self,
        component: &str,
        data: &T,
    ) -> anyhow::Result<Option<SplitTemplateRenderer>> {
        self.close_component()?;
        let old_component = self.set_current_component(component).await?;
        self.current_component
            .as_mut()
            .expect("just set the current component")
            .render_start(&mut self.writer, json!(data))?;
        Ok(old_component)
    }

    fn close_component(&mut self) -> anyhow::Result<()> {
        if let Some(old_component) = self.current_component.as_mut() {
            old_component.render_end(&mut self.writer)?;
        }
        Ok(())
    }

    pub async fn close(mut self) -> W {
        if let Some(old_component) = self.current_component.as_mut() {
            let res = old_component
                .render_end(&mut self.writer)
                .map_err(|e| format_err!("Unable to render the component closing: {e}"));
            self.handle_result_and_log(&res).await;
        }
        let res = self
            .shell_renderer
            .render_end(&mut self.writer)
            .map_err(|e| format_err!("Unable to render the shell closing: {e}"));
        self.handle_result_and_log(&res).await;
        self.writer
    }
}

fn handle_log_component(
    source_path: &Path,
    current_statement: Option<usize>,
    data: &JsonValue,
) -> anyhow::Result<()> {
    let mut log_level = log::Level::Info;
    if let Some(priority) = get_object_str(data, "priority") {
        if let Ok(level) = log::Level::from_str(priority) {
            log_level = level;
        }
    }

    let current_statement_string = if let Some(option) = current_statement {
        &format!("statement {option}")
    } else {
        "header"
    };

    let target = format!(
        "sqlpage::log from file \"{}\" in {}",
        source_path.display(),
        current_statement_string
    );

    if let Some(message) = get_object_str(data, "message") {
        log::log!(target: &target, log_level, "{message}");
    } else {
        return Err(anyhow::anyhow!(
            "message undefined for log in \"{}\" in {}",
            source_path.display(),
            current_statement_string
        ));
    }

    Ok(())
}

pub(super) fn get_backtrace_as_strings(error: &anyhow::Error) -> Vec<String> {
    let mut backtrace = vec![];
    let mut source = error.source();
    while let Some(s) = source {
        backtrace.push(format!("{s}"));
        source = s.source();
    }
    backtrace
}

struct HandlebarWriterOutput<W: std::io::Write>(W);

impl<W: std::io::Write> handlebars::Output for HandlebarWriterOutput<W> {
    fn write(&mut self, seg: &str) -> std::io::Result<()> {
        std::io::Write::write_all(&mut self.0, seg.as_bytes())
    }
}

pub struct SplitTemplateRenderer {
    split_template: Arc<SplitTemplate>,
    // LocalVars is a large struct, so we store it on the heap
    local_vars: Option<Box<handlebars::LocalVars>>,
    ctx: Box<handlebars::Context>,
    app_state: Arc<AppState>,
    row_index: usize,
    component_index: usize,
    nonce: u64,
}

const _: () = assert!(
    std::mem::size_of::<SplitTemplateRenderer>() <= 64,
    "SplitTemplateRenderer should be small enough to be allocated on the stack"
);

impl SplitTemplateRenderer {
    fn new(
        split_template: Arc<SplitTemplate>,
        app_state: Arc<AppState>,
        component_index: usize,
        nonce: u64,
    ) -> Self {
        Self {
            split_template,
            local_vars: None,
            app_state,
            row_index: 0,
            ctx: Box::new(handlebars::Context::null()),
            component_index,
            nonce,
        }
    }
    fn name(&self) -> &str {
        self.split_template
            .list_content
            .name
            .as_deref()
            .unwrap_or_default()
    }

    fn render_start<W: std::io::Write>(
        &mut self,
        writer: W,
        data: JsonValue,
    ) -> Result<(), RenderError> {
        log::trace!(
            "Starting rendering of a template{} with the following top-level parameters: {data}",
            self.split_template
                .name()
                .map(|n| format!(" ('{n}')"))
                .unwrap_or_default(),
        );
        let mut render_context = handlebars::RenderContext::new(None);
        let blk = render_context
            .block_mut()
            .expect("context created without block");
        blk.set_local_var("component_index", self.component_index.into());
        blk.set_local_var("csp_nonce", self.nonce.into());

        *self.ctx.data_mut() = data;
        let mut output = HandlebarWriterOutput(writer);
        self.split_template.before_list.render(
            &self.app_state.all_templates.handlebars,
            &self.ctx,
            &mut render_context,
            &mut output,
        )?;
        let blk = render_context.block_mut();
        if let Some(blk) = blk {
            let local_vars = std::mem::take(blk.local_variables_mut());
            self.local_vars = Some(Box::new(local_vars));
        }
        self.row_index = 0;
        Ok(())
    }

    fn render_item<W: std::io::Write>(
        &mut self,
        writer: W,
        data: JsonValue,
    ) -> Result<(), RenderError> {
        log::trace!("Rendering a new item in the page: {data:?}");
        if let Some(local_vars) = self.local_vars.take() {
            let mut render_context = handlebars::RenderContext::new(None);
            let blk = render_context
                .block_mut()
                .expect("context created without block");
            *blk.local_variables_mut() = *local_vars;
            let mut blk = BlockContext::new();
            blk.set_base_value(data);
            blk.set_local_var("component_index", self.component_index.into());
            blk.set_local_var("row_index", self.row_index.into());
            blk.set_local_var("csp_nonce", self.nonce.into());
            render_context.push_block(blk);
            let mut output = HandlebarWriterOutput(writer);
            self.split_template.list_content.render(
                &self.app_state.all_templates.handlebars,
                &self.ctx,
                &mut render_context,
                &mut output,
            )?;
            render_context.pop_block();
            let blk = render_context.block_mut();
            if let Some(blk) = blk {
                let local_vars = std::mem::take(blk.local_variables_mut());
                self.local_vars = Some(Box::new(local_vars));
            }
            self.row_index += 1;
        }
        Ok(())
    }

    fn render_end<W: std::io::Write>(&mut self, writer: W) -> Result<(), RenderError> {
        log::trace!(
            "Closing a template {}",
            self.split_template
                .name()
                .map(|name| format!("('{name}')"))
                .unwrap_or_default(),
        );
        if let Some(mut local_vars) = self.local_vars.take() {
            let mut render_context = handlebars::RenderContext::new(None);
            local_vars.put("row_index", self.row_index.into());
            local_vars.put("component_index", self.component_index.into());
            local_vars.put("csp_nonce", self.nonce.into());
            log::trace!("Rendering the after_list template with the following local variables: {local_vars:?}");
            *render_context
                .block_mut()
                .expect("ctx created without block")
                .local_variables_mut() = *local_vars;
            let mut output = HandlebarWriterOutput(writer);
            self.split_template.after_list.render(
                &self.app_state.all_templates.handlebars,
                &self.ctx,
                &mut render_context,
                &mut output,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config;
    use crate::templates::split_template;
    use handlebars::Template;

    #[actix_web::test]
    async fn test_split_template_render() -> anyhow::Result<()> {
        let template = Template::compile(
            "Hello {{name}} !\
        {{#each_row}} ({{x}} : {{../name}}) {{/each_row}}\
        Goodbye {{name}}",
        )?;
        let split = split_template(template);
        let mut output = Vec::new();
        let config = app_config::tests::test_config();
        let app_state = Arc::new(AppState::init(&config).await.unwrap());
        let mut rdr = SplitTemplateRenderer::new(Arc::new(split), app_state, 0, 0);
        rdr.render_start(&mut output, json!({"name": "SQL"}))?;
        rdr.render_item(&mut output, json!({"x": 1}))?;
        rdr.render_item(&mut output, json!({"x": 2}))?;
        rdr.render_end(&mut output)?;
        assert_eq!(
            String::from_utf8_lossy(&output),
            "Hello SQL ! (1 : SQL)  (2 : SQL) Goodbye SQL"
        );
        Ok(())
    }

    #[actix_web::test]
    async fn test_delayed() -> anyhow::Result<()> {
        let template = Template::compile(
            "{{#each_row}}<b> {{x}} {{#delay}} {{x}} </b>{{/delay}}{{/each_row}}{{flush_delayed}}",
        )?;
        let split = split_template(template);
        let mut output = Vec::new();
        let config = app_config::tests::test_config();
        let app_state = Arc::new(AppState::init(&config).await.unwrap());
        let mut rdr = SplitTemplateRenderer::new(Arc::new(split), app_state, 0, 0);
        rdr.render_start(&mut output, json!(null))?;
        rdr.render_item(&mut output, json!({"x": 1}))?;
        rdr.render_item(&mut output, json!({"x": 2}))?;
        rdr.render_end(&mut output)?;
        assert_eq!(
            String::from_utf8_lossy(&output),
            "<b> 1 <b> 2  2 </b> 1 </b>"
        );
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum HeaderComponent {
    StatusCode,
    HttpHeader,
    Redirect,
    Json,
    Csv,
    Cookie,
    Authentication,
    Download,
    Log,
}

impl TryFrom<&str> for HeaderComponent {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "status_code" => Ok(Self::StatusCode),
            "http_header" => Ok(Self::HttpHeader),
            "redirect" => Ok(Self::Redirect),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "cookie" => Ok(Self::Cookie),
            "authentication" => Ok(Self::Authentication),
            "download" => Ok(Self::Download),
            "log" => Ok(Self::Log),
            _ => Err(()),
        }
    }
}
