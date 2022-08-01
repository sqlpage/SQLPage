use std::fs::DirEntry;
use crate::http::ResponseWriter;
use handlebars::{template::TemplateElement, Template, Handlebars, TemplateError, handlebars_helper};
use sqlx::{Column, Row};
use crate::AppState;
use serde::{Serialize, Serializer};

pub struct RenderContext<'a> {
    app_state: &'a AppState,
    writer: ResponseWriter,
    current_component: Option<String>,
}

const DEFAULT_COMPONENT: &str = "default";

impl RenderContext<'_> {
    pub fn new(app_state: &AppState, writer: ResponseWriter) -> RenderContext {
        let mut this = RenderContext { app_state, writer, current_component: None };
        this.render_template("shell_before");
        this
    }

    pub async fn handle_row(&mut self, row: sqlx::any::AnyRow) -> Result<(), handlebars::RenderError> {
        log::trace!("handle_row: {:?}", row.columns());
        if self.current_component.is_none() {
            let component = row.try_get("component").unwrap_or_else(|_| DEFAULT_COMPONENT.to_string());
            self.open_component(component)
        };
        self.render_current_template_with_data(&&SerializeRow(row));
        Ok(())
    }

    pub async fn finish_query(&mut self, result: sqlx::any::AnyQueryResult) -> Result<(), handlebars::RenderError> {
        log::trace!("finish_query: {:?}", result);
        self.close_component();
        Ok(())
    }

    pub fn handle_error(&mut self, error: &impl std::error::Error) {
        log::warn!("SQL error {}", error);
        if self.current_component.is_some() {
            self.close_component();
        }
        self.open_component("error".to_string());
        self.render_current_template_with_data(&format!("{}", error));
        self.close_component();
    }


    pub fn handle_result<R, E: std::error::Error>(&mut self, result: &Result<R, E>) {
        if let Err(error) = result {
            self.handle_error(error)
        }
    }

    pub async fn close(&mut self) {
        log::warn!("close");
    }


    fn render_template(&mut self, name: &str) {
        self.render_template_with_data(name, &())
    }

    fn render_template_with_data<T: Serialize>(&mut self, name: &str, data: &T) {
        self.handle_result(&self.app_state.all_templates.handlebars.render_to_write(name, data, &self.writer));
    }

    fn render_current_template_with_data<T: Serialize>(&mut self, data: &T) {
        let name = self.current_component.as_ref().unwrap();
        self.handle_result(&self.app_state.all_templates.handlebars.render_to_write(name, data, &self.writer));
    }

    fn open_component(&mut self, component: String) {
        self.render_template(&[&component, "_before"].join(""));
        self.current_component = Some(component);
    }

    fn close_component(&mut self) {
        if let Some(component) = self.current_component.take() {
            self.render_template(&(component + "_after"));
            self.render_template("shell");
        }
    }
}

impl Drop for RenderContext<'_> {
    fn drop(&mut self) {
        if let Some(component) = self.current_component.take() {
            self.render_template(&(component + "_after"));
        }
        self.render_template("shell_after");
    }
}


struct SerializeRow<R: Row>(R);

impl<'r, R: Row> Serialize for &'r SerializeRow<R>
    where usize: sqlx::ColumnIndex<R>,
          &'r str: sqlx::Decode<'r, <R as Row>::Database>,
          f64: sqlx::Decode<'r, <R as Row>::Database>,
          i64: sqlx::Decode<'r, <R as Row>::Database>,
          bool: sqlx::Decode<'r, <R as Row>::Database>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
        use serde::ser::SerializeMap;
        use sqlx::{decode::Decode, TypeInfo, ValueRef};
        use serde::ser::Error;
        let columns = self.0.columns();
        let mut map = serializer.serialize_map(Some(columns.len()))?;
        for col in columns {
            let key = col.name();
            if let Ok(raw_value) = self.0.try_get_raw(col.ordinal()) {
                if raw_value.is_null() {
                    map.serialize_entry(key, &())?;
                    continue;
                }
                match raw_value.type_info().name() {
                    "REAL" | "FLOAT" => {
                        let value: f64 = Decode::decode(raw_value).map_err(Error::custom)?;
                        map.serialize_entry(key, &value)?;
                    }
                    "INT" | "INTEGER" => {
                        let value: i64 = Decode::decode(raw_value).map_err(Error::custom)?;
                        map.serialize_entry(key, &value)?;
                    }
                    "BOOL" | "BOOLEAN" => {
                        let value: bool = Decode::decode(raw_value).map_err(Error::custom)?;
                        map.serialize_entry(key, &value)?;
                    }
                    _ => { // Deserialize as a string by default
                        let value: &str = Decode::decode(raw_value).map_err(Error::custom)?;
                        map.serialize_entry(key, value)?;
                    }
                }
            }
        }
        map.end()
    }
}

struct SplitTemplate {
    before_list: Template,
    list_content: Template,
    after_list: Template,
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
    use handlebars::Path::*;
    use Parameter::*;
    matches!(element,
                    TemplateElement::HelperBlock(tpl)
                        if matches!((&tpl.name, &tpl.params[..]),
                                    (Name(name), [Path(Relative((_, param)))]) if name == "each" && param == "items"))
}


pub struct AllTemplates {
    handlebars: Handlebars<'static>,
}

impl AllTemplates {
    pub fn init() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars_helper!(stringify: |v: Json| v.to_string());
        handlebars.register_helper("stringify", Box::new(stringify));
        let mut this = Self { handlebars };
        this.register_split("shell", include_str!("../templates/shell.handlebars"))
            .expect("Embedded shell template contains an error");
        this.register_split("error", include_str!("../templates/error.handlebars"))
            .expect("Embedded shell template contains an error");
        this.register_dir();
        this
    }

    fn register_split(&mut self, name: &str, tpl_str: &str) -> Result<(), TemplateError> {
        let mut tpl = Template::compile(tpl_str)?;
        tpl.name = Some(name.to_string());
        let split = split_template(tpl);
        self.handlebars.register_template(&[name, "before"].join("_"), split.before_list);
        self.handlebars.register_template(name, split.list_content);
        self.handlebars.register_template(&[name, "after"].join("_"), split.after_list);
        Ok(())
    }

    fn register_dir(&mut self) {
        let mut errors = vec![];
        match std::fs::read_dir("templates") {
            Ok(dir) => {
                for f in dir {
                    errors.extend(self.register_dir_entry(f).err());
                }
            }
            Err(e) => errors.push(Box::new(e))
        }
        for err in errors {
            log::error!("Unable to register a template: {}", err);
        }
    }

    fn register_dir_entry(&mut self, entry: std::io::Result<DirEntry>) -> Result<(), Box<dyn std::error::Error>> {
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
fn test_custom_template() {
    let template = handlebars::Template::compile(
        "
    <h1> 
        Hello {{name}} ! 
    {{#each items}}
        <li>{{this}}</li>
    {{/each}}
    </h1>",
    )
        .unwrap();
    assert_eq!(template.elements, vec![]);
}
