//! `claude-agent` — native Rust driver for the Claude CLI subprocess.
//!
//! This crate implements the `--output-format stream-json` protocol used by
//! `@anthropic-ai/claude-agent-sdk`, as a first-class Rust library so the
//! `sdlc` workspace can call Claude without a Node.js runtime.
//!
//! # Architecture
//!
//! ```text
//! QueryOptions
//!     │
//!     ▼
//! ClaudeProcess   ← spawns `claude --input-format stream-json --output-format stream-json …`
//!     │              sends prompt via stdin, reads JSONL from stdout
//!     ▼
//! QueryStream     ← implements futures::Stream<Item = Result<Message>>
//!     │              background task + mpsc channel
//!     ▼
//! Message enum    ← fully typed from sdk.d.ts; no Value escape hatches
//! ```
//!
//! # Quick start
//!
//! ```rust,ignore
//! use claude_agent::{query, Message, QueryOptions};
//! use futures::StreamExt;
//!
//! let opts = QueryOptions {
//!     model: Some("claude-sonnet-4-6".into()),
//!     max_turns: Some(10),
//!     ..Default::default()
//! };
//!
//! let mut stream = query("Write a hello-world Rust function.", opts);
//! while let Some(msg) = stream.next().await {
//!     match msg? {
//!         Message::Result(r) => println!("{}", r.result_text().unwrap_or("")),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # Current status
//!
//! - Types (`types.rs`): ✅ Week 1
//! - Error types (`error.rs`): ✅ Week 1
//! - Process driver (`process.rs`): ✅ Week 2
//! - Async stream (`stream.rs`): ✅ Week 2
//! - Session persistence (`session.rs`): ✅ Week 2
//! - MCP tool infrastructure (`sdlc mcp` command + `tools/`): ✅ Week 3
//! - Agent runner (`runner.rs`): ✅ Week 4

pub mod error;
pub mod provider;
pub mod runner;
pub mod types;

pub(crate) mod process;
pub mod session;
pub mod stream;

#[cfg(test)]
mod tests;

pub use error::{AgentError, ClaudeAgentError};
pub use provider::claude::ClaudeProvider;
pub use provider::codex::CodexProvider;
pub use provider::opencode::OpenCodeProvider;
pub use provider::AgentProvider;
pub use runner::{run as agent_run, RunConfig, RunResult};
pub use session::SessionStore;
pub use stream::{AgentStream, QueryStream};
pub use types::{
    AgentEvent, AssistantContent, AssistantMessage, ContentBlock, Effort, McpServerConfig, Message,
    PermissionMode, QueryOptions, ResultError, ResultMessage, ResultSuccess, SystemMessage,
    SystemPayload, ThinkingBlock, TokenUsage, ToolCall, ToolResultEvent, UserMessage,
};

/// Convenience `Result` alias for this crate.
pub type Result<T> = std::result::Result<T, ClaudeAgentError>;

/// Drive a single agentic query against the Claude CLI.
///
/// Returns a [`QueryStream`] that yields [`Message`] values as they arrive
/// from the subprocess. The stream terminates after the first
/// [`Message::Result`] or on process exit.
///
/// # Example
///
/// ```rust,ignore
/// use claude_agent::{query, Message, QueryOptions};
/// use futures::StreamExt;
///
/// let stream = query("say hello", QueryOptions::default());
/// let messages: Vec<_> = stream.collect().await;
/// ```
pub fn query(prompt: impl Into<String>, opts: QueryOptions) -> QueryStream {
    QueryStream::new(prompt.into(), opts)
}

/// Drive a query using a specific [`AgentProvider`].
///
/// Returns an [`AgentStream`] that yields provider-neutral [`AgentEvent`]
/// values. Use this when you need to support multiple agent backends.
pub fn query_with(
    prompt: impl Into<String>,
    opts: QueryOptions,
    provider: &dyn AgentProvider,
) -> AgentStream {
    AgentStream::new(prompt.into(), opts, provider)
}
