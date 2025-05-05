use actix_web::http::header::{
    HeaderName, HeaderValue, TryIntoHeaderPair, CONTENT_SECURITY_POLICY,
};
use actix_web::HttpResponseBuilder;
use awc::http::header::InvalidHeaderValue;
use rand::random;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::AppState;

pub const DEFAULT_CONTENT_SECURITY_POLICY: &str = "script-src 'self' 'nonce-{NONCE}'";

#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
    app_state: Arc<AppState>,
}

impl ContentSecurityPolicy {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            nonce: random(),
            app_state,
        }
    }

    pub fn apply_to_response(&self, response: &mut HttpResponseBuilder) {
        if self.is_enabled() {
            response.insert_header(self);
        }
    }

    fn template_string(&self) -> &str {
        &self.app_state.config.content_security_policy
    }

    fn is_enabled(&self) -> bool {
        !self.app_state.config.content_security_policy.is_empty()
    }
}

impl Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let template = self.template_string();
        if let Some((before, after)) = template.split_once("{NONCE}") {
            write!(f, "{before}{nonce}{after}", nonce = self.nonce)
        } else {
            write!(f, "{}", template)
        }
    }
}

impl TryIntoHeaderPair for &ContentSecurityPolicy {
    type Error = InvalidHeaderValue;

    fn try_into_pair(self) -> Result<(HeaderName, HeaderValue), Self::Error> {
        Ok((
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_maybe_shared(self.to_string())?,
        ))
    }
}
