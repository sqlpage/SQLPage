use anyhow::{self, Context as _};
use serde_json::Value as JsonValue;

use crate::webserver::database::DbItem;

const MAX_RECURSION_DEPTH: u8 = 127;

/// the raw query results can include (potentially nested) rows with a 'component' column that has the value 'dynamic'.
/// in that case we need to parse the JSON in the 'properties' column, and emit a row for each value in the resulting json array.
#[must_use] pub fn parse_dynamic_rows(db_item: DbItem) -> Box<dyn Iterator<Item = DbItem>> {
    if let DbItem::Row(row) = db_item {
        parse_dynamic_rows_json(row, 0)
    } else {
        Box::new(std::iter::once(db_item))
    }
}

fn parse_dynamic_rows_json(mut row: JsonValue, depth: u8) -> Box<dyn Iterator<Item = DbItem>> {
    if depth >= MAX_RECURSION_DEPTH {
        return Box::new(std::iter::once(DbItem::Error(anyhow::anyhow!(
            "Too many nested dynamic components: \n\
            The 'dynamic' component can be used to render another 'dynamic' component, \
            but the recursion cannot exceed {depth} layers."
        ))));
    }
    if let Some(properties) = extract_dynamic_properties(&mut row) {
        match dynamic_properties_to_iter(properties) {
            Ok(iter) => Box::new(iter.flat_map(move |v| parse_dynamic_rows_json(v, depth + 1))),
            Err(e) => Box::new(std::iter::once(DbItem::Error(e))),
        }
    } else {
        Box::new(std::iter::once(DbItem::Row(row)))
    }
}

/// if row.component == 'dynamic', return Some(row.properties), otherwise return None
fn extract_dynamic_properties(data: &mut JsonValue) -> Option<JsonValue> {
    let component = data.get("component").and_then(|v| v.as_str());
    if component == Some("dynamic") {
        let properties = data.get_mut("properties").map(JsonValue::take);
        Some(properties.unwrap_or_default())
    } else {
        None
    }
}

fn dynamic_properties_to_iter(
    mut properties_obj: JsonValue,
) -> anyhow::Result<Box<dyn Iterator<Item = JsonValue>>> {
    if let JsonValue::String(s) = properties_obj {
        properties_obj = serde_json::from_str::<JsonValue>(&s).with_context(|| {
            format!(
                "Unable to parse the 'properties' property of the dynamic component as JSON.\n\
                    Invalid json: {s}"
            )
        })?;
    }
    match properties_obj {
        obj @ JsonValue::Object(_) => Ok(Box::new(std::iter::once(obj))),
        JsonValue::Array(values) => Ok(Box::new(values.into_iter())),
        other => anyhow::bail!(
            "Dynamic component expected properties of type array or object, got {other} instead."
        ),
    }
}
