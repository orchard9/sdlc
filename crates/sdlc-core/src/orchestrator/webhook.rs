//! Webhook payload and route data models for the orchestrator.
//!
//! - `WebhookPayload` — raw record stored when an external HTTP sender POSTs to
//!   `POST /api/orchestrator/webhooks/*path`. Stored in the `WEBHOOKS` redb table.
//! - `WebhookRoute` — persistent mapping from a URL path to a tool invocation.
//!   Stored in the `WEBHOOK_ROUTES` redb table. The tick loop matches pending
//!   payloads against registered routes, renders the input template, and
//!   dispatches the tool.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, SdlcError};

/// A raw webhook payload received via `POST /api/orchestrator/webhooks/*path`.
///
/// Stored in the `WEBHOOKS` redb table. The tick loop reads pending payloads
/// via `ActionDb::all_pending_webhooks()` and deletes them after dispatch
/// via `ActionDb::delete_webhook(id)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Unique identifier for this payload, used as the redb key.
    pub id: Uuid,
    /// The path from the URL (e.g. `/hooks/github`).
    pub route_path: String,
    /// Raw bytes as received — not decoded or validated.
    ///
    /// Serialized as a JSON byte array (base64 via serde).
    pub raw_body: Vec<u8>,
    /// RFC 3339 timestamp when the payload arrived.
    pub received_at: DateTime<Utc>,
    /// The `Content-Type` header from the request, if present.
    pub content_type: Option<String>,
}

impl WebhookPayload {
    /// Construct a new `WebhookPayload` with a fresh UUID and the current timestamp.
    pub fn new(
        route_path: impl Into<String>,
        raw_body: Vec<u8>,
        content_type: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            route_path: route_path.into(),
            raw_body,
            received_at: Utc::now(),
            content_type,
        }
    }
}

// ---------------------------------------------------------------------------
// WebhookRoute
// ---------------------------------------------------------------------------

/// A registered mapping from an HTTP webhook path to a tool invocation.
///
/// When a payload arrives for a path, the tick loop finds the matching route,
/// renders `input_template` (substituting `{{payload}}` with the JSON-escaped
/// webhook body), and calls `run_tool()` with the rendered input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRoute {
    /// Unique identifier, used as the redb key.
    pub id: Uuid,
    /// The URL path this route handles (e.g. `/hooks/github`).
    /// Always starts with `/`.
    pub path: String,
    /// Tool slug matching a directory under `.sdlc/tools/<name>/`.
    pub tool_name: String,
    /// JSON template for the tool input. The literal string `{{payload}}` is
    /// replaced with the JSON-escaped string representation of the raw webhook
    /// body before the template is parsed as JSON.
    ///
    /// Example: `{"body": {{payload}}, "source": "github"}`
    pub input_template: String,
    pub created_at: DateTime<Utc>,
}

impl WebhookRoute {
    /// Create a new `WebhookRoute` with a fresh UUID and the current timestamp.
    pub fn new(
        path: impl Into<String>,
        tool_name: impl Into<String>,
        input_template: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            path: path.into(),
            tool_name: tool_name.into(),
            input_template: input_template.into(),
            created_at: Utc::now(),
        }
    }

    /// Render `input_template` by substituting `{{payload}}` with the
    /// JSON-escaped string form of `raw_payload`, then parse as JSON.
    ///
    /// The substitution produces a JSON string literal so templates can embed
    /// the payload as a string field. After substitution the rendered text must
    /// be valid JSON; returns `Err(SdlcError::OrchestratorDb(...))` if parsing fails.
    pub fn render_input(&self, raw_payload: &[u8]) -> Result<serde_json::Value> {
        let payload_str = String::from_utf8_lossy(raw_payload);
        // Produces a quoted, escaped JSON string value — e.g. "\"foo\\\"bar\""
        let payload_json = serde_json::to_string(payload_str.as_ref())
            .map_err(|e| SdlcError::OrchestratorDb(format!("payload serialization failed: {e}")))?;
        let rendered = self.input_template.replace("{{payload}}", &payload_json);
        let value: serde_json::Value = serde_json::from_str(&rendered).map_err(|e| {
            SdlcError::OrchestratorDb(format!(
                "template render produced invalid JSON: {e} (rendered: {rendered})"
            ))
        })?;
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn route(template: &str) -> WebhookRoute {
        WebhookRoute::new("/hooks/test", "my-tool", template)
    }

    #[test]
    fn render_input_substitutes_payload() {
        let r = route(r#"{"body": {{payload}}, "source": "github"}"#);
        let payload = b"{\"action\":\"push\"}";
        let value = r.render_input(payload).expect("render should succeed");
        let obj = value.as_object().expect("result is object");
        assert_eq!(obj["source"].as_str(), Some("github"));
        // The payload is embedded as a JSON string
        let body = obj["body"].as_str().expect("body is a string");
        assert_eq!(body, "{\"action\":\"push\"}");
    }

    #[test]
    fn render_input_invalid_json_after_render() {
        // Template is not valid JSON after substitution (missing closing brace)
        let r = route(r#"{"body": {{payload}}"#);
        let err = r.render_input(b"{}").unwrap_err();
        assert!(
            matches!(err, SdlcError::OrchestratorDb(_)),
            "expected OrchestratorDb error, got {err:?}"
        );
    }

    #[test]
    fn render_input_no_placeholder() {
        // Template with no {{payload}} — parses as-is, ignores raw_payload.
        let r = route(r#"{"static": true}"#);
        let value = r.render_input(b"ignored").expect("render should succeed");
        assert_eq!(value["static"].as_bool(), Some(true));
    }

    #[test]
    fn render_input_binary_payload_uses_lossy_utf8() {
        let r = route(r#"{"body": {{payload}}}"#);
        // Non-UTF8 bytes → lossy conversion, should not panic
        let payload = &[0xFF, 0xFE, 0x41];
        let result = r.render_input(payload);
        assert!(result.is_ok(), "lossy UTF-8 payload should not hard-error");
    }

    #[test]
    fn webhook_route_new_sets_fields_correctly() {
        let r = WebhookRoute::new("/hooks/svc", "my-tool", r#"{"x": 1}"#);
        assert_eq!(r.path, "/hooks/svc");
        assert_eq!(r.tool_name, "my-tool");
        assert_eq!(r.input_template, r#"{"x": 1}"#);
    }
}
