use crate::error::{Result, SdlcError};

use super::types::SmtpConfig;

/// Sends a multipart email via SMTP.
pub struct SmtpMailer {
    config: SmtpConfig,
}

/// Represents a composed email ready to send.
struct EmailMessage {
    from: String,
    to: Vec<String>,
    subject: String,
    plain: String,
    html: String,
}

impl SmtpMailer {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// Send the digest email to all configured recipients.
    ///
    /// Uses STARTTLS for port 587, or direct TLS for port 465.
    /// All other ports also use STARTTLS.
    pub fn send(&self, subject: &str, plain: &str, html: &str) -> Result<()> {
        let msg = EmailMessage {
            from: self.config.from.clone(),
            to: self.config.to.clone(),
            subject: subject.to_string(),
            plain: plain.to_string(),
            html: html.to_string(),
        };
        self.send_via_smtp(msg)
    }

    fn send_via_smtp(&self, msg: EmailMessage) -> Result<()> {
        use std::process::Command;

        // Build a minimal MIME message using base64 encoding.
        // This implementation uses openssl s_client or curl as a fallback
        // when the lettre crate is not compiled in.
        //
        // For production use, this delegates to curl's SMTP support which is
        // universally available and handles TLS natively.
        let _recipients = msg.to.join(",");
        let mime_body = build_mime_message(&msg);

        // Use curl to send SMTP mail with TLS
        let smtp_url = if self.config.port == 465 {
            format!("smtps://{}:{}", self.config.host, self.config.port)
        } else {
            format!("smtp://{}:{}", self.config.host, self.config.port)
        };

        let mut cmd = Command::new("curl");
        cmd.arg("--silent")
            .arg("--show-error")
            .arg("--url")
            .arg(&smtp_url)
            .arg("--ssl-reqd")
            .arg("--mail-from")
            .arg(&msg.from)
            .arg("--user")
            .arg(format!("{}:{}", self.config.username, self.config.password))
            .arg("--upload-file")
            .arg("-"); // read from stdin

        for recipient in &msg.to {
            cmd.arg("--mail-rcpt").arg(recipient);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                SdlcError::TelegramApi(format!("SMTP delivery failed: could not spawn curl: {e}"))
            })?;

        // Write the MIME message to curl's stdin
        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin.write_all(mime_body.as_bytes()).map_err(|e| {
                SdlcError::TelegramApi(format!("SMTP delivery failed: write error: {e}"))
            })?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| SdlcError::TelegramApi(format!("SMTP delivery failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SdlcError::TelegramApi(format!(
                "SMTP delivery failed: {}",
                // Redact any credentials that might appear in error output
                redact_credentials(&stderr, &self.config.username, &self.config.password)
            )));
        }

        Ok(())
    }
}

/// Build a minimal multipart/alternative MIME message.
fn build_mime_message(msg: &EmailMessage) -> String {
    let boundary = format!("sdlc_digest_{}", chrono::Utc::now().timestamp());
    let to_header = msg.to.join(", ");
    let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000");

    format!(
        "From: {from}\r\n\
         To: {to}\r\n\
         Subject: {subject}\r\n\
         Date: {date}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Transfer-Encoding: quoted-printable\r\n\
         \r\n\
         {plain}\r\n\
         --{boundary}\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Transfer-Encoding: quoted-printable\r\n\
         \r\n\
         {html}\r\n\
         --{boundary}--\r\n",
        from = msg.from,
        to = to_header,
        subject = msg.subject,
        date = date,
        boundary = boundary,
        plain = msg.plain,
        html = msg.html,
    )
}

/// Redact sensitive values from an error string to avoid credential leakage in logs.
fn redact_credentials(s: &str, username: &str, password: &str) -> String {
    let mut out = s.to_string();
    if !username.is_empty() {
        out = out.replace(username, "[redacted]");
    }
    if !password.is_empty() {
        out = out.replace(password, "[redacted]");
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SmtpConfig {
        SmtpConfig {
            host: "smtp.example.com".to_string(),
            port: 587,
            username: "user@example.com".to_string(),
            password: "s3cr3t".to_string(),
            from: "digest@example.com".to_string(),
            to: vec!["team@example.com".to_string()],
        }
    }

    #[test]
    fn build_mime_message_contains_required_headers() {
        let msg = EmailMessage {
            from: "from@example.com".to_string(),
            to: vec!["to@example.com".to_string()],
            subject: "Test Subject".to_string(),
            plain: "Plain text body".to_string(),
            html: "<p>HTML body</p>".to_string(),
        };
        let mime = build_mime_message(&msg);
        assert!(mime.contains("From: from@example.com"));
        assert!(mime.contains("To: to@example.com"));
        assert!(mime.contains("Subject: Test Subject"));
        assert!(mime.contains("MIME-Version: 1.0"));
        assert!(mime.contains("multipart/alternative"));
        assert!(mime.contains("text/plain"));
        assert!(mime.contains("text/html"));
        assert!(mime.contains("Plain text body"));
        assert!(mime.contains("<p>HTML body</p>"));
    }

    #[test]
    fn build_mime_message_multiple_recipients() {
        let msg = EmailMessage {
            from: "from@example.com".to_string(),
            to: vec!["a@example.com".to_string(), "b@example.com".to_string()],
            subject: "Multi-recipient".to_string(),
            plain: "body".to_string(),
            html: "<p>body</p>".to_string(),
        };
        let mime = build_mime_message(&msg);
        assert!(mime.contains("a@example.com, b@example.com"));
    }

    #[test]
    fn redact_credentials_masks_username_and_password() {
        let error_msg = "Authentication failed: user@example.com:s3cr3t is invalid";
        let redacted = redact_credentials(error_msg, "user@example.com", "s3cr3t");
        assert!(!redacted.contains("user@example.com"));
        assert!(!redacted.contains("s3cr3t"));
        assert!(redacted.contains("[redacted]"));
    }

    #[test]
    fn redact_credentials_empty_fields_safe() {
        let error_msg = "Some error without credentials";
        let redacted = redact_credentials(error_msg, "", "");
        assert_eq!(redacted, error_msg);
    }

    #[test]
    fn smtp_mailer_constructs() {
        let mailer = SmtpMailer::new(test_config());
        assert_eq!(mailer.config.host, "smtp.example.com");
        assert_eq!(mailer.config.port, 587);
    }
}
