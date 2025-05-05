use actix_web::http::header::{
    HeaderName, HeaderValue, TryIntoHeaderPair, CONTENT_SECURITY_POLICY,
};
use actix_web::HttpResponseBuilder;
use awc::http::header::InvalidHeaderValue;
use rand::random;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub const DEFAULT_CONTENT_SECURITY_POLICY: &str = "script-src 'self' 'nonce-{NONCE}'";

#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
    template: ContentSecurityPolicyTemplate,
}

/// A template for the Content Security Policy header.
/// The template is a string that contains the nonce placeholder.
/// The nonce placeholder is replaced with the nonce value when the Content Security Policy is applied to a response.
/// This struct is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSecurityPolicyTemplate {
    pub before_nonce: Arc<str>,
    pub after_nonce: Option<Arc<str>>,
}

impl Default for ContentSecurityPolicyTemplate {
    fn default() -> Self {
        Self::from(DEFAULT_CONTENT_SECURITY_POLICY)
    }
}

impl From<&str> for ContentSecurityPolicyTemplate {
    fn from(s: &str) -> Self {
        if let Some((before, after)) = s.split_once("{NONCE}") {
            Self {
                before_nonce: Arc::from(before),
                after_nonce: Some(Arc::from(after)),
            }
        } else {
            Self {
                before_nonce: Arc::from(s),
                after_nonce: None,
            }
        }
    }
}

impl<'de> Deserialize<'de> for ContentSecurityPolicyTemplate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Self::from(s))
    }
}

impl ContentSecurityPolicy {
    pub fn new(template: ContentSecurityPolicyTemplate) -> Self {
        Self {
            nonce: random(),
            template,
        }
    }

    pub fn apply_to_response(&self, response: &mut HttpResponseBuilder) {
        if self.is_enabled() {
            response.insert_header(self);
        }
    }

    fn is_enabled(&self) -> bool {
        !self.template.before_nonce.is_empty() || self.template.after_nonce.is_some()
    }
}

impl Display for ContentSecurityPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let before = self.template.before_nonce.as_ref();
        if let Some(after) = &self.template.after_nonce {
            let nonce = self.nonce;
            write!(f, "{before}{nonce}{after}")
        } else {
            write!(f, "{before}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_security_policy_display() {
        let template = ContentSecurityPolicyTemplate::from(
            "script-src 'self' 'nonce-{NONCE}' 'unsafe-inline'",
        );
        let csp = ContentSecurityPolicy::new(template.clone());
        let csp_str = csp.to_string();
        assert!(csp_str.starts_with("script-src 'self' 'nonce-"));
        assert!(csp_str.ends_with("' 'unsafe-inline'"));
        let second_csp = ContentSecurityPolicy::new(template);
        let second_csp_str = second_csp.to_string();
        assert_ne!(
            csp_str, second_csp_str,
            "We should not generate the same nonce twice"
        );
    }
}
