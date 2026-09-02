use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use crate::{app_config::AppConfig, utils::static_filename};
use anyhow::Context as _;
use handlebars::{
    Context, Handlebars, HelperDef, JsonTruthy, PathAndJson, RenderError, RenderErrorReason,
    Renderable, ScopedJson, handlebars_helper,
};
use serde_json::Value as JsonValue;

/// Simple json to json helper
type H = fn(&JsonValue) -> JsonValue;
/// Simple json to json helper with error handling
type EH = fn(&JsonValue) -> anyhow::Result<JsonValue>;
/// Helper that takes two arguments
type HH = fn(&JsonValue, &JsonValue) -> JsonValue;
/// Helper that takes three arguments
#[allow(clippy::upper_case_acronyms)]
type HHH = fn(&JsonValue, &JsonValue, &JsonValue) -> JsonValue;

pub fn register_all_helpers(h: &mut Handlebars<'_>, config: &AppConfig) {
    let site_prefix = config.site_prefix.clone();

    register_helper(h, "all", HelperCheckTruthy(false));
    register_helper(h, "any", HelperCheckTruthy(true));

    register_helper(h, "stringify", stringify_helper as H);
    register_helper(h, "parse_json", parse_json_helper as EH);
    register_helper(h, "default", default_helper as HH);
    register_helper(h, "entries", entries_helper as H);
    register_helper(h, "replace", replace_helper as HHH);
    // delay helper: store a piece of information in memory that can be output later with flush_delayed
    h.register_helper("delay", Box::new(delay_helper));
    h.register_helper("flush_delayed", Box::new(flush_delayed_helper));
    register_helper(h, "plus", plus_helper as HH);
    register_helper(h, "minus", minus_helper as HH);
    h.register_helper("sum", Box::new(sum_helper));
    register_helper(h, "loose_eq", loose_eq_helper as HH);
    register_helper(h, "starts_with", starts_with_helper as HH);

    // to_array: convert a value to a single-element array. If the value is already an array, return it as-is.
    register_helper(h, "to_array", to_array_helper as H);

    // array_contains: check if an array contains an element. If the first argument is not an array, it is compared to the second argument.
    handlebars_helper!(array_contains: |array: Json, element: Json| match array {
        JsonValue::Array(arr) => arr.contains(element),
        other => other == element
    });
    h.register_helper("array_contains", Box::new(array_contains));

    // array_contains_case_insensitive: check if an array contains an element case-insensitively. If the first argument is not an array, it is compared to the second argument case-insensitively.
    handlebars_helper!(array_contains_case_insensitive: |array: Json, element: Json| {
        match array {
            JsonValue::Array(arr) => arr.iter().any(|v| json_eq_case_insensitive(v, element)),
            other => json_eq_case_insensitive(other, element),
        }
    });
    h.register_helper(
        "array_contains_case_insensitive",
        Box::new(array_contains_case_insensitive),
    );

    // static_path helper: generate a path to a static file. Replaces sqpage.js by sqlpage.<hash>.js
    register_helper(h, "static_path", StaticPathHelper(site_prefix.clone()));
    register_helper(h, "app_config", AppConfigHelper(config.clone()));

    // icon helper: generate an image with the specified icon
    h.register_helper("icon_img", Box::new(IconImgHelper));
    register_helper(h, "markdown", MarkdownHelper::new(config));
    register_helper(h, "buildinfo", buildinfo_helper as EH);
    register_helper(h, "typeof", typeof_helper as H);
    register_helper(h, "rfc2822_date", rfc2822_date_helper as EH);
    register_helper(h, "url_encode", url_encode_helper as H);
    register_helper(h, "csv_escape", csv_escape_helper as HH);
}

fn json_eq_case_insensitive(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::String(a), JsonValue::String(b)) => a.eq_ignore_ascii_case(b),
        _ => a == b,
    }
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
    arithmetic_helper(a, b, i64::checked_add, |a, b| a + b)
}

fn minus_helper(a: &JsonValue, b: &JsonValue) -> JsonValue {
    arithmetic_helper(a, b, i64::checked_sub, |a, b| a - b)
}

/// Integer inputs are kept exact where they fit in an `i64`, and fall back to
/// floating point rather than overflowing.
fn arithmetic_helper(
    a: &JsonValue,
    b: &JsonValue,
    exact: fn(i64, i64) -> Option<i64>,
    approximate: fn(f64, f64) -> f64,
) -> JsonValue {
    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64())
        && let Some(result) = exact(a, b)
    {
        return result.into();
    }
    if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
        return approximate(a, b).into();
    }
    JsonValue::Null
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

/// Generate the full path to a builtin sqlpage asset. Struct Param is the site prefix
struct StaticPathHelper(String);

impl CanHelp for StaticPathHelper {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        let static_file = match args {
            [v] => v.value(),
            _ => return Err("expected one argument".to_string()),
        };
        let name = static_file
            .as_str()
            .ok_or_else(|| format!("static_path: not a string: {static_file}"))?;
        let path = match name {
            "sqlpage.js" => static_filename!("sqlpage.js"),
            "sqlpage.css" => static_filename!("sqlpage.css"),
            "apexcharts.js" => static_filename!("apexcharts.js"),
            "tomselect.js" => static_filename!("tomselect.js"),
            "favicon.svg" => static_filename!("favicon.svg"),
            other => return Err(format!("unknown static file: {other:?}")),
        };
        Ok(format!("{}{}", self.0, path).into())
    }
}

/// Generate the full path to a builtin sqlpage asset. Struct Param is the site prefix
struct AppConfigHelper(AppConfig);

impl CanHelp for AppConfigHelper {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        let static_file = match args {
            [v] => v.value(),
            _ => return Err("expected one argument".to_string()),
        };
        let name = static_file
            .as_str()
            .ok_or_else(|| format!("app_config: not a string: {static_file}"))?;
        match name {
            "max_uploaded_file_size" => Ok(JsonValue::Number(self.0.max_uploaded_file_size.into())),
            "environment" => serde_json::to_value(self.0.environment).map_err(|e| e.to_string()),
            "site_prefix" => Ok(self.0.site_prefix.clone().into()),
            other => Err(format!("unknown app config property: {other:?}")),
        }
    }
}

pub static ICON_MAP: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| include!(concat!(env!("OUT_DIR"), "/icons.rs")).into());

/// Generate an image with the specified icon.
struct IconImgHelper;
impl HelperDef for IconImgHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        helper: &handlebars::Helper<'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        writer: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let null = JsonValue::Null;
        let [name, size] = [0, 1].map(|i| helper.params().get(i).map_or(&null, PathAndJson::value));
        let size = size.as_u64().unwrap_or(24);
        let content = name.as_str().and_then(|name| ICON_MAP.get(name));
        let Some(&inner_content) = content else {
            log::warn!("icon_img: icon {name} not found");
            return Ok(());
        };

        write!(
            writer,
            r#"<svg viewBox="0 0 24 24" width="{size}" height="{size}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{inner_content}</svg>"#
        )?;
        Ok(())
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

pub trait MarkdownConfig {
    fn allow_dangerous_html(&self) -> bool;
    fn allow_dangerous_protocol(&self) -> bool;
}

impl MarkdownConfig for AppConfig {
    fn allow_dangerous_html(&self) -> bool {
        self.markdown_allow_dangerous_html
    }

    fn allow_dangerous_protocol(&self) -> bool {
        self.markdown_allow_dangerous_protocol
    }
}

/// Renders a markdown source string to HTML using the application's markdown configuration.
/// Uses the same GFM options as the `markdown` Handlebars helper with the "default" preset.
pub fn render_markdown_to_html(
    config: &impl MarkdownConfig,
    markdown_src: &str,
) -> Result<String, String> {
    let mut options = markdown::Options::gfm();
    options.compile.allow_dangerous_html = config.allow_dangerous_html();
    options.compile.allow_dangerous_protocol = config.allow_dangerous_protocol();
    options.compile.allow_any_img_src = true;
    markdown::to_html_with_options(markdown_src, &options).map_err(|e| e.to_string())
}

/// Helper to render markdown with configurable options
#[derive(Default)]
struct MarkdownHelper {
    allow_dangerous_html: bool,
    allow_dangerous_protocol: bool,
}

impl MarkdownHelper {
    fn new(config: &impl MarkdownConfig) -> Self {
        Self {
            allow_dangerous_html: config.allow_dangerous_html(),
            allow_dangerous_protocol: config.allow_dangerous_protocol(),
        }
    }

    fn get_preset_options(&self, preset_name: &str) -> Result<markdown::Options, String> {
        let mut options = markdown::Options::gfm();
        options.compile.allow_dangerous_html = self.allow_dangerous_html;
        options.compile.allow_dangerous_protocol = self.allow_dangerous_protocol;
        options.compile.allow_any_img_src = true;

        match preset_name {
            "default" => {}
            "allow_unsafe" => {
                options.compile.allow_dangerous_html = true;
                options.compile.allow_dangerous_protocol = true;
            }
            _ => return Err(format!("unknown markdown preset: {preset_name}")),
        }

        Ok(options)
    }
}

impl CanHelp for MarkdownHelper {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        let (markdown_src_value, preset_name) = match args {
            [v] => (v.value(), "default"),
            [v, preset] => {
                let value = v.value();
                let preset_name_value = preset.value();
                let preset = preset_name_value.as_str()
                    .ok_or_else(|| format!("markdown template helper expects a string as preset name. Got: {preset_name_value}"))?;
                (value, preset)
            }
            _ => return Err("markdown template helper expects one or two arguments".to_string()),
        };
        let markdown_src = match markdown_src_value {
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

        let options = self.get_preset_options(preset_name)?;
        markdown::to_html_with_options(&markdown_src, &options)
            .map(JsonValue::String)
            .map_err(|e| e.to_string())
    }
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

// Percent-encode a string
fn url_encode_helper(v: &JsonValue) -> JsonValue {
    let as_str = match v {
        JsonValue::String(s) => s,
        other => &other.to_string(),
    };
    percent_encoding::percent_encode(as_str.as_bytes(), percent_encoding::NON_ALPHANUMERIC)
        .to_string()
        .into()
}

// Percent-encode a string
fn csv_escape_helper(v: &JsonValue, separator: &JsonValue) -> JsonValue {
    let as_str = match v {
        JsonValue::String(s) => s,
        other => &other.to_string(),
    };
    let separator = separator.as_str().unwrap_or(",");
    if as_str.contains(separator) || as_str.contains('"') || as_str.contains('\n') {
        format!(r#""{}""#, as_str.replace('"', r#""""#)).into()
    } else {
        as_str.to_owned().into()
    }
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

/// Compare two values loosely, i.e. treat all values as strings. (42 == "42")
fn loose_eq_helper(a: &JsonValue, b: &JsonValue) -> JsonValue {
    match (a, b) {
        (JsonValue::String(a), JsonValue::String(b)) => a == b,
        (JsonValue::String(a), non_str) => a == &non_str.to_string(),
        (non_str, JsonValue::String(b)) => &non_str.to_string() == b,
        (a, b) => a == b,
    }
    .into()
}
/// Helper that returns the first argument with the given truthiness, or the last argument if none have it.
/// Equivalent to a && b && c && ... if the truthiness is false,
/// or a || b || c || ... if the truthiness is true.
pub struct HelperCheckTruthy(bool);

impl CanHelp for HelperCheckTruthy {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        for arg in args {
            if arg.value().is_truthy(false) == self.0 {
                return Ok(arg.value().clone());
            }
        }
        if let Some(last) = args.last() {
            Ok(last.value().clone())
        } else {
            Err("expected at least one argument".to_string())
        }
    }
}

trait CanHelp: Send + Sync + 'static {
    fn call(&self, v: &[PathAndJson<'_>]) -> Result<JsonValue, String>;
}

impl CanHelp for H {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        match args {
            [v] => Ok(self(v.value())),
            _ => Err("expected one argument".to_string()),
        }
    }
}

impl CanHelp for EH {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        match args {
            [v] => self(v.value()).map_err(|e| e.to_string()),
            _ => Err("expected one argument".to_string()),
        }
    }
}

impl CanHelp for HH {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        match args {
            [a, b] => Ok(self(a.value(), b.value())),
            _ => Err("expected two arguments".to_string()),
        }
    }
}

impl CanHelp for HHH {
    fn call(&self, args: &[PathAndJson<'_>]) -> Result<JsonValue, String> {
        match args {
            [a, b, c] => Ok(self(a.value(), b.value(), c.value())),
            _ => Err("expected three arguments".to_string()),
        }
    }
}

struct JFun<F: CanHelp> {
    name: &'static str,
    fun: F,
}
impl<F: CanHelp> HelperDef for JFun<F> {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &handlebars::Helper<'rc>,
        _r: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let result = self
            .fun
            .call(helper.params().as_slice())
            .map_err(|s| RenderErrorReason::Other(format!("{}: {}", self.name, s)))?;
        Ok(ScopedJson::Derived(result))
    }
}

fn register_helper(h: &mut Handlebars<'_>, name: &'static str, fun: impl CanHelp) {
    h.register_helper(name, Box::new(JFun { name, fun }));
}

fn replace_helper(text: &JsonValue, original: &JsonValue, replacement: &JsonValue) -> JsonValue {
    let text_str = match text {
        JsonValue::String(s) => s,
        other => &other.to_string(),
    };
    let original_str = match original {
        JsonValue::String(s) => s,
        other => &other.to_string(),
    };
    let replacement_str = match replacement {
        JsonValue::String(s) => s,
        other => &other.to_string(),
    };

    text_str.replace(original_str, replacement_str).into()
}

#[cfg(test)]
mod tests {
    use crate::template_helpers::{CanHelp, MarkdownHelper, rfc2822_date_helper};
    use handlebars::{JsonValue, PathAndJson, ScopedJson};
    use serde_json::Value;

    const CONTENT_KEY: &str = "contents_md";

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

    #[test]
    fn test_basic_gfm_markdown() {
        let helper = MarkdownHelper::default();

        let contents = Value::String("# Heading".to_string());
        let actual = helper.call(&as_args(&contents)).unwrap();

        assert_eq!(Some("<h1>Heading</h1>"), actual.as_str());
    }

    // Optionally allow potentially unsafe html blocks
    // See https://spec.commonmark.org/0.31.2/#html-blocks
    mod markdown_html_blocks {

        use super::*;

        const UNSAFE_MARKUP: &str = "<table><tr><td>";
        const ESCAPED_UNSAFE_MARKUP: &str = "&lt;table&gt;&lt;tr&gt;&lt;td&gt;";
        #[test]
        fn test_html_blocks_with_various_settings() {
            struct TestCase {
                name: &'static str,
                preset: Option<Value>,
                expected_output: Result<&'static str, String>,
            }

            let helper = MarkdownHelper::default();
            let content = contents();

            let test_cases = [
                TestCase {
                    name: "default settings",
                    preset: Some(Value::String("default".to_string())),
                    expected_output: Ok(ESCAPED_UNSAFE_MARKUP),
                },
                TestCase {
                    name: "allow_unsafe preset",
                    preset: Some(Value::String("allow_unsafe".to_string())),
                    expected_output: Ok(UNSAFE_MARKUP),
                },
                TestCase {
                    name: "undefined allow_unsafe",
                    preset: Some(Value::Null),
                    expected_output: Err(
                        "markdown template helper expects a string as preset name. Got: null"
                            .to_string(),
                    ),
                },
                TestCase {
                    name: "allow_unsafe is false",
                    preset: Some(Value::Bool(false)),
                    expected_output: Err(
                        "markdown template helper expects a string as preset name. Got: false"
                            .to_string(),
                    ),
                },
            ];

            for case in test_cases {
                let args = match case.preset {
                    None => &as_args(&content)[..],
                    Some(ref preset) => &as_args_with_unsafe(&content, preset)[..],
                };

                match helper.call(args) {
                    Ok(actual) => assert_eq!(
                        case.expected_output.unwrap(),
                        actual.as_str().unwrap(),
                        "Failed on case: {}",
                        case.name
                    ),
                    Err(e) => assert_eq!(
                        case.expected_output.unwrap_err(),
                        e,
                        "Failed on case: {}",
                        case.name
                    ),
                }
            }
        }

        fn as_args_with_unsafe<'a>(
            contents: &'a Value,
            allow_unsafe: &'a Value,
        ) -> [PathAndJson<'a>; 2] {
            [
                as_helper_arg(CONTENT_KEY, contents),
                as_helper_arg("allow_unsafe", allow_unsafe),
            ]
        }

        fn contents() -> Value {
            Value::String(UNSAFE_MARKUP.to_string())
        }
    }

    fn as_args(contents: &Value) -> [PathAndJson<'_>; 1] {
        [as_helper_arg(CONTENT_KEY, contents)]
    }

    fn as_helper_arg<'a>(path: &'a str, value: &'a Value) -> PathAndJson<'a> {
        let json_context = as_json_context(path, value);
        to_path_and_json(path, json_context)
    }

    fn to_path_and_json<'a>(path: &'a str, value: ScopedJson<'a>) -> PathAndJson<'a> {
        PathAndJson::new(Some(path.to_string()), value)
    }

    fn as_json_context<'a>(path: &'a str, value: &'a Value) -> ScopedJson<'a> {
        ScopedJson::Context(value, vec![path.to_string()])
    }

    mod documented_helpers {
        use crate::app_config::tests::test_config;
        use crate::template_helpers::{
            csv_escape_helper, entries_helper, loose_eq_helper, minus_helper, plus_helper,
            register_all_helpers, starts_with_helper, to_array_helper, url_encode_helper,
        };
        use handlebars::Handlebars;
        use serde_json::json;

        fn render(template: &str) -> String {
            let mut registry = Handlebars::new();
            register_all_helpers(&mut registry, &test_config());
            registry.render_template(template, &json!({})).unwrap()
        }

        #[test]
        fn sum_adds_every_argument() {
            assert_eq!(render("{{sum 1 2 3}}"), "6");
            assert_eq!(render("{{sum 1.5 2.25}}"), "3.75");
        }

        #[test]
        fn all_and_any_short_circuit_on_truthiness() {
            assert_eq!(render("{{all 1 2 3}}"), "3");
            assert_eq!(render("{{all 1 0 3}}"), "0");
            assert_eq!(render("{{any 0 false 7}}"), "7");
            assert_eq!(render("{{any 0 false}}"), "false");
        }

        #[test]
        fn starts_with_distinguishes_internal_columns() {
            assert_eq!(
                starts_with_helper(&json!("_sqlpage_id"), &json!("_sqlpage_")),
                json!(true)
            );
            assert_eq!(
                starts_with_helper(&json!("price"), &json!("_sqlpage_")),
                json!(false)
            );
            assert_eq!(
                starts_with_helper(&json!([1, 2, 3]), &json!([1, 2])),
                json!(true)
            );
        }

        #[test]
        fn plus_and_minus_operate_on_numbers_only() {
            assert_eq!(plus_helper(&json!(2), &json!(3)), json!(5));
            assert_eq!(minus_helper(&json!(7), &json!(2)), json!(5));
            assert_eq!(plus_helper(&json!(2.5), &json!(0.25)), json!(2.75));
            assert_eq!(minus_helper(&json!(2.5), &json!(0.25)), json!(2.25));
            assert_eq!(plus_helper(&json!("a"), &json!(1)), json!(null));
            assert_eq!(minus_helper(&json!("a"), &json!(1)), json!(null));
        }

        #[test]
        fn integer_arithmetic_falls_back_to_floats_instead_of_overflowing() {
            let overflowed = plus_helper(&json!(i64::MAX), &json!(1));
            assert!(
                overflowed.as_f64().is_some_and(|v| v > 9.2e18),
                "{overflowed}"
            );

            let underflowed = minus_helper(&json!(i64::MIN), &json!(1));
            assert!(
                underflowed.as_f64().is_some_and(|v| v < -9.2e18),
                "{underflowed}"
            );
        }

        #[test]
        fn csv_fields_are_quoted_whenever_they_need_to_be() {
            for needs_quoting in ["a,b", "a\"b", "a\nb"] {
                let escaped = csv_escape_helper(&json!(needs_quoting), &json!(","));
                assert!(
                    escaped.as_str().unwrap().starts_with('"'),
                    "{needs_quoting:?} was left unquoted as {escaped}"
                );
            }
            assert_eq!(
                csv_escape_helper(&json!("a\"b"), &json!(",")),
                json!("\"a\"\"b\"")
            );
            assert_eq!(
                csv_escape_helper(&json!("plain"), &json!(",")),
                json!("plain")
            );
            assert_eq!(
                csv_escape_helper(&json!("a;b"), &json!(";")),
                json!("\"a;b\"")
            );
        }

        #[test]
        fn url_encode_percent_encodes_reserved_characters() {
            assert_eq!(
                url_encode_helper(&json!("hello world")),
                json!("hello%20world")
            );
        }

        #[test]
        fn entries_exposes_objects_and_arrays_as_key_value_pairs() {
            assert_eq!(
                entries_helper(&json!({"a": 1})),
                json!([{"key": "a", "value": 1}])
            );
            assert_eq!(
                entries_helper(&json!([10, 20])),
                json!([{"key": 0, "value": 10}, {"key": 1, "value": 20}])
            );
            assert_eq!(entries_helper(&json!("scalar")), json!([]));
        }

        #[test]
        fn to_array_parses_json_list_strings() {
            assert_eq!(to_array_helper(&json!("[1,2]")), json!([1, 2]));
            assert_eq!(to_array_helper(&json!("nope")), json!(["nope"]));
            assert_eq!(to_array_helper(&json!([1, 2])), json!([1, 2]));
            assert_eq!(to_array_helper(&json!(null)), json!([]));
        }

        #[test]
        fn loose_eq_compares_values_as_strings() {
            assert_eq!(loose_eq_helper(&json!(42), &json!("42")), json!(true));
            assert_eq!(loose_eq_helper(&json!("42"), &json!(42)), json!(true));
            assert_eq!(loose_eq_helper(&json!("a"), &json!("a")), json!(true));
            assert_eq!(loose_eq_helper(&json!(1), &json!(2)), json!(false));
        }
    }

    mod app_config_markdown_defaults {
        use crate::app_config::tests::test_config;
        use crate::template_helpers::render_markdown_to_html;

        #[test]
        fn raw_html_is_escaped() {
            let html = render_markdown_to_html(&test_config(), "<table><tr><td>").unwrap();
            assert_eq!(html, "&lt;table&gt;&lt;tr&gt;&lt;td&gt;");
        }

        #[test]
        fn dangerous_protocols_are_stripped() {
            let html =
                render_markdown_to_html(&test_config(), "[click](javascript:alert(1))").unwrap();
            assert_eq!(html, "<p><a href=\"\">click</a></p>");
        }
    }
}
