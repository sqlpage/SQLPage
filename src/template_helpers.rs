use std::borrow::Cow;

use anyhow::Context as _;
use handlebars::{
    handlebars_helper, Context, Handlebars, PathAndJson, RenderError, RenderErrorReason,
    Renderable, ScopedJson,
};
use serde_json::Value as JsonValue;

use crate::utils::static_filename;

/// Simple json to json helper
type H = fn(&JsonValue) -> JsonValue;
/// Simple json to json helper with error handling
type EH = fn(&JsonValue) -> anyhow::Result<JsonValue>;
/// Helper that takes two arguments
type HH = fn(&JsonValue, &JsonValue) -> JsonValue;

pub fn register_all_helpers(h: &mut Handlebars<'_>) {
    register_helper(h, "stringify", stringify_helper as H);
    register_helper(h, "parse_json", parse_json_helper as EH);
    register_helper(h, "default", default_helper as HH);
    register_helper(h, "entries", entries_helper as H);
    // delay helper: store a piece of information in memory that can be output later with flush_delayed
    h.register_helper("delay", Box::new(delay_helper));
    h.register_helper("flush_delayed", Box::new(flush_delayed_helper));
    register_helper(h, "plus", plus_helper as HH);
    register_helper(h, "minus", minus_helper as HH);
    h.register_helper("sum", Box::new(sum_helper));
    register_helper(h, "starts_with", starts_with_helper as HH);

    // to_array: convert a value to a single-element array. If the value is already an array, return it as-is.
    register_helper(h, "to_array", to_array_helper as H);

    // array_contains: check if an array contains an element. If the first argument is not an array, it is compared to the second argument.
    handlebars_helper!(array_contains: |array: Json, element: Json| match array {
        JsonValue::Array(arr) => arr.contains(element),
        other => other == element
    });
    h.register_helper("array_contains", Box::new(array_contains));

    // static_path helper: generate a path to a static file. Replaces sqpage.js by sqlpage.<hash>.js
    register_helper(h, "static_path", static_path_helper as EH);

    // icon helper: generate an image with the specified icon
    h.register_helper("icon_img", Box::new(icon_img_helper));
    register_helper(h, "markdown", markdown_helper as EH);
    register_helper(h, "buildinfo", buildinfo_helper as EH);
    register_helper(h, "typeof", typeof_helper as H);
    register_helper(h, "rfc2822_date", rfc2822_date_helper as EH);
}

fn stringify_helper(v: &JsonValue) -> JsonValue {
    v.to_string().into()
}

fn parse_json_helper(v: &JsonValue) -> Result<JsonValue, anyhow::Error> {
    Ok(match v {
        serde_json::value::Value::String(s) => serde_json::from_str(s)?,
        other => other.clone(),
    })
}

fn default_helper(v: &JsonValue, default: &JsonValue) -> JsonValue {
    if v.is_null() {
        default.clone()
    } else {
        v.clone()
    }
}

fn plus_helper(a: &JsonValue, b: &JsonValue) -> JsonValue {
    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
        (a + b).into()
    } else if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        (a + b).into()
    } else {
        JsonValue::Null
    }
}

fn minus_helper(a: &JsonValue, b: &JsonValue) -> JsonValue {
    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
        (a - b).into()
    } else if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        (a - b).into()
    } else {
        JsonValue::Null
    }
}

fn starts_with_helper(a: &JsonValue, b: &JsonValue) -> JsonValue {
    if let (Some(a), Some(b)) = (a.as_str(), b.as_str()) {
        a.starts_with(b)
    } else if let (Some(arr1), Some(arr2)) = (a.as_array(), b.as_array()) {
        arr1.starts_with(arr2)
    } else {
        false
    }
    .into()
}
fn entries_helper(v: &JsonValue) -> JsonValue {
    match v {
        serde_json::value::Value::Object(map) => map
            .into_iter()
            .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
            .collect(),
        serde_json::value::Value::Array(values) => values
            .iter()
            .enumerate()
            .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
            .collect(),
        _ => vec![],
    }
    .into()
}

fn to_array_helper(v: &JsonValue) -> JsonValue {
    match v {
        JsonValue::Array(arr) => arr.clone(),
        JsonValue::Null => vec![],
        JsonValue::String(s) if s.starts_with('[') => {
            if let Ok(JsonValue::Array(r)) = serde_json::from_str(s) {
                r
            } else {
                vec![JsonValue::String(s.clone())]
            }
        }
        other => vec![other.clone()],
    }
    .into()
}

fn static_path_helper(v: &JsonValue) -> anyhow::Result<JsonValue> {
    match v.as_str().with_context(|| "static_path: not a string")? {
        "sqlpage.js" => Ok(static_filename!("sqlpage.js").into()),
        "sqlpage.css" => Ok(static_filename!("sqlpage.css").into()),
        "apexcharts.js" => Ok(static_filename!("apexcharts.js").into()),
        other => Err(anyhow::anyhow!("unknown static file: {other:?}")),
    }
}

fn typeof_helper(v: &JsonValue) -> JsonValue {
    match v {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "boolean",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
    .into()
}

fn markdown_helper(x: &JsonValue) -> anyhow::Result<JsonValue> {
    let as_str = match x {
        JsonValue::String(s) => Cow::Borrowed(s),
        JsonValue::Array(arr) => Cow::Owned(
            arr.iter()
                .map(|v| v.as_str().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        JsonValue::Null => Cow::Owned(String::new()),
        other => Cow::Owned(other.to_string()),
    };
    markdown::to_html_with_options(&as_str, &markdown::Options::gfm())
        .map(JsonValue::String)
        .map_err(|e| anyhow::anyhow!("markdown error: {e}"))
}

fn buildinfo_helper(x: &JsonValue) -> anyhow::Result<JsonValue> {
    match x {
        JsonValue::String(s) if s == "CARGO_PKG_NAME" => Ok(env!("CARGO_PKG_NAME").into()),
        JsonValue::String(s) if s == "CARGO_PKG_VERSION" => Ok(env!("CARGO_PKG_VERSION").into()),
        other => Err(anyhow::anyhow!("unknown buildinfo key: {other:?}")),
    }
}

// rfc2822_date: take an ISO date and convert it to an RFC 2822 date
fn rfc2822_date_helper(v: &JsonValue) -> anyhow::Result<JsonValue> {
    let date: chrono::DateTime<chrono::FixedOffset> = match v {
        JsonValue::String(s) => {
            // we accept both dates with and without time
            chrono::DateTime::parse_from_rfc3339(s)
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset())
                })
                .with_context(|| format!("invalid date: {s}"))?
        }
        JsonValue::Number(n) => {
            chrono::DateTime::from_timestamp(n.as_i64().with_context(|| "not a timestamp")?, 0)
                .with_context(|| "invalid timestamp")?
                .into()
        }
        other => anyhow::bail!("expected a date, got {other:?}"),
    };
    // format: Thu, 01 Jan 1970 00:00:00 +0000
    Ok(date.format("%a, %d %b %Y %T %z").to_string().into())
}

fn with_each_block<'a, 'reg, 'rc>(
    rc: &'a mut handlebars::RenderContext<'reg, 'rc>,
    mut action: impl FnMut(&mut handlebars::BlockContext<'rc>, bool) -> Result<(), RenderError>,
) -> Result<(), RenderError> {
    let mut blks = Vec::new();
    while let Some(mut top) = rc.block_mut().map(std::mem::take) {
        rc.pop_block();
        action(&mut top, rc.block().is_none())?;
        blks.push(top);
    }
    while let Some(blk) = blks.pop() {
        rc.push_block(blk);
    }
    Ok(())
}

pub(crate) const DELAYED_CONTENTS: &str = "_delayed_contents";

fn delay_helper<'reg, 'rc>(
    h: &handlebars::Helper<'rc>,
    r: &'reg Handlebars<'reg>,
    ctx: &'rc Context,
    rc: &mut handlebars::RenderContext<'reg, 'rc>,
    _out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let inner = h
        .template()
        .ok_or(RenderErrorReason::BlockContentRequired)?;
    let mut str_out = handlebars::StringOutput::new();
    inner.render(r, ctx, rc, &mut str_out)?;
    let mut delayed_render = str_out.into_string()?;
    with_each_block(rc, |block, is_last| {
        if is_last {
            let old_delayed_render = block
                .get_local_var(DELAYED_CONTENTS)
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            delayed_render += old_delayed_render;
            let contents = JsonValue::String(std::mem::take(&mut delayed_render));
            block.set_local_var(DELAYED_CONTENTS, contents);
        }
        Ok(())
    })?;
    Ok(())
}

fn flush_delayed_helper<'reg, 'rc>(
    _h: &handlebars::Helper<'rc>,
    _r: &'reg Handlebars<'reg>,
    _ctx: &'rc Context,
    rc: &mut handlebars::RenderContext<'reg, 'rc>,
    writer: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    with_each_block(rc, |block_context, _last| {
        let delayed = block_context
            .get_local_var(DELAYED_CONTENTS)
            .and_then(JsonValue::as_str)
            .filter(|s| !s.is_empty());
        if let Some(contents) = delayed {
            writer.write(contents)?;
            block_context.set_local_var(DELAYED_CONTENTS, JsonValue::Null);
        }
        Ok(())
    })
}

fn sum_helper<'reg, 'rc>(
    helper: &handlebars::Helper<'rc>,
    _r: &'reg Handlebars<'reg>,
    _ctx: &'rc Context,
    _rc: &mut handlebars::RenderContext<'reg, 'rc>,
    writer: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let mut sum = 0f64;
    for v in helper.params() {
        sum += v
            .value()
            .as_f64()
            .ok_or(RenderErrorReason::InvalidParamType("number"))?;
    }
    write!(writer, "{sum}")?;
    Ok(())
}

fn icon_img_helper<'reg, 'rc>(
    helper: &handlebars::Helper<'rc>,
    _r: &'reg Handlebars<'reg>,
    _ctx: &'rc Context,
    _rc: &mut handlebars::RenderContext<'reg, 'rc>,
    writer: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let null = handlebars::JsonValue::Null;
    let params = [0, 1].map(|i| helper.params().get(i).map_or(&null, PathAndJson::value));
    let name = match params[0] {
        JsonValue::String(s) => s,
        other => {
            log::debug!("icon_img: {other:?} is not an icon name, not rendering anything");
            return Ok(());
        }
    };
    let size = params[1].as_u64().unwrap_or(24);
    write!(
        writer,
        "<svg width={size} height={size}><use href=\"/{}#tabler-{name}\" /></svg>",
        static_filename!("tabler-icons.svg")
    )?;
    Ok(())
}

trait CanHelp: Send + Sync + 'static {
    fn call(&self, v: &[PathAndJson]) -> Result<JsonValue, String>;
}

impl CanHelp for H {
    fn call(&self, args: &[PathAndJson]) -> Result<JsonValue, String> {
        match args {
            [v] => Ok(self(v.value())),
            _ => Err("expected one argument".to_string()),
        }
    }
}

impl CanHelp for EH {
    fn call(&self, args: &[PathAndJson]) -> Result<JsonValue, String> {
        match args {
            [v] => self(v.value()).map_err(|e| e.to_string()),
            _ => Err("expected one argument".to_string()),
        }
    }
}

impl CanHelp for HH {
    fn call(&self, args: &[PathAndJson]) -> Result<JsonValue, String> {
        match args {
            [a, b] => Ok(self(a.value(), b.value())),
            _ => Err("expected two arguments".to_string()),
        }
    }
}

struct JFun<F: CanHelp> {
    name: &'static str,
    fun: F,
}
impl<F: CanHelp> handlebars::HelperDef for JFun<F> {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &handlebars::Helper<'rc>,
        _r: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let result = self
            .fun
            .call(helper.params().as_slice())
            .map_err(|s| RenderErrorReason::Other(format!("{}: {}", self.name, s)))?;
        Ok(ScopedJson::Derived(result))
    }
}

fn register_helper(h: &mut Handlebars, name: &'static str, fun: impl CanHelp) {
    h.register_helper(name, Box::new(JFun { name, fun }));
}

#[test]
fn test_rfc2822_date() {
    assert_eq!(
        rfc2822_date_helper(&JsonValue::String("1970-01-02T03:04:05+02:00".into()))
            .unwrap()
            .as_str()
            .unwrap(),
        "Fri, 02 Jan 1970 03:04:05 +0200"
    );
    assert_eq!(
        rfc2822_date_helper(&JsonValue::String("1970-01-02".into()))
            .unwrap()
            .as_str()
            .unwrap(),
        "Fri, 02 Jan 1970 00:00:00 +0000"
    );
}
