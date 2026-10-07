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

    #[test]
    fn lookup_preserves_null_and_source_policies() {
        let get = param_map([
            ("same".into(), "get".into()),
            ("array[]".into(), "a".into()),
            ("array[]".into(), "b".into()),
        ]);
        let post = param_map([
            ("same".into(), "post".into()),
            ("post_only".into(), "post".into()),
        ]);
        let set = RefCell::new(SetVariablesMap::from([("same".into(), None)]));
        let view = VariableAccess::new(&get, &post, &set);
        assert!(matches!(
            view.lookup("absent", LookupPolicy::GetOnly),
            VariableValue::Missing
        ));
        assert!(matches!(
            view.lookup("same", LookupPolicy::GetOnly),
            VariableValue::Text(Cow::Borrowed("get"))
        ));
        assert!(matches!(
            view.lookup("same", LookupPolicy::SetThenGet),
            VariableValue::Null
        ));
        assert!(matches!(
            view.lookup("same", LookupPolicy::SetThenPost),
            VariableValue::Null
        ));
        assert!(matches!(
            view.lookup("post_only", LookupPolicy::SetThenGet),
            VariableValue::Missing
        ));
        assert!(matches!(
            view.lookup("post_only", LookupPolicy::SetThenPost),
            VariableValue::Text(Cow::Borrowed("post"))
        ));
        let VariableValue::Text(array) = view.lookup("array", LookupPolicy::GetOnly) else {
            panic!("missing array")
        };
        assert_eq!(array, r#"["a","b"]"#);
        set.borrow_mut()
            .insert("same".into(), Some(SingleOrVec::Single("set".into())));
        let VariableValue::Text(value) = view.lookup("same", LookupPolicy::SetThenGet) else {
            panic!("missing SET")
        };
        set.borrow_mut().clear();
        assert_eq!(value, "set");
        assert!(matches!(
            view.lookup("same", LookupPolicy::SetThenGet),
            VariableValue::Text(Cow::Borrowed("get"))
        ));
        assert!(matches!(
            view.lookup("same", LookupPolicy::SetThenPost),
            VariableValue::Text(Cow::Borrowed("post"))
        ));
    }

    #[test]
    fn merged_enumeration_keeps_set_post_get_precedence_and_arrays() {
        let get = param_map([
            ("same".into(), "get".into()),
            ("collision".into(), "get".into()),
            ("get_only".into(), "get".into()),
        ]);
        let post = param_map([
            ("same".into(), "post".into()),
            ("collision".into(), "post".into()),
            ("array[]".into(), "a".into()),
            ("array[]".into(), "b".into()),
        ]);
        let set = RefCell::new(SetVariablesMap::from([
            ("same".into(), None),
            ("set_only".into(), Some(SingleOrVec::Single("set".into()))),
        ]));
        let view = VariableAccess::new(&get, &post, &set);
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            serde_json::json!({
                "same": null,
                "collision": "post",
                "get_only": "get",
                "array": ["a", "b"],
                "set_only": "set"
            })
        );
    }
}
