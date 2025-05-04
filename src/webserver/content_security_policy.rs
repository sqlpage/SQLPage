use actix_web::http::header::{
    HeaderName, HeaderValue, TryIntoHeaderPair, CONTENT_SECURITY_POLICY,
};
use awc::http::header::InvalidHeaderValue;
use rand::random;
use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(from = "String")]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
    value: String,
}

impl ContentSecurityPolicy {
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !self.value.is_empty()
    }

    fn new<S: Into<String>>(value: S) -> Self {
        Self {
            nonce: random(),
            value: value.into(),
        }
    }

    #[allow(dead_code)]
    fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self::new("script-src 'self' 'nonce-{NONCE}'")
    }
}

impl Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = self
            .value
            .replace("{NONCE}", self.nonce.to_string().as_str());

        write!(f, "{value}")
    }
}

impl From<String> for ContentSecurityPolicy {
    fn from(input: String) -> Self {
        ContentSecurityPolicy::new(input)
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
    fn default_csp_contains_random_nonce() {
        let mut csp = ContentSecurityPolicy::default();
        csp.set_nonce(0);

        assert_eq!(csp.to_string().as_str(), "script-src 'self' 'nonce-0'");
        assert!(csp.is_enabled());
    }

    #[test]
    fn custom_csp_without_nonce() {
        let csp: ContentSecurityPolicy = String::from("object-src 'none';").into();
        assert_eq!("object-src 'none';", csp.to_string().as_str());
        assert!(csp.is_enabled());
    }

    #[test]
    fn blank_csp() {
        let csp: ContentSecurityPolicy = String::from("").into();
        assert_eq!("", csp.to_string().as_str());
        assert!(!csp.is_enabled());
    }

    #[test]
    fn custom_csp_with_nonce() {
        let mut csp: ContentSecurityPolicy =
            String::from("script-src 'self' 'nonce-{NONCE}'; object-src 'none';").into();
        csp.set_nonce(0);

        assert_eq!(
            "script-src 'self' 'nonce-0'; object-src 'none';",
            csp.to_string().as_str()
        );
        assert!(csp.is_enabled());
    }
}
