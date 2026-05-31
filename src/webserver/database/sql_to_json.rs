use crate::utils::add_value_to_map;
use crate::webserver::database::blob_to_data_url;
use crate::webserver::database::driver::{DbColumn, DbKind, DbRow, DbValue};
use serde_json::{self, Map, Value};

pub fn row_to_json(row: &DbRow) -> Value {
    let mut map = Map::new();
    for (col, value) in row.columns.iter().zip(row.values.iter()) {
        let key = canonical_col_name(col, row.kind);
        let value = sql_value_to_json(value);
        map = add_value_to_map(map, (key, value));
    }
    Value::Object(map)
}

fn canonical_col_name(col: &DbColumn, kind: DbKind) -> String {
    if matches!(kind, DbKind::Odbc)
        && col
            .name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_')
    {
        col.name.to_ascii_lowercase()
    } else {
        col.name.clone()
    }
}

pub fn sql_value_to_json(value: &DbValue) -> Value {
    match value {
        DbValue::Null => Value::Null,
        DbValue::Integer(i) => (*i).into(),
        DbValue::Real(f) => (*f).into(),
        DbValue::Text(s) => Value::String(s.clone()),
        DbValue::Bytes(bytes) => blob_to_data_url::vec_to_data_uri_value(bytes),
    }
}

pub fn row_to_string(row: &DbRow) -> Option<String> {
    let first = row.values.first()?;
    match sql_value_to_json(first) {
        Value::String(s) => Some(s),
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_columns_become_arrays() {
        let row = DbRow {
            columns: vec![
                DbColumn {
                    name: "value".into(),
                    type_name: None,
                },
                DbColumn {
                    name: "value".into(),
                    type_name: None,
                },
            ],
            values: vec![DbValue::Integer(1), DbValue::Integer(2)],
            kind: DbKind::Sqlite,
        };
        assert_eq!(row_to_json(&row), serde_json::json!({"value": [1, 2]}));
    }

    #[test]
    fn odbc_uppercase_columns_are_lowercased() {
        let row = DbRow {
            columns: vec![DbColumn {
                name: "TITLE_TEXT".into(),
                type_name: None,
            }],
            values: vec![DbValue::Text("hello".into())],
            kind: DbKind::Odbc,
        };
        assert_eq!(row_to_json(&row), serde_json::json!({"title_text": "hello"}));
    }
}
