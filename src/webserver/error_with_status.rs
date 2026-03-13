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

pub trait StatusCodeResultExt<T, E> {
    fn with_status(self, status: StatusCode) -> anyhow::Result<T>;
    fn with_status_from(self, get_status: impl FnOnce(&E) -> StatusCode) -> anyhow::Result<T>;
    fn with_response_status(self) -> anyhow::Result<T>
    where
        Self: Sized,
        E: ResponseError;
}

impl<T, E> StatusCodeResultExt<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_status(self, status: StatusCode) -> anyhow::Result<T> {
        self.map_err(|err| anyhow::anyhow!(ErrorWithStatus { status }).context(err.to_string()))
    }

    fn with_status_from(self, get_status: impl FnOnce(&E) -> StatusCode) -> anyhow::Result<T> {
        self.map_err(|err| {
            let status = get_status(&err);
            anyhow::anyhow!(ErrorWithStatus { status }).context(err.to_string())
        })
    }

    fn with_response_status(self) -> anyhow::Result<T>
    where
        E: ResponseError,
    {
        self.with_status_from(ResponseError::status_code)
    }
}

pub trait ActixErrorStatusExt<T> {
    fn with_actix_error_status(self) -> anyhow::Result<T>;
}

impl<T> ActixErrorStatusExt<T> for Result<T, actix_web::Error> {
    fn with_actix_error_status(self) -> anyhow::Result<T> {
        // Snapshot the HTTP status before converting to anyhow, which does not preserve Actix's response mapping for later inspection.
        self.with_status_from(|e| e.as_response_error().status_code())
    }
}
