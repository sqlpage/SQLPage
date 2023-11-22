use super::http::RequestInfo;
use super::http::SingleOrVec;
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::http::header::Header;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::web::Form;
use actix_web::FromRequest;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Basic;
use futures_util::TryStreamExt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::StreamExt;

pub(crate) async fn extract_request_info(
    req: &mut ServiceRequest,
    app_state: Arc<AppState>,
) -> RequestInfo {
    let (http_req, payload) = req.parts_mut();
    let (post_variables, uploaded_files) =
        extract_post_data(http_req, payload).await;

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
        uploaded_files: HashMap::from_iter(uploaded_files),
        client_ip,
        cookies: param_map(cookies),
        basic_auth,
        app_state,
    }
}

struct UploadedFile {
    filename: String,
    content_type: String,
    data: Vec<u8>,
}

async fn extract_post_data(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
) -> (Vec<(String, String)>, Vec<(String, UploadedFile)>) {
    if let Ok(post_urlencoded_data) = extract_urlencoded_post_variables(http_req, payload).await {
        (post_urlencoded_data, Vec::new())
    } else {
        extract_multipart_post_data(http_req, payload).await
    }
}

async fn extract_urlencoded_post_variables(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
) -> actix_web::Result<Vec<(String, String)>> {
    Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(Form::into_inner)
}

async fn extract_multipart_post_data(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
) -> (Vec<(String, String)>, Vec<(String, UploadedFile)>) {
    let mut post_variables = Vec::new();
    let mut uploaded_files = Vec::new();

    let mut multipart = match Multipart::from_request(http_req, payload).await {
        Ok(multipart) => multipart,
        Err(err) => {
            log::error!("Failed to parse request: {}", err);
            return (post_variables, uploaded_files);
        }
    };

    while let Some(part) = multipart.next().await {
        match part {
            Ok(field) => {
                // test if field is a file
                let filename = field.content_disposition().get_filename();
                let field_name = field
                    .content_disposition()
                    .get_name()
                    .unwrap_or_default()
                    .to_string();
                if let Some(filename) = filename {
                    uploaded_files.push((field_name, extract_file(field, filename).await));
                } else {
                    post_variables.push((field_name, extract_text(field).await));
                }
            }
            Err(err) => {
                log::error!("Failed to parse multipart field: {}", err);
                return (post_variables, uploaded_files);
            }
        }
    }
    (post_variables, uploaded_files)
}

async fn extract_text(field: actix_multipart::Field) -> String {
    // field is an async stream of Result<Bytes> objects, we collect them into a Vec<u8>
    let data = field
        .try_fold(Vec::new(), |mut data, bytes| async move {
            data.extend_from_slice(&bytes);
            Ok(data)
        }).await
        .unwrap_or_default();
    // convert the Vec<u8> into a String
    String::from_utf8_lossy(&data).to_string()
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
