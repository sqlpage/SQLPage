use super::*;

pub(super) async fn header<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    let lower_name = name.to_ascii_lowercase();
    request
        .headers
        .get(&lower_name)
        .map(SingleOrVec::as_json_str)
}
