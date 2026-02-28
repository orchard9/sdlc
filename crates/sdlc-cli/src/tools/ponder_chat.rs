use super::SdlcTool;
use std::path::Path;

pub struct PonderChatTool;

impl SdlcTool for PonderChatTool {
    fn name(&self) -> &str {
        "sdlc_ponder_chat"
    }

    fn description(&self) -> &str {
        "Start a ponder session for an idea, optionally seeded with a message. \
         The agent will explore the idea, recruit thought partners, and write a session log."
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Ponder entry slug"
                },
                "message": {
                    "type": "string",
                    "description": "Seed message to start the session with. \
                                    If omitted, the agent opens a fresh session."
                }
            },
            "required": ["slug"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"].as_str().ok_or("slug is required")?;

        // Verify the ponder entry exists
        sdlc_core::ponder::PonderEntry::load(root, slug)
            .map_err(|e| format!("ponder entry '{}' not found: {}", slug, e))?;

        let session = sdlc_core::ponder::next_session_number(root, slug).unwrap_or(1);

        Ok(serde_json::json!({
            "status": "ready",
            "slug": slug,
            "next_session": session,
            "message": args["message"].as_str().unwrap_or(""),
            "instruction": format!(
                "Run `sdlc ponder session log {} --file /tmp/ponder-session-{}.md` \
                 at the end of your session to register the log.",
                slug, slug
            )
        }))
    }
}
