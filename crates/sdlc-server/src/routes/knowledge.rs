use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppError;
use crate::routes::runs::{sdlc_query_options, spawn_agent_run};
use crate::state::{AppState, SseMessage};

// ---------------------------------------------------------------------------
// Parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SessionPath {
    pub slug: String,
    pub n: u32,
}

#[derive(serde::Deserialize, Default)]
pub struct ListKnowledgeQuery {
    pub code: Option<String>,
    pub tag: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/catalog
// ---------------------------------------------------------------------------

pub async fn get_catalog(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let catalog = sdlc_core::knowledge::load_catalog(&root)?;
        let classes: Vec<serde_json::Value> = catalog
            .classes
            .iter()
            .map(|c| {
                let divisions: Vec<serde_json::Value> = c
                    .divisions
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "code": d.code,
                            "name": d.name,
                            "description": d.description,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "code": c.code,
                    "name": c.name,
                    "description": c.description,
                    "divisions": divisions,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "classes": classes,
            "updated_at": catalog.updated_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge
// ---------------------------------------------------------------------------

pub async fn list_knowledge(
    State(app): State<AppState>,
    Query(params): Query<ListKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut entries = match params.code.as_deref() {
            Some(prefix) => sdlc_core::knowledge::list_by_code_prefix(&root, prefix)?,
            None => sdlc_core::knowledge::list(&root)?,
        };

        if let Some(tag) = params.tag.as_deref() {
            entries.retain(|e| e.tags.iter().any(|t| t == tag));
        }

        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "code": e.code,
                    "status": e.status.to_string(),
                    "summary": e.summary,
                    "tags": e.tags,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateKnowledgeBody {
    pub slug: Option<String>,
    pub title: String,
    #[serde(default = "default_code")]
    pub code: String,
    #[serde(default)]
    pub content: Option<String>,
}

fn default_code() -> String {
    "uncategorized".to_string()
}

pub async fn create_knowledge(
    State(app): State<AppState>,
    Json(body): Json<CreateKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let slug = body
            .slug
            .unwrap_or_else(|| slugify_title_server(&body.title));

        let entry = sdlc_core::knowledge::create(&root, &slug, &body.title, &body.code)?;

        if let Some(text) = body.content.as_deref() {
            sdlc_core::knowledge::append_content(&root, &slug, text)?;
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug
// ---------------------------------------------------------------------------

pub async fn get_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::knowledge::load(&root, &slug)?;
        let content = sdlc_core::knowledge::read_content(&root, &slug).unwrap_or_default();
        let artifacts = sdlc_core::knowledge::list_named_artifacts(&root, &slug)?;

        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                })
            })
            .collect();

        let sources: Vec<serde_json::Value> = entry
            .sources
            .iter()
            .map(|s| {
                serde_json::json!({
                    "type": s.source_type.to_string(),
                    "url": s.url,
                    "path": s.path,
                    "workspace": s.workspace,
                    "captured_at": s.captured_at,
                })
            })
            .collect();

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
            "summary": entry.summary,
            "tags": entry.tags,
            "sources": sources,
            "related": entry.related,
            "origin": entry.origin.to_string(),
            "harvested_from": entry.harvested_from,
            "last_verified_at": entry.last_verified_at,
            "staleness_flags": entry.staleness_flags,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "content": content,
            "artifacts": artifact_list,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// PUT /api/knowledge/:slug
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct UpdateKnowledgeBody {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub tags_add: Option<Vec<String>>,
    #[serde(default)]
    pub related_add: Option<Vec<String>>,
}

pub async fn update_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let status = body
            .status
            .as_deref()
            .map(|s| {
                s.parse::<sdlc_core::knowledge::KnowledgeStatus>()
                    .map_err(|_| sdlc_core::SdlcError::InvalidKnowledgeStatus(s.to_string()))
            })
            .transpose()?;

        let tags_add = body.tags_add.unwrap_or_default();
        let related_add = body.related_add.unwrap_or_default();

        let entry = sdlc_core::knowledge::update(
            &root,
            &slug,
            body.title.as_deref(),
            body.code.as_deref(),
            status,
            body.summary.as_deref(),
            &tags_add,
            &related_add,
        )?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "code": entry.code,
            "status": entry.status.to_string(),
            "updated_at": entry.updated_at,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/:slug/capture
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CaptureKnowledgeBody {
    pub filename: String,
    pub content: String,
}

pub async fn capture_knowledge_artifact(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CaptureKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::knowledge::capture_named_artifact(&root, &slug, &body.filename, &body.content)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "filename": body.filename,
            "captured": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug/sessions
// ---------------------------------------------------------------------------

pub async fn list_knowledge_sessions(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = sdlc_core::knowledge::list_sessions(&root, &slug)?;
        let list: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/:slug/sessions/:n
// ---------------------------------------------------------------------------

pub async fn get_knowledge_session(
    State(app): State<AppState>,
    Path(SessionPath { slug, n }): Path<SessionPath>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let content = sdlc_core::knowledge::read_session(&root, &slug, n)?;
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "session": n,
            "content": content,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/:slug/research
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct ResearchKnowledgeBody {
    pub topic: Option<String>,
}

pub async fn research_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<ResearchKnowledgeBody>,
) -> Result<impl IntoResponse, AppError> {
    let root = app.root.clone();
    let slug_clone = slug.clone();

    // Load or create the knowledge entry, capturing the title for the prompt.
    let title = tokio::task::spawn_blocking(move || {
        match sdlc_core::knowledge::load(&root, &slug_clone) {
            Ok(e) => Ok::<String, sdlc_core::SdlcError>(e.title),
            Err(_) => {
                // Entry does not exist — create it with Research origin.
                let e =
                    sdlc_core::knowledge::create(&root, &slug_clone, &slug_clone, "uncategorized")?;
                Ok::<String, sdlc_core::SdlcError>(e.title)
            }
        }
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    // Emit KnowledgeResearchStarted SSE before spawning the agent.
    let _ = app
        .event_tx
        .send(SseMessage::KnowledgeResearchStarted { slug: slug.clone() });

    let topic = body.topic.unwrap_or_else(|| slug.clone());
    let prompt = build_research_prompt(&slug, &title, &topic, &app.root);
    let mut opts = sdlc_query_options(app.root.clone(), 20);
    opts.allowed_tools.push("WebSearch".into());
    opts.allowed_tools.push("WebFetch".into());

    let completion = Some(SseMessage::KnowledgeResearchCompleted { slug: slug.clone() });
    let result = spawn_agent_run(
        format!("knowledge:{slug}"),
        prompt,
        opts,
        &app,
        "knowledge",
        &format!("Research: {title}"),
        completion,
    )
    .await?;

    Ok((StatusCode::ACCEPTED, result).into_response())
}

fn build_research_prompt(slug: &str, title: &str, topic: &str, root: &std::path::Path) -> String {
    let root_str = root.display();
    format!(
        r#"You are a knowledge researcher. Your task is to research the topic "{topic}" and synthesize findings into the knowledge base entry with slug "{slug}" (title: "{title}").

Root directory: {root_str}
Knowledge entry path: {root_str}/.sdlc/knowledge/{slug}/

Follow these steps in order:

1. **Web research** — Use `WebSearch` to find 3–5 authoritative external sources on "{topic}". For each promising result, use `WebFetch` to read the full content. Capture key facts, patterns, and references, and note the source URLs for citation.

2. **Local codebase context** — Use `Grep` and `Read` to search the project at {root_str} for any existing usage, configuration, or documentation related to "{topic}". Note any internal conventions, prior art, or gaps.

3. **Synthesize and write `content.md`** — Write a comprehensive Markdown document to {root_str}/.sdlc/knowledge/{slug}/content.md with this structure:
   ```
   ## External Findings
   <summary of web research with source URLs as Markdown links>

   ## Local Context
   <summary of what exists in the codebase, or "None found" if absent>

   ## Summary
   <2–4 sentence synthesis combining both sources>
   ```

4. **Update entry summary** — Run:
   ```
   sdlc knowledge update {slug} --summary "<one-line summary of the topic>"
   ```

5. **Log the session** — Run:
   ```
   sdlc knowledge session log {slug} --content "<brief description of what was researched and found>"
   ```

Focus on accuracy and citation. Only include external claims that you fetched and read directly — do not hallucinate sources.
When done, confirm completion with a brief summary of findings.
"#
    )
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/maintain
// ---------------------------------------------------------------------------

pub async fn maintain_knowledge(
    State(app): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let root = app.root.clone();
    let _ = app.event_tx.send(SseMessage::KnowledgeMaintenanceStarted);

    let prompt = format!(
        r#"You are a knowledge librarian agent running a maintenance pass on the project knowledge base.

Project root: {root}

## Maintenance checklist

Run all six checks in order:

1. **Stale entries** — Run `sdlc knowledge list` and for each entry older than 30 days, run
   `sdlc knowledge update <slug> --summary "<updated summary>"` to refresh the summary.

2. **Broken cross-references** — Scan `.sdlc/knowledge/*/content.md` with Grep for
   `[[` cross-reference links; verify that referenced slugs exist with `sdlc knowledge show <slug>`.
   Log any broken references as a note in the source entry.

3. **Duplicate detection** — Compare entry titles and tags; if two entries cover the same topic,
   add a `see_also` note in both and flag for consolidation.

4. **Tag consistency** — Run `sdlc knowledge list` and check that tags follow lowercase-hyphen
   convention. Rename inconsistent tags with `sdlc knowledge update <slug> --tags "<new tags>"`.

5. **Orphan cleanup** — Identify entries with no sessions and no content beyond the default
   template. Mark them with `sdlc knowledge update <slug> --summary "orphan: no content yet"`.

6. **Log completion** — Write a maintenance session log to `/tmp/knowledge-maintenance.md`
   summarising what was found and fixed, then run:
   `sdlc knowledge session log maintenance --file /tmp/knowledge-maintenance.md`

Report the number of actions taken in a final line: `ACTIONS_TAKEN: <n>`
"#,
        root = root.display()
    );

    let opts = sdlc_query_options(root, 50);

    let result = spawn_agent_run(
        "knowledge:maintain".to_string(),
        prompt,
        opts,
        &app,
        "knowledge_maintain",
        "Knowledge maintenance",
        Some(SseMessage::KnowledgeMaintenanceCompleted { actions_taken: 0 }),
    )
    .await;

    match result {
        Ok(_) => Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "started": true })),
        )
            .into_response()),
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/harvest
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct HarvestWorkspaceBody {
    pub r#type: String,
    pub slug: String,
}

pub async fn harvest_knowledge_workspace(
    State(app): State<AppState>,
    Json(body): Json<HarvestWorkspaceBody>,
) -> Result<impl IntoResponse, AppError> {
    let workspace_type = body.r#type.clone();
    let workspace_slug = body.slug.clone();

    if !matches!(workspace_type.as_str(), "investigation" | "ponder") {
        return Err(AppError(anyhow::anyhow!(
            "unsupported workspace type '{}'; must be 'investigation' or 'ponder'",
            workspace_type
        )));
    }

    let root = app.root.clone();
    let label = format!("Harvest {workspace_type}/{workspace_slug}");
    let run_key = format!("knowledge:harvest:{workspace_slug}");

    let prompt = format!(
        r#"You are a knowledge librarian agent harvesting a workspace into the knowledge base.

Workspace type: {workspace_type}
Workspace slug: {workspace_slug}
Project root: {root}

## Steps

1. Run `sdlc knowledge librarian harvest --type {workspace_type} --slug {workspace_slug}`
   to create or update the knowledge entry for this workspace.

2. If the command reports `created: true`, add a brief summary:
   `sdlc knowledge update {workspace_type}-{workspace_slug} --summary "<one-sentence summary>"`

3. Log the harvest session:
   a. Write `/tmp/knowledge-harvest-{workspace_slug}.md` with: what was harvested, whether it was
      new or updated, and any notable content.
   b. Run: `sdlc knowledge session log {workspace_type}-{workspace_slug} --file /tmp/knowledge-harvest-{workspace_slug}.md`

4. Output a final line: `HARVEST_COMPLETE: {workspace_type}/{workspace_slug}`
"#,
        root = root.display()
    );

    let opts = sdlc_query_options(root, 20);

    let result = spawn_agent_run(
        run_key,
        prompt,
        opts,
        &app,
        "knowledge_harvest",
        &label,
        None,
    )
    .await;

    match result {
        Ok(_) => Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "type": workspace_type,
                "slug": workspace_slug,
                "started": true,
            })),
        )
            .into_response()),
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// GET /api/knowledge/relevant
// ---------------------------------------------------------------------------

/// Query parameters for the relevance endpoint.
#[derive(serde::Deserialize, Default)]
pub struct RelevantKnowledgeQuery {
    /// Comma-separated list of tags to match against.
    pub tags: Option<String>,
    /// Maximum number of results to return (default 10, max 50).
    pub limit: Option<usize>,
}

/// GET /api/knowledge/relevant?tags=a,b&limit=10
///
/// Returns the top-N published knowledge entries most relevant to the given
/// tags, ranked by tag-overlap count then recency.
pub async fn get_relevant_knowledge(
    State(app): State<AppState>,
    Query(params): Query<RelevantKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let tags: Vec<String> = params
        .tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let limit = params.limit.unwrap_or(10).min(50);
    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::knowledge::relevant_entries(&root, &tags, limit)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "code": e.code,
                    "status": e.status.to_string(),
                    "summary": e.summary,
                    "tags": e.tags,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/knowledge/ask
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct AskKnowledgeBody {
    pub question: String,
}

pub async fn ask_knowledge(
    State(app): State<AppState>,
    Json(body): Json<AskKnowledgeBody>,
) -> Result<impl IntoResponse, AppError> {
    let question = body.question.trim().to_string();
    if question.is_empty() {
        return Err(AppError::bad_request("question cannot be empty"));
    }

    let root = app.root.clone();
    let question_clone = question.clone();

    // Load catalog and all entries in a blocking task.
    let (catalog_summary, entries_summary) = tokio::task::spawn_blocking(move || {
        let catalog = sdlc_core::knowledge::load_catalog(&root).unwrap_or_else(|_| {
            sdlc_core::knowledge::Catalog {
                classes: Vec::new(),
                updated_at: chrono::Utc::now(),
            }
        });
        let entries = sdlc_core::knowledge::list(&root).unwrap_or_default();
        let catalog_text = if catalog.classes.is_empty() {
            "(catalog not initialized)".to_string()
        } else {
            catalog
                .classes
                .iter()
                .map(|c| format!("  {} \u{2014} {}", c.code, c.name))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let entries_text: Vec<String> = entries
            .iter()
            .map(|e| {
                format!(
                    "- [{code}] {title} (slug: {slug})",
                    code = e.code,
                    title = e.title,
                    slug = e.slug,
                )
            })
            .collect();
        (catalog_text, entries_text.join("\n"))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?;

    let root_str = app.root.display().to_string();
    let prompt = format!(
        "You are the project knowledge librarian. Answer the following question using the\nproject knowledge base.\n\n## Question\n{question}\n\n## Knowledge catalog (class codes and names)\n{catalog_summary}\n\n## Knowledge entries\n{entries_summary}\n\n## Instructions\n\n1. Use Bash to read relevant entry content:\n   ```\n   cat {root_str}/.sdlc/knowledge/<slug>/content.md\n   ```\n2. Synthesize a clear, factual answer from the knowledge base.\n3. If the knowledge base does not fully cover the question, supplement with WebSearch/WebFetch.\n4. Output your answer in this EXACT format:\n\n```\nANSWER:\n<your answer here>\n\nCITED:\n<slug1> | <code1> | <title1>\n\nGAP: YES or NO\nGAP_SUGGESTION: <one sentence on what should be added, or NONE>\n```\n\nRules:\n- Each CITED line must be `slug | code | title` with pipe separators.\n- Only cite entries you actually read and used.\n- GAP is YES only if the question cannot be fully answered from the knowledge base.\n- GAP_SUGGESTION is required only when GAP is YES.\n"
    );

    let _ = app.event_tx.send(SseMessage::KnowledgeQueryStarted {
        question: question_clone.clone(),
    });

    let key: String = question_clone
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(40)
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let mut opts = sdlc_query_options(app.root.clone(), 20);
    opts.allowed_tools.push("WebSearch".into());
    opts.allowed_tools.push("WebFetch".into());

    spawn_agent_run(
        format!("knowledge:ask:{key}"),
        prompt,
        opts,
        &app,
        "knowledge_ask",
        &format!("Knowledge ask: {question_clone}"),
        Some(SseMessage::KnowledgeQueryCompleted {
            answer: String::new(),
            cited_entries: Vec::new(),
            gap_detected: false,
            gap_suggestion: None,
        }),
    )
    .await
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn slugify_title_server(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut result = String::new();
    let mut last_was_dash = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            last_was_dash = false;
        } else if !last_was_dash && !result.is_empty() {
            result.push('-');
            last_was_dash = true;
        }
    }
    while result.ends_with('-') {
        result.pop();
    }
    result.chars().take(40).collect()
}
