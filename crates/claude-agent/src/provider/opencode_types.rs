use serde::Deserialize;

/// Response from `POST /session`.
#[derive(Debug, Deserialize)]
pub struct CreateSessionResponse {
    pub id: String,
}

/// Top-level SSE event from OpenCode's `/event` endpoint.
///
/// The SSE `event:` field contains the dotted type (e.g. `message.part.updated`),
/// and the `data:` field is JSON matching one of these variants.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OpenCodeEvent {
    #[serde(rename = "message.updated")]
    MessageUpdated {
        #[serde(default, rename = "sessionID")]
        session_id: Option<String>,
        message: Option<serde_json::Value>,
    },
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated {
        #[serde(default, rename = "sessionID")]
        session_id: Option<String>,
        part: Option<MessagePart>,
    },
    #[serde(rename = "session.idle")]
    SessionIdle {
        #[serde(default, rename = "sessionID")]
        session_id: Option<String>,
    },
    #[serde(rename = "permission.updated")]
    PermissionUpdated {
        #[serde(default, rename = "sessionID")]
        session_id: Option<String>,
        permission: Option<PermissionInfo>,
    },
    #[serde(other)]
    Unknown,
}

/// A single part of an OpenCode message.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePart {
    #[serde(rename = "text")]
    Text {
        #[serde(default)]
        content: Option<String>,
    },
    #[serde(rename = "tool-invocation")]
    ToolInvocation {
        #[serde(default, rename = "toolName")]
        tool_name: Option<String>,
        /// One of: pending, running, done, error
        #[serde(default)]
        state: Option<String>,
        #[serde(default)]
        args: Option<serde_json::Value>,
        #[serde(default)]
        result: Option<String>,
    },
    #[serde(rename = "tool-result")]
    ToolResult {
        #[serde(default)]
        content: Option<String>,
        #[serde(default, rename = "isError")]
        is_error: Option<bool>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        #[serde(default)]
        content: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

/// Permission request info from OpenCode.
#[derive(Debug, Deserialize)]
pub struct PermissionInfo {
    #[serde(default)]
    pub id: Option<String>,
    /// e.g. "pending", "granted", "denied"
    #[serde(default)]
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_idle() {
        let json = r#"{"type":"session.idle","sessionID":"s1"}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::SessionIdle { session_id } => {
                assert_eq!(session_id.as_deref(), Some("s1"));
            }
            _ => panic!("expected SessionIdle"),
        }
    }

    #[test]
    fn parse_message_part_text() {
        let json = r#"{"type":"message.part.updated","sessionID":"s1","part":{"type":"text","content":"hello"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::MessagePartUpdated {
                part: Some(MessagePart::Text { content }),
                ..
            } => {
                assert_eq!(content.as_deref(), Some("hello"));
            }
            _ => panic!("expected MessagePartUpdated with Text"),
        }
    }

    #[test]
    fn parse_tool_invocation() {
        let json = r#"{"type":"message.part.updated","sessionID":"s1","part":{"type":"tool-invocation","toolName":"Bash","state":"done","result":"ok"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::MessagePartUpdated {
                part:
                    Some(MessagePart::ToolInvocation {
                        tool_name,
                        state,
                        result,
                        ..
                    }),
                ..
            } => {
                assert_eq!(tool_name.as_deref(), Some("Bash"));
                assert_eq!(state.as_deref(), Some("done"));
                assert_eq!(result.as_deref(), Some("ok"));
            }
            _ => panic!("expected ToolInvocation"),
        }
    }

    #[test]
    fn unknown_event_doesnt_fail() {
        let json = r#"{"type":"file.edited","path":"foo.rs"}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, OpenCodeEvent::Unknown));
    }
}
