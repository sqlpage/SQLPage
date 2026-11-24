use super::Seed;
use anyhow::{Context, Result};
use schemars::generate::SchemaSettings;
use serde_json::Value;
use sqlpage::app_config::AppConfig;

pub(super) fn insert(seed: &mut Seed) -> Result<()> {
    let schema = SchemaSettings::draft2020_12()
        .with(|settings| settings.inline_subschemas = true)
        .into_generator()
        .into_root_schema_for::<AppConfig>();
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .context("Missing AppConfig properties")?;
    for (name, option) in properties {
        let description = option["description"]
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .with_context(|| format!("{name} needs a Rust doc comment"))?;
        let kind = schema_type(option).with_context(|| format!("Documenting {name}"))?;
        let default = option["x-default-description"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| {
                option.get("default").map(|value| {
                    let value = value.to_string();
                    let ticks = value.split(|c| c != '`').map(str::len).max().unwrap_or(0);
                    let delimiter = "`".repeat(ticks + 1);
                    format!("{delimiter}{value}{delimiter}")
                })
            });
        seed.row(
            "INSERT INTO config_option (name, type, default_md, description_md) VALUES (?, ?, ?, ?)",
            &[&name, &kind, &default, &description],
        )?;
    }
    Ok(())
}

fn schema_type(schema: &Value) -> Result<String> {
    if let Some(values) = schema["enum"].as_array() {
        return Ok(values
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join(" | "));
    }
    if let Some(types) = schema["anyOf"].as_array() {
        return Ok(types
            .iter()
            .map(schema_type)
            .collect::<Result<Vec<_>>>()?
            .join(" or "));
    }
    if let Some(types) = schema["type"].as_array() {
        return Ok(types
            .iter()
            .map(|kind| format_type(schema, kind))
            .collect::<Result<Vec<_>>>()?
            .join(" or "));
    }
    format_type(schema, &schema["type"])
}

fn format_type(schema: &Value, kind: &Value) -> Result<String> {
    let kind = kind.as_str().context("Missing configuration option type")?;
    Ok(match kind {
        "array" => format!("{}[]", schema_type(&schema["items"])?),
        "integer" | "number" => match (schema.get("minimum"), schema.get("maximum")) {
            (Some(min), Some(max)) => format!("{kind} ({min} to {max})"),
            (Some(min), None) => format!("{kind} (≥ {min})"),
            (None, Some(max)) => format!("{kind} (≤ {max})"),
            (None, None) => kind.to_owned(),
        },
        _ => kind.to_owned(),
    })
}
