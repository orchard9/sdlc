use crate::tools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::Path;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 protocol types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct ToolContent {
    r#type: &'static str,
    text: String,
}

#[derive(Debug, Serialize)]
struct ToolCallResult {
    content: Vec<ToolContent>,
    #[serde(rename = "isError")]
    is_error: bool,
}

// ---------------------------------------------------------------------------
// Server loop
// ---------------------------------------------------------------------------

pub fn run(root: &Path) -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let tools = tools::all_tools();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let raw: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("parse error: {e}"),
                    }),
                };
                let mut out = stdout.lock();
                serde_json::to_writer(&mut out, &resp)?;
                writeln!(out)?;
                continue;
            }
        };

        // Notifications have no "id" key — do not respond
        if !raw
            .as_object()
            .map(|o| o.contains_key("id"))
            .unwrap_or(false)
        {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_value(raw) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32600,
                        message: format!("invalid request: {e}"),
                    }),
                };
                let mut out = stdout.lock();
                serde_json::to_writer(&mut out, &resp)?;
                writeln!(out)?;
                continue;
            }
        };

        let response = handle_request(&request, &tools, root);
        let mut out = stdout.lock();
        serde_json::to_writer(&mut out, &response)?;
        writeln!(out)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Request dispatch (pub for unit tests)
// ---------------------------------------------------------------------------

pub fn handle_request(
    req: &JsonRpcRequest,
    tools: &[Box<dyn tools::SdlcTool>],
    root: &Path,
) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0",
            id: req.id.clone(),
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "sdlc",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },

        "tools/list" => {
            let tool_list: Vec<Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name(),
                        "description": t.description(),
                        "inputSchema": t.schema()
                    })
                })
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0",
                id: req.id.clone(),
                result: Some(serde_json::json!({ "tools": tool_list })),
                error: None,
            }
        }

        "tools/call" => {
            let params = match &req.params {
                Some(p) => p,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: req.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "missing params".to_string(),
                        }),
                    };
                }
            };

            let tool_name = match params["name"].as_str() {
                Some(n) => n,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: req.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "missing tool name in params".to_string(),
                        }),
                    };
                }
            };

            let args = params.get("arguments").cloned().unwrap_or(Value::Null);

            match tools.iter().find(|t| t.name() == tool_name) {
                None => JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("tool not found: {tool_name}"),
                    }),
                },
                Some(tool) => {
                    let (text, is_error) = match tool.call(args, root) {
                        Ok(v) => (
                            serde_json::to_string_pretty(&v)
                                .unwrap_or_else(|e| format!("serialization error: {e}")),
                            false,
                        ),
                        Err(e) => (e, true),
                    };

                    let call_result = ToolCallResult {
                        content: vec![ToolContent {
                            r#type: "text",
                            text,
                        }],
                        is_error,
                    };

                    JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: req.id.clone(),
                        result: Some(
                            serde_json::to_value(&call_result)
                                .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})),
                        ),
                        error: None,
                    }
                }
            }
        }

        other => JsonRpcResponse {
            jsonrpc: "2.0",
            id: req.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("method not found: {other}"),
            }),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::{config::Config, feature::Feature, state::State};
    use tempfile::TempDir;

    fn setup(dir: &TempDir) {
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();
        let config = Config::new("test");
        std::fs::write(
            dir.path().join(".sdlc/config.yaml"),
            serde_yaml::to_string(&config).unwrap(),
        )
        .unwrap();
        let state = State::new("test");
        std::fs::write(
            dir.path().join(".sdlc/state.yaml"),
            serde_yaml::to_string(&state).unwrap(),
        )
        .unwrap();
    }

    fn make_req(id: i64, method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(Value::Number(id.into())),
            method: method.to_string(),
            params,
        }
    }

    #[test]
    fn initialize_returns_capabilities() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let tools = tools::all_tools();
        let req = make_req(
            1,
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0.0.1"}
            })),
        );

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(result["serverInfo"]["name"], "sdlc");
    }

    #[test]
    fn tools_list_returns_all_seven() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let tools = tools::all_tools();
        let req = make_req(2, "tools/list", Some(serde_json::json!({})));

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tool_list = result["tools"].as_array().unwrap();
        assert_eq!(tool_list.len(), 7);

        let names: Vec<&str> = tool_list
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"sdlc_get_directive"));
        assert!(names.contains(&"sdlc_write_artifact"));
        assert!(names.contains(&"sdlc_approve_artifact"));
        assert!(names.contains(&"sdlc_reject_artifact"));
        assert!(names.contains(&"sdlc_add_task"));
        assert!(names.contains(&"sdlc_complete_task"));
        assert!(names.contains(&"sdlc_add_comment"));
    }

    #[test]
    fn tools_call_unknown_tool_returns_error() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let tools = tools::all_tools();
        let req = make_req(
            3,
            "tools/call",
            Some(serde_json::json!({
                "name": "nonexistent_tool",
                "arguments": {}
            })),
        );

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn tools_call_get_directive_success() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();
        let tools = tools::all_tools();

        let req = make_req(
            4,
            "tools/call",
            Some(serde_json::json!({
                "name": "sdlc_get_directive",
                "arguments": {"slug": "my-feat"}
            })),
        );

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let content = &result["content"][0]["text"].as_str().unwrap();
        assert!(content.contains("my-feat"));
        assert_eq!(result["isError"], false);
    }

    #[test]
    fn tools_call_get_directive_error_returns_is_error_true() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let tools = tools::all_tools();

        let req = make_req(
            5,
            "tools/call",
            Some(serde_json::json!({
                "name": "sdlc_get_directive",
                "arguments": {"slug": "no-such-feature"}
            })),
        );

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
    }

    #[test]
    fn unknown_method_returns_method_not_found() {
        let dir = TempDir::new().unwrap();
        let tools = tools::all_tools();
        let req = make_req(6, "unknown/method", None);

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.result.is_none());
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("method not found"));
    }

    #[test]
    fn tools_call_missing_params_returns_error() {
        let dir = TempDir::new().unwrap();
        let tools = tools::all_tools();
        let req = make_req(7, "tools/call", None);

        let resp = handle_request(&req, &tools, dir.path());
        assert!(resp.result.is_none());
        assert_eq!(resp.error.unwrap().code, -32602);
    }
}
