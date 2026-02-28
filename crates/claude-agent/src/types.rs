use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Outer Message enum ───────────────────────────────────────────────────

/// Every message emitted by `claude --output-format stream-json`.
/// Discriminated by the JSON `"type"` field.
///
/// Source: `@anthropic-ai/claude-agent-sdk/sdk.d.ts` — `SDKMessage` union type.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    System(SystemMessage),
    Assistant(AssistantMessage),
    User(UserMessage),
    Result(ResultMessage),
    /// `stream_event` — partial assistant message chunks (--include-partial-messages)
    StreamEvent(StreamEventMessage),
    /// `tool_progress` — progress updates during tool execution
    ToolProgress(ToolProgressMessage),
    /// `tool_use_summary` — summary after tool calls complete
    ToolUseSummary(ToolUseSummaryMessage),
    /// `auth_status` — authentication status during session init
    AuthStatus(AuthStatusMessage),
}

impl Message {
    pub fn session_id(&self) -> &str {
        match self {
            Message::System(m) => &m.session_id,
            Message::Assistant(m) => &m.session_id,
            Message::User(m) => &m.session_id,
            Message::Result(m) => m.session_id(),
            Message::StreamEvent(m) => &m.session_id,
            Message::ToolProgress(m) => &m.session_id,
            Message::ToolUseSummary(m) => &m.session_id,
            Message::AuthStatus(m) => &m.session_id,
        }
    }

    /// Returns `Some(&ResultMessage)` if this is the terminal result message.
    pub fn as_result(&self) -> Option<&ResultMessage> {
        if let Message::Result(r) = self {
            Some(r)
        } else {
            None
        }
    }
}

// ─── System messages ──────────────────────────────────────────────────────

/// `type = "system"` — further distinguished by `subtype`.
///
/// Uses `#[serde(flatten)]` to allow the inner `SystemPayload` enum
/// (tagged by `subtype`) to consume remaining fields after `session_id`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemMessage {
    pub session_id: String,
    #[serde(flatten)]
    pub payload: SystemPayload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum SystemPayload {
    /// First message — contains model, tools, MCP servers, permission mode
    Init(SystemInit),
    /// Status update during session
    Status(SystemStatus),
    /// Compact context boundary (auto/manual compaction)
    CompactBoundary(CompactBoundaryPayload),
    /// Subtask started via the Task tool
    TaskStarted(TaskStartedPayload),
    /// Subtask progress update
    TaskProgress(TaskProgressPayload),
    /// Subtask completed/failed
    TaskNotification(TaskNotificationPayload),
    /// Any future/unknown system subtype — safe to ignore
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemInit {
    pub model: String,
    pub tools: Vec<String>,
    pub mcp_servers: Vec<McpServerStatus>,
    /// Permission mode — CLI sends camelCase (`permissionMode`)
    #[serde(alias = "permissionMode")]
    pub permission_mode: String,
    pub claude_code_version: String,
    pub cwd: String,
    // ── Fields added in Claude CLI 2.x ──
    #[serde(default, alias = "apiKeySource")]
    pub api_key_source: Option<String>,
    #[serde(default, alias = "outputStyle")]
    pub output_style: Option<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub slash_commands: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<serde_json::Value>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default, alias = "fastModeState")]
    pub fast_mode_state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpServerStatus {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactBoundaryPayload {
    pub compact_metadata: CompactMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactMetadata {
    pub trigger: String,
    pub pre_tokens: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskStartedPayload {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskProgressPayload {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    pub description: String,
    pub usage: TaskUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskNotificationPayload {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    pub status: String,
    pub output_file: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TaskUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskUsage {
    pub total_tokens: u64,
    pub tool_uses: u64,
    pub duration_ms: u64,
}

// ─── Assistant messages ───────────────────────────────────────────────────

/// `type = "assistant"` — the model's response, including content blocks.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantMessage {
    pub message: AssistantContent,
    pub parent_tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// The `BetaMessage` shape from Anthropic SDK, as it appears in stream-json.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantContent {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    pub usage: TokenUsage,
}

/// Content blocks within an assistant message.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        /// Tool inputs are schema-polymorphic (varies per tool), so Value is correct here.
        input: serde_json::Value,
    },
    Thinking {
        thinking: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
}

// ─── User messages ────────────────────────────────────────────────────────

/// `type = "user"` — typically tool results fed back to the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserMessage {
    pub message: UserContent,
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_synthetic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_replay: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserContent {
    pub role: String,
    pub content: Vec<UserContentBlock>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    Text {
        text: String,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<ToolResultContent>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContent {
    Text { text: String },
}

// ─── Result messages ──────────────────────────────────────────────────────

/// `type = "result"` — the terminal message in every query stream.
///
/// `subtype` distinguishes success from the various error conditions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum ResultMessage {
    Success(ResultSuccess),
    ErrorDuringExecution(ResultError),
    ErrorMaxTurns(ResultError),
    ErrorMaxBudgetUsd(ResultError),
    ErrorMaxStructuredOutputRetries(ResultError),
}

impl ResultMessage {
    pub fn session_id(&self) -> &str {
        match self {
            ResultMessage::Success(r) => &r.session_id,
            ResultMessage::ErrorDuringExecution(r)
            | ResultMessage::ErrorMaxTurns(r)
            | ResultMessage::ErrorMaxBudgetUsd(r)
            | ResultMessage::ErrorMaxStructuredOutputRetries(r) => &r.session_id,
        }
    }

    pub fn is_error(&self) -> bool {
        !matches!(self, ResultMessage::Success(_))
    }

    /// The final result text. `None` for error subtypes.
    pub fn result_text(&self) -> Option<&str> {
        if let ResultMessage::Success(r) = self {
            Some(&r.result)
        } else {
            None
        }
    }

    pub fn total_cost_usd(&self) -> f64 {
        match self {
            ResultMessage::Success(r) => r.total_cost_usd,
            ResultMessage::ErrorDuringExecution(r)
            | ResultMessage::ErrorMaxTurns(r)
            | ResultMessage::ErrorMaxBudgetUsd(r)
            | ResultMessage::ErrorMaxStructuredOutputRetries(r) => r.total_cost_usd,
        }
    }

    pub fn num_turns(&self) -> u32 {
        match self {
            ResultMessage::Success(r) => r.num_turns,
            ResultMessage::ErrorDuringExecution(r)
            | ResultMessage::ErrorMaxTurns(r)
            | ResultMessage::ErrorMaxBudgetUsd(r)
            | ResultMessage::ErrorMaxStructuredOutputRetries(r) => r.num_turns,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultSuccess {
    pub session_id: String,
    pub result: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub stop_reason: Option<String>,
    pub total_cost_usd: f64,
    pub usage: ResultUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultError {
    pub session_id: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub stop_reason: Option<String>,
    pub total_cost_usd: f64,
    pub usage: ResultUsage,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
}

// ─── Ancillary message types ──────────────────────────────────────────────

/// `type = "stream_event"` — partial chunks (only with --include-partial-messages).
/// We don't process partial chunks, but we must not fail to parse them.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamEventMessage {
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// `type = "tool_progress"` — emitted periodically while a tool is running.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolProgressMessage {
    pub tool_use_id: String,
    pub tool_name: String,
    pub parent_tool_use_id: Option<String>,
    pub elapsed_time_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// `type = "tool_use_summary"` — emitted after a batch of tool calls.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolUseSummaryMessage {
    pub summary: String,
    pub preceding_tool_use_ids: Vec<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// `type = "auth_status"` — authentication status (SSO flows, API key issues).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthStatusMessage {
    #[serde(rename = "isAuthenticating")]
    pub is_authenticating: bool,
    pub output: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

// ─── QueryOptions ─────────────────────────────────────────────────────────

/// Options for driving a Claude subprocess query.
///
/// Maps to the `Options` type in `@anthropic-ai/claude-agent-sdk/sdk.d.ts`.
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Claude model name (e.g. `"claude-sonnet-4-6"`)
    pub model: Option<String>,
    /// Maximum number of agentic turns before stopping with `error_max_turns`
    pub max_turns: Option<u32>,
    /// Maximum budget in USD before stopping with `error_max_budget_usd`
    pub max_budget_usd: Option<f64>,
    /// Effort level for reasoning depth
    pub effort: Option<Effort>,
    /// Tool names that are auto-approved without user prompting
    pub allowed_tools: Vec<String>,
    /// Tool names that are explicitly disallowed
    pub disallowed_tools: Vec<String>,
    /// Permission mode for tool execution
    pub permission_mode: PermissionMode,
    /// Override system prompt
    pub system_prompt: Option<String>,
    /// Text to append to the default system prompt
    pub append_system_prompt: Option<String>,
    /// Session ID to resume (loads conversation history)
    pub resume: Option<String>,
    /// Continue the most recent conversation
    pub continue_conversation: bool,
    /// Session ID for a specific conversation
    pub session_id: Option<String>,
    /// MCP servers to register for this session
    pub mcp_servers: Vec<McpServerConfig>,
    /// Working directory for the subprocess (default: current dir)
    pub cwd: Option<std::path::PathBuf>,
    /// Additional environment variables for the subprocess
    pub env: HashMap<String, String>,
    /// Additional working directories (`--add-dir`)
    pub additional_directories: Vec<String>,
    /// Custom path to the `claude` binary (default: `"claude"`)
    pub path_to_executable: Option<String>,
    /// Enable debug mode (`--debug`)
    pub debug: bool,
    /// Include partial/streaming messages (`--include-partial-messages`)
    pub include_partial_messages: bool,
    /// Disable session persistence (`--no-session-persistence`)
    pub no_session_persistence: bool,
}

/// Effort level for Claude reasoning depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Low,
    Medium,
    High,
    Max,
}

impl Effort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Max => "max",
        }
    }
}

/// Permission mode — controls how tool executions are authorized.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PermissionMode {
    /// Standard: prompts for dangerous operations
    #[default]
    Default,
    /// Auto-accept file edit operations
    AcceptEdits,
    /// Bypass all permission checks (requires allowDangerouslySkipPermissions)
    BypassPermissions,
    /// Planning mode — no actual tool execution
    Plan,
    /// Don't prompt; deny if not pre-approved
    DontAsk,
}

impl PermissionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionMode::Default => "default",
            PermissionMode::AcceptEdits => "acceptEdits",
            PermissionMode::BypassPermissions => "bypassPermissions",
            PermissionMode::Plan => "plan",
            PermissionMode::DontAsk => "dontAsk",
        }
    }
}

/// MCP server configuration for stdio transport (the most common case).
///
/// Maps to `McpStdioServerConfig` in the SDK.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Logical name for this server (used in tool names as `mcp__<name>__<tool>`)
    pub name: String,
    /// Executable to spawn
    pub command: String,
    /// Arguments for the executable
    pub args: Vec<String>,
    /// Additional environment variables for the server process
    pub env: HashMap<String, String>,
}
