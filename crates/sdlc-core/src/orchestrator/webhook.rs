//! Webhook payload and route data models for the orchestrator.
//!
//! - `WebhookPayload` -- raw record stored when an external HTTP sender POSTs to
//!   `POST /api/orchestrator/webhooks/*path`. Stored in the `WEBHOOKS` redb table.
//! - `WebhookRoute` -- persistent mapping from a URL path to a tool invocation.
//!   Stored in the `WEBHOOK_ROUTES` redb table. The tick loop matches pending
//!   payloads against registered routes, renders the input template, and
//!   dispatches the tool.
//! - `WebhookEvent` -- immutable audit record written on every webhook arrival
//!   and dispatch outcome. Stored in the `WEBHOOK_EVENTS` redb table (ring
//!   buffer, max 500 entries).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, SdlcError};

/// A raw webhook payload received via `POST /api/orchestrator/webhooks/*path`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: Uuid,
    pub route_path: String,
    pub raw_body: Vec<u8>,
    pub received_at: DateTime<Utc>,
    pub content_type: Option<String>,
}

impl WebhookPayload {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRoute {
    pub id: Uuid,
    pub path: String,
    pub tool_name: String,
    pub input_template: String,
    pub created_at: DateTime<Utc>,
    /// When true, payloads are stored but never dispatched to the tool.
    #[serde(default)]
    pub store_only: bool,
    /// Optional shared secret. If set, incoming requests must supply a
    /// matching `X-Webhook-Secret` header or the payload is rejected (401).
    #[serde(default)]
    pub secret_token: Option<String>,
}

impl WebhookRoute {
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
            store_only: false,
            secret_token: None,
        }
    }

    pub fn render_input(&self, raw_payload: &[u8]) -> Result<serde_json::Value> {
        let payload_str = String::from_utf8_lossy(raw_payload);
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
// WebhookEvent
// ---------------------------------------------------------------------------

/// Outcome of a webhook dispatch attempt, recorded in a `WebhookEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WebhookEventOutcome {
    Received,
    NoRoute,
    Routed { route_id: String, tool_name: String },
    DispatchError { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub seq: u64,
    pub route_path: String,
    pub content_type: Option<String>,
    pub body_bytes: usize,
    pub received_at: DateTime<Utc>,
    pub outcome: WebhookEventOutcome,
}

impl WebhookEvent {
    pub fn new(
        route_path: impl Into<String>,
        content_type: Option<String>,
        body_bytes: usize,
        outcome: WebhookEventOutcome,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            seq: 0,
            route_path: route_path.into(),
            content_type,
            body_bytes,
            received_at: Utc::now(),
            outcome,
        }
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
        let body = obj["body"].as_str().expect("body is a string");
        assert_eq!(body, "{\"action\":\"push\"}");
    }

    #[test]
    fn render_input_invalid_json_after_render() {
        let r = route(r#"{"body": {{payload}}"#);
        let err = r.render_input(b"{}").unwrap_err();
        assert!(matches!(err, SdlcError::OrchestratorDb(_)));
    }

    #[test]
    fn render_input_no_placeholder() {
        let r = route(r#"{"static": true}"#);
        let value = r.render_input(b"ignored").expect("render should succeed");
        assert_eq!(value["static"].as_bool(), Some(true));
    }

    #[test]
    fn render_input_binary_payload_uses_lossy_utf8() {
        let r = route(r#"{"body": {{payload}}}"#);
        let payload = &[0xFF, 0xFE, 0x41];
        let result = r.render_input(payload);
        assert!(result.is_ok());
    }

    #[test]
    fn webhook_route_new_sets_fields_correctly() {
        let r = WebhookRoute::new("/hooks/svc", "my-tool", r#"{"x": 1}"#);
        assert_eq!(r.path, "/hooks/svc");
        assert_eq!(r.tool_name, "my-tool");
        assert_eq!(r.input_template, r#"{"x": 1}"#);
    }

    #[test]
    fn webhook_event_new_sets_fields_correctly() {
        let ev = WebhookEvent::new(
            "/hooks/github",
            Some("application/json".to_string()),
            512,
            WebhookEventOutcome::Received,
        );
        assert_eq!(ev.route_path, "/hooks/github");
        assert_eq!(ev.body_bytes, 512);
        assert_eq!(ev.seq, 0);
        assert!(matches!(ev.outcome, WebhookEventOutcome::Received));
    }

    #[test]
    fn webhook_event_outcome_received_serialization() {
        let outcome = WebhookEventOutcome::Received;
        let json = serde_json::to_string(&outcome).expect("serialize");
        let back: WebhookEventOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(outcome, back);
        assert!(json.contains("\"kind\":\"received\""));
    }

    #[test]
    fn webhook_event_outcome_no_route_serialization() {
        let outcome = WebhookEventOutcome::NoRoute;
        let json = serde_json::to_string(&outcome).expect("serialize");
        let back: WebhookEventOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(outcome, back);
        assert!(json.contains("\"kind\":\"no_route\""));
    }

    #[test]
    fn webhook_event_outcome_routed_serialization() {
        let outcome = WebhookEventOutcome::Routed {
            route_id: "abc-123".to_string(),
            tool_name: "my-tool".to_string(),
        };
        let json = serde_json::to_string(&outcome).expect("serialize");
        let back: WebhookEventOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(outcome, back);
        assert!(json.contains("\"kind\":\"routed\""));
    }

    #[test]
    fn webhook_event_outcome_dispatch_error_serialization() {
        let outcome = WebhookEventOutcome::DispatchError {
            reason: "tool not found".to_string(),
        };
        let json = serde_json::to_string(&outcome).expect("serialize");
        let back: WebhookEventOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(outcome, back);
        assert!(json.contains("\"kind\":\"dispatch_error\""));
    }

    #[test]
    fn webhook_event_round_trips_via_json() {
        let ev = WebhookEvent::new("/hooks/stripe", None, 0, WebhookEventOutcome::NoRoute);
        let json = serde_json::to_string(&ev).expect("serialize");
        let back: WebhookEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.route_path, ev.route_path);
        assert_eq!(back.body_bytes, ev.body_bytes);
        assert_eq!(back.outcome, ev.outcome);
    }
}
