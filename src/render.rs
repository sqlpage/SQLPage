use crate::templates::SplitTemplate;
use crate::AppState;
use actix_web::http::{header, StatusCode};
use actix_web::{HttpResponse, HttpResponseBuilder};
use anyhow::{bail, format_err, Context as AnyhowContext};
use async_recursion::async_recursion;
use handlebars::{BlockContext, Context, JsonValue, RenderError, Renderable};
use serde::Serialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::Arc;

pub enum PageContext<W: std::io::Write> {
    /// Indicates that we should stay in the header context
    Header(HeaderContext<W>),

    /// Indicates that we should start rendering the body
    Body {
        http_response: HttpResponseBuilder,
        renderer: RenderContext<W>,
    },

    /// The response is ready, and should be sent as is. No further statements should be executed
    Close(HttpResponse),
}

/// Handles the first SQL statements, before the headers have been sent to
pub struct HeaderContext<W: std::io::Write> {
    app_state: Arc<AppState>,
    pub writer: W,
    response: HttpResponseBuilder,
    has_status: bool,
}

impl<W: std::io::Write> HeaderContext<W> {
    pub fn new(app_state: Arc<AppState>, writer: W) -> Self {
        let mut response = HttpResponseBuilder::new(StatusCode::OK);
        response.content_type("text/html; charset=utf-8");
        Self {
            app_state,
            writer,
            response,
            has_status: false,
        }
    }
    pub async fn handle_row(self, data: JsonValue) -> anyhow::Result<PageContext<W>> {
        log::debug!("Handling header row: {data}");
        match get_object_str(&data, "component") {
            Some("status_code") => self.status_code(&data).map(PageContext::Header),
            Some("http_header") => self.add_http_header(&data).map(PageContext::Header),
            Some("redirect") => self.redirect(&data).map(PageContext::Close),
            Some("json") => self.json(&data).map(PageContext::Close),
            Some("cookie") => self.add_cookie(&data).map(PageContext::Header),
            Some("authentication") => self.authentication(&data),
            _ => self.start_body(data).await,
        }
    }

    pub async fn handle_error(self, err: anyhow::Error) -> anyhow::Result<PageContext<W>> {
        log::debug!("Handling header error: {err}");
        let data = json!({
            "component": "error",
            "description": err.to_string(),
            "backtrace": get_backtrace(&err),
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

        let remove = obj.get("remove");
        if remove == Some(&json!(true)) || remove == Some(&json!(1)) {
            self.response.cookie(cookie);
            log::trace!("Removing cookie {}", name);
            return Ok(self);
        }

        let value = obj
            .get("value")
            .and_then(JsonValue::as_str)
            .with_context(|| "cookie value must be a string")?;
        cookie.set_value(value);
        let http_only = obj.get("http_only");
        cookie.set_http_only(http_only != Some(&json!(false)) && http_only != Some(&json!(0)));
        let secure = obj.get("secure");
        cookie.set_secure(secure != Some(&json!(false)) && secure != Some(&json!(0)));
        let path = obj.get("path").and_then(JsonValue::as_str);
        if let Some(path) = path {
            cookie.set_path(path);
        }
        let domain = obj.get("domain").and_then(JsonValue::as_str);
        if let Some(domain) = domain {
            cookie.set_domain(domain);
        }
        let expires = obj.get("expires").and_then(JsonValue::as_i64);
        if let Some(expires) = expires {
            cookie.set_expires(actix_web::cookie::Expiration::DateTime(
                actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(expires)?,
            ));
        }
        log::trace!("Setting cookie {}", cookie);
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
    fn json(mut self, data: &JsonValue) -> anyhow::Result<HttpResponse> {
        let contents = data
            .get("contents")
            .with_context(|| "Missing 'contents' property for the json component")?;
        let json_response = if let Some(s) = contents.as_str() {
            s.as_bytes().to_owned()
        } else {
            serde_json::to_vec(contents)?
        };
        self.response
            .insert_header((header::CONTENT_TYPE, "application/json"));
        Ok(self.response.body(json_response))
    }

    fn authentication(mut self, data: &JsonValue) -> anyhow::Result<PageContext<W>> {
        use argon2::Argon2;
        use password_hash::PasswordHash;
        let password_hash = get_object_str(data, "password_hash");
        let password = get_object_str(data, "password");
        if let (Some(password), Some(password_hash)) = (password, password_hash) {
            match PasswordHash::new(password_hash)
                .map_err(|e| {
                    anyhow::anyhow!("invalid value for the password_hash property: {}", e)
                })?
                .verify_password(&[&Argon2::default()], password)
            {
                Ok(()) => return Ok(PageContext::Header(self)),
                Err(e) => log::info!("User authentication failed: {}", e),
            }
        }
        log::debug!(
            "Authentication failed with password_hash = {:?}",
            password_hash
        );
        // The authentication failed
        if let Some(link) = get_object_str(data, "link") {
            self.response.status(StatusCode::FOUND);
            self.response.insert_header((header::LOCATION, link));
            self.has_status = true;
        } else {
            self.response.status(StatusCode::UNAUTHORIZED);
            self.response
                .insert_header((header::WWW_AUTHENTICATE, "Basic realm=\"Auth required\""));
            self.has_status = true;
        }
        // Set an empty response body
        let http_response = self.response.body(());
        Ok(PageContext::Close(http_response))
    }

    async fn start_body(self, data: JsonValue) -> anyhow::Result<PageContext<W>> {
        let renderer = RenderContext::new(self.app_state, self.writer, data)
            .await
            .with_context(|| "Failed to create a render context from the header context.")?;
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

fn get_backtrace(error: &anyhow::Error) -> Vec<String> {
    let mut backtrace = vec![];
    let mut source = error.source();
    while let Some(s) = source {
        backtrace.push(format!("{s}"));
        source = s.source();
    }
    backtrace
}

fn get_object_str<'a>(json: &'a JsonValue, key: &str) -> Option<&'a str> {
    json.as_object()
        .and_then(|obj| obj.get(key))
        .and_then(JsonValue::as_str)
}

#[allow(clippy::module_name_repetitions)]
pub struct RenderContext<W: std::io::Write> {
    app_state: Arc<AppState>,
    pub writer: W,
    current_component: Option<SplitTemplateRenderer>,
    shell_renderer: SplitTemplateRenderer,
    recursion_depth: usize,
    current_statement: usize,
}

const DEFAULT_COMPONENT: &str = "debug";
const SHELL_COMPONENT: &str = "shell";
const DYNAMIC_COMPONENT: &str = "dynamic";
const MAX_RECURSION_DEPTH: usize = 256;

impl<W: std::io::Write> RenderContext<W> {
    pub async fn new(
        app_state: Arc<AppState>,
        mut writer: W,
        mut initial_row: JsonValue,
    ) -> anyhow::Result<RenderContext<W>> {
        log::debug!("Creating the shell component for the page");
        let mut shell_renderer = Self::create_renderer(SHELL_COMPONENT, Arc::clone(&app_state))
            .await
            .with_context(|| "The shell component should always exist")?;

        let mut initial_component =
            Some(get_object_str(&initial_row, "component").unwrap_or(DEFAULT_COMPONENT));
        let mut shell_properties = JsonValue::Null;
        match initial_component {
            Some(SHELL_COMPONENT) => {
                shell_properties = initial_row.take();
                initial_component = None;
            },
            Some(DYNAMIC_COMPONENT) => {
                let dynamic_properties = Self::extract_dynamic_properties(&initial_row)?;
                for prop in dynamic_properties {
                    match get_object_str(&prop, "component") {
                        None | Some(SHELL_COMPONENT) => {
                            shell_properties = prop.into_owned();
                            initial_component = None;
                        },
                        _ => bail!("Dynamic components at the top level are not supported, except for setting the shell component properties"),
                    }
                }
            },
            _ => log::trace!("The first row is not a shell component, so we will render a shell with default properties"),
        }

        log::debug!("Rendering the shell with properties: {shell_properties}");
        shell_renderer.render_start(&mut writer, shell_properties)?;

        let mut initial_context = RenderContext {
            app_state,
            writer,
            current_component: None,
            shell_renderer,
            recursion_depth: 0,
            current_statement: 1,
        };

        if let Some(component) = initial_component {
            log::trace!("The page starts with a component without a shell: {component}");
            initial_context
                .open_component_with_data(component, &initial_row)
                .await?;
        }

        Ok(initial_context)
    }

    async fn current_component(&mut self) -> anyhow::Result<&mut SplitTemplateRenderer> {
        if self.current_component.is_none() {
            let _old = self.set_current_component(DEFAULT_COMPONENT).await?;
        }
        Ok(self.current_component.as_mut().unwrap())
    }

    #[async_recursion(? Send)]
    pub async fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        log::debug!(
            "<- Processing database row: {}",
            serde_json::to_string(&data).unwrap_or_else(|e| e.to_string())
        );
        let new_component = get_object_str(data, "component");
        let current_component = self.current_component().await?.name();
        match (current_component, new_component) {
            (_current_component, Some(DYNAMIC_COMPONENT)) => {
                self.render_dynamic(data).await.with_context(|| {
                    format!("Unable to render dynamic component with properties {data}")
                })?;
            }
            (_, Some("http_header")) => {
                bail!("The http_header component can not be used in the body of the page, only as the very first component in the page. \
                       The HTTP headers have already be sent for the current page, they cannot be changed now.");
            }
            (_current_component, Some(new_component)) => {
                self.open_component_with_data(new_component, &data).await?;
            }
            (_, _) => {
                self.render_current_template_with_data(&data).await?;
            }
        }
        Ok(())
    }

    fn extract_dynamic_properties(data: &Value) -> anyhow::Result<Vec<Cow<'_, JsonValue>>> {
        let properties_key = "properties";
        let properties_obj = data
            .get(properties_key)
            .with_context(|| format!("Missing '{properties_key}' key."))?;
        Ok(match properties_obj {
            Value::String(s) => match serde_json::from_str::<JsonValue>(s)
                .with_context(|| "parsing json properties")?
            {
                Value::Array(values) => values.into_iter().map(Cow::Owned).collect(),
                obj @ Value::Object(_) => vec![Cow::Owned(obj)],
                other => bail!(
                    "Expected properties string to parse as array or object, got {other} instead."
                ),
            },
            obj @ Value::Object(_) => vec![Cow::Borrowed(obj)],
            Value::Array(values) => values.iter().map(Cow::Borrowed).collect(),
            other => bail!("Expected properties of type array or object, got {other} instead."),
        })
    }

    async fn render_dynamic(&mut self, data: &Value) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.recursion_depth <= MAX_RECURSION_DEPTH,
            "Maximum recursion depth exceeded in the dynamic component."
        );
        for dynamic_row_obj in Self::extract_dynamic_properties(data)? {
            self.recursion_depth += 1;
            let res = self.handle_row(&dynamic_row_obj).await;
            self.recursion_depth -= 1;
            res?;
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
        log::warn!("SQL error: {:?}", error);
        self.close_component()?;
        let description = error.to_string();
        let data = json!({
            "query_number": self.current_statement,
            "description": description,
            "backtrace": get_backtrace(error)
        });
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
            log::error!("{}", e);
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
    ) -> anyhow::Result<SplitTemplateRenderer> {
        let split_template = app_state
            .all_templates
            .get_template(&app_state, component)
            .await?;
        Ok(SplitTemplateRenderer::new(split_template, app_state))
    }

    /// Set a new current component and return the old one
    async fn set_current_component(
        &mut self,
        component: &str,
    ) -> anyhow::Result<Option<SplitTemplateRenderer>> {
        let new_component = Self::create_renderer(component, Arc::clone(&self.app_state)).await?;
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
        if let Some(old_component) = self.current_component.as_mut().take() {
            old_component.render_end(&mut self.writer)?;
        }
        Ok(())
    }

    pub async fn close(mut self) -> W {
        if let Some(old_component) = self.current_component.as_mut().take() {
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

struct HandlebarWriterOutput<W: std::io::Write>(W);

impl<W: std::io::Write> handlebars::Output for HandlebarWriterOutput<W> {
    fn write(&mut self, seg: &str) -> std::io::Result<()> {
        std::io::Write::write_all(&mut self.0, seg.as_bytes())
    }
}

pub struct SplitTemplateRenderer {
    split_template: Arc<SplitTemplate>,
    local_vars: Option<handlebars::LocalVars>,
    ctx: Context,
    app_state: Arc<AppState>,
    row_index: usize,
}

impl SplitTemplateRenderer {
    fn new(split_template: Arc<SplitTemplate>, app_state: Arc<AppState>) -> Self {
        Self {
            split_template,
            local_vars: None,
            app_state,
            row_index: 0,
            ctx: Context::null(),
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
        *self.ctx.data_mut() = data;
        let mut output = HandlebarWriterOutput(writer);
        self.split_template.before_list.render(
            &self.app_state.all_templates.handlebars,
            &self.ctx,
            &mut render_context,
            &mut output,
        )?;
        self.local_vars = render_context
            .block_mut()
            .map(|blk| std::mem::take(blk.local_variables_mut()));
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
            *blk.local_variables_mut() = local_vars;
            let mut blk = BlockContext::new();
            blk.set_base_value(data);
            blk.set_local_var("row_index", JsonValue::Number(self.row_index.into()));
            render_context.push_block(blk);
            let mut output = HandlebarWriterOutput(writer);
            self.split_template.list_content.render(
                &self.app_state.all_templates.handlebars,
                &self.ctx,
                &mut render_context,
                &mut output,
            )?;
            render_context.pop_block();
            self.local_vars = render_context
                .block_mut()
                .map(|blk| std::mem::take(blk.local_variables_mut()));
            self.row_index += 1;
        }
        Ok(())
    }

    fn render_end<W: std::io::Write>(&mut self, writer: W) -> Result<(), RenderError> {
        log::trace!(
            "Closing a template {}",
            self.split_template
                .name()
                .map(|n| format!("('{n}')"))
                .unwrap_or_default(),
        );
        if let Some(local_vars) = self.local_vars.take() {
            let mut render_context = handlebars::RenderContext::new(None);
            *render_context
                .block_mut()
                .expect("ctx created without block")
                .local_variables_mut() = local_vars;
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
        let mut rdr = SplitTemplateRenderer::new(Arc::new(split), app_state);
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
        let mut rdr = SplitTemplateRenderer::new(Arc::new(split), app_state);
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
