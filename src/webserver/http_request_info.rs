use super::http::RequestInfo;
use super::http::SingleOrVec;
use crate::AppState;
use actix_web::dev::ServiceRequest;
use actix_web::http::header::Header;
use actix_web::web;
use actix_web::web::Form;
use actix_web::FromRequest;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Basic;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) async fn extract_request_info(
    req: &mut ServiceRequest,
    app_state: Arc<AppState>,
) -> RequestInfo {
    let (http_req, payload) = req.parts_mut();
    let post_variables = Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(Form::into_inner)
        .unwrap_or_default();

    let headers = req.headers().iter().map(|(name, value)| {
        (
            name.to_string(),
            String::from_utf8_lossy(value.as_bytes()).to_string(),
        )
    });
    let get_variables = web::Query::<Vec<(String, String)>>::from_query(req.query_string())
        .map(web::Query::into_inner)
        .unwrap_or_default();
    let client_ip = req.peer_addr().map(|addr| addr.ip());

    let raw_cookies = req.cookies();
    let cookies = raw_cookies
        .iter()
        .flat_map(|c| c.iter())
        .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()));

    let basic_auth = Authorization::<Basic>::parse(req)
        .ok()
        .map(Authorization::into_scheme);

    RequestInfo {
        path: req.path().to_string(),
        headers: param_map(headers),
        get_variables: param_map(get_variables),
        post_variables: param_map(post_variables),
        client_ip,
        cookies: param_map(cookies),
        basic_auth,
        app_state,
    }
}

pub type ParamMap = HashMap<String, SingleOrVec>;

fn param_map<PAIRS: IntoIterator<Item = (String, String)>>(values: PAIRS) -> ParamMap {
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
