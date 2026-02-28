use axum::{
    extract::{Path, State},
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
async fn spawn_agent_run(
    key: String,
    prompt: String,
    opts: QueryOptions,
    app: &AppState,
    run_type: &str,
    label: &str,
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
            Sse::new(stream)
                .keep_alive(KeepAlive::default())
                .into_response()
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
            command: "sdlc".into(),
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
pub async fn start_run(
    Path(slug): Path<String>,
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let opts = sdlc_query_options(app.root.clone(), 200);
    let prompt = format!(
        "Drive feature '{}' through the sdlc state machine. \
         Run `sdlc next --for {} --json` to get the next action, \
         execute it, then loop until done or a HITL gate is reached.",
        slug, slug
    );
    let label = slug.clone();
    spawn_agent_run(slug, prompt, opts, &app, "feature", &label).await
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
    spawn_agent_run(key, prompt, opts, &app, "milestone_uat", &label).await
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
    spawn_agent_run(key, prompt, opts, &app, "milestone_prepare", &label).await
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
         This step is not optional when the commit signal is met. The web UI shows \
         the Commit button only when status is `converging`. Without this update the \
         user has no UI path to commit.",
        slug = slug,
        message_context = message_context,
    );

    let mut opts = sdlc_query_options(app.root.clone(), 100);
    // Ponder sessions also need the ponder_chat tool available
    opts.allowed_tools
        .push("mcp__sdlc__sdlc_ponder_chat".into());

    let event_tx = app.event_tx.clone();
    let slug_for_completion = slug.clone();
    let session_n_for_completion = session_n;

    // Spawn using the shared helper, then hook completion signal
    let ponder_label = format!("ponder: {slug}");
    let _ = spawn_agent_run(run_key.clone(), prompt, opts, &app, "ponder", &ponder_label).await?;

    // After spawn, watch for completion and emit ponder_run_completed.
    // We do this by watching the agent_runs map: when our key disappears,
    // the run finished. A short-polling task is sufficient here.
    let agent_runs = app.agent_runs.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let still_running = agent_runs.lock().await.contains_key(&run_key);
            if !still_running {
                let _ = event_tx.send(SseMessage::PonderRunCompleted {
                    slug: slug_for_completion,
                    session: session_n_for_completion,
                });
                break;
            }
        }
    });

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

    let event_tx = app.event_tx.clone();
    let slug_for_completion = slug.clone();
    let session_n_for_completion = session_n;

    // Spawn using the shared helper, then hook completion signal
    let investigation_label = format!("investigate: {slug}");
    let _ = spawn_agent_run(
        run_key.clone(),
        prompt,
        opts,
        &app,
        "investigation",
        &investigation_label,
    )
    .await?;

    // After spawn, watch for completion and emit investigation_run_completed.
    let agent_runs = app.agent_runs.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let still_running = agent_runs.lock().await.contains_key(&run_key);
            if !still_running {
                let _ = event_tx.send(SseMessage::InvestigationRunCompleted {
                    slug: slug_for_completion,
                    session: session_n_for_completion,
                });
                break;
            }
        }
    });

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
