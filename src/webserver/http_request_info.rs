use crate::AppState;
use actix_multipart::form::bytes::Bytes;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::FieldReader;
use actix_multipart::form::Limits;
use actix_multipart::Multipart;
use actix_web::dev::ServiceRequest;
use actix_web::http::header::Header;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::web;
use actix_web::web::Form;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Basic;
use anyhow::anyhow;
use anyhow::Context;
use std::collections::HashMap;
use std::net::IpAddr;
use std::rc::Rc;
use std::sync::Arc;
use tokio_stream::StreamExt;

use super::request_variables::param_map;
use super::request_variables::ParamMap;

#[derive(Debug)]
pub struct RequestInfo {
    pub method: actix_web::http::Method,
    pub path: String,
    pub protocol: String,
    pub get_variables: ParamMap,
    pub post_variables: ParamMap,
    pub uploaded_files: Rc<HashMap<String, TempFile>>,
    pub headers: ParamMap,
    pub client_ip: Option<IpAddr>,
    pub cookies: ParamMap,
    pub basic_auth: Option<Basic>,
    pub app_state: Arc<AppState>,
    pub clone_depth: u8,
}

impl RequestInfo {
    #[must_use]
    pub fn clone_without_variables(&self) -> Self {
        Self {
            method: self.method.clone(),
            path: self.path.clone(),
            protocol: self.protocol.clone(),
            get_variables: ParamMap::new(),
            post_variables: ParamMap::new(),
            uploaded_files: self.uploaded_files.clone(),
            headers: self.headers.clone(),
            client_ip: self.client_ip,
            cookies: self.cookies.clone(),
            basic_auth: self.basic_auth.clone(),
            app_state: self.app_state.clone(),
            clone_depth: self.clone_depth + 1,
        }
    }
}

impl Clone for RequestInfo {
    fn clone(&self) -> Self {
        let mut clone = self.clone_without_variables();
        clone.get_variables.clone_from(&self.get_variables);
        clone.post_variables.clone_from(&self.post_variables);
        clone
    }
}

pub(crate) async fn extract_request_info(
    req: &mut ServiceRequest,
    app_state: Arc<AppState>,
) -> anyhow::Result<RequestInfo> {
    let (http_req, payload) = req.parts_mut();
    let method = http_req.method().clone();
    let protocol = http_req.connection_info().scheme().to_string();
    let config = &app_state.config;
    let (post_variables, uploaded_files) = extract_post_data(http_req, payload, config).await?;
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

    Ok(RequestInfo {
        method,
        path: req.path().to_string(),
        headers: param_map(headers),
        get_variables: param_map(get_variables),
        post_variables: param_map(post_variables),
        uploaded_files: Rc::new(HashMap::from_iter(uploaded_files)),
        client_ip,
        cookies: param_map(cookies),
        basic_auth,
        app_state,
        protocol,
        clone_depth: 0,
    })
}

async fn extract_post_data(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
    config: &crate::app_config::AppConfig,
) -> anyhow::Result<(Vec<(String, String)>, Vec<(String, TempFile)>)> {
    let content_type = http_req
        .headers()
        .get(&CONTENT_TYPE)
        .map(AsRef::as_ref)
        .unwrap_or_default();
    if content_type.starts_with(b"application/x-www-form-urlencoded") {
        let vars = extract_urlencoded_post_variables(http_req, payload).await?;
        Ok((vars, Vec::new()))
    } else if content_type.starts_with(b"multipart/form-data") {
        extract_multipart_post_data(http_req, payload, config).await
    } else {
        let ct_str = String::from_utf8_lossy(content_type);
        log::debug!("Not parsing POST data from request without known content type {ct_str}");
        Ok((Vec::new(), Vec::new()))
    }
}

async fn extract_urlencoded_post_variables(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
) -> anyhow::Result<Vec<(String, String)>> {
    Form::<Vec<(String, String)>>::from_request(http_req, payload)
        .await
        .map(Form::into_inner)
        .map_err(|e| anyhow!("could not parse request as urlencoded form data: {e}"))
}

async fn extract_multipart_post_data(
    http_req: &mut actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
    config: &crate::app_config::AppConfig,
) -> anyhow::Result<(Vec<(String, String)>, Vec<(String, TempFile)>)> {
    let mut post_variables = Vec::new();
    let mut uploaded_files = Vec::new();

    let mut multipart = Multipart::from_request(http_req, payload)
        .await
        .map_err(|e| anyhow!("could not parse request as multipart form data: {e}"))?;

    let mut limits = Limits::new(config.max_uploaded_file_size, config.max_uploaded_file_size);
    log::trace!(
        "Parsing multipart form data with a {:?} KiB limit",
        limits.total_limit_remaining / 1024
    );

    while let Some(part) = multipart.next().await {
        let field = part.map_err(|e| anyhow!("unable to read form field: {e}"))?;
        let content_disposition = field
            .content_disposition()
            .ok_or_else(|| anyhow!("missing Content-Disposition in form field"))?;
        // test if field is a file
        let filename = content_disposition.get_filename();
        let field_name = content_disposition
            .get_name()
            .unwrap_or_default()
            .to_string();
        log::trace!("Parsing multipart field: {}", field_name);
        if let Some(filename) = filename {
            log::debug!("Extracting file: {field_name} ({filename})");
            let extracted = extract_file(http_req, field, &mut limits)
                .await
                .with_context(|| {
                    format!(
                        "Failed to extract file {field_name:?}. Max file size: {} kiB",
                        config.max_uploaded_file_size / 1_024
                    )
                })?;
            log::trace!("Extracted file {field_name} to {:?}", extracted.file.path());
            if is_file_field_empty(&extracted).await? {
                log::debug!("Ignoring empty file field: {field_name}");
                continue;
            }
            uploaded_files.push((field_name, extracted));
        } else {
            let text_contents = extract_text(http_req, field, &mut limits).await?;
            log::trace!("Extracted field as text: {field_name} = {text_contents:?}");
            post_variables.push((field_name, text_contents));
        }
    }
    Ok((post_variables, uploaded_files))
}

async fn extract_text(
    req: &HttpRequest,
    field: actix_multipart::Field,
    limits: &mut Limits,
) -> anyhow::Result<String> {
    // field is an async stream of Result<Bytes> objects, we collect them into a Vec<u8>
    let data = Bytes::read_field(req, field, limits)
        .await
        .map(|bytes| bytes.data)
        .map_err(|e| anyhow!("failed to read form field data: {e}"))?;
    Ok(String::from_utf8(data.to_vec())?)
}

async fn extract_file(
    req: &HttpRequest,
    field: actix_multipart::Field,
    limits: &mut Limits,
) -> anyhow::Result<TempFile> {
    // extract a tempfile from the field
    let file = TempFile::read_field(req, field, limits)
        .await
        .map_err(|e| anyhow!("Failed to save uploaded file: {e}"))?;
    Ok(file)
}

/// file upload form fields that are left blank result in the browser sending an empty file, with a mime type of application/octet-stream.
/// We don't want to treat this the same as actual empty files, so we check for this case.
async fn is_file_field_empty(
    uploaded_file: &actix_multipart::form::tempfile::TempFile,
) -> anyhow::Result<bool> {
    Ok(
        uploaded_file.content_type == Some(mime_guess::mime::APPLICATION_OCTET_STREAM)
            && uploaded_file
                .file_name
                .as_ref()
                .filter(|x| !x.is_empty())
                .is_none()
            && tokio::fs::metadata(&uploaded_file.file.path()).await?.len() == 0,
    )
}

#[cfg(test)]
mod test {
    use super::super::http::SingleOrVec;
    use super::*;
    use crate::app_config::AppConfig;
    use actix_web::{http::header::ContentType, test::TestRequest};

    #[actix_web::test]
    async fn test_extract_empty_request() {
        let config =
            serde_json::from_str::<AppConfig>(r#"{"listen_on": "localhost:1234"}"#).unwrap();
        let mut service_request = TestRequest::default().to_srv_request();
        let app_data = Arc::new(AppState::init(&config).await.unwrap());
        let request_info = extract_request_info(&mut service_request, app_data)
            .await
            .unwrap();
        assert_eq!(request_info.post_variables.len(), 0);
        assert_eq!(request_info.uploaded_files.len(), 0);
        assert_eq!(request_info.get_variables.len(), 0);
    }

    #[actix_web::test]
    async fn test_extract_urlencoded_request() {
        let config =
            serde_json::from_str::<AppConfig>(r#"{"listen_on": "localhost:1234"}"#).unwrap();
        let mut service_request = TestRequest::get()
            .uri("/?my_array[]=5")
            .insert_header(ContentType::form_url_encoded())
            .set_payload("my_array[]=3&my_array[]=Hello%20World&repeated=1&repeated=2")
            .to_srv_request();
        let app_data = Arc::new(AppState::init(&config).await.unwrap());
        let request_info = extract_request_info(&mut service_request, app_data)
            .await
            .unwrap();
        assert_eq!(
            request_info.post_variables,
            vec![
                (
                    "my_array".to_string(),
                    SingleOrVec::Vec(vec!["3".to_string(), "Hello World".to_string()])
                ),
                ("repeated".to_string(), SingleOrVec::Single("2".to_string())), // without brackets, only the last value is kept
            ]
            .into_iter()
            .collect::<ParamMap>()
        );
        assert_eq!(request_info.uploaded_files.len(), 0);
        assert_eq!(
            request_info.get_variables,
            vec![(
                "my_array".to_string(),
                SingleOrVec::Vec(vec!["5".to_string()])
            )] // with brackets, even if there is only one value, it is kept as a vector
            .into_iter()
            .collect::<ParamMap>()
        );
    }

    #[actix_web::test]
    async fn test_extract_multipart_form_data() {
        env_logger::init();
        let config =
            serde_json::from_str::<AppConfig>(r#"{"listen_on": "localhost:1234"}"#).unwrap();
        let mut service_request = TestRequest::get()
            .insert_header(("content-type", "multipart/form-data;boundary=xxx"))
            .set_payload(
                "--xxx\r\n\
                Content-Disposition: form-data; name=\"my_array[]\"\r\n\
                Content-Type: text/plain\r\n\
                \r\n\
                3\r\n\
                --xxx\r\n\
                Content-Disposition: form-data; name=\"my_uploaded_file\"; filename=\"test.txt\"\r\n\
                Content-Type: text/plain\r\n\
                \r\n\
                Hello World\r\n\
                --xxx--\r\n"
            )
            .to_srv_request();
        let app_data = Arc::new(AppState::init(&config).await.unwrap());
        let request_info = extract_request_info(&mut service_request, app_data)
            .await
            .unwrap();
        assert_eq!(
            request_info.post_variables,
            vec![(
                "my_array".to_string(),
                SingleOrVec::Vec(vec!["3".to_string()])
            ),]
            .into_iter()
            .collect::<ParamMap>()
        );
        assert_eq!(request_info.uploaded_files.len(), 1);
        let my_upload = &request_info.uploaded_files["my_uploaded_file"];
        assert_eq!(my_upload.file_name.as_ref().unwrap(), "test.txt");
        assert_eq!(request_info.get_variables.len(), 0);
        assert_eq!(std::fs::read(&my_upload.file).unwrap(), b"Hello World");
        assert_eq!(request_info.get_variables.len(), 0);
    }
}
