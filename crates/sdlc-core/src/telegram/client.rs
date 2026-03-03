use crate::error::{Result, SdlcError};
use serde::Deserialize;

use super::types::TelegramUpdate;

/// Thin wrapper around the Telegram Bot API `getUpdates` endpoint.
pub struct TelegramClient {
    token: String,
    base_url: String,
    client: reqwest::blocking::Client,
}

/// Raw API response wrapper.
#[derive(Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    description: Option<String>,
    result: Option<T>,
}

impl TelegramClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            base_url: "https://api.telegram.org".to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Override the base URL (used in tests with a mock server).
    #[cfg(test)]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Fetch the most recent `limit` updates from the Bot API.
    ///
    /// This is a stateless call — no offset is tracked between invocations.
    /// The caller is responsible for filtering by timestamp.
    pub fn get_updates(&self, limit: u32) -> Result<Vec<TelegramUpdate>> {
        let url = format!(
            "{}/bot{}/getUpdates?limit={}&allowed_updates=[\"message\"]",
            self.base_url, self.token, limit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| SdlcError::TelegramApi(format!("network error: {e}")))?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SdlcError::TelegramApi(
                "authentication failed: check your bot token".to_string(),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(SdlcError::TelegramApi(
                "rate limited (429): retry after the specified delay".to_string(),
            ));
        }
        if !status.is_success() {
            return Err(SdlcError::TelegramApi(format!(
                "unexpected HTTP status: {status}"
            )));
        }

        let body: ApiResponse<Vec<TelegramUpdate>> = response
            .json()
            .map_err(|e| SdlcError::TelegramApi(format!("failed to parse response: {e}")))?;

        if !body.ok {
            let desc = body
                .description
                .unwrap_or_else(|| "unknown error".to_string());
            return Err(SdlcError::TelegramApi(format!("API error: {desc}")));
        }

        Ok(body.result.unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_updates_json(messages: &[(&str, i64, &str)]) -> String {
        let items: Vec<String> = messages
            .iter()
            .enumerate()
            .map(|(i, (chat_title, chat_id, text))| {
                format!(
                    r#"{{
                        "update_id": {update_id},
                        "message": {{
                            "message_id": {msg_id},
                            "from": {{"id": 42, "first_name": "Alice", "is_bot": false}},
                            "chat": {{"id": {chat_id}, "title": "{chat_title}", "type": "supergroup"}},
                            "date": 1740000000,
                            "text": "{text}"
                        }}
                    }}"#,
                    update_id = 100 + i,
                    msg_id = 200 + i,
                )
            })
            .collect();
        format!(r#"{{"ok": true, "result": [{}]}}"#, items.join(","))
    }

    #[test]
    fn get_updates_success() {
        let mut server = mockito::Server::new();
        let body = make_updates_json(&[
            ("Project Alpha", -1001, "hello"),
            ("Dev Ops", -1002, "world"),
        ]);
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create();

        let client = TelegramClient::new("test-token").with_base_url(server.url());
        let updates = client.get_updates(100).unwrap();
        assert_eq!(updates.len(), 2);
        mock.assert();
    }

    #[test]
    fn get_updates_401_unauthorized() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(401)
            .with_body(r#"{"ok":false,"description":"Unauthorized"}"#)
            .create();

        let client = TelegramClient::new("bad-token").with_base_url(server.url());
        let err = client.get_updates(10).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("authentication failed") || msg.contains("401"),
            "unexpected: {msg}"
        );
        mock.assert();
    }

    #[test]
    fn get_updates_429_rate_limit() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(429)
            .with_body(r#"{"ok":false,"description":"Too Many Requests"}"#)
            .create();

        let client = TelegramClient::new("test-token").with_base_url(server.url());
        let err = client.get_updates(10).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("rate limited") || msg.contains("429"),
            "unexpected: {msg}"
        );
        mock.assert();
    }

    #[test]
    fn get_updates_malformed_json() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok": true, "result": "not-an-array"}"#)
            .create();

        let client = TelegramClient::new("test-token").with_base_url(server.url());
        let err = client.get_updates(10).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("parse") || msg.contains("Telegram"),
            "unexpected: {msg}"
        );
        mock.assert();
    }

    #[test]
    fn get_updates_api_ok_false() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok": false, "description": "Bot was blocked by the user"}"#)
            .create();

        let client = TelegramClient::new("test-token").with_base_url(server.url());
        let err = client.get_updates(10).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("blocked") || msg.contains("API error"),
            "unexpected: {msg}"
        );
        mock.assert();
    }
}
