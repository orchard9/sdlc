//! Thin HTTP client for the notify transactional email service.
//!
//! In dev/test mode (when `NOTIFY_URL` is unset), `send_otp` logs to stdout
//! and returns immediately. In production, it fires a background task so
//! email delivery never blocks the HTTP response path.

use std::sync::Arc;

use serde_json::json;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct NotifyClient {
    base_url: String,
    api_key: String,
    host: String,
    from: String,
    http: reqwest::Client,
}

impl NotifyClient {
    pub fn new(base_url: String, api_key: String, host: String, from: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            base_url,
            api_key,
            host,
            from,
            http,
        }
    }

    /// Build from environment variables. Returns `None` if `NOTIFY_URL` is unset.
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("NOTIFY_URL").ok()?;
        let api_key = std::env::var("NOTIFY_API_KEY").unwrap_or_default();
        let host = std::env::var("NOTIFY_HOST").unwrap_or_default();
        let from = std::env::var("NOTIFY_FROM").unwrap_or_default();
        Some(Self::new(base_url, api_key, host, from))
    }

    /// Send an OTP email. Fires and logs — errors do not propagate to callers.
    ///
    /// This method is synchronous-looking but cheap: it clones `Arc` references
    /// and spawns a background task. Call it from within a Tokio context.
    pub fn send_otp_background(self: Arc<Self>, email: String, otp: String) {
        tokio::spawn(async move {
            if let Err(e) = self.send_otp_inner(&email, &otp).await {
                tracing::warn!(email = %email, error = %e, "notify: OTP send failed");
            }
        });
    }

    async fn send_otp_inner(&self, email: &str, otp: &str) -> anyhow::Result<()> {
        let idempotency_key = format!("otp:{email}:{otp}");
        let from = if self.from.contains('<') {
            self.from.clone()
        } else {
            format!("Ponder <{}>", self.from)
        };

        let content = crate::email::render_otp(email, otp);

        let body = json!({
            "to": email,
            "from": from,
            "content": {
                "subject": content.subject,
                "html": content.html,
                "text": content.text,
            },
            "meta": {
                "host": self.host,
                "category": "critical",
                "tags": ["auth", "otp"],
            },
            "options": {
                "idempotency_key": idempotency_key,
            },
        });

        let resp = self
            .http
            .post(format!("{}/email", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        // 202 = new queued, 200 = idempotent duplicate — both are success
        if status == 202 || status == 200 {
            let data: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg_id = data
                .get("message_id")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            tracing::info!(email = %email, message_id = %msg_id, "notify: OTP queued");
            return Ok(());
        }

        let body_text = resp.text().await.unwrap_or_default();

        // 422 SUPPRESSED — log and treat as success (terminal, no retry)
        if status == 422 && body_text.contains("SUPPRESSED") {
            tracing::warn!(email = %email, "notify: address suppressed, skipping");
            return Ok(());
        }

        anyhow::bail!("notify HTTP {status}: {body_text}");
    }
}
