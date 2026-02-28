use super::SdlcTool;
use std::path::Path;

pub struct ProjectPhaseTool;

impl SdlcTool for ProjectPhaseTool {
    fn name(&self) -> &str {
        "sdlc_project_phase"
    }

    fn description(&self) -> &str {
        "Get the current project lifecycle phase (idle, pondering, planning, executing, verifying)"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn call(&self, _args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let phase = sdlc_core::prepare::project_phase(root).map_err(|e| e.to_string())?;
        serde_json::to_value(&phase).map_err(|e| e.to_string())
    }
}
