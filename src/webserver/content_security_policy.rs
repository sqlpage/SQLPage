use actix_web::http::header::CONTENT_SECURITY_POLICY;
use actix_web::HttpResponseBuilder;
use rand::random;
use serde::Deserialize;

pub const DEFAULT_CONTENT_SECURITY_POLICY: &str = "script-src 'self' 'nonce-{NONCE}' 'unsafe-eval'";
pub const NONCE_PLACEHOLDER: &str = "{NONCE}";

#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub nonce: u64,
}

/// A template for the Content Security Policy header.
/// The template is a string that contains the nonce placeholder.
/// The nonce placeholder is replaced with the nonce value when the Content Security Policy is applied to a response.
/// This struct is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSecurityPolicyTemplate {
    pub template: String,
    pub nonce_position: Option<usize>,
}

impl ContentSecurityPolicyTemplate {
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.nonce_position.is_some()
    }

    fn format_nonce(&self, nonce: u64) -> String {
        if let Some(pos) = self.nonce_position {
            format!(
                "{}{}{}",
                &self.template[..pos],
                nonce,
                &self.template[pos + NONCE_PLACEHOLDER.len()..]
            )
        } else {
            self.template.clone()
        }
    }
}

impl Default for ContentSecurityPolicyTemplate {
    fn default() -> Self {
        Self::from(DEFAULT_CONTENT_SECURITY_POLICY)
    }
}

impl From<&str> for ContentSecurityPolicyTemplate {
    fn from(s: &str) -> Self {
        let nonce_position = s.find(NONCE_PLACEHOLDER);
        Self {
            template: s.to_owned(),
            nonce_position,
        }
    }
}

impl<'de> Deserialize<'de> for ContentSecurityPolicyTemplate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(Self::from(s.as_str()))
    }
}

impl ContentSecurityPolicy {
    #[must_use]
    pub fn with_random_nonce() -> Self {
        Self { nonce: random() }
    }

    pub fn apply_to_response(
        &self,
        template: &ContentSecurityPolicyTemplate,
        response: &mut HttpResponseBuilder,
    ) {
        if template.is_enabled() {
            response.insert_header((CONTENT_SECURITY_POLICY, template.format_nonce(self.nonce)));
        }
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
        let csp = ContentSecurityPolicy::with_random_nonce();
        let csp_str = template.format_nonce(csp.nonce);
        assert!(csp_str.starts_with("script-src 'self' 'nonce-"));
        assert!(csp_str.ends_with("' 'unsafe-inline'"));
        let second_csp = ContentSecurityPolicy::with_random_nonce();
        let second_csp_str = template.format_nonce(second_csp.nonce);
        assert_ne!(
            csp_str, second_csp_str,
            "We should not generate the same nonce twice"
        );
    }
}
