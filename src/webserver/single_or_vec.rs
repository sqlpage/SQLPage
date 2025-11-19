use serde::de::Error;
use std::borrow::Cow;
use std::mem;

#[derive(Debug, serde::Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum SingleOrVec {
    Single(String),
    Vec(Vec<String>),
}

impl<'de> serde::Deserialize<'de> for SingleOrVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(SingleOrVec::Single(s)),
            serde_json::Value::Array(values) => {
                let mut strings = Vec::with_capacity(values.len());
                for (idx, item) in values.into_iter().enumerate() {
                    match item {
                        serde_json::Value::String(s) => strings.push(s),
                        other => {
                            return Err(D::Error::custom(format!(
                                "expected an array of strings, but item at index {idx} is {other}"
                            )))
                        }
                    }
                }
                Ok(SingleOrVec::Vec(strings))
            }
            other => Err(D::Error::custom(format!(
                "expected a string or an array of strings, but found {other}"
            ))),
        }
    }
}

impl std::fmt::Display for SingleOrVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleOrVec::Single(x) => write!(f, "{x}"),
            SingleOrVec::Vec(v) => {
                write!(f, "[")?;
                let mut it = v.iter();
                if let Some(first) = it.next() {
                    write!(f, "{first}")?;
                }
                for item in it {
                    write!(f, ", {item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

impl SingleOrVec {
    pub(crate) fn merge(&mut self, other: Self) {
        match (self, other) {
            (Self::Single(old), Self::Single(new)) => *old = new,
            (old, mut new) => {
                let mut v = old.take_vec();
                v.extend_from_slice(&new.take_vec());
                *old = Self::Vec(v);
            }
        }
    }

    fn take_vec(&mut self) -> Vec<String> {
        match self {
            SingleOrVec::Single(x) => vec![mem::take(x)],
            SingleOrVec::Vec(v) => mem::take(v),
        }
    }

    #[must_use]
    pub fn as_json_str(&self) -> Cow<'_, str> {
        match self {
            SingleOrVec::Single(x) => Cow::Borrowed(x),
            SingleOrVec::Vec(v) => Cow::Owned(serde_json::to_string(v).unwrap()),
        }
    }
}

#[cfg(test)]
mod single_or_vec_tests {
    use super::SingleOrVec;

    #[test]
    fn deserializes_string_and_array_values() {
        let single: SingleOrVec = serde_json::from_str(r#""hello""#).unwrap();
        assert_eq!(single, SingleOrVec::Single("hello".to_string()));
        let array: SingleOrVec = serde_json::from_str(r#"["a","b"]"#).unwrap();
        assert_eq!(
            array,
            SingleOrVec::Vec(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn rejects_non_string_items() {
        let err = serde_json::from_str::<SingleOrVec>(r#"["a", 1]"#).unwrap_err();
        assert!(
            err.to_string()
                .contains("expected an array of strings, but item at index 1 is 1"),
            "{err}"
        );
    }

    #[test]
    fn displays_single_value() {
        let single = SingleOrVec::Single("hello".to_string());
        assert_eq!(single.to_string(), "hello");
    }

    #[test]
    fn displays_array_values() {
        let array = SingleOrVec::Vec(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(array.to_string(), "[a, b]");
    }
}
