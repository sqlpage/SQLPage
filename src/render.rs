use std::collections::HashMap;
use crate::http::ResponseWriter;
use crate::{AppState, TEMPLATES_DIR};
use handlebars::{handlebars_helper, template::TemplateElement, Handlebars, Template, TemplateError, Renderable, JsonValue, Context, BlockContext};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use sqlx::{Column, Database, Decode, Row};
use std::fs::DirEntry;
use serde_json::json;

type TplRenderer<'a> = SplitTemplateRenderer<'a, ResponseWriter>;

pub struct RenderContext<'a> {
    app_state: &'a AppState,
    writer: ResponseWriter,
    current_component: Option<TplRenderer<'a>>,
    shell_renderer: TplRenderer<'a>,

}

const DEFAULT_COMPONENT: &str = "default";

impl RenderContext<'_> {
    pub fn new(app_state: &AppState, writer: ResponseWriter) -> RenderContext {
        let shell_renderer = Self::create_renderer("shell", app_state, &writer).expect("shell must always exist");
        RenderContext {
            app_state,
            writer,
            current_component: None,
            shell_renderer,
        }
    }

    pub async fn handle_row(
        &mut self,
        row: sqlx::any::AnyRow,
    ) -> anyhow::Result<()> {
        let data = SerializeRow(row);
        log::debug!("Processing database row: {:?}", json!(data));
        let new_component = data.0.try_get::<&str, &str>("component");
        let current_component = self.current_component.as_ref().map(|c| c.name());
        match (current_component, new_component) {
            (None, Ok("head")) | (None, Err(_)) => {
                self.shell_renderer.render_start(json!(&&data))?;
                self.open_component_with_data(DEFAULT_COMPONENT, &&data)?;
            }
            (None, new_component) => {
                self.shell_renderer.render_start(json!(null))?;
                let component = new_component.unwrap_or(DEFAULT_COMPONENT);
                self.open_component_with_data(component, &&data)?;
            }
            (Some(current_component), Ok(new_component)) if new_component != current_component => {
                self.open_component_with_data(new_component, &&data)?;
            }
            (Some(_), _) => {
                self.render_current_template_with_data(&&data)?;
            }
        }
        Ok(())
    }

    pub async fn finish_query(
        &mut self,
        result: sqlx::any::AnyQueryResult,
    ) -> anyhow::Result<()> {
        log::trace!("finish_query: {:?}", result);
        Ok(())
    }

    /// Handles the rendering of an error.
    /// Returns whether the error is irrecoverable and the rendering must stop
    pub fn handle_error(&mut self, error: &impl std::error::Error) -> anyhow::Result<()> {
        log::warn!("SQL error: {:?}", error);
        self.close_component()?;
        let saved_component = self.current_component.take();
        self.open_component("error")?;
        let description = format!("{}", error);
        let mut backtrace = vec![];
        let mut source = error.source();
        while let Some(s) = source {
            backtrace.push(format!("{}", s));
            source = s.source()
        }
        self.render_current_template_with_data(&json!({
            "description": description,
            "backtrace": backtrace
        }))?;
        self.close_component()?;
        self.current_component = saved_component;
        Ok(())
    }

    pub fn handle_result<R, E: std::error::Error>(&mut self, result: &Result<R, E>) -> anyhow::Result<()> {
        if let Err(error) = result {
            self.handle_error(error)
        } else {
            Ok(())
        }
    }

    pub fn handle_result_and_log<R, E: std::error::Error>(&mut self, result: &Result<R, E>) {
        if let Err(e) = self.handle_result(result) {
            log::error!("{}", e);
        }
    }

    fn render_current_template_with_data<T: Serialize>(&mut self, data: &T) -> anyhow::Result<()> {
        use anyhow::Context;
        let rdr = self.current_component.as_mut()
            .with_context(|| format!("Tried to render the following data but no component is selected: {}", serde_json::to_string(data).unwrap_or_default()))?;
        rdr.render_item(json!(data))?;
        self.shell_renderer.render_item(JsonValue::Null)?;
        Ok(())
    }

    fn open_component(&mut self, component: &str) -> anyhow::Result<()> {
        self.open_component_with_data(component, &json!(null))
    }

    fn create_renderer<'a>(component: &str, app_state: &'a AppState, writer: &ResponseWriter) -> anyhow::Result<TplRenderer<'a>> {
        use anyhow::Context;
        let split_template = app_state.all_templates.split_templates
            .get(component)
            .with_context(|| format!("The component '{component}' was not found."))?;
        Ok(SplitTemplateRenderer::new(
            split_template,
            &app_state.all_templates.handlebars,
            writer.clone(),
        ))
    }

    fn set_current_component(&mut self, component: &str) -> anyhow::Result<()> {
        self.current_component = Some(Self::create_renderer(
            component,
            self.app_state,
            &self.writer,
        )?);
        Ok(())
    }

    fn open_component_with_data<T: Serialize>(&mut self, component: &str, data: &T) -> anyhow::Result<()> {
        self.close_component()?;
        self.set_current_component(component)?;
        self.current_component.as_mut().unwrap().render_start(json!(data))?;
        Ok(())
    }

    fn close_component(&mut self) -> anyhow::Result<()> {
        if let Some(component) = &mut self.current_component {
            component.render_end()?;
        }
        Ok(())
    }
}

impl Drop for RenderContext<'_> {
    fn drop(&mut self) {
        if let Some(mut component) = self.current_component.take() {
            self.handle_result_and_log(&component.render_end());
        }
        let res = self.shell_renderer.render_end();
        self.handle_result_and_log(&res);
    }
}

struct SerializeRow<R: Row>(R);

impl<'r, R: Row> Serialize for &'r SerializeRow<R>
    where
        usize: sqlx::ColumnIndex<R>,
        &'r str: sqlx::Decode<'r, <R as Row>::Database>,
        f64: sqlx::Decode<'r, <R as Row>::Database>,
        i64: sqlx::Decode<'r, <R as Row>::Database>,
        bool: sqlx::Decode<'r, <R as Row>::Database>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        use sqlx::{TypeInfo, ValueRef};
        let columns = self.0.columns();
        let mut map = serializer.serialize_map(Some(columns.len()))?;
        for col in columns {
            let key = col.name();
            match self.0.try_get_raw(col.ordinal()) {
                Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                    "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                        map_serialize::<_, _, f64>(&mut map, key, raw_value)
                    }
                    "INT" | "INTEGER" | "INT8" | "INT2" | "INT4" | "TINYINT" | "SMALLINT"
                    | "BIGINT" => map_serialize::<_, _, i64>(&mut map, key, raw_value),
                    "BOOL" | "BOOLEAN" => map_serialize::<_, _, bool>(&mut map, key, raw_value),
                    // Deserialize as a string by default
                    _ => map_serialize::<_, _, &str>(&mut map, key, raw_value),
                },
                _ => map.serialize_entry(key, &()), // Serialize null
            }?
        }
        map.end()
    }
}

fn map_serialize<'r, M: SerializeMap, DB: Database, T: Decode<'r, DB> + Serialize>(
    map: &mut M,
    key: &str,
    raw_value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
) -> Result<(), M::Error> {
    let val = T::decode(raw_value).map_err(serde::ser::Error::custom)?;
    map.serialize_entry(key, &val)
}

struct SplitTemplate {
    before_list: Template,
    list_content: Template,
    after_list: Template,
}


impl handlebars::Output for ResponseWriter {
    fn write(&mut self, seg: &str) -> std::io::Result<()> {
        std::io::Write::write_all(self, seg.as_bytes())
    }
}

struct SplitTemplateRenderer<'registry, OUTPUT: handlebars::Output> {
    split_template: &'registry SplitTemplate,
    block_context: Option<BlockContext<'registry>>,
    registry: &'registry Handlebars<'registry>,
    output: OUTPUT,
}

impl<'reg, OUTPUT: handlebars::Output> SplitTemplateRenderer<'reg, OUTPUT> {
    fn new(
        split_template: &'reg SplitTemplate,
        registry: &'reg Handlebars<'reg>,
        output: OUTPUT,
    ) -> Self {
        Self {
            split_template,
            block_context: None,
            registry,
            output,
        }
    }
    fn name(&self) -> &str {
        self.split_template.list_content.name.as_deref().unwrap_or_default()
    }

    fn render_start(&mut self, data: JsonValue) -> Result<(), handlebars::RenderError> {
        let mut render_context = handlebars::RenderContext::new(None);
        let mut ctx = Context::from(data);
        self.split_template.before_list.render(self.registry, &ctx, &mut render_context, &mut self.output)?;
        let mut blk = render_context.block_mut().map(std::mem::take).unwrap_or_default();
        blk.set_base_value(std::mem::take(ctx.data_mut()));
        self.block_context = Some(blk);
        Ok(())
    }

    fn render_item(&mut self, data: JsonValue) -> Result<(), handlebars::RenderError> {
        if let Some(block_context) = self.block_context.take() {
            let mut render_context = handlebars::RenderContext::new(None);
            render_context.push_block(block_context);
            let mut blk = BlockContext::new();
            blk.set_base_value(data);
            render_context.push_block(blk);
            let ctx = Context::null();
            self.split_template.list_content.render(self.registry, &ctx, &mut render_context, &mut self.output)?;
            render_context.pop_block();
            self.block_context = render_context.block_mut().map(std::mem::take);
        }
        Ok(())
    }

    fn render_end(&mut self) -> Result<(), handlebars::RenderError> {
        if let Some(block_context) = self.block_context.take() {
            let mut render_context = handlebars::RenderContext::new(None);
            render_context.push_block(block_context);
            let ctx = Context::null();
            self.split_template.after_list.render(self.registry, &ctx, &mut render_context, &mut self.output)?;
        }
        Ok(())
    }
}

fn split_template(mut original: Template) -> SplitTemplate {
    let mut elements_after = Vec::new();
    let mut mapping_after = Vec::new();
    let mut items_template = None;
    let found = original.elements.iter().position(is_template_list_item);
    if let Some(idx) = found {
        elements_after = original.elements.split_off(idx + 1);
        mapping_after = original.mapping.split_off(idx + 1);
        if let Some(TemplateElement::HelperBlock(tpl)) = original.elements.pop() {
            original.mapping.pop();
            items_template = tpl.template
        }
    }
    let mut list_content = items_template.unwrap_or_default();
    list_content.name = original.name.clone();
    SplitTemplate {
        before_list: Template {
            name: original.name.clone(),
            elements: original.elements,
            mapping: original.mapping,
        },
        list_content,
        after_list: Template {
            name: original.name,
            elements: elements_after,
            mapping: mapping_after,
        },
    }
}

fn is_template_list_item(element: &TemplateElement) -> bool {
    use handlebars::template::*;
    use Parameter::*;
    matches!(element,
                    TemplateElement::HelperBlock(tpl)
                        if matches!(&tpl.name, Name(name) if name == "each_row"))
}

pub struct AllTemplates {
    handlebars: Handlebars<'static>,
    split_templates: HashMap<String, SplitTemplate>,
}

impl AllTemplates {
    pub fn init() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars_helper!(stringify: |v: Json| v.to_string());
        handlebars.register_helper("stringify", Box::new(stringify));
        handlebars_helper!(default: |a: Json, b:Json| if a.is_null() {b} else {a}.clone());
        handlebars.register_helper("default", Box::new(default));
        handlebars_helper!(entries: |v: Json | match v {
            serde_json::value::Value::Object(map) =>
                map.into_iter()
                    .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
                    .collect(),
            serde_json::value::Value::Array(values) =>
                values.iter()
                    .enumerate()
                    .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
                    .collect(),
            _ => vec![]
        });
        handlebars.register_helper("entries", Box::new(entries));
        let split_templates = HashMap::new();
        let mut this = Self { handlebars, split_templates };
        this.register_split("shell", include_str!("../sqlsite/templates/shell.handlebars"))
            .expect("Embedded shell template contains an error");
        this.register_split("error", include_str!("../sqlsite/templates/error.handlebars"))
            .expect("Embedded shell template contains an error");
        this.register_dir();
        this
    }

    fn register_split<S: ToString>(&mut self, name: S, tpl_str: &str) -> Result<(), TemplateError> {
        let mut tpl = Template::compile(tpl_str)?;
        tpl.name = Some(name.to_string());
        let split = split_template(tpl);
        self.split_templates.insert(name.to_string(), split);
        Ok(())
    }

    fn register_dir(&mut self) {
        let mut errors = vec![];
        match std::fs::read_dir(TEMPLATES_DIR) {
            Ok(dir) => {
                for f in dir {
                    errors.extend(self.register_dir_entry(f).err());
                }
            }
            Err(e) => errors.push(Box::new(e)),
        }
        for err in errors {
            log::error!("Unable to register a template: {}", err);
        }
    }

    fn register_dir_entry(
        &mut self,
        entry: std::io::Result<DirEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = entry?.path();
        if matches!(path.extension(), Some(x) if x == "handlebars") {
            let tpl_str = std::fs::read_to_string(&path)?;
            let name = path.file_stem().unwrap().to_string_lossy();
            self.register_split(&name, &tpl_str)?;
        }
        Ok(())
    }
}

#[test]
fn test_split_template() {
    let template = Template::compile(
        "Hello {{name}} ! \
        {{#each_row}}<li>{{this}}</li>{{/each_row}}\
        end",
    ).unwrap();
    let split = split_template(template);
    assert_eq!(split.before_list.elements, Template::compile("Hello {{name}} ! ").unwrap().elements);
    assert_eq!(split.list_content.elements, Template::compile("<li>{{this}}</li>").unwrap().elements);
    assert_eq!(split.after_list.elements, Template::compile("end").unwrap().elements);
}

#[test]
fn test_split_template_render() -> anyhow::Result<()> {
    let reg = Handlebars::new();
    let template = Template::compile(
        "Hello {{name}} !\
        {{#each_row}} ({{x}} : {{../name}}) {{/each_row}}\
        Goodbye {{name}}",
    )?;
    let split = split_template(template);
    let mut rdr = SplitTemplateRenderer::new(
        &split,
        &reg,
        handlebars::StringOutput::new(),
    );
    rdr.render_start(json!({"name": "SQL"}))?;
    rdr.render_item(json!({"x": 1}))?;
    rdr.render_item(json!({"x": 2}))?;
    rdr.render_end()?;
    assert_eq!(rdr.output.into_string()?,
               "Hello SQL ! (1 : SQL)  (2 : SQL) Goodbye SQL");
    Ok(())
}