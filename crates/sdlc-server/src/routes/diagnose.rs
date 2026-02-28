use axum::{extract::State, Json};
use claude_agent::{query, Message, PermissionMode, QueryOptions};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt as _;

use crate::{error::AppError, state::AppState};

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DiagnoseRequest {
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnoseResult {
    pub title: String,
    pub problem_statement: String,
    pub root_cause: String,
    pub files_affected: Vec<String>,
    /// "high" | "medium" | "low" | "none"
    /// "none" means the agent determined the input is not a software issue.
    pub confidence: String,
}

// ---------------------------------------------------------------------------
// Agent options — filesystem-only, no sdlc MCP
// ---------------------------------------------------------------------------

fn diagnose_options(root: std::path::PathBuf) -> QueryOptions {
    QueryOptions {
        permission_mode: PermissionMode::AcceptEdits,
        mcp_servers: vec![],
        allowed_tools: vec!["Bash".into(), "Read".into(), "Glob".into(), "Grep".into()],
        cwd: Some(root),
        max_turns: Some(20),
        no_session_persistence: true,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// JSON extraction helpers
// ---------------------------------------------------------------------------

/// Pull the outermost `{ ... }` from an arbitrary string and parse it.
fn extract_json(text: &str) -> Option<serde_json::Value> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

/// Best-effort title from raw description when the agent output can't be parsed.
fn slug_from_description(description: &str) -> String {
    let words: Vec<&str> = description
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .take(6)
        .collect();
    if words.is_empty() {
        "fix-issue".to_string()
    } else {
        words.join("-").to_lowercase()
    }
}

fn fallback_result(description: &str) -> DiagnoseResult {
    DiagnoseResult {
        title: slug_from_description(description),
        problem_statement: description.chars().take(500).collect(),
        root_cause: "Could not automatically determine root cause.".to_string(),
        files_affected: vec![],
        confidence: "low".to_string(),
    }
}

fn parse_result(json: &serde_json::Value, description: &str) -> DiagnoseResult {
    let title = json["title"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| slug_from_description(description));

    let problem_statement = json["problem_statement"]
        .as_str()
        .unwrap_or(description)
        .to_string();

    let root_cause = json["root_cause"]
        .as_str()
        .unwrap_or("Unknown root cause.")
        .to_string();

    let files_affected = json["files_affected"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let confidence = json["confidence"].as_str().unwrap_or("low").to_string();

    DiagnoseResult {
        title,
        problem_statement,
        root_cause,
        files_affected,
        confidence,
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

const DIAGNOSE_TIMEOUT_SECS: u64 = 60;

/// POST /api/diagnose — run a lightweight agent to triage an issue before
/// creating a feature. The agent reads relevant files from the codebase,
/// forms a root-cause hypothesis, and returns structured JSON.
///
/// The request body is `{ "description": "..." }` — any text the user pasted
/// (stack trace, vague description, or anything else). The agent determines
/// what to do with it; random/non-software input yields `confidence: "none"`.
pub async fn diagnose(
    State(app): State<AppState>,
    Json(body): Json<DiagnoseRequest>,
) -> Result<Json<DiagnoseResult>, AppError> {
    let description = body.description.trim().to_string();
    if description.is_empty() {
        return Err(AppError::bad_request("description is required"));
    }

    let opts = diagnose_options(app.root.clone());

    // Braces in the JSON schema must be doubled to escape Rust's format! macro.
    let prompt = format!(
        r#"You are a senior engineer doing fast bug triage on a codebase.

The user submitted this to "Fix Right Away":
---
{description}
---

STEPS:
1. Assess what was given:
   - Real error or stack trace with file paths → read those files (up to 5), search for root cause
   - Vague description → search the codebase for relevant code (up to 5 searches), form a hypothesis
   - Completely unrelated to software (random text, not a bug) → set confidence to "none", skip file reads

2. Form a clear hypothesis about the root cause.

3. Output ONLY a raw JSON object — no markdown, no code fences, no prose before or after:
{{
  "title": "short-imperative-title-slug-style (4-7 words, lowercase, hyphen-separated, no 'fix-' prefix)",
  "problem_statement": "2-3 sentences: what is broken and what is the impact",
  "root_cause": "one sentence: your best hypothesis for the underlying cause",
  "files_affected": ["relative/path/to/file.ts"],
  "confidence": "high|medium|low|none"
}}

Use confidence "none" only when the input is clearly not a software issue (e.g. random words, a personal question, a poem).
For vague descriptions with no file paths, still search the codebase and set confidence to "low" or "medium".
"#,
        description = description
    );

    let mut stream = query(prompt, opts);
    let mut result_text = String::new();

    let collect = async {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Result(ref r)) => {
                    if let Some(text) = r.result_text() {
                        result_text = text.to_string();
                    }
                    break;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "diagnosis agent error");
                    break;
                }
                _ => {}
            }
        }
    };

    // Cap diagnosis at DIAGNOSE_TIMEOUT_SECS — fall back gracefully on timeout.
    if tokio::time::timeout(
        std::time::Duration::from_secs(DIAGNOSE_TIMEOUT_SECS),
        collect,
    )
    .await
    .is_err()
    {
        tracing::warn!("diagnosis agent timed out after {DIAGNOSE_TIMEOUT_SECS}s");
        return Ok(Json(fallback_result(&description)));
    }

    if result_text.is_empty() {
        return Ok(Json(fallback_result(&description)));
    }

    match extract_json(&result_text) {
        Some(v) => Ok(Json(parse_result(&v, &description))),
        None => Ok(Json(fallback_result(&description))),
    }
}
