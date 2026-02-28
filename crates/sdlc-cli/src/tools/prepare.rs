use super::SdlcTool;
use std::path::Path;

pub struct PrepareTool;

impl SdlcTool for PrepareTool {
    fn name(&self) -> &str {
        "sdlc_prepare"
    }

    fn description(&self) -> &str {
        "Survey a milestone — find gaps, organize features into parallelizable execution waves"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "milestone": {
                    "type": "string",
                    "description": "Milestone slug (auto-detects if omitted)"
                }
            },
            "required": []
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let milestone = args["milestone"].as_str();
        let result = sdlc_core::prepare::prepare(root, milestone).map_err(|e| e.to_string())?;
        serde_json::to_value(&result).map_err(|e| e.to_string())
    }
}
