use serde::Deserialize;

/// Events emitted by `codex exec --json` as JSONL on stdout.
///
/// Based on the OpenAI Codex CLI source — each line is a JSON object
/// with a `type` field discriminating the event kind.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexEvent {
    /// Session initialized.
    ThreadStarted {
        #[serde(default)]
        thread_id: Option<String>,
        #[serde(default)]
        model: Option<String>,
    },
    /// A new agent turn began.
    TurnStarted {
        #[serde(default)]
        turn_number: Option<u32>,
    },
    /// A turn completed successfully.
    TurnCompleted {
        #[serde(default)]
        turn_number: Option<u32>,
    },
    /// A turn failed.
    TurnFailed {
        #[serde(default)]
        turn_number: Option<u32>,
        #[serde(default)]
        error: Option<String>,
    },
    /// An item (message, command, file change) started processing.
    ItemStarted {
        #[serde(default)]
        item_type: Option<String>,
        #[serde(default)]
        item_id: Option<String>,
    },
    /// An item completed.
    ItemCompleted {
        #[serde(default)]
        item_type: Option<String>,
        #[serde(default)]
        item_id: Option<String>,
        #[serde(default)]
        content: Option<serde_json::Value>,
    },
    /// Streaming text delta from the agent.
    ItemAgentMessageDelta {
        #[serde(default)]
        delta: Option<String>,
    },
    /// Token usage update.
    TokenUsageUpdated {
        #[serde(default)]
        input_tokens: Option<u64>,
        #[serde(default)]
        output_tokens: Option<u64>,
        #[serde(default)]
        total_tokens: Option<u64>,
    },
    /// Catch-all for unrecognized event types.
    #[serde(other)]
    Unknown,
}
