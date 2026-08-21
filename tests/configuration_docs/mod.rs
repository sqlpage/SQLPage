use serde::Deserialize;
use serde::de::value::Error as DeError;
use serde::de::{Deserializer, Visitor};
use sqlpage::app_config::AppConfig;
use std::collections::BTreeSet;

#[test]
fn configuration_md_documents_every_option() {
    let documented = documented_option_names();
    let implemented = app_config_option_names();

    let undocumented: Vec<_> = implemented.difference(&documented).collect();
    let unimplemented: Vec<_> = documented.difference(&implemented).collect();

    assert!(
        undocumented.is_empty() && unimplemented.is_empty(),
        "configuration.md and AppConfig disagree.\n\
         Missing from configuration.md: {undocumented:?}\n\
         Documented but not in AppConfig: {unimplemented:?}"
    );
}

fn documented_option_names() -> BTreeSet<String> {
    let table = std::fs::read_to_string("configuration.md").unwrap();
    table
        .lines()
        .filter_map(|line| line.strip_prefix("| `")?.split('`').next())
        .map(str::to_owned)
        .collect()
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
