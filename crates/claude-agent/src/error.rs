use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClaudeAgentError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse stream-json line: {source}\n  line: {line}")]
    Parse {
        line: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Process error: {0}")]
    Process(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Session not found for slug: {0}")]
    SessionNotFound(String),
}
