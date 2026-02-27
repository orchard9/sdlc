/// Round-trip deserialization tests for `Message` using representative
/// stream-json payloads captured from the Claude CLI protocol.
#[cfg(test)]
mod unit {
    use crate::types::{Message, ResultMessage, SystemPayload};

    fn parse(json: &str) -> Message {
        serde_json::from_str(json).expect("failed to parse message")
    }

    #[test]
    fn parse_system_init() {
        let json = r#"{
            "type": "system",
            "subtype": "init",
            "session_id": "abc-123",
            "model": "claude-sonnet-4-6",
            "tools": ["Read", "Bash", "Edit"],
            "mcp_servers": [{"name": "sdlc", "status": "connected"}],
            "permission_mode": "acceptEdits",
            "claude_code_version": "1.0.0",
            "cwd": "/tmp"
        }"#;
        let msg = parse(json);
        let Message::System(sys) = msg else {
            panic!("expected System")
        };
        assert_eq!(sys.session_id, "abc-123");
        let SystemPayload::Init(init) = sys.payload else {
            panic!("expected Init")
        };
        assert_eq!(init.model, "claude-sonnet-4-6");
        assert_eq!(init.tools.len(), 3);
        assert_eq!(init.mcp_servers[0].name, "sdlc");
    }

    #[test]
    fn parse_system_unknown_subtype() {
        let json = r#"{
            "type": "system",
            "subtype": "some_future_subtype",
            "session_id": "abc-123"
        }"#;
        let msg = parse(json);
        let Message::System(sys) = msg else {
            panic!("expected System")
        };
        assert!(matches!(sys.payload, SystemPayload::Unknown));
    }

    #[test]
    fn parse_result_success() {
        let json = r#"{
            "type": "result",
            "subtype": "success",
            "session_id": "abc-123",
            "result": "Done! I wrote the spec.",
            "duration_ms": 5000,
            "duration_api_ms": 4800,
            "is_error": false,
            "num_turns": 3,
            "stop_reason": "end_turn",
            "total_cost_usd": 0.0042,
            "usage": {
                "input_tokens": 1200,
                "output_tokens": 400
            }
        }"#;
        let msg = parse(json);
        let Message::Result(result) = msg else {
            panic!("expected Result")
        };
        assert!(!result.is_error());
        assert_eq!(result.session_id(), "abc-123");
        assert_eq!(result.result_text(), Some("Done! I wrote the spec."));
        assert_eq!(result.num_turns(), 3);
        assert!((result.total_cost_usd() - 0.0042).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_result_error_max_turns() {
        let json = r#"{
            "type": "result",
            "subtype": "error_max_turns",
            "session_id": "abc-123",
            "duration_ms": 10000,
            "duration_api_ms": 9500,
            "is_error": true,
            "num_turns": 10,
            "stop_reason": null,
            "total_cost_usd": 0.02,
            "usage": {"input_tokens": 5000, "output_tokens": 1000},
            "errors": ["Reached maximum turn limit"]
        }"#;
        let msg = parse(json);
        let Message::Result(result) = msg else {
            panic!("expected Result")
        };
        assert!(result.is_error());
        assert!(matches!(result, ResultMessage::ErrorMaxTurns(_)));
        assert_eq!(result.result_text(), None);
    }

    #[test]
    fn parse_assistant_message() {
        let json = r#"{
            "type": "assistant",
            "session_id": "abc-123",
            "parent_tool_use_id": null,
            "message": {
                "id": "msg_abc",
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "Let me read the file."},
                    {"type": "tool_use", "id": "tu_1", "name": "Read", "input": {"file_path": "/tmp/foo.txt"}}
                ],
                "model": "claude-sonnet-4-6",
                "stop_reason": "tool_use",
                "usage": {"input_tokens": 100, "output_tokens": 50}
            }
        }"#;
        let msg = parse(json);
        let Message::Assistant(asst) = msg else {
            panic!("expected Assistant")
        };
        assert_eq!(asst.session_id, "abc-123");
        assert_eq!(asst.message.content.len(), 2);
    }

    #[test]
    fn parse_tool_progress() {
        let json = r#"{
            "type": "tool_progress",
            "tool_use_id": "tu_1",
            "tool_name": "Bash",
            "parent_tool_use_id": null,
            "elapsed_time_seconds": 2.5,
            "session_id": "abc-123"
        }"#;
        let msg = parse(json);
        let Message::ToolProgress(tp) = msg else {
            panic!("expected ToolProgress")
        };
        assert_eq!(tp.tool_name, "Bash");
        assert!((tp.elapsed_time_seconds - 2.5).abs() < f64::EPSILON);
    }
}
