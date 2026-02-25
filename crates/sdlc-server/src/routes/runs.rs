use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use std::convert::Infallible;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::error::AppError;
use crate::state::{AppState, RunEvent};
use crate::subprocess;

/// Only these command prefixes are allowed via the generic run-command endpoint.
const ALLOWED_PREFIXES: &[&str] = &["sdlc", "xadk"];

/// POST /api/run/:slug — classify, spawn backend subprocess, return run_id.
pub async fn run_feature(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();

    // Classify on blocking thread
    let classification = tokio::task::spawn_blocking({
        let root = root.clone();
        let slug = slug.clone();
        move || {
            let config = sdlc_core::config::Config::load(&root)?;
            let state = sdlc_core::state::State::load(&root)?;
            let feature = sdlc_core::feature::Feature::load(&root, &slug)?;

            let ctx = sdlc_core::classifier::EvalContext {
                feature: &feature,
                state: &state,
                config: &config,
                root: &root,
            };
            let classifier =
                sdlc_core::classifier::Classifier::new(sdlc_core::rules::default_rules());
            let c = classifier.classify(&ctx);

            // Build argv from classification (mirroring cmd/run.rs logic)
            let backend = config.agents.backend_for(c.action);
            let context_str = build_context_str(&c, &root);
            let argv = build_argv(backend, &context_str);

            Ok::<_, sdlc_core::SdlcError>((c, argv))
        }
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    let (classification, argv) = classification;

    if classification.action == sdlc_core::types::ActionType::Done {
        return Ok(Json(serde_json::json!({
            "status": "done",
            "message": classification.message,
        })));
    }

    if argv.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "human_required",
            "message": classification.message,
            "next_command": classification.next_command,
        })));
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let handle = subprocess::spawn_process(argv, &root);

    // Sweep completed runs to prevent memory leaks, then insert new one.
    app.sweep_completed_runs().await;
    app.runs.write().await.insert(run_id.clone(), handle);

    Ok(Json(serde_json::json!({
        "run_id": run_id,
        "action": classification.action,
        "message": classification.message,
    })))
}

#[derive(serde::Deserialize)]
pub struct RunCommandBody {
    pub argv: Vec<String>,
}

/// POST /api/run-command — generic subprocess for setup wizard.
pub async fn run_command(
    State(app): State<AppState>,
    Json(body): Json<RunCommandBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.argv.is_empty() {
        return Err(AppError(anyhow::anyhow!("argv must not be empty")));
    }

    let cmd = &body.argv[0];
    if !ALLOWED_PREFIXES.iter().any(|p| cmd.starts_with(p)) {
        return Err(AppError(anyhow::anyhow!(
            "Command not allowed: only sdlc and xadk commands are permitted"
        )));
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let handle = subprocess::spawn_process(body.argv, &app.root);

    app.sweep_completed_runs().await;
    app.runs.write().await.insert(run_id.clone(), handle);

    Ok(Json(serde_json::json!({
        "run_id": run_id,
    })))
}

/// GET /api/runs/:run_id/stream — SSE stream of subprocess output.
pub async fn stream_run(
    State(app): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let runs = app.runs.read().await;
    let handle = runs
        .get(&run_id)
        .ok_or_else(|| AppError(anyhow::anyhow!("run not found: {run_id}")))?;

    // Take the pre-subscribed receiver if available (first subscriber gets all
    // events from the start, preventing the race where a fast subprocess finishes
    // before any SSE client subscribes). Subsequent subscribers get a fresh
    // subscription and may miss events that already fired.
    let rx = handle
        .initial_rx
        .lock()
        .unwrap()
        .take()
        .unwrap_or_else(|| handle.tx.subscribe());
    drop(runs);

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let data = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(data)))
        }
        Err(BroadcastStreamRecvError::Lagged(n)) => {
            let event = RunEvent::Error {
                message: format!("Dropped {n} events (output too fast)"),
            };
            let data = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(data)))
        }
    });

    Ok(Sse::new(stream))
}

// --- helpers (mirroring cmd/run.rs) ---

fn build_context_str(c: &sdlc_core::classifier::Classification, root: &std::path::Path) -> String {
    let mut parts = vec![
        format!("Feature: {}", c.feature),
        format!("Title: {}", c.title),
    ];
    if let Some(ref desc) = c.description {
        parts.push(format!("Description: {desc}"));
    }
    parts.push(format!("Phase: {}", c.current_phase));
    parts.push(format!("Action: {}", c.action));
    parts.push(c.message.clone());
    if !c.next_command.is_empty() {
        parts.push(format!("Next command: {}", c.next_command));
    }
    if let Some(ref path) = c.output_path {
        parts.push(format!("Output path: {path}"));
    }

    let vision_path = sdlc_core::paths::vision_md_path(root);
    if let Ok(vision) = std::fs::read_to_string(&vision_path) {
        if !vision.trim().is_empty() {
            parts.push(String::new());
            parts.push("--- VISION.md ---".to_string());
            parts.push(vision);
        }
    }

    parts.join("\n")
}

fn build_argv(backend: &sdlc_core::config::AgentBackend, context: &str) -> Vec<String> {
    match backend {
        sdlc_core::config::AgentBackend::Xadk { agent_id, .. } => vec![
            "python".to_string(),
            "-m".to_string(),
            "xadk".to_string(),
            agent_id.clone(),
            "--prompt".to_string(),
            context.to_string(),
        ],
        sdlc_core::config::AgentBackend::ClaudeAgentSdk {
            model,
            allowed_tools,
            permission_mode,
            timeout_minutes,
        } => {
            let mut argv = vec![
                "claude".to_string(),
                "-p".to_string(),
                context.to_string(),
                "--model".to_string(),
                model.clone(),
            ];
            if !allowed_tools.is_empty() {
                argv.push("--allowedTools".to_string());
                argv.push(allowed_tools.join(","));
            }
            if let Some(mode) = permission_mode {
                argv.push("--permission-mode".to_string());
                argv.push(mode.clone());
            }
            if let Some(t) = timeout_minutes {
                argv.push("--timeout".to_string());
                argv.push(format!("{}", t * 60));
            }
            argv
        }
        sdlc_core::config::AgentBackend::Human => Vec::new(),
    }
}
