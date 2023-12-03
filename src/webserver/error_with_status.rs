use actix_web::{
    http::{header::ContentType, StatusCode},
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
        actix_web::HttpResponse::build(self.status)
            .content_type(ContentType::plaintext())
            .body(self.status.to_string())
    }
}
