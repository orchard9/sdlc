use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;

use super::opencode_types::{CreateSessionResponse, MessagePart, OpenCodeEvent, PermissionInfo};
use super::AgentProvider;
use crate::error::AgentError;
use crate::types::{AgentEvent, PermissionMode, QueryOptions, ThinkingBlock, ToolResultEvent};

/// OpenCode provider — communicates with a running OpenCode HTTP server
/// via REST + SSE (not subprocess JSONL like Claude/Codex).
#[derive(Debug, Clone, Default)]
pub struct OpenCodeProvider;

impl AgentProvider for OpenCodeProvider {
    fn spawn(
        &self,
        prompt: String,
        opts: QueryOptions,
        tx: mpsc::Sender<Result<AgentEvent, AgentError>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send>> {
        Box::pin(async move {
            let base_url = opts
                .env
                .get("OPENCODE_URL")
                .cloned()
                .unwrap_or_else(|| "http://localhost:3000".to_string());

            let client = reqwest::Client::new();
            let ts = || chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

            // Emit Init event
            let _ = tx
                .send(Ok(AgentEvent::Init {
                    model: opts.model.clone().unwrap_or_else(|| "opencode".to_string()),
                    tools_count: 0,
                    mcp_servers: vec![],
                    timestamp: ts(),
                }))
                .await;

            // Create session (or resume existing one)
            let session_id = if let Some(resume_id) = &opts.resume {
                resume_id.clone()
            } else {
                let resp = client
                    .post(format!("{base_url}/session"))
                    .json(&serde_json::json!({}))
                    .send()
                    .await
                    .map_err(|e| AgentError::Process(format!("failed to create session: {e}")))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    return Err(AgentError::Process(format!(
                        "POST /session failed ({status}): {body}"
                    )));
                }

                let session: CreateSessionResponse = resp
                    .json()
                    .await
                    .map_err(|e| AgentError::Process(format!("invalid session response: {e}")))?;
                session.id
            };

            // Connect to SSE stream BEFORE sending message
            let sse_resp = client
                .get(format!("{base_url}/event"))
                .header("Accept", "text/event-stream")
                .send()
                .await
                .map_err(|e| AgentError::Process(format!("failed to connect SSE: {e}")))?;

            if !sse_resp.status().is_success() {
                return Err(AgentError::Process(format!(
                    "GET /event failed ({})",
                    sse_resp.status()
                )));
            }

            // Send message
            let msg_resp = client
                .post(format!("{base_url}/session/{session_id}/message"))
                .json(&serde_json::json!({ "content": prompt }))
                .send()
                .await
                .map_err(|e| AgentError::Process(format!("failed to send message: {e}")))?;

            if !msg_resp.status().is_success() {
                let status = msg_resp.status();
                let body = msg_resp.text().await.unwrap_or_default();
                return Err(AgentError::Process(format!(
                    "POST /session/{session_id}/message failed ({status}): {body}"
                )));
            }

            // Process SSE stream
            let auto_grant = matches!(
                opts.permission_mode,
                PermissionMode::BypassPermissions | PermissionMode::AcceptEdits
            );
            let max_turns = opts.max_turns;

            let mut accumulated_text = String::new();
            let mut turn_count: u32 = 0;
            let mut got_error = false;
            let mut error_text: Option<String> = None;

            use futures::StreamExt;
            let mut byte_stream = sse_resp.bytes_stream();
            let mut line_buf = String::new();

            loop {
                match byte_stream.next().await {
                    None => break,
                    Some(Err(e)) => {
                        let _ = tx
                            .send(Err(AgentError::Process(format!("SSE read error: {e}"))))
                            .await;
                        break;
                    }
                    Some(Ok(chunk)) => {
                        let text = String::from_utf8_lossy(&chunk);
                        line_buf.push_str(&text);

                        // Process complete SSE frames (separated by blank lines)
                        while let Some(frame_end) = line_buf.find("\n\n") {
                            let frame = line_buf[..frame_end].to_string();
                            line_buf = line_buf[frame_end + 2..].to_string();

                            let (_event_type, data) = parse_sse_frame(&frame);
                            if data.is_empty() {
                                continue;
                            }

                            let event: OpenCodeEvent = match serde_json::from_str(&data) {
                                Ok(e) => e,
                                Err(_) => continue,
                            };

                            // Filter to our session
                            if !event_matches_session(&event, &session_id) {
                                continue;
                            }

                            match &event {
                                OpenCodeEvent::MessagePartUpdated {
                                    part: Some(part), ..
                                } => {
                                    if let Some(agent_event) =
                                        part_to_agent_event(part, &mut accumulated_text, &ts)
                                    {
                                        if tx.send(Ok(agent_event)).await.is_err() {
                                            return Ok(());
                                        }
                                    }
                                    // Track errors from tool invocations
                                    if let MessagePart::ToolInvocation {
                                        state: Some(s),
                                        result,
                                        ..
                                    } = part
                                    {
                                        if s == "error" {
                                            got_error = true;
                                            error_text = result.clone();
                                        }
                                    }
                                }
                                OpenCodeEvent::MessageUpdated { .. } => {
                                    // Full message snapshot — we use part updates instead
                                    turn_count += 1;
                                }
                                OpenCodeEvent::PermissionUpdated {
                                    permission: Some(perm),
                                    ..
                                } => {
                                    if auto_grant {
                                        auto_grant_permission(
                                            &client,
                                            &base_url,
                                            &session_id,
                                            perm,
                                        )
                                        .await;
                                    }
                                }
                                OpenCodeEvent::SessionIdle { .. } => {
                                    // Session done — break out
                                    break;
                                }
                                _ => {}
                            }

                            // Check max turns
                            if let Some(max) = max_turns {
                                if turn_count >= max {
                                    // Abort session
                                    let _ = client
                                        .post(format!("{base_url}/session/{session_id}/abort"))
                                        .send()
                                        .await;
                                    got_error = true;
                                    error_text = Some("max turns exceeded".to_string());
                                    break;
                                }
                            }
                        }

                        // Check if we got SessionIdle (inner break only breaks the while)
                        // Re-check: if we just processed a SessionIdle, the outer
                        // match arm broke out of the while-loop but not the outer loop.
                        // We need a flag.
                    }
                }
            }

            // Synthesize Result event
            let result_event = AgentEvent::Result {
                is_error: got_error,
                is_max_turns: max_turns.map(|m| turn_count >= m).unwrap_or(false),
                text: accumulated_text,
                cost_usd: 0.0, // OpenCode doesn't expose cost
                turns: turn_count,
                session_id: Some(session_id),
                stop_reason: if got_error {
                    error_text.or_else(|| Some("error".to_string()))
                } else {
                    Some("end_turn".to_string())
                },
                timestamp: ts(),
            };
            let _ = tx.send(Ok(result_event)).await;

            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "opencode"
    }

    fn credential_env_var(&self) -> &'static str {
        "OPENCODE_API_KEY"
    }
}

/// Parse an SSE frame into (event_type, data).
/// Frame format: `event: <type>\ndata: <json>\n` (one or more lines).
fn parse_sse_frame(frame: &str) -> (String, String) {
    let mut event_type = String::new();
    let mut data_lines = Vec::new();

    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_type = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim().to_string());
        } else if line.starts_with(':') {
            // SSE comment — skip
        }
    }

    (event_type, data_lines.join("\n"))
}

/// Check if an event belongs to the given session.
fn event_matches_session(event: &OpenCodeEvent, session_id: &str) -> bool {
    match event {
        OpenCodeEvent::MessageUpdated {
            session_id: sid, ..
        }
        | OpenCodeEvent::MessagePartUpdated {
            session_id: sid, ..
        }
        | OpenCodeEvent::SessionIdle {
            session_id: sid, ..
        }
        | OpenCodeEvent::PermissionUpdated {
            session_id: sid, ..
        } => sid.as_deref().is_none_or(|s| s == session_id),
        OpenCodeEvent::Unknown => true,
    }
}

/// Convert a message part into an AgentEvent.
fn part_to_agent_event(
    part: &MessagePart,
    accumulated_text: &mut String,
    ts: &dyn Fn() -> String,
) -> Option<AgentEvent> {
    match part {
        MessagePart::Text {
            content: Some(text),
        } => {
            accumulated_text.push_str(text);
            Some(AgentEvent::Assistant {
                text: text.clone(),
                tools: vec![],
                thinking: vec![],
                timestamp: ts(),
            })
        }
        MessagePart::ToolInvocation {
            tool_name,
            state: Some(state),
            result,
            ..
        } => {
            let name = tool_name.as_deref().unwrap_or("unknown").to_string();
            match state.as_str() {
                "running" | "pending" => Some(AgentEvent::ToolProgress {
                    tool: name,
                    elapsed_seconds: 0.0,
                    timestamp: ts(),
                }),
                "done" => Some(AgentEvent::User {
                    tool_results: vec![ToolResultEvent {
                        event_type: "tool_result".to_string(),
                        tool_use_id: name.clone(),
                        is_error: false,
                        content: result.clone().unwrap_or_default(),
                    }],
                    timestamp: ts(),
                }),
                "error" => Some(AgentEvent::User {
                    tool_results: vec![ToolResultEvent {
                        event_type: "tool_result".to_string(),
                        tool_use_id: name.clone(),
                        is_error: true,
                        content: result.clone().unwrap_or_else(|| "tool error".to_string()),
                    }],
                    timestamp: ts(),
                }),
                _ => None,
            }
        }
        MessagePart::Reasoning {
            content: Some(text),
        } => Some(AgentEvent::Assistant {
            text: String::new(),
            tools: vec![],
            thinking: vec![ThinkingBlock {
                block_type: "thinking".to_string(),
                thinking: text.clone(),
            }],
            timestamp: ts(),
        }),
        _ => None,
    }
}

/// Auto-grant a pending permission request.
async fn auto_grant_permission(
    client: &reqwest::Client,
    base_url: &str,
    session_id: &str,
    perm: &PermissionInfo,
) {
    let Some(perm_id) = &perm.id else { return };
    let status = perm.status.as_deref().unwrap_or("");
    if status != "pending" {
        return;
    }

    tracing::debug!(permission_id = %perm_id, "auto-granting permission");
    let _ = client
        .post(format!(
            "{base_url}/session/{session_id}/permissions/{perm_id}"
        ))
        .json(&serde_json::json!({ "grant": true }))
        .send()
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sse_frame_basic() {
        let frame = "event: message.part.updated\ndata: {\"type\":\"message.part.updated\",\"part\":{\"type\":\"text\",\"content\":\"hi\"}}";
        let (event_type, data) = parse_sse_frame(frame);
        assert_eq!(event_type, "message.part.updated");
        assert!(data.contains("\"content\":\"hi\""));
    }

    #[test]
    fn parse_sse_frame_with_comment() {
        let frame = ": keepalive\nevent: session.idle\ndata: {\"type\":\"session.idle\",\"sessionID\":\"s1\"}";
        let (event_type, data) = parse_sse_frame(frame);
        assert_eq!(event_type, "session.idle");
        assert!(data.contains("s1"));
    }

    #[test]
    fn text_part_maps_to_assistant() {
        let part = MessagePart::Text {
            content: Some("hello world".into()),
        };
        let mut acc = String::new();
        let ts = || "2026-01-01T00:00:00.000Z".to_string();
        let result = part_to_agent_event(&part, &mut acc, &ts);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "assistant");
        assert_eq!(json["text"], "hello world");
        assert_eq!(acc, "hello world");
    }

    #[test]
    fn tool_done_maps_to_user() {
        let part = MessagePart::ToolInvocation {
            tool_name: Some("Bash".into()),
            state: Some("done".into()),
            args: None,
            result: Some("output".into()),
        };
        let mut acc = String::new();
        let ts = || "t".to_string();
        let result = part_to_agent_event(&part, &mut acc, &ts);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "user");
        assert_eq!(json["tool_results"][0]["is_error"], false);
        assert_eq!(json["tool_results"][0]["content"], "output");
    }

    #[test]
    fn tool_error_maps_to_error_result() {
        let part = MessagePart::ToolInvocation {
            tool_name: Some("Read".into()),
            state: Some("error".into()),
            args: None,
            result: Some("file not found".into()),
        };
        let mut acc = String::new();
        let ts = || "t".to_string();
        let result = part_to_agent_event(&part, &mut acc, &ts);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "user");
        assert_eq!(json["tool_results"][0]["is_error"], true);
    }

    #[test]
    fn reasoning_maps_to_thinking() {
        let part = MessagePart::Reasoning {
            content: Some("thinking...".into()),
        };
        let mut acc = String::new();
        let ts = || "t".to_string();
        let result = part_to_agent_event(&part, &mut acc, &ts);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "assistant");
        assert_eq!(json["thinking"][0]["thinking"], "thinking...");
    }

    #[test]
    fn session_filter_matches_correct_session() {
        let event = OpenCodeEvent::SessionIdle {
            session_id: Some("s1".into()),
        };
        assert!(event_matches_session(&event, "s1"));
        assert!(!event_matches_session(&event, "s2"));
    }

    #[test]
    fn session_filter_matches_none_session() {
        let event = OpenCodeEvent::SessionIdle { session_id: None };
        assert!(event_matches_session(&event, "any"));
    }
}
