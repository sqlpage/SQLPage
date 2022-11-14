use crate::templates::SplitTemplate;
use crate::AppState;
use anyhow::{format_err, Context as AnyhowContext};
use async_recursion::async_recursion;
use handlebars::{BlockContext, Context, JsonValue, RenderError, Renderable};
use serde::Serialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::Arc;

#[allow(clippy::module_name_repetitions)]
pub struct RenderContext<W: std::io::Write> {
    app_state: Arc<AppState>,
    pub writer: W,
    current_component: Option<SplitTemplateRenderer>,
    shell_renderer: SplitTemplateRenderer,
    recursion_depth: usize,
    current_statement: usize,
}

const DEFAULT_COMPONENT: &str = "default";
const MAX_RECURSION_DEPTH: usize = 256;

impl<W: std::io::Write> RenderContext<W> {
    pub async fn new(app_state: Arc<AppState>, writer: W) -> RenderContext<W> {
        let shell_renderer = Self::create_renderer("shell", Arc::clone(&app_state))
            .await
            .expect("shell must always exist");
        RenderContext {
            app_state,
            writer,
            current_component: None,
            shell_renderer,
            recursion_depth: 0,
            current_statement: 1,
        }
    }

    #[async_recursion(? Send)]
    pub async fn handle_row(&mut self, data: &JsonValue) -> anyhow::Result<()> {
        log::debug!(
            "<- Processing database row: {}",
            serde_json::to_string(&data).unwrap_or_else(|e| e.to_string())
        );
        let new_component = data
            .as_object()
            .and_then(|o| o.get("component"))
            .and_then(JsonValue::as_str);
        let current_component = self
            .current_component
            .as_ref()
            .map(SplitTemplateRenderer::name);
        match (current_component, new_component) {
            (None, Some("head") | None) => {
                self.shell_renderer
                    .render_start(&mut self.writer, json!(&data))?;
                self.open_component_with_data(DEFAULT_COMPONENT, &data)
                    .await?;
            }
            (None, new_component) => {
                self.shell_renderer
                    .render_start(&mut self.writer, json!(null))?;
                let component = new_component.unwrap_or(DEFAULT_COMPONENT);
                self.open_component_with_data(component, &data).await?;
            }
            (Some(_current_component), Some("dynamic")) => {
                self.render_dynamic(data).await?;
            }
            (Some(_current_component), Some(new_component)) => {
                self.open_component_with_data(new_component, &data).await?;
            }
            (Some(_), _) => {
                self.render_current_template_with_data(&data)?;
            }
        }
        Ok(())
    }

    async fn render_dynamic(&mut self, data: &Value) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.recursion_depth <= MAX_RECURSION_DEPTH,
            "Maximum recursion depth exceeded in the dynamic component."
        );
        let properties: Vec<Cow<JsonValue>> = data
            .get("properties")
            .and_then(|props| match props {
                Value::String(s) => match serde_json::from_str::<JsonValue>(s).ok()? {
                    Value::Array(values) => Some(values.into_iter().map(Cow::Owned).collect()),
                    obj @ Value::Object(_) => Some(vec![Cow::Owned(obj)]),
                    _ => None,
                },
                obj @ Value::Object(_) => Some(vec![Cow::Borrowed(obj)]),
                _ => None,
            })
            .context(
                "The dynamic component requires a parameter called 'parameters' that is a json ",
            )?;
        for p in properties {
            self.recursion_depth += 1;
            let res = self.handle_row(&p).await;
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
        if self.current_component.is_some() {
            self.close_component()?;
        } else {
            self.shell_renderer
                .render_start(&mut self.writer, json!(null))?;
        }
        let saved_component = self.current_component.take();
        self.open_component("error").await?;
        let description = error.to_string();
        let mut backtrace = vec![];
        let mut source = error.source();
        while let Some(s) = source {
            backtrace.push(format!("{s}"));
            source = s.source();
        }
        self.render_current_template_with_data(&json!({
            "query_number": self.current_statement,
            "description": description,
            "backtrace": backtrace
        }))?;
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

    fn render_current_template_with_data<T: Serialize>(&mut self, data: &T) -> anyhow::Result<()> {
        use anyhow::Context;
        let rdr = self.current_component.as_mut().with_context(|| {
            format!(
                "Tried to render the following data but no component is selected: {}",
                serde_json::to_string(data).unwrap_or_default()
            )
        })?;
        rdr.render_item(&mut self.writer, json!(data))?;
        self.shell_renderer
            .render_item(&mut self.writer, JsonValue::Null)?;
        Ok(())
    }

    async fn open_component(&mut self, component: &str) -> anyhow::Result<()> {
        self.open_component_with_data(component, &json!(null)).await
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

    async fn set_current_component(&mut self, component: &str) -> anyhow::Result<()> {
        self.current_component =
            Some(Self::create_renderer(component, Arc::clone(&self.app_state)).await?);
        Ok(())
    }

    async fn open_component_with_data<T: Serialize>(
        &mut self,
        component: &str,
        data: &T,
    ) -> anyhow::Result<()> {
        self.close_component()?;
        self.set_current_component(component).await?;
        self.current_component
            .as_mut()
            .unwrap()
            .render_start(&mut self.writer, json!(data))?;
        Ok(())
    }

    fn close_component(&mut self) -> anyhow::Result<()> {
        if let Some(component) = &mut self.current_component {
            component.render_end(&mut self.writer)?;
        }
        Ok(())
    }

    pub async fn close(mut self) -> W {
        if let Some(mut component) = self.current_component.take() {
            let res = component
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
        log::trace!("Starting rendering of a new page with the following page-level data: {data}");
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
        log::trace!("Closing the current page");
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
        let app_state = Arc::new(AppState::init().unwrap());
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
        let app_state = Arc::new(AppState::init().unwrap());
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
