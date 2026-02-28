use super::SdlcTool;
use std::path::Path;

pub struct MergeTool;

impl SdlcTool for MergeTool {
    fn name(&self) -> &str {
        "sdlc_merge"
    }

    fn description(&self) -> &str {
        "Finalize a feature in the Merge phase — transitions it to Released. Call when the directive action is 'merge'."
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug to merge"
                }
            },
            "required": ["slug"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;

        crate::cmd::merge::run(root, slug, true).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "slug": slug,
            "phase": "released",
            "merged": true
        }))
    }
}
