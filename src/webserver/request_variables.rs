use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry};

use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::webserver::single_or_vec::SingleOrVec;

pub type ParamMap = HashMap<String, SingleOrVec>;
pub type SetVariablesMap = HashMap<String, Option<SingleOrVec>>;

/// Request values stay borrowed; mutable SET values are copied before releasing
/// their `RefCell` borrow. A present SET NULL must suppress request fallbacks.
pub(crate) struct VariableAccess<'a> {
    get: &'a ParamMap,
    post: &'a ParamMap,
    set: &'a RefCell<SetVariablesMap>,
}

#[derive(Clone, Copy)]
pub(crate) enum LookupPolicy {
    GetOnly,
    SetThenGet,
    SetThenPost,
}

pub(crate) enum VariableValue<'a> {
    Missing,
    Null,
    Text(Cow<'a, str>),
}

impl<'a> VariableAccess<'a> {
    pub(crate) fn new(
        get: &'a ParamMap,
        post: &'a ParamMap,
        set: &'a RefCell<SetVariablesMap>,
    ) -> Self {
        Self { get, post, set }
    }

    pub(crate) fn lookup(&self, name: &str, policy: LookupPolicy) -> VariableValue<'a> {
        if !matches!(policy, LookupPolicy::GetOnly)
            && let Some(value) = self.set.borrow().get(name)
        {
            return value.as_ref().map_or(VariableValue::Null, |value| {
                VariableValue::Text(Cow::Owned(value.as_json_str().into_owned()))
            });
        }
        let values = match policy {
            LookupPolicy::SetThenPost => self.post,
            LookupPolicy::GetOnly | LookupPolicy::SetThenGet => self.get,
        };
        if matches!(policy, LookupPolicy::SetThenGet) && self.post.contains_key(name) {
            if values.contains_key(name) {
                log::warn!(
                    "Deprecation warning! There is both a URL parameter named '{name}' and a form field named '{name}'. SQLPage is using the URL parameter for ${name}. Please use :{name} to reference the form field explicitly."
                );
            } else {
                log::warn!(
                    "Deprecation warning! ${name} was used to reference a form field value (a POST variable). This now uses only URL parameters. Please use :{name} instead."
                );
            }
        }
        values.get(name).map_or(VariableValue::Missing, |value| {
            VariableValue::Text(value.as_json_str())
        })
    }
}

/// Serialize the merged SET > POST > GET view without cloning its values.
impl Serialize for VariableAccess<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let set = self.set.borrow();
        let mut map = serializer.serialize_map(None)?;
        for (key, value) in &*set {
            map.serialize_entry(key, value)?;
        }
        for (key, value) in self.post {
            if !set.contains_key(key) {
                map.serialize_entry(key, value)?;
            }
        }
        for (key, value) in self.get {
            if !set.contains_key(key) && !self.post.contains_key(key) {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

pub fn param_map<PAIRS: IntoIterator<Item = (String, String)>>(values: PAIRS) -> ParamMap {
    values
        .into_iter()
        .fold(HashMap::new(), |mut map, (mut k, v)| {
            let entry = if k.ends_with("[]") {
                k.replace_range(k.len() - 2.., "");
                SingleOrVec::Vec(vec![v])
            } else {
                SingleOrVec::Single(v)
            };
            match map.entry(k) {
                Entry::Occupied(mut s) => {
                    SingleOrVec::merge(s.get_mut(), entry);
                }
                Entry::Vacant(v) => {
                    v.insert(entry);
                }
            }
            map
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use LookupPolicy::{GetOnly, SetThenGet, SetThenPost};
    use VariableValue::{Missing, Null, Text};

    #[test]
    fn lookup_distinguishes_null_missing_and_releases_set_borrows() {
        let get = param_map([("value".into(), "get".into())]);
        let post = param_map([("value".into(), "post".into())]);
        for (policy, fallback) in [(GetOnly, "get"), (SetThenGet, "get"), (SetThenPost, "post")] {
            let set = RefCell::new(SetVariablesMap::from([("value".into(), None)]));
            let view = VariableAccess::new(&get, &post, &set);
            assert!(matches!(view.lookup("absent", policy), Missing));
            let get_only = matches!(policy, GetOnly);
            assert_eq!(matches!(view.lookup("value", policy), Null), !get_only);
            for (value, expected) in [
                (SingleOrVec::Single("set".into()), "set"),
                (
                    SingleOrVec::Vec(vec!["a".into(), "b".into()]),
                    r#"["a","b"]"#,
                ),
            ] {
                set.borrow_mut().insert("value".into(), Some(value));
                let Text(actual) = view.lookup("value", policy) else {
                    panic!("missing SET")
                };
                if expected.starts_with('[') {
                    assert_eq!(
                        serde_json::to_value(&view).unwrap()["value"],
                        serde_json::json!(["a", "b"])
                    );
                }
                set.borrow_mut().clear();
                assert_eq!(matches!(actual, Cow::Borrowed(_)), get_only);
                assert_eq!(actual, if get_only { "get" } else { expected });
                assert!(
                    matches!(view.lookup("value", policy), Text(Cow::Borrowed(value)) if value == fallback)
                );
            }
        }
    }
}
