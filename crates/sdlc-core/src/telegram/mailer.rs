use crate::error::{Result, SdlcError};

use super::types::ResendConfig;

/// Sends email via the Resend HTTP API.
pub struct ResendMailer {
    config: ResendConfig,
    base_url: String,
}

impl ResendMailer {
    pub fn new(config: ResendConfig) -> Self {
        Self {
            config,
            base_url: "https://api.resend.com".to_string(),
        }
    }

    /// Override the base URL (used in tests with a mock server).
    #[cfg(test)]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Send the digest email to all configured recipients via Resend.
    pub fn send(&self, subject: &str, plain: &str, html: &str) -> Result<()> {
        let url = format!("{}/emails", self.base_url);

        let body = serde_json::json!({
            "from": self.config.from,
            "to": self.config.to,
            "subject": subject,
            "text": plain,
            "html": html,
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .map_err(|e| {
                SdlcError::TelegramApi(format!("Resend delivery failed: network error: {e}"))
            })?;

        let status = response.status();
        if !status.is_success() {
            let detail = response.text().unwrap_or_default();
            return Err(SdlcError::TelegramApi(format!(
                "Resend delivery failed (HTTP {status}): {}",
                redact_api_key(&detail, &self.config.api_key)
            )));
        }

        Ok(())
    }
}

/// Redact the Resend API key from error strings to avoid credential leakage in logs.
fn redact_api_key(s: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        s.to_string()
    } else {
        s.replace(api_key, "[redacted]")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ResendConfig {
        ResendConfig {
            api_key: "re_test_key_abc123".to_string(),
            from: "digest@example.com".to_string(),
            to: vec!["team@example.com".to_string()],
        }
    }

    #[test]
    fn resend_mailer_constructs() {
        let mailer = ResendMailer::new(test_config());
        assert_eq!(mailer.config.from, "digest@example.com");
    }

    #[test]
    fn redact_api_key_masks_key() {
        let error_msg = "Authentication failed: Bearer re_test_key_abc123 is invalid";
        let redacted = redact_api_key(error_msg, "re_test_key_abc123");
        assert!(!redacted.contains("re_test_key_abc123"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn redact_api_key_empty_safe() {
        let error_msg = "Some error without credentials";
        let redacted = redact_api_key(error_msg, "");
        assert_eq!(redacted, error_msg);
    }

    #[test]
    fn send_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/emails")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"test-email-id"}"#)
            .create();

        let mailer = ResendMailer::new(test_config()).with_base_url(server.url());
        let result = mailer.send("Test Subject", "plain text", "<p>html</p>");
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        mock.assert();
    }

    #[test]
    fn send_401_returns_error() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/emails")
            .with_status(401)
            .with_body(r#"{"message":"API key is invalid"}"#)
            .create();

        let mailer = ResendMailer::new(test_config()).with_base_url(server.url());
        let result = mailer.send("Subject", "plain", "<html>");
        assert!(result.is_err(), "expected Err for 401");
        mock.assert();
    }

    #[test]
    fn send_error_redacts_api_key() {
        let mut server = mockito::Server::new();
        // Mock a response whose body contains the API key to verify redaction
        let mock = server
            .mock("POST", "/emails")
            .with_status(403)
            .with_body(r#"{"message":"re_test_key_abc123 is invalid"}"#)
            .create();

        let mailer = ResendMailer::new(test_config()).with_base_url(server.url());
        let result = mailer.send("Subject", "plain", "<html>");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            !err_msg.contains("re_test_key_abc123"),
            "API key must be redacted in error, got: {err_msg}"
        );
        assert!(
            err_msg.contains("[redacted]"),
            "error must contain [redacted], got: {err_msg}"
        );
        mock.assert();
    }
}
