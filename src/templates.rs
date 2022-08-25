use crate::TEMPLATES_DIR;
use handlebars::{
    handlebars_helper, template::TemplateElement, Context, Handlebars, JsonValue, RenderError,
    Renderable, Template, TemplateError,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;

pub(crate) const DELAYED_CONTENTS: &str = "_delayed_contents";

pub struct SplitTemplate {
    pub before_list: Template,
    pub list_content: Template,
    pub after_list: Template,
}

pub fn split_template(mut original: Template) -> SplitTemplate {
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
    pub handlebars: Handlebars<'static>,
    pub split_templates: HashMap<String, SplitTemplate>,
}

fn without_top_block<'a, 'reg, 'rc, R>(
    rc: &'a mut handlebars::RenderContext<'reg, 'rc>,
    action: impl FnOnce(&mut handlebars::RenderContext<'reg, 'rc>) -> R,
) -> R {
    let top = rc.block_mut().map(std::mem::take).unwrap_or_default();
    rc.pop_block();
    let r = action(rc);
    rc.push_block(top);
    r
}

fn delay_helper<'reg, 'rc>(
    h: &handlebars::Helper<'reg, 'rc>,
    r: &'reg Handlebars<'reg>,
    ctx: &'rc Context,
    rc: &mut handlebars::RenderContext<'reg, 'rc>,
    _out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let inner = h
        .template()
        .ok_or_else(|| RenderError::new("missing delayed contents"))?;
    let mut str_out = handlebars::StringOutput::new();
    inner.render(r, ctx, rc, &mut str_out)?;
    without_top_block(rc, move |rc| {
        let block = rc
            .block_mut()
            .ok_or_else(|| RenderError::new("Cannot use delayed output without a block context"))?;
        let old_delayed_render = block
            .get_local_var(DELAYED_CONTENTS)
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let delayed_render = str_out.into_string()? + old_delayed_render;
        block.set_local_var(DELAYED_CONTENTS, JsonValue::String(delayed_render));
        Ok::<_, RenderError>(())
    })?;
    Ok(())
}

fn flush_delayed_helper<'reg, 'rc>(
    h: &handlebars::Helper<'reg, 'rc>,
    r: &'reg Handlebars<'reg>,
    ctx: &'rc Context,
    rc: &mut handlebars::RenderContext<'reg, 'rc>,
    writer: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(block_context) = rc.block_mut() {
        let delayed = block_context
            .get_local_var(DELAYED_CONTENTS)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        if let Some(contents) = delayed {
            writer.write(contents)?;
            block_context.set_local_var(DELAYED_CONTENTS, JsonValue::Null);
            Ok(())
        } else {
            without_top_block(rc, |rc| flush_delayed_helper(h, r, ctx, rc, writer))
        }
    } else {
        Ok(())
    }
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
        handlebars.register_helper("delay", Box::new(delay_helper));
        handlebars.register_helper("flush_delayed", Box::new(flush_delayed_helper));
        let split_templates = HashMap::new();
        let mut this = Self {
            handlebars,
            split_templates,
        };
        this.register_split(
            "shell",
            include_str!("../sqlsite/templates/shell.handlebars"),
        )
        .expect("Embedded shell template contains an error");
        this.register_split(
            "error",
            include_str!("../sqlsite/templates/error.handlebars"),
        )
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
        let tpl_str = std::fs::read_to_string(&path)?;
        let name = path.file_stem().unwrap().to_string_lossy();
        if path.extension() == Some(OsStr::new("handlebars")) {
            self.register_split(&name, &tpl_str)?;
        } else {
            self.handlebars.register_partial(&name, &tpl_str)?;
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
    )
    .unwrap();
    let split = split_template(template);
    assert_eq!(
        split.before_list.elements,
        Template::compile("Hello {{name}} ! ").unwrap().elements
    );
    assert_eq!(
        split.list_content.elements,
        Template::compile("<li>{{this}}</li>").unwrap().elements
    );
    assert_eq!(
        split.after_list.elements,
        Template::compile("end").unwrap().elements
    );
}
