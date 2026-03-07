pub mod claude;
pub mod claude_types;
pub mod codex;
pub mod codex_types;
pub mod opencode;
pub mod opencode_types;

use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;

use crate::error::AgentError;
use crate::types::{AgentEvent, QueryOptions};

/// Abstraction over agent CLI backends (Claude, Codex, etc.).
///
/// Each provider knows how to spawn its CLI, map its native event stream
/// into the provider-neutral [`AgentEvent`] vocabulary, and report which
/// environment variable carries its credential.
pub trait AgentProvider: Send + Sync + 'static {
    /// Spawn the agent process, converting native events into [`AgentEvent`]s
    /// sent through `tx`. The future resolves when the process exits.
    fn spawn(
        &self,
        prompt: String,
        opts: QueryOptions,
        tx: mpsc::Sender<Result<AgentEvent, AgentError>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AgentError>> + Send>>;

    /// Human-readable provider name (e.g. `"claude"`, `"codex"`).
    fn name(&self) -> &'static str;

    /// Environment variable used to inject credentials (e.g.
    /// `"CLAUDE_CODE_OAUTH_TOKEN"` or `"OPENAI_API_KEY"`).
    fn credential_env_var(&self) -> &'static str;
}
