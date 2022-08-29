use serde_json::{Map, Value};

pub fn add_value_to_map(
    mut map: Map<String, Value>,
    (key, value): (String, Value),
) -> Map<String, Value> {
    use serde_json::map::Entry::*;
    use Value::Array;
    match map.entry(key) {
        Vacant(vacant) => {
            vacant.insert(value);
        }
        Occupied(mut old_entry) => match old_entry.get_mut() {
            Array(old_array) => old_array.push(value),
            old_scalar => *old_scalar = Array(vec![old_scalar.take(), value]),
        },
    }
    map
}