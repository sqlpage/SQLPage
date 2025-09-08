use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;

pub struct URLParameters(String);

impl URLParameters {
    fn encode_and_push(&mut self, v: &str) {
        let val: Cow<str> = percent_encode(v.as_bytes(), NON_ALPHANUMERIC).into();
        self.0.push_str(&val);
    }
    fn push_kv(&mut self, key: &str, value: &str) {
        if !self.0.is_empty() {
            self.0.push('&');
        }
        self.encode_and_push(key);
        self.0.push('=');
        self.encode_and_push(value);
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for URLParameters {
    fn deserialize<D>(deserializer: D) -> Result<URLParameters, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Visit an object and append keys and values to the string
        struct URLParametersVisitor;

        impl<'de> serde::de::Visitor<'de> for URLParametersVisitor {
            type Value = URLParameters;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_map<A>(self, mut map: A) -> Result<URLParameters, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut out = URLParameters(String::new());
                while let Some((key, value)) =
                    map.next_entry::<Cow<str>, Cow<serde_json::value::RawValue>>()?
                {
                    let value = value.get();
                    if let Ok(str_val) = serde_json::from_str::<Option<Cow<str>>>(value) {
                        if let Some(str_val) = str_val {
                            out.push_kv(&key, &str_val);
                        }
                    } else if let Ok(vec_val) =
                        serde_json::from_str::<Vec<serde_json::Value>>(value)
                    {
                        for val in vec_val {
                            if !out.0.is_empty() {
                                out.0.push('&');
                            }
                            out.encode_and_push(&key);
                            out.0.push_str("[]");
                            out.0.push('=');

                            let val = match val {
                                Value::String(s) => s,
                                other => other.to_string(),
                            };
                            out.encode_and_push(&val);
                        }
                    } else {
                        out.push_kv(&key, value);
                    }
                }

                Ok(out)
            }
        }

        deserializer.deserialize_map(URLParametersVisitor)
    }
}

#[test]
fn test_url_parameters_deserializer() {
    use serde_json::json;
    let json = json!({
        "x": "hello world",
        "num": 123,
        "arr": [1, 2, 3],
    });

    let url_parameters: URLParameters = serde_json::from_value(json).unwrap();
    assert_eq!(
        url_parameters.0,
        "x=hello%20world&num=123&arr[]=1&arr[]=2&arr[]=3"
    );
}

#[test]
fn test_url_parameters_null() {
    use serde_json::json;
    let json = json!({
        "null_should_be_omitted": null,
        "x": "hello",
    });

    let url_parameters: URLParameters = serde_json::from_value(json).unwrap();
    assert_eq!(url_parameters.0, "x=hello");
}

#[test]
fn test_url_parameters_deserializer_special_chars() {
    use serde_json::json;
    let json = json!({
        "chars": ["\n", " ", "\""],
    });

    let url_parameters: URLParameters = serde_json::from_value(json).unwrap();
    assert_eq!(url_parameters.0, "chars[]=%0A&chars[]=%20&chars[]=%22");
}

#[test]
fn test_url_parameters_deserializer_issue_879() {
    use serde_json::json;
    let json = json!({
        "name": "John Doe & Son's",
        "items": [1, "item 2 & 3", true],
        "special_char": "%&=+ ",
    });

    let url_parameters: URLParameters = serde_json::from_value(json).unwrap();
    assert_eq!(
        url_parameters.0,
        "name=John%20Doe%20%26%20Son%27s&items[]=1&items[]=item%202%20%26%203&items[]=true&special%5Fchar=%25%26%3D%2B%20"
    );
}
