use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;

use super::codex_types::CodexEvent;
use super::AgentProvider;
use crate::error::AgentError;
use crate::types::{AgentEvent, PermissionMode, QueryOptions};

/// OpenAI Codex CLI provider.
#[derive(Debug, Clone, Default)]
pub struct CodexProvider;

impl AgentProvider for CodexProvider {
    fn spawn(
        &self,
        prompt: String,
        opts: QueryOptions,
        tx: mpsc::Sender<Result<AgentEvent, AgentError>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send>> {
        Box::pin(async move {
            let mut cmd = build_codex_command(&prompt, &opts);

            // Apply additional env vars
            for (k, v) in &opts.env {
                cmd.env(k, v);
            }

            cmd.stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().map_err(AgentError::Io)?;

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| AgentError::Process("stdout not captured".into()))?;

            // Drain stderr in background
            let stderr_buf = Arc::new(Mutex::new(String::new()));
            if let Some(stderr) = child.stderr.take() {
                let buf = Arc::clone(&stderr_buf);
                tokio::spawn(async move {
                    let mut reader = tokio::io::BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if let Ok(mut b) = buf.lock() {
                            if !b.is_empty() {
                                b.push('\n');
                            }
                            b.push_str(&line);
                        }
                    }
                });
            }

            let mut lines = tokio::io::BufReader::new(stdout).lines();
            let mut turn_count: u32 = 0;
            let mut total_input_tokens: u64 = 0;
            let mut total_output_tokens: u64 = 0;
            let mut accumulated_text = String::new();
            let mut got_failure = false;
            let mut failure_error: Option<String> = None;

            loop {
                match lines.next_line().await {
                    Err(e) => {
                        let _ = tx.send(Err(AgentError::Io(e))).await;
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some(line)) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let event: CodexEvent = match serde_json::from_str(trimmed) {
                            Ok(e) => e,
                            Err(_) => continue, // skip unparseable lines
                        };
                        if let Some(agent_event) = codex_event_to_agent_event(
                            &event,
                            &mut turn_count,
                            &mut accumulated_text,
                        ) {
                            if tx.send(Ok(agent_event)).await.is_err() {
                                break;
                            }
                        }
                        match &event {
                            CodexEvent::TurnCompleted { .. } => {
                                turn_count += 1;
                            }
                            CodexEvent::TurnFailed { error, .. } => {
                                got_failure = true;
                                failure_error = error.clone();
                            }
                            CodexEvent::TokenUsageUpdated {
                                input_tokens,
                                output_tokens,
                                ..
                            } => {
                                if let Some(it) = input_tokens {
                                    total_input_tokens = *it;
                                }
                                if let Some(ot) = output_tokens {
                                    total_output_tokens = *ot;
                                }
                            }
                            CodexEvent::ItemAgentMessageDelta { delta: Some(d) } => {
                                accumulated_text.push_str(d);
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Synthesize a Result event from accumulated state
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let result_event = AgentEvent::Result {
                is_error: got_failure,
                is_max_turns: false,
                text: accumulated_text,
                cost_usd: estimate_cost(total_input_tokens, total_output_tokens),
                turns: turn_count,
                session_id: None,
                stop_reason: if got_failure {
                    failure_error.or_else(|| Some("turn_failed".to_string()))
                } else {
                    Some("end_turn".to_string())
                },
                timestamp: ts,
            };
            let _ = tx.send(Ok(result_event)).await;

            // Check exit status
            let status = child.wait().await.map_err(AgentError::Io)?;
            if !status.success() {
                let stderr = stderr_buf
                    .lock()
                    .ok()
                    .map(|b| b.clone())
                    .unwrap_or_default();
                let msg = if let Some(code) = status.code() {
                    format!("Codex process exited with code {code}: {stderr}")
                } else {
                    format!("Codex process terminated by signal: {stderr}")
                };
                // Don't send error — we already sent a Result event
                tracing::warn!("{}", msg);
            }

            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "codex"
    }

    fn credential_env_var(&self) -> &'static str {
        "OPENAI_API_KEY"
    }
}

/// Build the `codex exec` command from QueryOptions.
fn build_codex_command(prompt: &str, opts: &QueryOptions) -> Command {
    let exe = opts.path_to_executable.as_deref().unwrap_or("codex");
    let mut cmd = Command::new(exe);

    cmd.arg("exec").arg("--json");

    match opts.permission_mode {
        PermissionMode::BypassPermissions => {
            cmd.arg("--dangerously-bypass-approvals-and-sandbox");
        }
        PermissionMode::AcceptEdits => {
            cmd.arg("--full-auto");
        }
        _ => {}
    }

    if let Some(model) = &opts.model {
        cmd.arg("--model").arg(model);
    }

    if !opts.mcp_servers.is_empty() {
        if let Ok(json) = build_mcp_config_json(&opts.mcp_servers) {
            cmd.arg("--mcp-config").arg(json);
        }
    }

    if let Some(resume) = &opts.resume {
        // Codex uses `codex exec resume <id>` to resume
        cmd.arg("resume").arg(resume);
    }

    // Prompt is a positional argument for codex
    cmd.arg(prompt);

    if let Some(cwd) = &opts.cwd {
        cmd.current_dir(cwd);
    }

    // Remove CODEX_UNSAFE_ALLOW_NO_SANDBOX defensively
    cmd.env_remove("CODEX_UNSAFE_ALLOW_NO_SANDBOX");

    cmd
}

/// Build MCP config JSON (same format as Claude).
fn build_mcp_config_json(servers: &[crate::types::McpServerConfig]) -> crate::Result<String> {
    let mut mcp_servers = serde_json::Map::new();
    for srv in servers {
        let mut cfg = serde_json::Map::new();
        cfg.insert("type".into(), serde_json::Value::String("stdio".into()));
        cfg.insert(
            "command".into(),
            serde_json::Value::String(srv.command.clone()),
        );
        if !srv.args.is_empty() {
            cfg.insert(
                "args".into(),
                serde_json::Value::Array(
                    srv.args
                        .iter()
                        .map(|a| serde_json::Value::String(a.clone()))
                        .collect(),
                ),
            );
        }
        if !srv.env.is_empty() {
            let env: serde_json::Map<String, serde_json::Value> = srv
                .env
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            cfg.insert("env".into(), serde_json::Value::Object(env));
        }
        mcp_servers.insert(srv.name.clone(), serde_json::Value::Object(cfg));
    }
    Ok(serde_json::json!({ "mcpServers": mcp_servers }).to_string())
}

/// Convert a Codex event into an `AgentEvent`, if it maps to one.
///
/// Some events (like `TurnCompleted`, `TokenUsageUpdated`) are accumulated
/// rather than emitted directly — they return `None`.
fn codex_event_to_agent_event(
    event: &CodexEvent,
    turn_count: &mut u32,
    _accumulated_text: &mut String,
) -> Option<AgentEvent> {
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    match event {
        CodexEvent::ThreadStarted { model, .. } => Some(AgentEvent::Init {
            model: model.clone().unwrap_or_else(|| "codex".to_string()),
            tools_count: 0,
            mcp_servers: vec![],
            timestamp: ts,
        }),
        CodexEvent::TurnStarted { turn_number } => Some(AgentEvent::Status {
            status: format!("turn {} started", turn_number.unwrap_or(*turn_count + 1)),
            timestamp: ts,
        }),
        CodexEvent::ItemCompleted {
            item_type, content, ..
        } => {
            let it = item_type.as_deref().unwrap_or("");
            match it {
                "agent_message" => {
                    let text = content
                        .as_ref()
                        .and_then(|v| v.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(AgentEvent::Assistant {
                        text,
                        tools: vec![],
                        thinking: vec![],
                        timestamp: ts,
                    })
                }
                "command_execution" => {
                    let output = content
                        .as_ref()
                        .and_then(|v| v.get("output"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let is_err = content
                        .as_ref()
                        .and_then(|v| v.get("exit_code"))
                        .and_then(|v| v.as_i64())
                        .map(|c| c != 0)
                        .unwrap_or(false);
                    Some(AgentEvent::User {
                        tool_results: vec![ToolResultEvent {
                            event_type: "tool_result".to_string(),
                            tool_use_id: "codex-cmd".to_string(),
                            is_error: is_err,
                            content: output,
                        }],
                        timestamp: ts,
                    })
                }
                _ => None,
            }
        }
        CodexEvent::ItemStarted { item_type, .. } => {
            let it = item_type.as_deref().unwrap_or("");
            if it == "command_execution" {
                Some(AgentEvent::ToolProgress {
                    tool: "command".to_string(),
                    elapsed_seconds: 0.0,
                    timestamp: ts,
                })
            } else {
                None
            }
        }
        CodexEvent::TurnFailed { error, .. } => Some(AgentEvent::Result {
            is_error: true,
            is_max_turns: false,
            text: error.clone().unwrap_or_default(),
            cost_usd: 0.0,
            turns: *turn_count,
            session_id: None,
            stop_reason: Some("turn_failed".to_string()),
            timestamp: ts,
        }),
        // Accumulated events — don't emit directly
        CodexEvent::TurnCompleted { .. }
        | CodexEvent::TokenUsageUpdated { .. }
        | CodexEvent::ItemAgentMessageDelta { .. }
        | CodexEvent::Unknown => None,
    }
}

use crate::types::ToolResultEvent;

/// Rough cost estimate from token counts (GPT-4.1 pricing).
fn estimate_cost(input_tokens: u64, output_tokens: u64) -> f64 {
    // Approximate GPT-4.1 pricing: $2/M input, $8/M output
    (input_tokens as f64 * 2.0 / 1_000_000.0) + (output_tokens as f64 * 8.0 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_started_maps_to_init() {
        let event = CodexEvent::ThreadStarted {
            thread_id: Some("t1".into()),
            model: Some("gpt-4.1".into()),
        };
        let mut turns = 0;
        let mut text = String::new();
        let result = codex_event_to_agent_event(&event, &mut turns, &mut text);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "init");
        assert_eq!(json["model"], "gpt-4.1");
    }

    #[test]
    fn turn_failed_maps_to_error_result() {
        let event = CodexEvent::TurnFailed {
            turn_number: Some(3),
            error: Some("sandbox violation".into()),
        };
        let mut turns = 2;
        let mut text = String::new();
        let result = codex_event_to_agent_event(&event, &mut turns, &mut text);
        assert!(result.is_some());
        let json = serde_json::to_value(result.unwrap()).unwrap();
        assert_eq!(json["type"], "result");
        assert_eq!(json["is_error"], true);
        assert_eq!(json["text"], "sandbox violation");
    }

    #[test]
    fn token_usage_returns_none() {
        let event = CodexEvent::TokenUsageUpdated {
            input_tokens: Some(100),
            output_tokens: Some(50),
            total_tokens: Some(150),
        };
        let mut turns = 0;
        let mut text = String::new();
        let result = codex_event_to_agent_event(&event, &mut turns, &mut text);
        assert!(result.is_none());
    }

    #[test]
    fn build_codex_command_basic() {
        let opts = QueryOptions {
            model: Some("gpt-4.1".into()),
            permission_mode: PermissionMode::BypassPermissions,
            ..Default::default()
        };
        let cmd = build_codex_command("hello world", &opts);
        let prog = cmd.as_std().get_program().to_str().unwrap();
        assert_eq!(prog, "codex");
        let args: Vec<_> = cmd
            .as_std()
            .get_args()
            .map(|a| a.to_str().unwrap())
            .collect();
        assert!(args.contains(&"exec"));
        assert!(args.contains(&"--json"));
        assert!(args.contains(&"--dangerously-bypass-approvals-and-sandbox"));
        assert!(args.contains(&"--model"));
        assert!(args.contains(&"gpt-4.1"));
        assert!(args.contains(&"hello world"));
    }
}
