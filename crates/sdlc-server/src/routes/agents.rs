use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct AgentFrontmatter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    tools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentDefinition {
    name: String,
    description: String,
    model: String,
    tools: Vec<String>,
    content: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a Claude agent `.md` file into an `AgentDefinition`.
///
/// Agent files have YAML frontmatter between `---` delimiters followed by
/// the system-prompt body (Markdown).
fn parse_agent_file(stem: &str, raw: &str) -> AgentDefinition {
    // Try to split off frontmatter.  Must start with "---\n".
    let (frontmatter, body) = if let Some(rest) = raw.strip_prefix("---\n") {
        // Find the closing "---" line
        if let Some(pos) = rest.find("\n---\n") {
            let fm = &rest[..pos];
            let body_start = pos + "\n---\n".len();
            (fm, rest[body_start..].trim_start().to_string())
        } else if let Some(pos) = rest.find("\n---") {
            // trailing "---" at EOF without trailing newline
            let fm = &rest[..pos];
            let body_start = pos + "\n---".len();
            (fm, rest[body_start..].trim_start().to_string())
        } else {
            ("", raw.to_string())
        }
    } else {
        ("", raw.to_string())
    };

    let mut fm: AgentFrontmatter = if frontmatter.is_empty() {
        AgentFrontmatter::default()
    } else {
        serde_yaml::from_str(frontmatter).unwrap_or_default()
    };

    // Fall back to the filename stem if `name` was absent or empty.
    if fm.name.is_empty() {
        fm.name = stem.to_string();
    }

    AgentDefinition {
        name: fm.name,
        description: fm.description,
        model: fm.model,
        tools: fm.tools,
        content: body,
    }
}

fn validate_agent_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(format!(
            "Invalid agent name '{name}': must contain only letters, digits, hyphens, and underscores"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /api/agents — list all agents from ~/.claude/agents/
// ---------------------------------------------------------------------------

pub async fn list_agents() -> Result<Json<serde_json::Value>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let agents_dir = match sdlc_core::paths::user_claude_agents_dir() {
            Ok(d) => d,
            Err(_) => return Ok::<_, anyhow::Error>(serde_json::json!([])),
        };

        if !agents_dir.is_dir() {
            return Ok(serde_json::json!([]));
        }

        let mut agents: Vec<serde_json::Value> = Vec::new();

        let mut entries: Vec<_> = std::fs::read_dir(&agents_dir)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            // Only process .md files
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            match std::fs::read_to_string(&path) {
                Ok(raw) => {
                    let agent = parse_agent_file(&stem, &raw);
                    match serde_json::to_value(&agent) {
                        Ok(v) => agents.push(v),
                        Err(e) => warn!(agent = %stem, error = %e, "failed to serialize agent"),
                    }
                }
                Err(e) => warn!(agent = %stem, error = %e, "failed to read agent file"),
            }
        }

        Ok(serde_json::json!(agents))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/project/agents — list agents from <project_root>/.claude/agents/
// ---------------------------------------------------------------------------

pub async fn list_project_agents(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let agents_dir = sdlc_core::paths::project_claude_agents_dir(&root);

        if !agents_dir.is_dir() {
            return Ok::<_, anyhow::Error>(serde_json::json!([]));
        }

        let mut agents: Vec<serde_json::Value> = Vec::new();

        let mut entries: Vec<_> = std::fs::read_dir(&agents_dir)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            match std::fs::read_to_string(&path) {
                Ok(raw) => {
                    let agent = parse_agent_file(&stem, &raw);
                    match serde_json::to_value(&agent) {
                        Ok(v) => agents.push(v),
                        Err(e) => warn!(agent = %stem, error = %e, "failed to serialize agent"),
                    }
                }
                Err(e) => warn!(agent = %stem, error = %e, "failed to read agent file"),
            }
        }

        Ok(serde_json::json!(agents))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/agents/:name — single agent
// ---------------------------------------------------------------------------

pub async fn get_agent(Path(name): Path<String>) -> Result<Json<AgentDefinition>, AppError> {
    validate_agent_name(&name)?;

    let result = tokio::task::spawn_blocking(move || {
        let agents_dir = sdlc_core::paths::user_claude_agents_dir()
            .map_err(|e| AppError(anyhow::anyhow!("home dir not found: {e}")))?;

        let path = agents_dir.join(format!("{name}.md"));
        if !path.exists() {
            return Err(AppError::not_found(format!("agent '{name}' not found")));
        }

        let raw = std::fs::read_to_string(&path)
            .map_err(|e| AppError(anyhow::anyhow!("failed to read agent file: {e}")))?;

        Ok(parse_agent_file(&name, &raw))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_agent_with_frontmatter() {
        let raw = "---\nname: test-agent\ndescription: A test agent\nmodel: claude-sonnet-4-6\ntools:\n  - Read\n  - Write\n---\n\n# Test Agent\n\nSystem prompt here.";
        let agent = parse_agent_file("test-agent", raw);
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.description, "A test agent");
        assert_eq!(agent.model, "claude-sonnet-4-6");
        assert_eq!(agent.tools, vec!["Read", "Write"]);
        assert!(agent.content.contains("# Test Agent"));
    }

    #[test]
    fn parse_agent_falls_back_to_stem_when_name_missing() {
        let raw = "---\ndescription: No name field\n---\n\nContent.";
        let agent = parse_agent_file("my-fallback", raw);
        assert_eq!(agent.name, "my-fallback");
    }

    #[test]
    fn parse_agent_no_frontmatter() {
        let raw = "# Just content\n\nNo frontmatter here.";
        let agent = parse_agent_file("bare", raw);
        assert_eq!(agent.name, "bare");
        assert!(agent.content.contains("# Just content"));
    }

    #[test]
    fn validate_agent_name_accepts_valid() {
        assert!(validate_agent_name("kai-hoffmann").is_ok());
        assert!(validate_agent_name("brand_alchemist").is_ok());
        assert!(validate_agent_name("Agent123").is_ok());
    }

    #[test]
    fn validate_agent_name_rejects_invalid() {
        assert!(validate_agent_name("").is_err());
        assert!(validate_agent_name("bad name").is_err());
        assert!(validate_agent_name("../traversal").is_err());
        assert!(validate_agent_name("agent.md").is_err());
    }
}
