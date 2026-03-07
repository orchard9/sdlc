// Re-export all Claude-specific message types from the main types module.
// These are kept in `types.rs` for backward compatibility — this module
// simply re-exports them so provider code can import from a consistent path.
pub use crate::types::{
    AssistantContent, AssistantMessage, AuthStatusMessage, CompactBoundaryPayload, CompactMetadata,
    ContentBlock, Message, ResultError, ResultMessage, ResultSuccess, ResultUsage,
    StreamEventMessage, SystemInit, SystemMessage, SystemPayload, SystemStatus,
    TaskNotificationPayload, TaskProgressPayload, TaskStartedPayload, TaskUsage, TokenUsage,
    ToolProgressMessage, ToolResultContent, ToolUseSummaryMessage, UserContent, UserContentBlock,
    UserMessage,
};
