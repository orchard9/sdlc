use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;

use super::AgentProvider;
use crate::error::AgentError;
use crate::process::ClaudeProcess;
use crate::types::{
    AgentEvent, ContentBlock, Message, QueryOptions, ResultMessage, SystemPayload, ThinkingBlock,
    ToolCall, ToolResultContent, ToolResultEvent, UserContentBlock,
};

/// Maximum characters for tool result content in events.
const DISPLAY_TRUNCATE_CHARS: usize = 2000;

/// Claude Code CLI provider.
#[derive(Debug, Clone, Default)]
pub struct ClaudeProvider;

impl AgentProvider for ClaudeProvider {
    fn spawn(
        &self,
        prompt: String,
        opts: QueryOptions,
        tx: mpsc::Sender<Result<AgentEvent, AgentError>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send>> {
        Box::pin(async move {
            let mut process = ClaudeProcess::spawn(&prompt, &opts).await?;

            let mut got_result = false;
            loop {
                match process.next_message().await {
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some(msg)) => {
                        let is_terminal = matches!(msg, Message::Result(_));
                        if is_terminal {
                            got_result = true;
                        }
                        let event = claude_message_to_event(&msg);
                        if tx.send(Ok(event)).await.is_err() {
                            break;
                        }
                        if is_terminal {
                            break;
                        }
                    }
                }
            }

            if !got_result {
                if let Some(exit_err) = process.wait_exit_error().await {
                    let _ = tx.send(Err(exit_err)).await;
                }
            }

            process.kill().await;
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "claude"
    }

    fn credential_env_var(&self) -> &'static str {
        "CLAUDE_CODE_OAUTH_TOKEN"
    }
}

/// Truncate text by character count (not bytes), preserving valid UTF-8.
fn truncate_chars(input: &str, max_chars: usize) -> String {
    match input.char_indices().nth(max_chars) {
        Some((idx, _)) => input[..idx].to_string(),
        None => input.to_string(),
    }
}

/// Convert a Claude CLI `Message` into a provider-neutral `AgentEvent`.
///
/// This is the logic previously in `message_to_event()` in `runs.rs`,
/// moved here so the conversion happens at the provider boundary.
pub fn claude_message_to_event(msg: &Message) -> AgentEvent {
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    match msg {
        Message::System(sys) => match &sys.payload {
            SystemPayload::Init(init) => AgentEvent::Init {
                model: init.model.clone(),
                tools_count: init.tools.len(),
                mcp_servers: init.mcp_servers.iter().map(|s| s.name.clone()).collect(),
                timestamp: ts,
            },
            SystemPayload::Status(status) => AgentEvent::Status {
                status: status.status.clone(),
                timestamp: ts,
            },
            SystemPayload::TaskStarted(t) => AgentEvent::SubagentStarted {
                task_id: t.task_id.clone(),
                tool_use_id: t.tool_use_id.clone(),
                description: t.description.clone(),
                timestamp: ts,
            },
            SystemPayload::TaskProgress(t) => AgentEvent::SubagentProgress {
                task_id: t.task_id.clone(),
                last_tool_name: t.last_tool_name.clone(),
                total_tokens: t.usage.total_tokens,
                tool_uses: t.usage.tool_uses,
                duration_ms: t.usage.duration_ms,
                timestamp: ts,
            },
            SystemPayload::TaskNotification(t) => AgentEvent::SubagentCompleted {
                task_id: t.task_id.clone(),
                status: t.status.clone(),
                summary: t.summary.clone(),
                total_tokens: t.usage.as_ref().map(|u| u.total_tokens),
                duration_ms: t.usage.as_ref().map(|u| u.duration_ms),
                timestamp: ts,
            },
            _ => AgentEvent::System { timestamp: ts },
        },
        Message::Assistant(asst) => {
            let text: String = asst
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let ContentBlock::Text { text } = c {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            let tools: Vec<ToolCall> = asst
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let ContentBlock::ToolUse { name, input, .. } = c {
                        Some(ToolCall {
                            name: name.clone(),
                            input: input.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            let thinking: Vec<ThinkingBlock> = asst
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let ContentBlock::Thinking { thinking } = c {
                        Some(ThinkingBlock {
                            block_type: "thinking".to_string(),
                            thinking: thinking.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            AgentEvent::Assistant {
                text,
                tools,
                thinking,
                timestamp: ts,
            }
        }
        Message::User(user) => {
            let tool_results: Vec<ToolResultEvent> = user
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let UserContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } = c
                    {
                        let text = content
                            .as_ref()
                            .and_then(|blocks| {
                                blocks
                                    .iter()
                                    .map(|b| {
                                        let ToolResultContent::Text { text } = b;
                                        text.as_str()
                                    })
                                    .next()
                            })
                            .unwrap_or("");
                        let truncated = truncate_chars(text, DISPLAY_TRUNCATE_CHARS);
                        Some(ToolResultEvent {
                            event_type: "tool_result".to_string(),
                            tool_use_id: tool_use_id.clone(),
                            is_error: is_error.unwrap_or(false),
                            content: truncated,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            AgentEvent::User {
                tool_results,
                timestamp: ts,
            }
        }
        Message::Result(r) => AgentEvent::Result {
            is_error: r.is_error(),
            is_max_turns: matches!(r, ResultMessage::ErrorMaxTurns(_)),
            text: r.result_text().unwrap_or("").to_string(),
            cost_usd: r.total_cost_usd(),
            turns: r.num_turns(),
            session_id: Some(r.session_id().to_string()),
            stop_reason: r.stop_reason().map(|s| s.to_string()),
            timestamp: ts,
        },
        Message::ToolProgress(tp) => AgentEvent::ToolProgress {
            tool: tp.tool_name.clone(),
            elapsed_seconds: tp.elapsed_time_seconds,
            timestamp: ts,
        },
        Message::ToolUseSummary(ts_msg) => AgentEvent::ToolSummary {
            summary: ts_msg.summary.clone(),
            timestamp: ts,
        },
        Message::StreamEvent(_) => AgentEvent::StreamEvent { timestamp: ts },
        Message::AuthStatus(auth) => AgentEvent::AuthStatus {
            is_authenticating: auth.is_authenticating,
            timestamp: ts,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AssistantContent, AssistantMessage, ResultSuccess, ResultUsage, TokenUsage,
    };

    fn make_result_message() -> Message {
        Message::Result(ResultMessage::Success(ResultSuccess {
            session_id: "s1".to_string(),
            result: "done".to_string(),
            duration_ms: 100,
            duration_api_ms: 80,
            is_error: false,
            num_turns: 3,
            stop_reason: Some("end_turn".to_string()),
            total_cost_usd: 0.01,
            usage: ResultUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            uuid: None,
        }))
    }

    #[test]
    fn result_event_has_all_fields() {
        let msg = make_result_message();
        let event = claude_message_to_event(&msg);
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "result");
        assert_eq!(json["is_error"], false);
        assert_eq!(json["is_max_turns"], false);
        assert_eq!(json["text"], "done");
        assert_eq!(json["turns"], 3);
        assert!(json["timestamp"].as_str().unwrap().ends_with('Z'));
        assert_eq!(json["session_id"], "s1");
        assert_eq!(json["stop_reason"], "end_turn");
    }

    #[test]
    fn assistant_event_extracts_tools_and_thinking() {
        let msg = Message::Assistant(AssistantMessage {
            message: AssistantContent {
                id: "msg_1".into(),
                role: "assistant".into(),
                content: vec![
                    ContentBlock::Text {
                        text: "hello".into(),
                    },
                    ContentBlock::ToolUse {
                        id: "tu_1".into(),
                        name: "Read".into(),
                        input: serde_json::json!({"file": "foo.txt"}),
                    },
                    ContentBlock::Thinking {
                        thinking: "hmm".into(),
                    },
                ],
                model: "claude-sonnet-4-6".into(),
                stop_reason: None,
                usage: TokenUsage {
                    input_tokens: 10,
                    output_tokens: 5,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            },
            parent_tool_use_id: None,
            error: None,
            session_id: "s1".into(),
            uuid: None,
        });
        let event = claude_message_to_event(&msg);
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "assistant");
        assert_eq!(json["text"], "hello");
        assert_eq!(json["tools"].as_array().unwrap().len(), 1);
        assert_eq!(json["tools"][0]["name"], "Read");
        assert_eq!(json["thinking"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn max_turns_event_sets_is_max_turns() {
        let msg = Message::Result(ResultMessage::ErrorMaxTurns(crate::types::ResultError {
            session_id: "s1".into(),
            duration_ms: 100,
            duration_api_ms: 80,
            is_error: true,
            num_turns: 50,
            stop_reason: Some("max_turns".into()),
            total_cost_usd: 0.05,
            usage: ResultUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            errors: vec![],
            uuid: None,
        }));
        let event = claude_message_to_event(&msg);
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["is_max_turns"], true);
        assert_eq!(json["is_error"], true);
    }
}
