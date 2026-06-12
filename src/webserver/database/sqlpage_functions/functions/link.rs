use std::borrow::Cow;

use anyhow::Context;

use crate::webserver::database::sqlpage_functions::url_parameters::URLParameters;

/// Builds a URL from a file name and a JSON object conatining URL parameters.
/// For instance, if the file is "index.sql" and the parameters are {"x": "hello world"},
/// the result will be "index.sql?x=hello%20world".
pub(super) async fn link<'a>(
    file: Cow<'a, str>,
    parameters: Option<Cow<'a, str>>,
    hash: Option<Cow<'a, str>>,
) -> anyhow::Result<String> {
    let mut url = file.into_owned();
    if let Some(parameters) = parameters {
        let encoded = serde_json::from_str::<URLParameters>(&parameters)
            .with_context(|| format!("sqlpage.link: {parameters:?} is not a valid JSON object. The URL parameters should be passed as a json object with parameter names as keys."))?;
        encoded.append_to_path(&mut url);
    }
    if let Some(hash) = hash {
        url.push('#');
        url.push_str(&hash);
    }
    Ok(url)
}
