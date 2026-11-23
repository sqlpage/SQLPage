#![cfg(feature = "config-schema")]

use schemars::schema_for;
use serde::Deserialize;
use serde::de::value::Error as DeError;
use serde::de::{Deserializer, Visitor};
use serde_json::json;
use sqlpage::app_config::AppConfig;
use std::collections::BTreeSet;

#[test]
fn every_public_setting_has_a_description() {
    let schema = schema_for!(AppConfig);
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert_eq!(
        properties.keys().cloned().collect::<BTreeSet<_>>(),
        app_config_option_names()
    );
    for (name, property) in properties {
        assert!(
            property["description"]
                .as_str()
                .is_some_and(|text| !text.trim().is_empty()),
            "{name} needs a Rust doc comment"
        );
    }
}

#[test]
fn schema_documents_input_types_and_effective_defaults() {
    let schema = schema_for!(AppConfig);
    let properties = schema.get("properties").unwrap();
    let timeout = &properties["database_connection_idle_timeout_seconds"];
    assert_eq!(timeout["type"], json!(["number", "null"]));
    assert!(
        timeout["x-default-description"]
            .as_str()
            .unwrap()
            .contains("SQLite")
    );
    assert!(timeout.get("default").is_none());
    assert_eq!(properties["site_prefix"]["default"], "/");
    assert_eq!(properties["smtp_tls_mode"]["default"], "starttls");
    assert_eq!(properties["web_root"]["default"], ".");
    assert_eq!(
        properties["configuration_directory"]["default"],
        "./sqlpage"
    );
    assert_eq!(properties["content_security_policy"]["type"], "string");
    assert!(
        properties["content_security_policy"]["default"]
            .as_str()
            .unwrap()
            .contains("{NONCE}")
    );
    assert!(
        properties["system_root_ca_certificates"]
            .get("default")
            .is_none()
    );
    assert!(properties["database_url"].get("default").is_none());
    assert_eq!(properties["oidc_public_paths"]["default"], json!([]));
    assert!(
        properties["oidc_additional_trusted_audiences"]
            .get("default")
            .is_none()
    );
    assert!(schema.get("required").is_none());
}

fn app_config_option_names() -> BTreeSet<String> {
    let mut collector = OptionNameCollector::default();
    let _ = AppConfig::deserialize(&mut collector);
    collector
        .names
        .iter()
        .map(|name| (*name).to_owned())
        .collect()
}

/// Serde hands the names it expects, renames included, to `deserialize_struct`.
#[derive(Default)]
struct OptionNameCollector {
    names: &'static [&'static str],
}

impl<'de> Deserializer<'de> for &mut OptionNameCollector {
    type Error = DeError;

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.names = fields;
        Err(serde::de::Error::custom("names collected"))
    }

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(serde::de::Error::custom("AppConfig is not a struct"))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map enum
        identifier ignored_any
    }
}
