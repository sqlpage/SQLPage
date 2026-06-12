use std::borrow::Cow;

/// Returns a string containing a JSON-encoded match object, or `null` if no match was found.
/// The match object contains one key per capture group, with the value being the matched text.
/// For named capture groups (`(?<name>pattern)`), the key is the name.
/// For unnamed capture groups (`(pattern)`), the key is the index of the capture group as a string.
pub(super) async fn regex_match<'a>(
    pattern: Cow<'a, str>,
    text: Option<Cow<'a, str>>,
) -> Result<Option<String>, anyhow::Error> {
    use serde::{Serializer, ser::SerializeMap};
    let regex = regex::Regex::new(&pattern)?;
    let Some(text) = text else {
        return Ok(None);
    };
    let Some(match_obj) = regex.captures(&text) else {
        return Ok(None);
    };
    let mut result = Vec::with_capacity(64);
    let mut ser = serde_json::Serializer::new(&mut result);
    let mut map = ser.serialize_map(Some(match_obj.len()))?;
    for (idx, maybe_name) in regex.capture_names().enumerate() {
        if let Some(match_group) = match_obj.get(idx) {
            if let Some(name) = maybe_name {
                map.serialize_entry(name, match_group.as_str())?;
            } else {
                let key = idx.to_string();
                map.serialize_entry(&key, match_group.as_str())?;
            }
        }
    }
    map.end()?;
    Ok(Some(String::from_utf8(result)?))
}

#[tokio::test]
pub(super) async fn regex_match_serializes_named_and_unnamed_groups() {
    use std::borrow::Cow;
    let result = regex_match(
        Cow::Borrowed(r"(?<word>foo)(bar)"),
        Some(Cow::Borrowed("_foobar_")),
    )
    .await
    .unwrap();

    assert_eq!(
        result.as_deref(),
        Some(r#"{"0":"foobar","word":"foo","2":"bar"}"#)
    );
}
