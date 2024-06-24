use anyhow::{self, Context as _};
use serde_json::Value as JsonValue;

use crate::webserver::database::DbItem;

pub fn parse_dynamic_rows(row: DbItem) -> impl Iterator<Item = DbItem> {
    DynamicComponentIterator {
        stack: vec![],
        db_item: Some(row),
    }
}

struct DynamicComponentIterator {
    stack: Vec<anyhow::Result<JsonValue>>,
    db_item: Option<DbItem>,
}

impl Iterator for DynamicComponentIterator {
    type Item = DbItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(db_item) = self.db_item.take() {
            if let DbItem::Row(mut row) = db_item {
                match extract_dynamic_properties(&mut row) {
                    Ok(None) => {
                        // Most common case: just a regular row. We allocated nothing.
                        return Some(DbItem::Row(row));
                    }
                    Ok(Some(properties)) => {
                        self.stack = dynamic_properties_to_vec(properties);
                    }
                    Err(err) => {
                        return Some(DbItem::Error(err));
                    }
                }
            } else {
                return Some(db_item);
            }
        }
        expand_dynamic_stack(&mut self.stack);
        self.stack.pop().map(|result| match result {
            Ok(row) => DbItem::Row(row),
            Err(err) => DbItem::Error(err),
        })
    }
}

fn expand_dynamic_stack(stack: &mut Vec<anyhow::Result<JsonValue>>) {
    while let Some(mut next) = stack.pop() {
        let next_value = next.as_mut().ok();
        // .and_then(extract_dynamic_properties);
        let dyn_props = if let Some(val) = next_value {
            extract_dynamic_properties(val)
        } else {
            Ok(None)
        };
        match dyn_props {
            Ok(None) => {
                // If the properties are not dynamic, push the row back onto the stack
                stack.push(next);
                // return at the first non-dynamic row
                // we don't support non-dynamic rows after dynamic rows nested in the same array
                return;
            }
            Ok(Some(properties)) => {
                // if the properties contain new (nested) dynamic components, push them onto the stack
                stack.extend(dynamic_properties_to_vec(properties));
            }
            Err(err) => {
                // if an error occurs, push it onto the stack
                stack.push(Err(err));
            }
        }
    }
}

/// if row.component == 'dynamic', return Some(row.properties), otherwise return None
#[inline]
fn extract_dynamic_properties(data: &mut JsonValue) -> anyhow::Result<Option<JsonValue>> {
    let component = data.get("component").and_then(|v| v.as_str());
    if component == Some("dynamic") {
        let Some(properties) = data.get_mut("properties").map(JsonValue::take) else {
            anyhow::bail!(
                "The dynamic component requires a property named \"properties\". \
                Instead, it received the following: {data}"
            );
        };
        Ok(Some(properties))
    } else {
        Ok(None)
    }
}

/// reverse the order of the vec returned by `dynamic_properties_to_result_vec`,
/// and wrap each element in a Result
fn dynamic_properties_to_vec(properties_obj: JsonValue) -> Vec<anyhow::Result<JsonValue>> {
    dynamic_properties_to_result_vec(properties_obj).map_or_else(
        |err| vec![Err(err)],
        |vec| vec.into_iter().rev().map(Ok).collect::<Vec<_>>(),
    )
}

/// if properties is a string, parse it as JSON and return a vec with the parsed value
/// if properties is an array, return it as is
/// if properties is an object, return it as a single element vec
/// otherwise, return an error
fn dynamic_properties_to_result_vec(
    mut properties_obj: JsonValue,
) -> anyhow::Result<Vec<JsonValue>> {
    if let JsonValue::String(s) = properties_obj {
        properties_obj = serde_json::from_str::<JsonValue>(&s)
            .with_context(|| format!("Invalid json in dynamic component properties: {s}"))?;
    }
    match properties_obj {
        obj @ JsonValue::Object(_) => Ok(vec![obj]),
        JsonValue::Array(values) => {
            let mut vec = Vec::with_capacity(values.len());
            for value in values {
                vec.extend_from_slice(&dynamic_properties_to_result_vec(value)?);
            }
            Ok(vec)
        }
        other => anyhow::bail!(
            "Dynamic component expected properties of type array or object, got {other} instead."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_properties_to_result_vec() {
        let mut properties = JsonValue::String(r#"{"a": 1}"#.to_string());
        assert_eq!(
            dynamic_properties_to_result_vec(properties.clone()).unwrap(),
            vec![JsonValue::Object(
                serde_json::from_str(r#"{"a": 1}"#).unwrap()
            )]
        );

        properties = JsonValue::Array(vec![JsonValue::String(r#"{"a": 1}"#.to_string())]);
        assert_eq!(
            dynamic_properties_to_result_vec(properties.clone()).unwrap(),
            vec![serde_json::json!({"a": 1})]
        );

        properties = JsonValue::Object(serde_json::from_str(r#"{"a": 1}"#).unwrap());
        assert_eq!(
            dynamic_properties_to_result_vec(properties.clone()).unwrap(),
            vec![JsonValue::Object(
                serde_json::from_str(r#"{"a": 1}"#).unwrap()
            )]
        );

        properties = JsonValue::Null;
        assert!(dynamic_properties_to_result_vec(properties).is_err());
    }

    #[test]
    fn test_dynamic_properties_to_vec() {
        let properties = JsonValue::String(r#"{"a": 1}"#.to_string());
        assert_eq!(
            dynamic_properties_to_vec(properties.clone())
                .first()
                .unwrap()
                .as_ref()
                .unwrap(),
            &serde_json::json!({"a": 1})
        );
    }

    #[test]
    fn test_parse_dynamic_rows() {
        let row = DbItem::Row(serde_json::json!({
            "component": "dynamic",
            "properties": [
                {"a": 1},
                {"component": "dynamic", "properties": {"nested": 2}},
            ]
        }));
        let iter = parse_dynamic_rows(row)
            .map(|item| match item {
                DbItem::Row(row) => row,
                x => panic!("Expected a row, got {x:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            iter,
            vec![
                serde_json::json!({"a": 1}),
                serde_json::json!({"nested": 2}),
            ]
        );
    }

    #[test]
    fn test_parse_dynamic_array_json_strings() {
        let row = DbItem::Row(serde_json::json!({
            "component": "dynamic",
            "properties": [
                r#"{"a": 1}"#,
                r#"{"b": 2}"#,
            ]
        }));
        let iter = parse_dynamic_rows(row)
            .map(|item| match item {
                DbItem::Row(row) => row,
                x => panic!("Expected a row, got {x:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            iter,
            vec![serde_json::json!({"a": 1}), serde_json::json!({"b": 2}),]
        );
    }
}
