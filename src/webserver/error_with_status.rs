use actix_web::{
    http::{
        header::{self, ContentType},
        StatusCode,
    },
    ResponseError,
};

#[derive(Debug, PartialEq)]
pub struct ErrorWithStatus {
    pub status: StatusCode,
}
impl std::fmt::Display for ErrorWithStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)
    }
}
impl std::error::Error for ErrorWithStatus {}

impl ResponseError for ErrorWithStatus {
    fn status_code(&self) -> StatusCode {
        self.status
    }
    fn error_response(&self) -> actix_web::HttpResponse {
        let mut resp_builder = actix_web::HttpResponse::build(self.status);
        resp_builder.content_type(ContentType::plaintext());
        if self.status == StatusCode::UNAUTHORIZED {
            resp_builder.insert_header((
                header::WWW_AUTHENTICATE,
                header::HeaderValue::from_static(
                    "Basic realm=\"Authentication required\", charset=\"UTF-8\"",
                ),
            ));
            resp_builder.body("Sorry, but you are not authorized to access this page.")
        } else {
            resp_builder.body(self.status.to_string())
        }
    }
}
