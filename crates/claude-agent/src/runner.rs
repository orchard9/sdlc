use futures::StreamExt;

use crate::stream::QueryStream;
use crate::{query, ClaudeAgentError, Message, QueryOptions, Result};

// ─── RunConfig ────────────────────────────────────────────────────────────

/// Configuration for a single agentic Claude run.
///
/// Pass to [`run`] to drive a query to completion and receive a [`RunResult`].
#[derive(Debug)]
pub struct RunConfig {
    /// System prompt override (replaces Claude's default system prompt).
    pub system_prompt: Option<String>,
    /// The user-facing prompt Claude will act on.
    pub prompt: String,
    /// Query options: model, MCP servers, permission mode, allowed tools, etc.
    pub opts: QueryOptions,
}

// ─── RunResult ────────────────────────────────────────────────────────────

/// The terminal result of a completed agentic run.
#[derive(Debug)]
pub struct RunResult {
    pub session_id: String,
    /// The final text Claude produced (empty string for error subtypes).
    pub result_text: String,
    pub total_cost_usd: f64,
    pub num_turns: u32,
    /// `true` if the run ended with any error subtype (max_turns, budget, etc.).
    pub is_error: bool,
}

// ─── Public API ───────────────────────────────────────────────────────────

/// Drive a single agentic Claude query to completion.
///
/// Merges `config.system_prompt` into `config.opts`, starts a [`QueryStream`],
/// consumes all messages, and returns the terminal result message as a
/// [`RunResult`].
///
/// Returns `Err` if the stream ends without a `Result` message (e.g., process
/// crashed) or if any message fails to parse.
///
/// # Example
///
/// ```rust,ignore
/// use claude_agent::runner::{RunConfig, run};
/// use claude_agent::QueryOptions;
///
/// let result = run(RunConfig {
///     system_prompt: None,
///     prompt: "say hello".into(),
///     opts: QueryOptions::default(),
/// }).await?;
/// println!("{}", result.result_text);
/// ```
pub async fn run(config: RunConfig) -> Result<RunResult> {
    let mut opts = config.opts;
    if let Some(sp) = config.system_prompt {
        opts.system_prompt = Some(sp);
    }
    collect(query(config.prompt, opts)).await
}

// ─── Internal ─────────────────────────────────────────────────────────────

/// Consume a [`QueryStream`] and extract the terminal [`RunResult`].
///
/// Exposed as `pub(crate)` so tests can inject mock streams directly without
/// spawning a real Claude subprocess.
pub(crate) async fn collect(stream: QueryStream) -> Result<RunResult> {
    let mut stream = stream;
    let mut run_result: Option<RunResult> = None;

    while let Some(msg) = stream.next().await {
        if let Message::Result(r) = msg? {
            run_result = Some(RunResult {
                session_id: r.session_id().to_string(),
                result_text: r.result_text().unwrap_or("").to_string(),
                total_cost_usd: r.total_cost_usd(),
                num_turns: r.num_turns(),
                is_error: r.is_error(),
            });
            // Result is the terminal message — no need to consume further.
            break;
        }
    }

    run_result
        .ok_or_else(|| ClaudeAgentError::Process("stream ended without a result message".into()))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    use crate::types::{
        ResultError, ResultMessage, ResultSuccess, ResultUsage, SystemInit, SystemMessage,
        SystemPayload,
    };

    fn success_msg(text: &str) -> Message {
        Message::Result(ResultMessage::Success(ResultSuccess {
            session_id: "s1".into(),
            result: text.to_string(),
            duration_ms: 10,
            duration_api_ms: 8,
            is_error: false,
            num_turns: 3,
            stop_reason: Some("end_turn".into()),
            total_cost_usd: 0.012,
            usage: ResultUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            uuid: None,
        }))
    }

    fn error_msg() -> Message {
        Message::Result(ResultMessage::ErrorMaxTurns(ResultError {
            session_id: "s2".into(),
            duration_ms: 10,
            duration_api_ms: 8,
            is_error: true,
            num_turns: 10,
            stop_reason: Some("max_turns".into()),
            total_cost_usd: 0.005,
            usage: ResultUsage {
                input_tokens: 50,
                output_tokens: 20,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            errors: vec![],
            uuid: None,
        }))
    }

    fn system_init_msg() -> Message {
        Message::System(SystemMessage {
            session_id: "s1".into(),
            payload: SystemPayload::Init(SystemInit {
                model: "claude-sonnet-4-6".into(),
                tools: vec![],
                mcp_servers: vec![],
                permission_mode: "default".into(),
                claude_code_version: "0.0.0".into(),
                cwd: "/tmp".into(),
                api_key_source: None,
                output_style: None,
                agents: vec![],
                skills: vec![],
                slash_commands: vec![],
                plugins: vec![],
                uuid: None,
                fast_mode_state: None,
            }),
        })
    }

    fn mock_stream(messages: Vec<Result<Message>>) -> QueryStream {
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            for msg in messages {
                if tx.send(msg).await.is_err() {
                    break;
                }
            }
        });
        QueryStream::from_channel(rx)
    }

    #[tokio::test]
    async fn collect_success_returns_result_text() {
        let stream = mock_stream(vec![Ok(success_msg("hello world"))]);
        let result = collect(stream).await.unwrap();
        assert_eq!(result.result_text, "hello world");
        assert_eq!(result.session_id, "s1");
        assert_eq!(result.num_turns, 3);
        assert!((result.total_cost_usd - 0.012).abs() < 1e-9);
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn collect_error_subtype_sets_is_error_true() {
        let stream = mock_stream(vec![Ok(error_msg())]);
        let result = collect(stream).await.unwrap();
        assert!(result.is_error);
        assert_eq!(result.session_id, "s2");
        assert_eq!(result.num_turns, 10);
        assert_eq!(result.result_text, ""); // error subtypes have no result text
    }

    #[tokio::test]
    async fn collect_no_result_message_returns_err() {
        let (tx, rx) = mpsc::channel::<Result<Message>>(1);
        drop(tx); // drop sender immediately so the stream closes with no messages
        let stream = QueryStream::from_channel(rx);
        let err = collect(stream).await;
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("result message"));
    }

    #[tokio::test]
    async fn collect_skips_non_result_messages() {
        let stream = mock_stream(vec![Ok(system_init_msg()), Ok(success_msg("done"))]);
        let result = collect(stream).await.unwrap();
        assert_eq!(result.result_text, "done");
    }

    #[tokio::test]
    async fn collect_propagates_parse_error() {
        let stream = mock_stream(vec![Err(ClaudeAgentError::Process(
            "injected error".into(),
        ))]);
        let err = collect(stream).await;
        assert!(err.is_err());
    }
}
