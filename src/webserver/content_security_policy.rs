use std::fmt::Display;

use awc::http::header::InvalidHeaderValue;
use rand::random;

#[derive(Debug, Clone, Copy)]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
}

impl ContentSecurityPolicy {
    pub fn new() -> Self {
        Self { nonce: random() }
    }
}

impl Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "script-src 'self' 'nonce-{}'", self.nonce)
    }
}

impl actix_web::http::header::TryIntoHeaderPair for &ContentSecurityPolicy {
    type Error = InvalidHeaderValue;

    fn try_into_pair(
        self,
    ) -> Result<
        (
            actix_web::http::header::HeaderName,
            actix_web::http::header::HeaderValue,
        ),
        Self::Error,
    > {
        Ok((
            actix_web::http::header::CONTENT_SECURITY_POLICY,
            actix_web::http::header::HeaderValue::from_str(&self.to_string())?,
        ))
    }
}
