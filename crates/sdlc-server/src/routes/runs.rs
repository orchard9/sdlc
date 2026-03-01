use axum::{
    extract::{Path, State},
    http::{header, HeaderValue},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use claude_agent::{
    query,
    types::{ContentBlock, SystemPayload},
    McpServerConfig, Message, PermissionMode, QueryOptions,
};
use std::collections::HashMap;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use tracing::{error, info, warn};

use crate::{
    error::AppError,
    state::{
        enforce_retention, generate_run_id, persist_run, persist_run_events, AppState, RunRecord,
        SseMessage,
    },
};

// ---------------------------------------------------------------------------
// Input validation
// ---------------------------------------------------------------------------

/// Validate that a slug contains only safe characters: a-z, A-Z, 0-9, hyphen, underscore.
/// Returns 400 Bad Request if the slug contains anything else.
fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(format!(
            "Invalid slug '{slug}': must contain only letters, digits, hyphens, and underscores"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Spawn a Claude agent keyed by `key`, streaming events into the broadcast map.
/// Creates a RunRecord, persists it, and emits SSE lifecycle events.
///
/// `completion_event` is an optional domain-specific SSE message emitted after
/// `RunFinished` and after the run is removed from the active map. Used by
/// ponder and investigation handlers to emit `PonderRunCompleted` /
/// `InvestigationRunCompleted` without a separate polling task.
async fn spawn_agent_run(
    key: String,
    prompt: String,
    opts: QueryOptions,
    app: &AppState,
    run_type: &str,
    label: &str,
    completion_event: Option<SseMessage>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(key = %key, "spawn_agent_run: request received");

    // Create the broadcast channel and build the RunRecord before taking the lock.
    let run_id = generate_run_id();
    let target = key.split(':').next_back().unwrap_or(&key).to_string();
    let record = RunRecord {
        id: run_id.clone(),
        key: key.clone(),
        run_type: run_type.to_string(),
        target,
        label: label.to_string(),
        status: "running".to_string(),
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        cost_usd: None,
        turns: None,
        error: None,
    };

    let (tx, _) = tokio::sync::broadcast::channel::<String>(512);
    // Clone tx for the spawned task; keep the original to store in the map.
    let tx_task = tx.clone();

    let key_clone = key.clone();
    let agent_runs = app.agent_runs.clone();
    let run_history = app.run_history.clone();
    let event_tx = app.event_tx.clone();
    let root = app.root.clone();
    let run_id_clone = run_id.clone();

    info!(key = %key, "spawn_agent_run: spawning agent task");
    let handle = tokio::spawn(async move {
        let tx = tx_task;
        let mut stream = query(prompt, opts);
        let mut message_count: u64 = 0;
        let mut accumulated_events: Vec<serde_json::Value> = Vec::new();
        let mut final_cost: Option<f64> = None;
        let mut final_turns: Option<u64> = None;
        let mut is_error = false;
        let mut error_msg: Option<String> = None;

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(message) => {
                    message_count += 1;
                    let event = message_to_event(&message);
                    accumulated_events.push(event.clone());
                    let json = match serde_json::to_string(&event) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let _ = tx.send(json);

                    if let Message::Result(ref r) = message {
                        is_error = r.is_error();
                        final_cost = Some(r.total_cost_usd());
                        final_turns = Some(r.num_turns() as u64);
                        if is_error {
                            error_msg = r.result_text().map(|s| s.to_string());
                        }
                        info!(key = %key_clone, message_count, "agent run completed");
                        break;
                    }
                }
                Err(e) => {
                    error!(key = %key_clone, error = %e, message_count, "agent run error");
                    is_error = true;
                    error_msg = Some(e.to_string());
                    let event = serde_json::json!({
                        "type": "error",
                        "message": e.to_string()
                    });
                    accumulated_events.push(event.clone());
                    let _ = tx.send(event.to_string());
                    break;
                }
            }
        }

        // Determine final status
        let status = if is_error { "failed" } else { "completed" };
        let completed_at = chrono::Utc::now().to_rfc3339();

        // Update in-memory history
        {
            let mut history = run_history.lock().await;
            if let Some(rec) = history.iter_mut().find(|r| r.id == run_id_clone) {
                rec.status = status.to_string();
                rec.completed_at = Some(completed_at.clone());
                rec.cost_usd = final_cost;
                rec.turns = final_turns;
                rec.error = error_msg.clone();
            }
        }

        // Persist record + events sidecar
        {
            let root2 = root.clone();
            let id2 = run_id_clone.clone();
            let full_rec = {
                let history = run_history.lock().await;
                history.iter().find(|r| r.id == run_id_clone).cloned()
            };
            if let Some(full_rec) = full_rec {
                tokio::task::spawn_blocking(move || {
                    persist_run(&root2, &full_rec);
                    persist_run_events(&root2, &id2, &accumulated_events);
                    enforce_retention(&root2, 50);
                })
                .await
                .ok();
            }
        }

        // Emit RunFinished SSE
        let _ = event_tx.send(SseMessage::RunFinished {
            id: run_id_clone,
            key: key_clone.clone(),
            status: status.to_string(),
        });

        // Clean up active run
        info!(key = %key_clone, message_count, "agent run cleanup");
        agent_runs.lock().await.remove(&key_clone);

        // Emit domain-specific completion event (e.g. PonderRunCompleted)
        if let Some(evt) = completion_event {
            let _ = event_tx.send(evt);
        }
    });
    let abort_handle = handle.abort_handle();

    // Atomically check for a duplicate and insert — single lock window, no async work inside.
    {
        let mut runs = app.agent_runs.lock().await;
        if runs.contains_key(&key) {
            warn!(key = %key, "spawn_agent_run: agent already running");
            handle.abort();
            return Err(AppError::conflict(format!(
                "Agent already running for '{key}'"
            )));
        }
        runs.insert(key.clone(), (tx.clone(), abort_handle));
    }

    // Async I/O happens after the lock is released.
    {
        let root = app.root.clone();
        let rec = record.clone();
        tokio::task::spawn_blocking(move || persist_run(&root, &rec))
            .await
            .ok();
    }
    app.run_history.lock().await.insert(0, record.clone());

    // Emit RunStarted SSE
    let _ = app.event_tx.send(SseMessage::RunStarted {
        id: run_id.clone(),
        key: key.clone(),
        label: label.to_string(),
    });

    Ok(Json(serde_json::json!({
        "status": "started",
        "message": format!("Agent started for '{key}'"),
        "run_id": run_id,
    })))
}

/// Subscribe to SSE events for a given run key.
async fn get_run_events(key: &str, app: &AppState) -> Response {
    info!(key = %key, "get_run_events: SSE subscribe");
    let rx = {
        let runs = app.agent_runs.lock().await;
        runs.get(key).map(|(tx, _)| tx.subscribe())
    };

    match rx {
        Some(rx) => {
            let stream = BroadcastStream::new(rx).filter_map(|msg| {
                msg.ok()
                    .map(|data| Ok::<Event, Infallible>(Event::default().event("agent").data(data)))
            });
            let mut response = Sse::new(stream)
                .keep_alive(KeepAlive::default())
                .into_response();
            // Disable Cloudflare (and nginx) buffering so SSE events are
            // delivered immediately rather than being held until the buffer fills.
            let h = response.headers_mut();
            h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            h.insert(
                header::HeaderName::from_static("x-accel-buffering"),
                HeaderValue::from_static("no"),
            );
            response
        }
        None => (
            // AppError cannot be used here: this fn returns Response directly (SSE vs JSON branch)
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no active run for this key"})),
        )
            .into_response(),
    }
}

/// Stop a running agent by removing it from the broadcast map.
/// Also updates the RunRecord status and emits RunFinished.
async fn stop_run_by_key(key: &str, app: &AppState) -> Json<serde_json::Value> {
    info!(key = %key, "stop_run_by_key: request received");
    let removed = app.agent_runs.lock().await.remove(key);
    match removed {
        Some((_, abort_handle)) => {
            abort_handle.abort();
            info!(key = %key, "stop_run_by_key: agent stopped");

            // Update RunRecord in history
            let run_id = {
                let mut history = app.run_history.lock().await;
                if let Some(rec) = history
                    .iter_mut()
                    .find(|r| r.key == key && r.status == "running")
                {
                    rec.status = "stopped".to_string();
                    rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
                    let root = app.root.clone();
                    let rec_clone = rec.clone();
                    tokio::task::spawn_blocking(move || persist_run(&root, &rec_clone));
                    Some(rec.id.clone())
                } else {
                    None
                }
            };

            if let Some(id) = run_id {
                let _ = app.event_tx.send(SseMessage::RunFinished {
                    id,
                    key: key.to_string(),
                    status: "stopped".to_string(),
                });
            }

            Json(serde_json::json!({
                "status": "stopped",
                "message": format!("Agent stopped for '{key}'")
            }))
        }
        None => {
            warn!(key = %key, "stop_run_by_key: no agent running");
            Json(serde_json::json!({
                "status": "not_running",
                "message": format!("No agent running for '{key}'")
            }))
        }
    }
}

/// Build the standard sdlc MCP query options.
fn sdlc_query_options(root: std::path::PathBuf, max_turns: u32) -> QueryOptions {
    QueryOptions {
        permission_mode: PermissionMode::AcceptEdits,
        mcp_servers: vec![McpServerConfig {
            name: "sdlc".into(),
            command: std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("sdlc"))
                .to_string_lossy()
                .into_owned(),
            args: vec!["mcp".into()],
            env: HashMap::new(),
        }],
        allowed_tools: vec![
            "Bash".into(),
            "Read".into(),
            "Write".into(),
            "Edit".into(),
            "Glob".into(),
            "Grep".into(),
            "mcp__sdlc__sdlc_get_directive".into(),
            "mcp__sdlc__sdlc_write_artifact".into(),
            "mcp__sdlc__sdlc_approve_artifact".into(),
            "mcp__sdlc__sdlc_reject_artifact".into(),
            "mcp__sdlc__sdlc_add_task".into(),
            "mcp__sdlc__sdlc_complete_task".into(),
            "mcp__sdlc__sdlc_add_comment".into(),
            "mcp__sdlc__sdlc_merge".into(),
        ],
        cwd: Some(root),
        max_turns: Some(max_turns),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Feature run endpoints
// ---------------------------------------------------------------------------

/// POST /api/run/{slug} — spawn a Claude agent that drives a feature through
/// the sdlc state machine via MCP tools.
///
/// Accepts an optional JSON body `{ "context": "..." }`. When provided, the
/// context is injected into the agent prompt as additional user-supplied detail
/// (e.g. the description typed in the Fix Right Away modal).
pub async fn start_run(
    Path(slug): Path<String>,
    State(app): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;

    // Parse optional body — empty or missing body is fine.
    let body_bytes = axum::body::to_bytes(request.into_body(), 8 * 1024)
        .await
        .unwrap_or_default();
    let context: Option<String> = if body_bytes.is_empty() {
        None
    } else {
        serde_json::from_slice::<serde_json::Value>(&body_bytes)
            .ok()
            .and_then(|v| {
                v.get("context")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            })
    };

    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = match context.as_deref() {
        Some(ctx) if !ctx.is_empty() => format!(
            "Drive feature '{}' through the sdlc state machine. \
             User context: \"{}\". Use this as the core problem statement \
             when writing artifacts. \
             Run `sdlc next --for {} --json` to get the next action, \
             execute it, then loop until done or a HITL gate is reached.",
            slug, ctx, slug
        ),
        _ => format!(
            "Drive feature '{}' through the sdlc state machine. \
             Run `sdlc next --for {} --json` to get the next action, \
             execute it, then loop until done or a HITL gate is reached.",
            slug, slug
        ),
    };
    let label = slug.clone();
    spawn_agent_run(slug, prompt, opts, &app, "feature", &label, None).await
}

/// GET /api/run/{slug}/events — SSE stream of agent messages for an active run.
pub async fn run_events(Path(slug): Path<String>, State(app): State<AppState>) -> Response {
    get_run_events(&slug, &app).await
}

/// POST /api/run/{slug}/stop — stop a running agent.
pub async fn stop_run(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    stop_run_by_key(&slug, &app).await
}

// ---------------------------------------------------------------------------
// Milestone UAT endpoints
// ---------------------------------------------------------------------------

/// POST /api/milestone/{slug}/uat — spawn a Claude agent that runs the
/// acceptance test for a milestone.
pub async fn start_milestone_uat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let key = format!("milestone-uat:{slug}");
    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = format!(
        "Run the acceptance test for milestone '{slug}'. \
         Call `sdlc milestone info {slug} --json` to load the milestone and acceptance test. \
         Execute every checklist step. Write results to uat_results.md via \
         `sdlc milestone uat-results {slug} --file ...`. \
         Then call `sdlc milestone complete {slug}` if all steps pass.",
    );
    let label = format!("UAT: {slug}");
    spawn_agent_run(key, prompt, opts, &app, "milestone_uat", &label, None).await
}

/// GET /api/milestone/{slug}/uat/events — SSE stream of milestone UAT agent messages.
pub async fn milestone_uat_events(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Response {
    let key = format!("milestone-uat:{slug}");
    get_run_events(&key, &app).await
}

/// POST /api/milestone/{slug}/uat/stop — stop a milestone UAT agent.
pub async fn stop_milestone_uat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    let key = format!("milestone-uat:{slug}");
    stop_run_by_key(&key, &app).await
}

// ---------------------------------------------------------------------------
// Milestone prepare endpoints
// ---------------------------------------------------------------------------

/// POST /api/milestone/{slug}/prepare — spawn a Claude agent that surveys
/// the milestone, fixes blocker gaps, and reports the wave plan.
pub async fn start_milestone_prepare(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let key = format!("milestone-prepare:{slug}");
    let opts = sdlc_query_options(app.root.clone(), 100);
    let prompt = format!(
        "Survey milestone '{slug}' using `sdlc project prepare --milestone {slug} --json`. \
         Parse the result: report project phase, milestone progress, gaps, and the wave plan. \
         If there are blocker-severity gaps, fix them now: \
         missing descriptions with `sdlc feature update <slug> --description \"...\"`, \
         broken dependency refs with `sdlc feature update <slug> --depends-on <correct-slug>`. \
         Then re-run prepare and present the final wave plan with next steps.",
    );
    let label = format!("prepare: {slug}");
    spawn_agent_run(key, prompt, opts, &app, "milestone_prepare", &label, None).await
}

/// GET /api/milestone/{slug}/prepare/events — SSE stream of prepare agent messages.
pub async fn milestone_prepare_events(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Response {
    let key = format!("milestone-prepare:{slug}");
    get_run_events(&key, &app).await
}

/// POST /api/milestone/{slug}/prepare/stop — stop a running prepare agent.
pub async fn stop_milestone_prepare(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    let key = format!("milestone-prepare:{slug}");
    stop_run_by_key(&key, &app).await
}

// ---------------------------------------------------------------------------
// Milestone run-wave endpoints
// ---------------------------------------------------------------------------

/// POST /api/milestone/{slug}/run-wave — spawn a Claude agent that executes
/// the current wave of a milestone in parallel.
pub async fn start_milestone_run_wave(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let key = format!("milestone-run-wave:{slug}");
    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = format!(
        "Execute the current wave of milestone '{slug}' in parallel. \
         Run `sdlc project prepare --milestone {slug} --json` to get the live wave plan. \
         Wave 1 of the output is the current wave. \
         For each feature in Wave 1 that does not need a worktree, \
         spawn a parallel Agent call running `/sdlc-run <feature-slug>`. \
         Wait for all agents to complete, then re-run prepare and report the updated wave plan.",
    );
    let label = format!("run-wave: {slug}");
    spawn_agent_run(key, prompt, opts, &app, "milestone_run_wave", &label, None).await
}

/// GET /api/milestone/{slug}/run-wave/events — SSE stream of run-wave agent messages.
pub async fn milestone_run_wave_events(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Response {
    let key = format!("milestone-run-wave:{slug}");
    get_run_events(&key, &app).await
}

/// POST /api/milestone/{slug}/run-wave/stop — stop a running run-wave agent.
pub async fn stop_milestone_run_wave(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    let key = format!("milestone-run-wave:{slug}");
    stop_run_by_key(&key, &app).await
}

// ---------------------------------------------------------------------------
// Message → JSON event conversion
// ---------------------------------------------------------------------------

fn message_to_event(msg: &Message) -> serde_json::Value {
    match msg {
        Message::System(sys) => match &sys.payload {
            SystemPayload::Init(init) => serde_json::json!({
                "type": "init",
                "model": init.model,
                "tools_count": init.tools.len(),
                "mcp_servers": init.mcp_servers.iter().map(|s| &s.name).collect::<Vec<_>>()
            }),
            SystemPayload::Status(status) => serde_json::json!({
                "type": "status",
                "status": status.status,
            }),
            _ => serde_json::json!({"type": "system"}),
        },
        Message::Assistant(asst) => {
            let texts: Vec<&str> = asst
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let ContentBlock::Text { text } = c {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let tools: Vec<serde_json::Value> = asst
                .message
                .content
                .iter()
                .filter_map(|c| {
                    if let ContentBlock::ToolUse { name, input, .. } = c {
                        Some(serde_json::json!({"name": name, "input": input}))
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::json!({
                "type": "assistant",
                "text": texts.join(""),
                "tools": tools,
            })
        }
        Message::User(_) => serde_json::json!({"type": "user"}),
        Message::Result(r) => serde_json::json!({
            "type": "result",
            "is_error": r.is_error(),
            "text": r.result_text().unwrap_or(""),
            "cost_usd": r.total_cost_usd(),
            "turns": r.num_turns(),
        }),
        Message::ToolProgress(tp) => serde_json::json!({
            "type": "tool_progress",
            "tool": tp.tool_name,
            "elapsed_seconds": tp.elapsed_time_seconds,
        }),
        Message::ToolUseSummary(ts) => serde_json::json!({
            "type": "tool_summary",
            "summary": ts.summary,
        }),
        Message::StreamEvent(_) => serde_json::json!({"type": "stream_event"}),
        Message::AuthStatus(auth) => serde_json::json!({
            "type": "auth_status",
            "is_authenticating": auth.is_authenticating,
        }),
    }
}

// ---------------------------------------------------------------------------
// Ponder chat endpoints
// ---------------------------------------------------------------------------

/// Request body for POST /api/ponder/:slug/chat
#[derive(serde::Deserialize)]
pub struct PonderChatRequest {
    /// Optional seed message from the owner to kick off the session.
    pub message: Option<String>,
}

/// POST /api/ponder/:slug/chat — start a ponder agent session.
///
/// The agent runs /sdlc-ponder <slug>, seeded with the owner's message if
/// provided. Returns the session number and owner name so the UI can render
/// the owner message optimistically before the session file lands.
///
/// Returns 409 if a session is already running for this ponder.
pub async fn start_ponder_chat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
    Json(body): Json<PonderChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let run_key = format!("ponder:{slug}");

    // 409 if already running
    if app.agent_runs.lock().await.contains_key(&run_key) {
        return Err(AppError::conflict(format!(
            "Session already running for '{slug}'"
        )));
    }

    // Get next session number and git user name (best-effort, non-blocking)
    let root = app.root.clone();
    let slug_clone = slug.clone();
    let (session_n, owner_name) = tokio::task::spawn_blocking(move || {
        let n = sdlc_core::ponder::next_session_number(&root, &slug_clone).unwrap_or(1);
        let name = read_git_user_name(&root).unwrap_or_else(|| "Owner".to_string());
        (n, name)
    })
    .await
    .unwrap_or((1, "Owner".to_string()));

    // Emit run_started before spawning so the UI can lock immediately
    let _ = app.event_tx.send(SseMessage::PonderRunStarted {
        slug: slug.clone(),
        session: session_n,
    });

    // Build agent prompt
    let message_context = match &body.message {
        Some(msg) if !msg.trim().is_empty() => format!(
            "\n\nThe owner ({owner_name}) has seeded this session with the following message — \
             include it verbatim at the top of your session log as \
             `**{owner_name} · Owner**\\n{msg}`:\n\n> {msg}",
            owner_name = owner_name,
            msg = msg.trim(),
        ),
        _ => String::new(),
    };

    let prompt = format!(
        "You are running a ponder session for the idea '{slug}' in the sdlc workspace.\
         {message_context}\n\
         \n\
         ## Step 1 — Load context\n\
         \n\
         ```bash\n\
         sdlc ponder show {slug}\n\
         sdlc ponder session list {slug}\n\
         ```\n\
         \n\
         Read the manifest, every scrapbook artifact, and the most recent session log \
         (`sdlc ponder session read {slug} <N>`) to restore full context. \
         Load team member agent definitions from .claude/agents/. \
         Orient from the orientation strip (WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL).\n\
         \n\
         ## Step 2 — Run the session\n\
         \n\
         You are a facilitator. Channel recruited thought partners by name — voice their \
         perspectives, let them push back, surface tensions. Interrogate the brief: push \
         past stated solutions to find real problems. Capture insights as scrapbook artifacts:\n\
         ```bash\n\
         sdlc ponder capture {slug} --content \"<markdown>\" --as <name>.md\n\
         ```\n\
         Use inline markers: `⚑  Decided:` for resolved points, `?  Open:` for live tensions.\n\
         \n\
         ## Step 3 — Log the session (MANDATORY)\n\
         \n\
         Before ending, you MUST log the session. This is not optional — skipping it means \
         the session is invisible to the web UI and to future agents.\n\
         \n\
         The ONLY correct procedure:\n\
         1. Write the complete session Markdown to a temp file using the Write tool:\n\
            `/tmp/ponder-session-{slug}.md`\n\
         2. Run: `sdlc ponder session log {slug} --file /tmp/ponder-session-{slug}.md`\n\
         \n\
         This command auto-numbers the file, places it in the `sessions/` subdirectory, \
         increments the session counter, and mirrors orientation to the manifest.\n\
         \n\
         NEVER do these — they create scrapbook artifacts, not sessions:\n\
         - Write tool directly to `.sdlc/roadmap/{slug}/session-N.md`\n\
         - `sdlc ponder capture` with session content\n\
         - Any path other than the two-step Write → `sdlc ponder session log` flow\n\
         \n\
         Session file format:\n\
         ```markdown\n\
         ---\n\
         session: <N>\n\
         timestamp: <ISO-8601 UTC>\n\
         orientation:\n\
           current: \"<where the thinking is right now>\"\n\
           next: \"<concrete next action>\"\n\
           commit: \"<condition that unlocks commitment>\"\n\
         ---\n\
         \n\
         <full session dialogue>\n\
         ```\n\
         \n\
         ## Step 4 — Update status (MANDATORY when commit signal is met)\n\
         \n\
         After logging the session, update the ponder status based on thinking state:\n\
         - Commit signal met (idea is shaped, ready to build):\n\
           `sdlc ponder update {slug} --status converging`\n\
         - Still exploring: no update needed (status stays `exploring`)\n\
         - Idea shelved: `sdlc ponder update {slug} --status parked`\n\
         \n\
         This step is not optional when the commit signal is met.",
        slug = slug,
        message_context = message_context,
    );

    let mut opts = sdlc_query_options(app.root.clone(), 100);
    // Ponder sessions also need the ponder_chat tool available
    opts.allowed_tools
        .push("mcp__sdlc__sdlc_ponder_chat".into());

    let ponder_label = format!("ponder: {slug}");
    let completion = SseMessage::PonderRunCompleted {
        slug: slug.clone(),
        session: session_n,
    };
    let _ = spawn_agent_run(
        run_key,
        prompt,
        opts,
        &app,
        "ponder",
        &ponder_label,
        Some(completion),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "status": "started",
        "session": session_n,
        "owner_name": owner_name,
    })))
}

/// DELETE /api/ponder/:slug/chat/current — stop a running ponder session.
pub async fn stop_ponder_chat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    let run_key = format!("ponder:{slug}");
    let result = stop_run_by_key(&run_key, &app).await;
    let _ = app.event_tx.send(SseMessage::PonderRunStopped { slug });
    result
}

// ---------------------------------------------------------------------------
// Ponder commit endpoint
// ---------------------------------------------------------------------------

/// POST /api/ponder/:slug/commit — spawn a headless agent that synthesizes
/// milestones/features from the ponder and marks it committed.
///
/// The agent reads the ponder manifest, scrapbook, and sessions; sizes the
/// idea; creates milestones and features via the sdlc CLI; then calls
/// `sdlc ponder update <slug> --status committed --committed-to <slugs>`.
///
/// Tracked as a normal agent run (key `ponder-commit:<slug>`) — visible in
/// the FAB run panel. Returns 409 if a commit run is already in progress.
pub async fn commit_ponder(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let run_key = format!("ponder-commit:{slug}");

    let prompt = format!(
        "You are running the commit flow for ponder '{slug}' in the sdlc workspace.\n\
         \n\
         ## Step 1 — Load context\n\
         \n\
         ```bash\n\
         sdlc ponder show {slug}\n\
         sdlc ponder artifacts {slug}\n\
         sdlc ponder session list {slug}\n\
         ```\n\
         \n\
         Read every scrapbook artifact and all session logs \
         (`sdlc ponder session read {slug} <N>`) to understand the full idea. \
         Also check existing milestones to avoid duplication:\n\
         ```bash\n\
         sdlc milestone list\n\
         sdlc feature list\n\
         ```\n\
         \n\
         ## Step 2 — Size the idea\n\
         \n\
         Based on scope:\n\
         - Single concern → one feature, add to an existing milestone if one fits\n\
         - Medium idea → one new milestone with 2–5 features\n\
         - Large idea → multiple milestones\n\
         \n\
         ## Step 3 — Create milestones and features\n\
         \n\
         ```bash\n\
         sdlc milestone create <slug> --title \"<title>\" --vision \"<one-line why>\"\n\
         sdlc feature create <slug> --title \"<title>\"\n\
         sdlc milestone add-feature <milestone-slug> <feature-slug>\n\
         ```\n\
         \n\
         Track every milestone slug you create or update.\n\
         \n\
         ## Step 4 — Mark committed (MANDATORY)\n\
         \n\
         After creating all milestones and features, close the ponder:\n\
         ```bash\n\
         sdlc ponder update {slug} --status committed \
         --committed-to <milestone-slug> [--committed-to <milestone-2> ...]\n\
         ```\n\
         \n\
         This is not optional. It is the signal that closes the ponder loop.\n\
         \n\
         ## Step 5 — Report\n\
         \n\
         Output a brief summary: what was created, which milestones, suggested next command.",
        slug = slug,
    );

    let opts = sdlc_query_options(app.root.clone(), 100);
    let label = format!("commit: {slug}");
    spawn_agent_run(run_key, prompt, opts, &app, "ponder", &label, None).await
}

// ---------------------------------------------------------------------------
// Investigation chat endpoints
// ---------------------------------------------------------------------------

/// Request body for POST /api/investigation/:slug/chat
#[derive(serde::Deserialize)]
pub struct InvestigationChatRequest {
    /// Optional seed message from the user to kick off the session.
    pub message: Option<String>,
}

/// POST /api/investigation/:slug/chat — start an investigation agent session.
///
/// The agent runs a structured investigation session, captures artifacts,
/// and logs the session via `sdlc investigate session log`. Returns the
/// session number so the UI can render the seed message optimistically.
///
/// Returns 409 if a session is already running for this investigation.
pub async fn start_investigation_chat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
    Json(body): Json<InvestigationChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let run_key = format!("investigation:{slug}");

    // 409 if already running
    if app.agent_runs.lock().await.contains_key(&run_key) {
        return Err(AppError::conflict(format!(
            "Session already running for '{slug}'"
        )));
    }

    // Load investigation to get kind + context, get next session number
    let root = app.root.clone();
    let slug_clone = slug.clone();
    let (session_n, owner_name, kind_str, phase, context) =
        tokio::task::spawn_blocking(move || {
            let n = sdlc_core::workspace::next_session_number(
                &sdlc_core::paths::investigation_dir(&root, &slug_clone),
            )
            .unwrap_or(1);
            let name = read_git_user_name(&root).unwrap_or_else(|| "User".to_string());
            let (kind_str, phase, context) =
                match sdlc_core::investigation::load(&root, &slug_clone) {
                    Ok(e) => (e.kind.to_string(), e.phase, e.context.unwrap_or_default()),
                    Err(_) => (String::new(), String::new(), String::new()),
                };
            (n, name, kind_str, phase, context)
        })
        .await
        .unwrap_or((
            1,
            "User".to_string(),
            String::new(),
            String::new(),
            String::new(),
        ));

    // Emit run_started before spawning so the UI can lock immediately
    let _ = app.event_tx.send(SseMessage::InvestigationRunStarted {
        slug: slug.clone(),
        session: session_n,
    });

    // Build agent prompt
    let message_context = match &body.message {
        Some(msg) if !msg.trim().is_empty() => format!(
            "\n\nThe user ({owner_name}) has seeded this session:\n\n> {msg}",
            owner_name = owner_name,
            msg = msg.trim(),
        ),
        _ => String::new(),
    };

    let prompt = format!(
        "You are running an investigation session for '{slug}' (kind: {kind_str}, phase: {phase}).\
         {message_context}\n\
         \n\
         Context: {context}\n\
         \n\
         ## Step 1 — Load context\n\
         \n\
         ```bash\n\
         sdlc investigate show {slug}\n\
         sdlc investigate session list {slug}\n\
         ```\n\
         \n\
         Read the manifest and every workspace artifact. Read the most recent session log \
         (`sdlc investigate session read {slug} <N>`) to restore full context. \
         Orient from the orientation strip (WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL).\n\
         \n\
         ## Step 2 — Run the session\n\
         \n\
         Execute the investigation work appropriate to the current phase. \
         Capture insights and findings as artifacts:\n\
         ```bash\n\
         sdlc investigate capture {slug} --content \"<markdown>\" --as <name>.md\n\
         ```\n\
         Use inline markers: `⚑  Decided:` for resolved points, `?  Open:` for live tensions.\n\
         \n\
         ### Root-cause artifact conventions\n\
         \n\
         For `root_cause` investigations, the five investigation areas map to these exact filenames:\n\
         ```\n\
         Area 1 — Code Paths    → area-1-code-paths.md\n\
         Area 2 — Bottlenecks   → area-2-bottlenecks.md\n\
         Area 3 — Data Flow     → area-3-data-flow.md\n\
         Area 4 — Auth Chain    → area-4-auth-chain.md\n\
         Area 5 — Environment   → area-5-environment.md\n\
         ```\n\
         \n\
         Each area artifact MUST begin with YAML frontmatter so the UI can render the progress cards:\n\
         ```markdown\n\
         ---\n\
         area: code_paths\n\
         status: finding    # pending | investigating | finding | hypothesis\n\
         confidence: 72     # 0-100, required when status=hypothesis\n\
         ---\n\
         One-line finding summary here.\n\
         \n\
         [rest of investigation notes]\n\
         ```\n\
         \n\
         Valid `area` values: `code_paths`, `bottlenecks`, `data_flow`, `auth_chain`, `environment`.\n\
         When you reach a hypothesis for an area, set `status: hypothesis` and include `confidence`.\n\
         Write the synthesis document as `synthesis.md` once the cross-area picture is clear.\n\
         \n\
         ### Evolve artifact conventions\n\
         \n\
         For `evolve` investigations, use these exact artifact filenames at each phase:\n\
         ```\n\
         survey    → survey.md       (system structure, entry points, docs state, TODOs/FIXMEs)\n\
         analyze   → lens-analysis.md (maturity table: Low/Medium/High/Excellent per lens + gaps)\n\
         paths     → paths.md        (2-4 evolution paths: name, vision, effort 1-5, impact 1-5)\n\
         roadmap   → roadmap.md      (proper solution → enabling changes → extended vision)\n\
         ```\n\
         \n\
         After writing `lens-analysis.md`, record the lens scores in the manifest:\n\
         ```bash\n\
         sdlc investigate update {slug} --lens pit_of_success=<low|medium|high|excellent>\n\
         sdlc investigate update {slug} --lens coupling=<low|medium|high|excellent>\n\
         sdlc investigate update {slug} --lens growth_readiness=<low|medium|high|excellent>\n\
         sdlc investigate update {slug} --lens self_documenting=<low|medium|high|excellent>\n\
         sdlc investigate update {slug} --lens failure_modes=<low|medium|high|excellent>\n\
         ```\n\
         The five lenses: Pit of Success (do defaults lead to good outcomes?), \
         Coupling (are related things together?), Growth Readiness (will it scale 10x?), \
         Self-Documenting (can you understand it from the code?), \
         Failure Modes (what happens when it breaks?).\n\
         \n\
         Apply strategic step-back before Roadmap: challenge each path for YAGNI, \
         hidden stakeholders, and execution reality.\n\
         \n\
         ### Phase advancement\n\
         \n\
         After writing a phase-gate artifact, advance the phase explicitly:\n\
         ```bash\n\
         sdlc investigate update {slug} --phase <next-phase>\n\
         ```\n\
         Root-cause sequence: `triage` → `investigate` → `synthesize` → `output`\n\
         Root-cause gate artifacts: `triage.md` unlocks `investigate`, `synthesis.md` unlocks `output`.\n\
         \n\
         Evolve sequence: `survey` → `analyze` → `paths` → `roadmap` → `output`\n\
         Evolve gate artifacts: each phase artifact unlocks the next phase.\n\
         Call the update command immediately after capturing the gate artifact — do not wait.\n\
         \n\
         ## Step 3 — Log the session (MANDATORY)\n\
         \n\
         Before ending, you MUST log the session. This is not optional — skipping it means \
         the session is invisible to the web UI and to future agents.\n\
         \n\
         The ONLY correct procedure:\n\
         1. Write the complete session Markdown to a temp file using the Write tool:\n\
            `/tmp/investigation-session-{slug}.md`\n\
         2. Run: `sdlc investigate session log {slug} --file /tmp/investigation-session-{slug}.md`\n\
         \n\
         This command auto-numbers the file, places it in the `sessions/` subdirectory, \
         increments the session counter, and mirrors orientation to the manifest.\n\
         \n\
         NEVER do these — they create artifacts, not sessions:\n\
         - Write tool directly to `.sdlc/investigations/{slug}/session-N.md`\n\
         - `sdlc investigate capture` with session content\n\
         - Any path other than the two-step Write → `sdlc investigate session log` flow\n\
         \n\
         Session file format:\n\
         ```markdown\n\
         ---\n\
         session: <N>\n\
         timestamp: <ISO-8601 UTC>\n\
         orientation:\n\
           current: \"<where the investigation is right now>\"\n\
           next: \"<concrete next action>\"\n\
           commit: \"<condition that unlocks the next phase>\"\n\
         ---\n\
         \n\
         <full session content>\n\
         ```",
        slug = slug,
        kind_str = kind_str,
        phase = phase,
        context = context,
        message_context = message_context,
    );

    let opts = sdlc_query_options(app.root.clone(), 100);

    let investigation_label = format!("investigate: {slug}");
    let completion = SseMessage::InvestigationRunCompleted {
        slug: slug.clone(),
        session: session_n,
    };
    let _ = spawn_agent_run(
        run_key,
        prompt,
        opts,
        &app,
        "investigation",
        &investigation_label,
        Some(completion),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "status": "started",
        "session": session_n,
        "owner_name": owner_name,
    })))
}

/// DELETE /api/investigation/:slug/chat/current — stop a running investigation session.
pub async fn stop_investigation_chat(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Json<serde_json::Value> {
    let run_key = format!("investigation:{slug}");
    let result = stop_run_by_key(&run_key, &app).await;
    let _ = app
        .event_tx
        .send(SseMessage::InvestigationRunStopped { slug });
    result
}

// ---------------------------------------------------------------------------
// Doc alignment handlers
// ---------------------------------------------------------------------------

/// POST /api/vision/run — generate or align VISION.md.
///
/// For fresh projects (no VISION.md): reads `.sdlc/config.yaml` for the project
/// name and description and writes VISION.md from scratch.
/// For existing projects: aligns the document with current feature/milestone state.
/// Fires `vision_align_completed` SSE on finish so the page re-fetches.
pub async fn start_vision_align(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let key = "vision-align".to_string();
    let opts = sdlc_query_options(app.root.clone(), 40);
    let prompt = "Check whether VISION.md exists in the project root.\n\n\
        If VISION.md does NOT exist: read `.sdlc/config.yaml` to get the project name \
        and description. Write VISION.md from scratch — what this project is, who it is \
        for, what problem it solves, and what success looks like. Ground every claim in \
        the project name and description. Be specific, aspirational, and concise.\n\n\
        If VISION.md DOES exist: read it along with the current project state — active \
        features, milestones, and their artifact content — using the available sdlc and \
        filesystem tools. Identify where the project's trajectory has refined or extended \
        the vision through implementation: assumptions validated or invalidated, scope \
        that became clearer, direction that evolved through building. Update VISION.md to \
        capture what was actually learned while preserving strategic intent and \
        aspirational language. Do not water down ambition — sharpen it with what we now know.\n\n\
        Write the result directly to VISION.md in the project root using the Write tool.";
    spawn_agent_run(
        key,
        prompt.to_string(),
        opts,
        &app,
        "vision_align",
        "align: vision",
        Some(SseMessage::VisionAlignCompleted),
    )
    .await
}

/// POST /api/architecture/run — generate or align ARCHITECTURE.md.
///
/// For fresh projects (no ARCHITECTURE.md): reads `.sdlc/config.yaml` and scans
/// the codebase to write ARCHITECTURE.md from scratch.
/// For existing projects: aligns the document with what was actually built.
/// Fires `architecture_align_completed` SSE on finish so the page re-fetches.
pub async fn start_architecture_align(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let key = "architecture-align".to_string();
    let opts = sdlc_query_options(app.root.clone(), 40);
    let prompt = "Check whether ARCHITECTURE.md exists in the project root.\n\n\
        If ARCHITECTURE.md does NOT exist: read `.sdlc/config.yaml` for the project name \
        and description, then scan the codebase — key source files, directory structure, \
        frameworks, and dependencies — using filesystem tools (Read, Glob, Grep). \
        Write ARCHITECTURE.md from scratch: tech stack, key components, interfaces, \
        data flows, and design decisions based on what you find.\n\n\
        If ARCHITECTURE.md DOES exist: read it and scan the actual codebase — key source \
        files, module structure, interfaces, data flows, and component boundaries. \
        Identify where the documented architecture has drifted from what was actually \
        built: renamed components, new modules, changed interfaces, evolved data flows, \
        or patterns that emerged during implementation. Update ARCHITECTURE.md to \
        accurately describe the real system as it exists today.\n\n\
        Write the result directly to ARCHITECTURE.md in the project root using the Write tool.";
    spawn_agent_run(
        key,
        prompt.to_string(),
        opts,
        &app,
        "architecture_align",
        "align: architecture",
        Some(SseMessage::ArchitectureAlignCompleted),
    )
    .await
}

/// POST /api/team/recruit — recruit 2-5 perspective agents tailored to this project.
///
/// Reads VISION.md, ARCHITECTURE.md, and `.sdlc/config.yaml` to understand the
/// project, then writes 2-5 agent `.md` files to `~/.claude/agents/` — one per
/// thought partner. Fires `team_recruit_completed` SSE on finish so the setup
/// page can fetch and display the new agents.
pub async fn start_team_recruit(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let key = "team-recruit".to_string();
    let opts = sdlc_query_options(app.root.clone(), 40);
    let prompt = "You are recruiting a high-impact AI thought-partner team for this project.\n\n\
        Read context files to understand the project:\n\
        1. Read `.sdlc/config.yaml` for project name and description\n\
        2. Read `VISION.md` if it exists\n\
        3. Read `ARCHITECTURE.md` if it exists\n\n\
        Based on the project domain, tech stack, and goals, identify 2-5 distinct expert \
        roles that would provide the most valuable perspectives. Each should cover a \
        different critical dimension — e.g. UX, security, performance, product strategy, \
        domain expertise. Do not create generic roles; make them specific to this project.\n\n\
        For each role, write a perspective agent file to `~/.claude/agents/<slug>.md` using \
        the Write tool. The filename should be a kebab-case slug like \
        `ux-researcher.md` or `security-architect.md`. Use this exact format:\n\n\
        ```\n\
        ---\n\
        name: <Full Name or Role Title>\n\
        description: <one sentence — when to use this agent>\n\
        ---\n\n\
        You are <Name>, a <role> with deep expertise in <domain>. \
        [2-3 sentences establishing background and perspective.]\n\n\
        When consulted, you:\n\
        - [distinctive behaviour 1]\n\
        - [distinctive behaviour 2]\n\
        - [distinctive behaviour 3]\n\
        ```\n\n\
        Write exactly 2-5 agents. Quality over quantity — only create roles that will \
        genuinely shape decisions for this specific project.";
    spawn_agent_run(
        key,
        prompt.to_string(),
        opts,
        &app,
        "team_recruit",
        "recruit: team",
        Some(SseMessage::TeamRecruitCompleted),
    )
    .await
}

// ---------------------------------------------------------------------------
// Run history endpoints
// ---------------------------------------------------------------------------

/// GET /api/runs — list all RunRecords (no events).
pub async fn list_runs(State(app): State<AppState>) -> Json<serde_json::Value> {
    let history = app.run_history.lock().await;
    Json(serde_json::json!(history.as_slice()))
}

/// GET /api/runs/{id} — single RunRecord + events (loaded from disk sidecar).
pub async fn get_run(Path(id): Path<String>, State(app): State<AppState>) -> Response {
    let record = {
        let history = app.run_history.lock().await;
        history.iter().find(|r| r.id == id).cloned()
    };

    match record {
        Some(rec) => {
            let root = app.root.clone();
            let id_clone = id.clone();
            let events = tokio::task::spawn_blocking(move || {
                crate::state::load_run_events(&root, &id_clone)
            })
            .await
            .unwrap_or_else(|_| vec![]);

            let mut value =
                match serde_json::to_value(&rec) {
                    Ok(v) => v,
                    Err(e) => return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(
                            serde_json::json!({"error": format!("failed to serialize run: {e}")}),
                        ),
                    )
                        .into_response(),
                };
            if let Some(obj) = value.as_object_mut() {
                obj.insert("events".to_string(), serde_json::json!(events));
            }
            Json(value).into_response()
        }
        None => (
            // AppError cannot be used here: this fn returns Response directly (SSE vs JSON branch)
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Run '{id}' not found")})),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// AMA answer synthesis endpoint
// ---------------------------------------------------------------------------

/// Request body for POST /api/tools/ama/answer
#[derive(serde::Deserialize)]
pub struct AmaAnswerRequest {
    pub question: String,
    pub sources: Vec<serde_json::Value>,
    /// Formatted prior Q&A turns — injected as context for follow-up questions.
    pub thread_context: Option<String>,
    /// Prevents run key collision when the same question is asked multiple times in a thread.
    pub turn_index: Option<u32>,
}

/// POST /api/tools/ama/answer — spawn a short synthesis agent that answers a
/// developer question using the search result excerpts returned by the AMA tool.
///
/// The agent is given the question + source excerpts (path, lines, excerpt,
/// score) and writes a concise prose answer grounded in those excerpts.
///
/// Returns `{ status, run_id, run_key }`. The caller should subscribe to
/// `GET /api/run/{run_key}/events` to stream the agent output.
///
/// Returns 400 if question is empty or sources is empty.
/// Returns 409 if an answer synthesis is already in flight for this question.
pub async fn answer_ama(
    State(app): State<AppState>,
    Json(body): Json<AmaAnswerRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let question = body.question.trim().to_string();
    if question.is_empty() {
        return Err(AppError::bad_request("question must not be empty"));
    }
    if body.sources.is_empty() {
        return Err(AppError::bad_request("sources must not be empty"));
    }

    // Derive a stable, URL-safe run key from a hash of the question + turn index
    let hash = short_hash_question(&question);
    let turn = body.turn_index.unwrap_or(0);
    let key = format!("ama-answer-{hash}-{turn}");

    // Format sources as labelled context blocks
    let sources_text: String = body
        .sources
        .iter()
        .map(|s| {
            let path = s.get("path").and_then(|v| v.as_str()).unwrap_or("?");
            let lines = s
                .get("lines")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    let start = arr.first().and_then(|v| v.as_u64()).unwrap_or(0);
                    let end = arr.last().and_then(|v| v.as_u64()).unwrap_or(0);
                    format!("{start}-{end}")
                })
                .unwrap_or_else(|| "?-?".to_string());
            let score = s.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let score_pct = (score * 100.0) as u64;
            let excerpt = s.get("excerpt").and_then(|v| v.as_str()).unwrap_or("");
            format!("--- {path}:{lines} (score: {score_pct}%) ---\n{excerpt}")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let n = body.sources.len();
    let thread_prefix = if let Some(ctx) = &body.thread_context {
        if !ctx.trim().is_empty() {
            format!(
                "[Prior conversation — use for context only]\n\
                 {ctx}\n\
                 ---\n\
                 [Current question]\n\
                 \n",
                ctx = ctx.trim()
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let prompt = format!(
        "{thread_prefix}\
         You are answering a developer question about this codebase using \
         search results from the AMA tool.\n\
         \n\
         Question: {question}\n\
         \n\
         Search results ({n} sources):\n\
         \n\
         {sources_text}\n\
         \n\
         ---\n\
         \n\
         Using ONLY the information in the search results above, write a clear, \
         concise answer to the question.\n\
         - If the results are sufficient, explain exactly what the code does and where\n\
         - If the results are insufficient or seem stale, say so and suggest re-running setup\n\
         - Reference specific file paths and line numbers in your answer\n\
         - Do not make up code or behaviors not visible in the excerpts\n\
         - Be concise — this is a quick developer answer, not an essay\n\
         \n\
         Write only the answer. No preamble like \"Based on the search results...\". \
         Just directly answer the question.",
        thread_prefix = thread_prefix,
        question = question,
        n = n,
        sources_text = sources_text,
    );

    let opts = sdlc_query_options(app.root.clone(), 5);
    let label_prefix: String = question.chars().take(40).collect();
    let label = format!("AMA: {label_prefix}");

    let result =
        spawn_agent_run(key.clone(), prompt, opts, &app, "ama_answer", &label, None).await?;

    // Inject run_key so the frontend can subscribe to the agent event stream
    let mut resp = result.0;
    if let Some(obj) = resp.as_object_mut() {
        obj.insert("run_key".to_string(), serde_json::json!(key));
    }
    Ok(Json(resp))
}

/// POST /api/tools/quality-check/reconfigure — spawn an agent that detects the
/// project stack and reconfigures `.sdlc/tools/quality-check/config.yaml` with
/// appropriate quality gates, then reinstalls the pre-commit hook.
///
/// Uses the `sdlc-setup-quality-gates` skill workflow:
///   1. Detect languages (Go, TypeScript, Rust, Python)
///   2. Update config.yaml checks for detected stack + available tooling
///   3. Run `sdlc tool setup quality-check` to reinstall the hook
///
/// Returns `{ status, run_id, run_key }`. Caller subscribes to
/// `GET /api/run/{run_key}/events` to stream the agent output.
pub async fn reconfigure_quality_gates(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let key = "quality-reconfigure".to_string();

    let prompt = "You are reconfiguring quality gates for this project using the two-phase quality-gates approach.\n\
        \n\
        ## Step 1 — Detect languages\n\
        \n\
        ```bash\n\
        ls go.mod Cargo.toml package.json pyproject.toml requirements.txt 2>/dev/null\n\
        ```\n\
        \n\
        ## Step 2 — Check available tooling\n\
        \n\
        For each detected language, check which tools are installed:\n\
        ```bash\n\
        which goimports golangci-lint 2>/dev/null\n\
        which prettier eslint 2>/dev/null\n\
        which rustfmt cargo 2>/dev/null\n\
        which ruff black mypy 2>/dev/null\n\
        ```\n\
        \n\
        ## Step 3 — Write the pre-commit hook (two-phase)\n\
        \n\
        Write `.githooks/pre-commit` with the two-phase quality-gates pattern:\n\
        \n\
        **Phase 1: Auto-fix** — run formatters on staged files, run linters with --fix, re-stage fixed files.\n\
        **Phase 2: Verify** — call the quality-check tool for structured check results.\n\
        \n\
        ```bash\n\
        #!/usr/bin/env bash\n\
        set -euo pipefail\n\
        ROOT=\"$(git rev-parse --show-toplevel)\"\n\
        \n\
        # Get staged files by type\n\
        staged_by_ext() { git diff --cached --name-only --diff-filter=ACM | grep -E \"$1\" || true; }\n\
        \n\
        STAGED_GO=$(staged_by_ext '\\.go$')\n\
        STAGED_TS=$(staged_by_ext '\\.(ts|tsx)$')\n\
        STAGED_RS=$(staged_by_ext '\\.rs$')\n\
        STAGED_PY=$(staged_by_ext '\\.py$')\n\
        \n\
        # Phase 1: Auto-fix (only include sections for detected languages)\n\
        [[ -n \"$STAGED_GO\" ]] && gofmt -w $STAGED_GO && git add $STAGED_GO\n\
        [[ -n \"$STAGED_TS\" ]] && npx prettier --write $STAGED_TS && git add $STAGED_TS\n\
        [[ -n \"$STAGED_RS\" ]] && rustfmt $STAGED_RS 2>/dev/null && git add $STAGED_RS\n\
        [[ -n \"$STAGED_PY\" ]] && ruff format $STAGED_PY && ruff check --fix $STAGED_PY && git add $STAGED_PY\n\
        \n\
        # Phase 2: Verify via quality-check tool (structured per-check results)\n\
        exec bun run \"$ROOT/.sdlc/tools/quality-check/tool.ts\" --run < /dev/null\n\
        ```\n\
        \n\
        Make it executable: `chmod +x .githooks/pre-commit`\n\
        Configure git: `git config core.hooksPath .githooks`\n\
        \n\
        Only include Phase 1 sections for languages that are BOTH detected AND have auto-fix tools available.\n\
        \n\
        ## Step 4 — Update config.yaml (Phase 2 verify checks)\n\
        \n\
        Write `.sdlc/tools/quality-check/config.yaml` with the verify-phase checks.\n\
        These are run by Phase 2 (`tool.ts --run`) and shown as structured results in the UI.\n\
        \n\
        Rules:\n\
        - Use single-line scripts only (no YAML block literals — use `&&` chains)\n\
        - Include: format verification, vet/lint, build, fast tests\n\
        - Exclude: slow integration tests, coverage reports\n\
        - Keep total runtime under 30s\n\
        - For Go: `go vet ./...`, `golangci-lint run ./...`, `go build ./...`, `go test ./... -count=1 -timeout 30s`\n\
        - For TypeScript: `tsc --noEmit`, `eslint .` (if available)\n\
        - For Rust: `cargo clippy -- -D warnings`, `cargo build`, `cargo test`\n\
        - For Python: `ruff check .`, `mypy .` (if available)\n\
        \n\
        Format (no block literals):\n\
        ```yaml\n\
        name: quality-check\n\
        version: \"0.2.0\"\n\
        checks:\n\
          - name: <check-name>\n\
            description: <what it does>\n\
            script: <single-line shell command using && chains>\n\
        ```\n\
        \n\
        ## Step 5 — Report\n\
        \n\
        List:\n\
        - Languages detected\n\
        - Phase 1 auto-fix tools installed (and any missing)\n\
        - Phase 2 verify checks configured (name + script)\n\
        - Hook status (installed at .githooks/pre-commit)\n\
        ".to_string();

    let opts = sdlc_query_options(app.root.clone(), 10);

    let result = spawn_agent_run(
        key.clone(),
        prompt,
        opts,
        &app,
        "quality_reconfigure",
        "Reconfigure quality gates",
        None,
    )
    .await?;

    let mut resp = result.0;
    if let Some(obj) = resp.as_object_mut() {
        obj.insert("run_key".to_string(), serde_json::json!(key));
    }
    Ok(Json(resp))
}

#[derive(serde::Deserialize)]
pub struct QualityFixRequest {
    /// The failed CheckResult objects from the quality-check tool run.
    pub failed_checks: Vec<serde_json::Value>,
}

/// POST /api/tools/quality-check/fix — spawn an agent that reads the failed
/// check results and applies a fix strategy scaled to the number of failures:
///
/// - 1 failure  → fix-forward (targeted patch on root cause)
/// - 2–5 failures → fix-all (seven-dimension review + fix)
/// - 6+ failures  → remediate (systemic — enforcement + pattern fix)
///
/// Returns `{ status, run_id, run_key }`. Caller subscribes to
/// `GET /api/run/{run_key}/events` to stream the agent output.
pub async fn fix_quality_issues(
    State(app): State<AppState>,
    Json(body): Json<QualityFixRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.failed_checks.is_empty() {
        return Err(AppError::bad_request("no failed checks provided"));
    }

    let key = "quality-fix".to_string();
    let count = body.failed_checks.len();

    let skill = if count == 1 {
        "/fix-forward"
    } else if count <= 5 {
        "/fix-all"
    } else {
        "/remediate"
    };

    let checks_summary = body
        .failed_checks
        .iter()
        .filter_map(|c| {
            let name = c.get("name")?.as_str()?;
            let output = c.get("output")?.as_str()?;
            let preview: String = output.chars().take(300).collect();
            Some(format!("**{name}**:\n```\n{preview}\n```"))
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "Quality gate check(s) failed. Fix them using `{skill}`.\n\
        \n\
        ## Failed checks ({count})\n\
        \n\
        {checks_summary}\n\
        \n\
        ## Steps\n\
        \n\
        1. Invoke `{skill}` with the failed check names and output above as context.\n\
        2. After `{skill}` completes, run `sdlc tool run quality-check` to confirm all checks pass.\n\
        3. Report the result.\n\
        "
    );

    let opts = sdlc_query_options(app.root.clone(), 20);

    let result = spawn_agent_run(
        key.clone(),
        prompt,
        opts,
        &app,
        "quality_fix",
        "Fix quality gate failures",
        None,
    )
    .await?;

    let mut resp = result.0;
    if let Some(obj) = resp.as_object_mut() {
        obj.insert("run_key".to_string(), serde_json::json!(key));
    }
    Ok(Json(resp))
}

/// Derive a short URL-safe hex hash from the question string.
fn short_hash_question(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:04x}", hasher.finish() & 0xFFFF)
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Read `git config user.name` from the given directory. Returns None on any error.
fn read_git_user_name(root: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["config", "user.name"])
        .current_dir(root)
        .output()
        .ok()?;
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    } else {
        None
    }
}
