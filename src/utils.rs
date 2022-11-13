use serde_json::{Map, Value};
use std::fmt::Display;

pub fn add_value_to_map(
    mut map: Map<String, Value>,
    (key, value): (String, Value),
) -> Map<String, Value> {
    use serde_json::map::Entry::{Occupied, Vacant};
    use Value::Array;
    match map.entry(key) {
        Vacant(vacant) => {
            vacant.insert(value);
        }
        Occupied(mut old_entry) => {
            let mut new_array = if let Array(v) = value { v } else { vec![value] };
            match old_entry.get_mut() {
                Array(old_array) => old_array.extend(new_array.into_iter()),
                old_scalar => {
                    new_array.insert(0, old_scalar.take());
                    *old_scalar = Array(new_array);
                }
            }
        }
    }
    map
}

pub fn log_error<R, E: Display>(r: &Result<R, E>) {
    if let Err(e) = r {
        log::error!("{e}");
    }
}
