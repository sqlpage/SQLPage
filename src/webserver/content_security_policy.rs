use actix_web::http::header::{
    HeaderName, HeaderValue, TryIntoHeaderPair, CONTENT_SECURITY_POLICY,
};
use actix_web::HttpResponseBuilder;
use awc::http::header::InvalidHeaderValue;
use rand::random;
use std::fmt::{Display, Formatter};

pub const DEFAULT_CONTENT_SECURITY_POLICY: &str = "script-src 'self' 'nonce-{NONCE}'";

#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
    policy: String,
}

impl ContentSecurityPolicy {
    pub fn new<S: Into<String>>(policy: S) -> Self {
        Self {
            nonce: random(),
            policy: policy.into(),
        }
    }

    pub fn apply_to_response(&self, response: &mut HttpResponseBuilder) {
        if self.is_enabled() {
            response.insert_header(self);
        }
    }

    fn is_enabled(&self) -> bool {
        !self.policy.is_empty()
    }

    #[allow(dead_code)]
    fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
}

impl Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self.policy.replace("{NONCE}", &self.nonce.to_string());

        write!(f, "{value}")
    }
}

impl TryIntoHeaderPair for &ContentSecurityPolicy {
    type Error = InvalidHeaderValue;

    fn try_into_pair(self) -> Result<(HeaderName, HeaderValue), Self::Error> {
        Ok((
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_str(&self.to_string())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_csp_response_contains_random_nonce() {
        let mut csp = ContentSecurityPolicy::new(DEFAULT_CONTENT_SECURITY_POLICY);
        csp.set_nonce(0);

        assert!(csp.is_enabled());
        assert_eq!(&csp.to_string(), "script-src 'self' 'nonce-0'");
    }

    #[test]
    fn custom_csp_response_without_nonce() {
        let csp = ContentSecurityPolicy::new("object-src 'none';");

        assert!(csp.is_enabled());
        assert_eq!("object-src 'none';", &csp.to_string());
    }

    #[test]
    fn blank_csp_response() {
        let csp = ContentSecurityPolicy::new("");

        assert!(!csp.is_enabled());
        assert_eq!("", &csp.to_string());
    }

    #[test]
    fn custom_csp_with_nonce() {
        let mut csp =
            ContentSecurityPolicy::new("script-src 'self' 'nonce-{NONCE}'; object-src 'none';");
        csp.set_nonce(0);

        assert!(csp.is_enabled());
        assert_eq!(
            "script-src 'self' 'nonce-0'; object-src 'none';",
            csp.to_string().as_str()
        );
    }
}
