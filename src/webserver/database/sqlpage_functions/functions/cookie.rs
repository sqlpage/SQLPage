use std::borrow::Cow;

use crate::webserver::{http_request_info::RequestInfo, single_or_vec::SingleOrVec};

pub(super) async fn cookie<'a>(request: &'a RequestInfo, name: Cow<'a, str>) -> Option<Cow<'a, str>> {
    request.cookies.get(&*name).map(SingleOrVec::as_json_str)
}
