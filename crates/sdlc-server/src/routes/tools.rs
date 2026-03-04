use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, warn};

use crate::error::AppError;
use crate::state::{generate_run_id, AppState, SseMessage};

// ---------------------------------------------------------------------------
// GET /api/tools — list all installed tools with their metadata
// ---------------------------------------------------------------------------

/// GET /api/tools — enumerate `.sdlc/tools/` and run `--meta` on each tool.
///
/// Tools that fail `--meta` (e.g. bad script, runtime error) are skipped with
/// a WARN log — the list is always a best-effort response, never a hard error.
///
/// Returns 503 if no JavaScript runtime is available and tools are present.
/// Returns an empty array if no tools are installed or the tools dir is absent.
pub async fn list_tools(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let tools_dir = sdlc_core::paths::tools_dir(&root);
        if !tools_dir.is_dir() {
            return Ok::<_, anyhow::Error>(serde_json::json!([]));
        }

        let mut metas: Vec<serde_json::Value> = Vec::new();

        let mut entries: Vec<_> = std::fs::read_dir(&tools_dir)?
            .filter_map(|e| e.ok())
            .collect();
        // Sort for stable ordering
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip _shared and any non-directory entries
            if name.starts_with('_') || !entry.path().is_dir() {
                continue;
            }

            let script = sdlc_core::paths::tool_script(&root, &name);
            if !script.exists() {
                continue;
            }

            match sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root, None) {
                Ok(stdout) => match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(mut meta) => {
                        if let Some(obj) = meta.as_object_mut() {
                            obj.insert(
                                "built_in".into(),
                                serde_json::json!(sdlc_core::tool_runner::is_managed_tool(&name)),
                            );
                        }
                        inject_missing_secrets(&mut meta);
                        metas.push(meta)
                    }
                    Err(e) => warn!(tool = %name, error = %e, "tool --meta returned invalid JSON"),
                },
                Err(e) => warn!(tool = %name, error = %e, "tool --meta failed"),
            }
        }

        Ok(serde_json::json!(metas))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /api/tools/:name — single tool metadata
// ---------------------------------------------------------------------------

/// GET /api/tools/:name — run `--meta` on the named tool and return its metadata.
///
/// Returns 400 if the name contains invalid characters.
/// Returns 404 if the tool is not installed.
/// Returns 503 if no JavaScript runtime is available.
pub async fn get_tool_meta(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&name)?;

    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let script = sdlc_core::paths::tool_script(&root, &name);
        if !script.exists() {
            return Err(AppError::not_found(format!("tool '{name}' not found")));
        }

        let stdout = sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root, None)?;
        let mut meta: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| AppError(anyhow::anyhow!("tool --meta returned invalid JSON: {e}")))?;
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "built_in".into(),
                serde_json::json!(sdlc_core::tool_runner::is_managed_tool(&name)),
            );
        }
        inject_missing_secrets(&mut meta);
        Ok(meta)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/tools/:name/run — invoke a tool's --run mode
// ---------------------------------------------------------------------------

/// POST /api/tools/:name/run — feed the request body (JSON) to the tool's stdin
/// and return the tool's JSON output.
///
/// The request body must be a JSON object matching the tool's `input_schema`.
/// An empty body is treated as `{}`.
///
/// For non-streaming tools: returns 200 with the full `ToolResult` JSON once
/// execution finishes (synchronous, unchanged behavior).
///
/// For streaming tools (`streaming: true` in `--meta`): returns 202 immediately
/// with `{ "job_id": "<id>", "streaming": true }`. The tool runs in the
/// background; progress lines are emitted as `ToolRunProgress` SSE events on
/// the `"tool"` channel. Completion or failure is signalled via
/// `ToolRunCompleted` / `ToolRunFailed`.
///
/// Returns 400 if the name is invalid.
/// Returns 404 if the tool is not installed.
/// Returns 422 with `{ missing_secrets: [...] }` if required env vars are absent.
/// Returns 503 if no JavaScript runtime is available.
pub async fn run_tool(
    State(app): State<AppState>,
    Path(name): Path<String>,
    body: Option<Json<serde_json::Value>>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    validate_tool_name(&name)?;

    let input = body.map(|b| b.0).unwrap_or_else(|| serde_json::json!({}));

    let root = app.root.clone();
    let server_url = format!("http://localhost:{}", app.port);
    let agent_token = (*app.agent_token).clone();

    // Phase 1 (sync): fetch meta and resolve secrets
    let name_for_meta = name.clone();
    let root_for_meta = root.clone();
    let server_url_meta = server_url.clone();
    let agent_token_meta = agent_token.clone();
    let (meta, extra_env, script_path) = tokio::task::spawn_blocking(move || {
        let script = sdlc_core::paths::tool_script(&root_for_meta, &name_for_meta);
        if !script.exists() {
            return Err(AppError::not_found(format!(
                "tool '{name_for_meta}' not found"
            )));
        }
        let meta_result =
            sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root_for_meta, None);
        let meta = match meta_result {
            Ok(meta_stdout) => {
                sdlc_core::tool_runner::parse_tool_meta(&meta_stdout).unwrap_or_default()
            }
            Err(_) => sdlc_core::tool_runner::ToolMeta::default(),
        };
        let mut extra_env = resolve_secrets(&name_for_meta, &meta)?;
        extra_env.insert("SDLC_SERVER_URL".to_string(), server_url_meta);
        extra_env.insert("SDLC_AGENT_TOKEN".to_string(), agent_token_meta);
        Ok((meta, extra_env, script))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    // Phase 2: branch on streaming flag
    // When persist_interactions == Some(false) the tool has explicitly opted out
    // of interaction recording (e.g. sensitive credential management tools).
    let should_persist = meta.persist_interactions != Some(false);

    if meta.streaming == Some(true) {
        let interaction_id = generate_run_id();
        let record_init = sdlc_core::tool_interaction::ToolInteractionRecord {
            id: interaction_id.clone(),
            tool_name: name.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            input: input.clone(),
            result: None,
            status: "streaming".to_string(),
            tags: Vec::new(),
            notes: None,
            streaming_log: true,
        };
        if should_persist {
            let _ = sdlc_core::tool_interaction::save_interaction(&root, &record_init);
        }

        let event_tx = app.event_tx.clone();
        let iid = interaction_id.clone();
        let tool_name = name.clone();
        let root_task = root.clone();
        let input_task = input.clone();

        tokio::spawn(async move {
            let _ = event_tx.send(SseMessage::ToolRunStarted {
                name: tool_name.clone(),
                interaction_id: iid.clone(),
            });

            let runtime = match sdlc_core::tool_runner::detect_runtime() {
                Some(r) => r,
                None => {
                    let _ = event_tx.send(SseMessage::ToolRunFailed {
                        name: tool_name,
                        interaction_id: iid,
                        error: "no JavaScript runtime found".to_string(),
                    });
                    return;
                }
            };

            let script_str = match script_path.to_str() {
                Some(s) => s.to_string(),
                None => {
                    let _ = event_tx.send(SseMessage::ToolRunFailed {
                        name: tool_name,
                        interaction_id: iid,
                        error: "script path contains non-UTF8 characters".to_string(),
                    });
                    return;
                }
            };

            let stdin_json = match serde_json::to_string(&input_task) {
                Ok(s) => s,
                Err(e) => {
                    let _ = event_tx.send(SseMessage::ToolRunFailed {
                        name: tool_name,
                        interaction_id: iid,
                        error: format!("failed to serialize input: {e}"),
                    });
                    return;
                }
            };

            let (program, args) =
                sdlc_core::tool_runner::tool_spawn_args(runtime, &script_str, "--run");
            let mut cmd = tokio::process::Command::new(program);
            cmd.args(&args);
            cmd.env("SDLC_ROOT", &root_task);
            cmd.current_dir(&root_task);
            for (k, v) in &extra_env {
                cmd.env(k, v);
            }
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::inherit());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let _ = event_tx.send(SseMessage::ToolRunFailed {
                        name: tool_name,
                        interaction_id: iid,
                        error: format!("failed to spawn tool: {e}"),
                    });
                    return;
                }
            };

            if let Some(mut stdin_handle) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin_handle.write_all(stdin_json.as_bytes()).await;
            }

            let stdout_pipe = match child.stdout.take() {
                Some(s) => s,
                None => {
                    let _ = event_tx.send(SseMessage::ToolRunFailed {
                        name: tool_name,
                        interaction_id: iid,
                        error: "could not capture tool stdout".to_string(),
                    });
                    return;
                }
            };

            let log_path =
                sdlc_core::tool_interaction::streaming_log_path(&root_task, &tool_name, &iid);
            if let Some(dir) = log_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let mut log_file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .await
                .ok();

            let mut reader = BufReader::new(stdout_pipe).lines();
            let mut last_valid_json: Option<serde_json::Value> = None;

            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                if let Some(file) = log_file.as_mut() {
                    use tokio::io::AsyncWriteExt;
                    let _ = file.write_all(line.as_bytes()).await;
                    let _ = file.write_all(b"\n").await;
                }
                match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(val) => {
                        last_valid_json = Some(val.clone());
                        let _ = event_tx.send(SseMessage::ToolRunProgress {
                            name: tool_name.clone(),
                            interaction_id: iid.clone(),
                            line: val,
                        });
                    }
                    Err(e) => {
                        debug!(tool = %tool_name, "streaming tool emitted non-JSON line: {e}");
                    }
                }
            }

            let exit_ok = child.wait().await.map(|s| s.success()).unwrap_or(false);

            // The final ToolResult line is the last valid JSON that has an "ok" key
            let final_result = last_valid_json.filter(|v| v.get("ok").is_some());
            let result_ok = final_result
                .as_ref()
                .and_then(|v| v["ok"].as_bool())
                .unwrap_or(false);
            let final_status = if exit_ok || result_ok {
                "completed"
            } else {
                "failed"
            };

            // Update the persisted record (only if this tool persists interactions)
            if should_persist {
                if let Ok(mut rec) =
                    sdlc_core::tool_interaction::load_interaction(&root_task, &tool_name, &iid)
                {
                    rec.status = final_status.to_string();
                    rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
                    rec.result = final_result;
                    let _ = sdlc_core::tool_interaction::save_interaction(&root_task, &rec);
                }
                sdlc_core::tool_interaction::enforce_interaction_retention(
                    &root_task, &tool_name, 200,
                );
            }

            if final_status == "completed" {
                let _ = event_tx.send(SseMessage::ToolRunCompleted {
                    name: tool_name,
                    interaction_id: iid,
                });
            } else {
                let _ = event_tx.send(SseMessage::ToolRunFailed {
                    name: tool_name,
                    interaction_id: iid,
                    error: "tool exited non-zero or produced no valid output".to_string(),
                });
            }
        });

        return Ok((
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "job_id": interaction_id,
                "streaming": true
            })),
        ));
    }

    // --- Non-streaming (synchronous) path ---
    let result = tokio::task::spawn_blocking(move || {
        let script = script_path;

        let interaction_id = generate_run_id();
        let created_at = chrono::Utc::now().to_rfc3339();
        let mut record = sdlc_core::tool_interaction::ToolInteractionRecord {
            id: interaction_id.clone(),
            tool_name: name.clone(),
            created_at: created_at.clone(),
            completed_at: None,
            input: input.clone(),
            result: None,
            status: "running".to_string(),
            tags: Vec::new(),
            notes: None,
            streaming_log: false,
        };
        if should_persist {
            let _ = sdlc_core::tool_interaction::save_interaction(&root, &record);
        }

        let stdin_json = serde_json::to_string(&input)
            .map_err(|e| AppError(anyhow::anyhow!("failed to serialize tool input: {e}")))?;
        let run_result = sdlc_core::tool_runner::run_tool(
            &script,
            "--run",
            Some(&stdin_json),
            &root,
            Some(&extra_env),
        );

        let (output, status) = match run_result {
            Ok(stdout) => match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(val) => (val, "completed"),
                Err(e) => {
                    return Err(AppError(anyhow::anyhow!(
                        "tool --run returned invalid JSON: {e}"
                    )));
                }
            },
            Err(e) => return Err(AppError(e.into())),
        };

        record.status = status.to_string();
        record.completed_at = Some(chrono::Utc::now().to_rfc3339());
        record.result = Some(output.clone());
        if should_persist {
            let _ = sdlc_core::tool_interaction::save_interaction(&root, &record);
            sdlc_core::tool_interaction::enforce_interaction_retention(&root, &name, 200);
        }

        Ok(output)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok((StatusCode::OK, Json(result)))
}

// ---------------------------------------------------------------------------
// GET /api/tools/:name/interactions — list tool run history
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct InteractionListParams {
    limit: Option<usize>,
}

/// GET /api/tools/:name/interactions?limit=50 — list recent run records for a tool.
pub async fn list_tool_interactions(
    State(app): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<InteractionListParams>,
) -> Result<Json<Vec<sdlc_core::tool_interaction::ToolInteractionRecord>>, AppError> {
    validate_tool_name(&name)?;

    let root = app.root.clone();
    let limit = params.limit.unwrap_or(50);
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::tool_interaction::list_interactions(&root, &name, limit)
            .map_err(|e| AppError(e.into()))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/tools/:name/interactions/:id — single interaction record.
pub async fn get_tool_interaction(
    State(app): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<sdlc_core::tool_interaction::ToolInteractionRecord>, AppError> {
    validate_tool_name(&name)?;

    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::tool_interaction::load_interaction(&root, &name, &id).map_err(|_| {
            AppError::not_found(format!("interaction '{id}' not found for tool '{name}'"))
        })
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// DELETE /api/tools/:name/interactions/:id — delete an interaction record.
pub async fn delete_tool_interaction(
    State(app): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&name)?;

    let root = app.root.clone();
    tokio::task::spawn_blocking(move || {
        sdlc_core::tool_interaction::delete_interaction(&root, &name, &id)
            .map_err(|_| AppError::not_found(format!("interaction '{id}' not found")))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------------------------
// POST /api/tools/:name/setup — invoke a tool's --setup mode
// ---------------------------------------------------------------------------

/// POST /api/tools/:name/setup — run the tool's one-time setup (e.g. index build).
///
/// Only meaningful for tools with `requires_setup: true`. For other tools this
/// is a no-op at the tool level (the tool handles it gracefully).
///
/// Returns 400 if the name is invalid.
/// Returns 404 if the tool is not installed.
/// Returns 503 if no JavaScript runtime is available.
/// Returns 422 if setup exits non-zero.
pub async fn setup_tool(
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&name)?;

    let root = app.root.clone();
    let server_url = format!("http://localhost:{}", app.port);
    let agent_token = (*app.agent_token).clone();
    let result = tokio::task::spawn_blocking(move || {
        let script = sdlc_core::paths::tool_script(&root, &name);
        if !script.exists() {
            return Err(AppError::not_found(format!("tool '{name}' not found")));
        }

        // Resolve secrets so setup can reach external services (e.g. bot token check)
        let mut extra_env =
            match sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root, None) {
                Ok(meta_stdout) => match sdlc_core::tool_runner::parse_tool_meta(&meta_stdout) {
                    Ok(meta) => resolve_secrets(&name, &meta)?,
                    Err(_) => std::collections::HashMap::new(),
                },
                Err(_) => std::collections::HashMap::new(),
            };
        // Inject server contact info so tools can call /api/tools/agent-call
        extra_env.insert("SDLC_SERVER_URL".to_string(), server_url);
        extra_env.insert("SDLC_AGENT_TOKEN".to_string(), agent_token);

        let stdout =
            sdlc_core::tool_runner::run_tool(&script, "--setup", None, &root, Some(&extra_env))?;
        let output: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| AppError(anyhow::anyhow!("tool --setup returned invalid JSON: {e}")))?;
        Ok(output)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// POST /api/tools — scaffold a new tool skeleton
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CreateToolBody {
    name: String,
    description: String,
}

/// POST /api/tools — scaffold a new tool skeleton in `.sdlc/tools/<name>/`.
///
/// Creates `tool.ts`, `config.yaml`, and `README.md` from built-in templates.
///
/// Returns 400 if the name is an invalid slug.
/// Returns 409 Conflict if a tool with that name already exists.
pub async fn create_tool(
    State(app): State<AppState>,
    Json(body): Json<CreateToolBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&body.name)?;

    let root = app.root.clone();
    let name = body.name.clone();
    let desc = body.description.clone();

    tokio::task::spawn_blocking(move || sdlc_core::tool_runner::scaffold_tool(&root, &name, &desc))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
        .map_err(|e| match e {
            sdlc_core::error::SdlcError::ToolExists(ref n) => {
                AppError::conflict(format!("tool '{n}' already exists"))
            }
            sdlc_core::error::SdlcError::InvalidSlug(ref s) => {
                AppError::bad_request(format!("invalid tool name '{s}'"))
            }
            other => AppError(other.into()),
        })?;

    Ok(Json(serde_json::json!({
        "name": body.name,
        "status": "scaffolded"
    })))
}

// ---------------------------------------------------------------------------
// POST /api/tools/:name/clone — copy a tool to a new user-owned name
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct CloneToolBody {
    new_name: String,
}

/// POST /api/tools/:name/clone — copy `.sdlc/tools/<name>/` to `.sdlc/tools/<new_name>/`.
///
/// Intended for cloning built-in (managed) tools into a user-editable copy.
///
/// Returns 400 if either name is invalid.
/// Returns 404 if the source tool does not exist.
/// Returns 409 if the destination already exists.
pub async fn clone_tool(
    State(app): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<CloneToolBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&name)?;
    validate_tool_name(&body.new_name)?;

    let root = app.root.clone();
    let new_name = body.new_name.clone();

    tokio::task::spawn_blocking(move || {
        let tools_dir = sdlc_core::paths::tools_dir(&root);
        let src_dir = tools_dir.join(&name);
        let dst_dir = tools_dir.join(&new_name);

        if !src_dir.is_dir() {
            return Err(AppError::not_found(format!("tool '{name}' not found")));
        }
        if dst_dir.exists() {
            return Err(AppError::conflict(format!(
                "tool '{new_name}' already exists"
            )));
        }

        std::fs::create_dir_all(&dst_dir).map_err(|e| {
            AppError(anyhow::anyhow!(
                "failed to create destination directory: {e}"
            ))
        })?;

        for entry in std::fs::read_dir(&src_dir)
            .map_err(|e| AppError(anyhow::anyhow!("failed to read source directory: {e}")))?
        {
            let entry =
                entry.map_err(|e| AppError(anyhow::anyhow!("directory read error: {e}")))?;
            // Only copy flat files — tools are single-level directories.
            if entry.path().is_file() {
                let dst = dst_dir.join(entry.file_name());
                std::fs::copy(entry.path(), dst)
                    .map_err(|e| AppError(anyhow::anyhow!("failed to copy file: {e}")))?;
            }
        }

        Ok(serde_json::json!({
            "name": new_name,
            "cloned_from": name,
            "status": "cloned"
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
    .map(Json)
}

// ---------------------------------------------------------------------------
// POST /api/tools/agent-call — invoke a Claude agent from within a tool
// ---------------------------------------------------------------------------

/// Request body for POST /api/tools/agent-call.
#[derive(serde::Deserialize)]
pub struct AgentCallRequest {
    pub prompt: String,
    pub agent_file: Option<String>,
    pub max_turns: Option<u32>,
}

/// POST /api/tools/agent-call — allow a tool subprocess to spawn a Claude agent run.
///
/// Validates the `Authorization: Bearer <token>` header against `app.agent_token`.
/// Spawns the agent via `spawn_agent_run`, then waits synchronously (up to 10 min)
/// for the run to complete and returns the result text.
///
/// This endpoint is safe to call only from tools with `streaming: true` in their meta.
/// Calling it from a synchronous (blocking) tool will deadlock the server's thread pool.
///
/// Returns 401 if the bearer token is missing or invalid.
/// Returns 400 if agent_file is specified but cannot be read.
/// Returns 504 if the agent run does not complete within 10 minutes.
/// Returns 500 if the agent run fails.
pub async fn agent_call(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AgentCallRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate bearer token
    let provided = extract_bearer_token(&headers)
        .ok_or_else(|| AppError::unauthorized("missing Authorization: Bearer header"))?;
    if provided != *app.agent_token {
        return Err(AppError::unauthorized("invalid agent token"));
    }

    // Optionally prepend agent file contents to the prompt
    let prompt = if let Some(ref path) = body.agent_file {
        let agent_path = app.root.join(path);
        let content = tokio::fs::read_to_string(&agent_path)
            .await
            .map_err(|e| AppError::bad_request(format!("cannot read agent_file '{path}': {e}")))?;
        format!("{content}\n\n{}", body.prompt)
    } else {
        body.prompt.clone()
    };

    let max_turns = body.max_turns.unwrap_or(20).min(100);
    let opts = crate::routes::runs::sdlc_query_options(app.root.clone(), max_turns, None);
    let run_key = format!("tool-agent:{}", generate_run_id());

    // Spawn the agent run — this inserts a (tx, abort_handle) into agent_runs synchronously.
    let _ = crate::routes::runs::spawn_agent_run(
        run_key.clone(),
        prompt,
        opts,
        &app,
        "tool-agent",
        "agent-call",
        None,
    )
    .await?;

    // Subscribe to the broadcast channel for this run key
    let rx = {
        let runs = app.agent_runs.lock().await;
        runs.get(&run_key).map(|(tx, _)| tx.subscribe())
    };

    let mut rx = match rx {
        Some(r) => r,
        None => {
            // Run already completed (very fast agent) — look up result from history
            return get_run_result_from_history(&run_key, &app).await;
        }
    };

    // Wait for a result or error event, with 10-minute timeout
    use tokio::time::{timeout, Duration};
    match timeout(Duration::from_secs(600), async {
        while let Ok(msg) = rx.recv().await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&msg) {
                let event_type = val.get("type").and_then(|t| t.as_str());
                if event_type == Some("result") {
                    return Ok::<serde_json::Value, AppError>(val);
                }
                if event_type == Some("error") {
                    let err_msg = val
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("agent run failed");
                    return Err(AppError(anyhow::anyhow!("agent run failed: {err_msg}")));
                }
            }
        }
        // Channel closed — run completed; fetch final result from history
        get_run_result_from_history(&run_key, &app)
            .await
            .map(|j| j.0)
    })
    .await
    {
        Ok(Ok(result_val)) => {
            let text = result_val
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let cost = result_val.get("cost_usd").and_then(|c| c.as_f64());
            let turns = result_val.get("turns").and_then(|t| t.as_u64());
            Ok(Json(serde_json::json!({
                "result": text,
                "cost_usd": cost,
                "turns": turns,
            })))
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError(anyhow::anyhow!(
            "agent-call timed out after 10 minutes"
        ))),
    }
}

/// Look up the result text for a completed run from the in-memory run history.
async fn get_run_result_from_history(
    run_key: &str,
    app: &AppState,
) -> Result<Json<serde_json::Value>, AppError> {
    let history = app.run_history.lock().await;
    if let Some(rec) = history.iter().find(|r| r.key == run_key) {
        if rec.status == "failed" {
            let msg = rec
                .error
                .clone()
                .unwrap_or_else(|| "agent run failed".into());
            return Err(AppError(anyhow::anyhow!("agent run failed: {msg}")));
        }
        return Ok(Json(serde_json::json!({
            "result": "",
            "cost_usd": rec.cost_usd,
            "turns": rec.turns,
        })));
    }
    Err(AppError(anyhow::anyhow!("run not found in history")))
}

/// Extract the token value from an `Authorization: Bearer <token>` header.
pub(crate) fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Inject a `missing_secrets` array into a tool meta JSON object.
///
/// Reads the `secrets` array from the meta (if present) and checks each
/// required entry against `std::env::var`. Missing required secrets are
/// collected and written back as `meta["missing_secrets"]`.
///
/// This is a best-effort operation — any parse failures are silently ignored
/// and the field is simply omitted.
fn inject_missing_secrets(meta: &mut serde_json::Value) {
    let Some(obj) = meta.as_object_mut() else {
        return;
    };

    let secrets_val = match obj.get("secrets") {
        Some(v) => v.clone(),
        None => return,
    };

    let secrets: Vec<sdlc_core::tool_runner::SecretRef> = match serde_json::from_value(secrets_val)
    {
        Ok(v) => v,
        Err(_) => return,
    };

    // Also export the secrets_env group if declared, so the UI shows accurate missing status.
    let exported: HashMap<String, String> = obj
        .get("secrets_env")
        .and_then(|v| v.as_str())
        .map(sdlc_core::tool_runner::export_secrets_env)
        .unwrap_or_default();

    let missing: Vec<String> = secrets
        .iter()
        .filter(|s| {
            s.required && std::env::var(&s.env_var).is_err() && !exported.contains_key(&s.env_var)
        })
        .map(|s| s.env_var.clone())
        .collect();

    obj.insert("missing_secrets".into(), serde_json::json!(missing));
}

/// Validate that a tool name contains only safe characters.
/// Tool names follow the same rules as slugs: a-z, A-Z, 0-9, hyphen, underscore.
fn validate_tool_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(format!(
            "Invalid tool name '{name}': must contain only letters, digits, hyphens, and underscores"
        )));
    }
    Ok(())
}

/// Resolve environment variable secrets declared in tool meta.
///
/// Resolution order for each declared secret:
///   1. The server process's own environment (`std::env::var`)
///   2. `sdlc secrets env export <secrets_env>` — when `meta.secrets_env` is set,
///      decrypts and imports the named secrets group before checking.
///
/// If any required secrets are still missing after both lookups, returns 422.
/// Collected values are returned as a HashMap for injection into the subprocess.
fn resolve_secrets(
    tool_name: &str,
    meta: &sdlc_core::tool_runner::ToolMeta,
) -> Result<HashMap<String, String>, AppError> {
    let Some(secrets) = &meta.secrets else {
        return Ok(HashMap::new());
    };

    // If the tool declares a secrets_env group, export it now and merge into our lookup map.
    let mut exported: HashMap<String, String> = HashMap::new();
    if let Some(group) = &meta.secrets_env {
        exported = sdlc_core::tool_runner::export_secrets_env(group);
    }

    let mut env_map = HashMap::new();
    let mut missing: Vec<String> = Vec::new();

    for secret in secrets {
        // Prefer live env var, fall back to exported secrets group.
        let val = std::env::var(&secret.env_var)
            .ok()
            .or_else(|| exported.get(&secret.env_var).cloned());

        match val {
            Some(v) => {
                env_map.insert(secret.env_var.clone(), v);
            }
            None => {
                if secret.required {
                    missing.push(secret.env_var.clone());
                }
            }
        }
    }

    if !missing.is_empty() {
        let group_hint = meta
            .secrets_env
            .as_deref()
            .map(|g| {
                format!(
                    " Run `sdlc secrets env set {g} {}=<value>` to add it.",
                    missing[0]
                )
            })
            .unwrap_or_default();
        let body = serde_json::json!({
            "error": format!(
                "Tool '{}' requires environment variable(s) that are not set: {}.{}",
                tool_name,
                missing.join(", "),
                group_hint,
            ),
            "missing_secrets": missing,
        });
        return Err(AppError::unprocessable_json(body));
    }

    Ok(env_map)
}

// ---------------------------------------------------------------------------
// POST /api/tools/agent-dispatch — fire-and-forget agent dispatch from tools
// ---------------------------------------------------------------------------

/// Request body for POST /api/tools/agent-dispatch.
#[derive(serde::Deserialize)]
pub struct AgentDispatchRequest {
    /// The prompt (slash command or free text) to send to the agent.
    pub prompt: String,
    /// Deduplication key — if a run with this key is already in flight,
    /// the endpoint returns 409 Conflict.
    pub run_key: String,
    /// Human-readable label shown in the activity feed.
    pub label: String,
    /// Maximum number of agent turns (default 40, capped at 100).
    pub max_turns: Option<u32>,
}

/// POST /api/tools/agent-dispatch — non-blocking agent dispatch for tool subprocesses.
///
/// Unlike `agent-call` (which blocks until completion), this endpoint returns
/// 202 Accepted immediately after inserting the run into `agent_runs`. The
/// agent runs in the background; its progress is visible via SSE and the
/// activity feed.
///
/// Validates the `Authorization: Bearer <token>` header against `app.agent_token`.
/// Returns 409 Conflict if a run with the same `run_key` is already in flight.
/// Returns 400 if `prompt`, `run_key`, or `label` are empty.
/// Returns 401 if the bearer token is missing or invalid.
pub async fn agent_dispatch(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AgentDispatchRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate bearer token
    let provided = extract_bearer_token(&headers)
        .ok_or_else(|| AppError::unauthorized("missing Authorization: Bearer header"))?;
    if provided != *app.agent_token {
        return Err(AppError::unauthorized("invalid agent token"));
    }

    // Validate required fields
    if body.prompt.is_empty() {
        return Err(AppError::bad_request("prompt must not be empty"));
    }
    if body.run_key.is_empty() {
        return Err(AppError::bad_request("run_key must not be empty"));
    }
    if body.label.is_empty() {
        return Err(AppError::bad_request("label must not be empty"));
    }

    let max_turns = body.max_turns.unwrap_or(40).min(100);
    let opts = crate::routes::runs::sdlc_query_options(app.root.clone(), max_turns, None);

    // spawn_agent_run returns 409 if a run with body.run_key is already in flight.
    let result = crate::routes::runs::spawn_agent_run(
        body.run_key.clone(),
        body.prompt.clone(),
        opts,
        &app,
        "dev-driver",
        &body.label,
        None,
    )
    .await?;

    // Return 202 immediately — the agent runs in the background.
    let run_id = result
        .0
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Json(serde_json::json!({
        "run_id": run_id,
        "run_key": body.run_key,
        "status": "started",
    })))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    // -----------------------------------------------------------------
    // list_tools
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn list_tools_returns_empty_array_when_no_tools_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_tools(State(app)).await.unwrap();
        assert_eq!(result.0, serde_json::json!([]));
    }

    #[tokio::test]
    async fn list_tools_returns_empty_array_when_only_shared_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        // Create _shared (no actual tools)
        std::fs::create_dir_all(dir.path().join(".sdlc/tools/_shared")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = list_tools(State(app)).await.unwrap();
        assert_eq!(result.0, serde_json::json!([]));
    }

    #[tokio::test]
    async fn list_tools_skips_dirs_without_tool_ts() {
        let dir = tempfile::TempDir::new().unwrap();
        // Create a tool dir with no tool.ts
        std::fs::create_dir_all(dir.path().join(".sdlc/tools/my-tool")).unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        // Should not error — just returns empty since tool.ts is absent
        let result = list_tools(State(app)).await.unwrap();
        assert_eq!(result.0, serde_json::json!([]));
    }

    // -----------------------------------------------------------------
    // get_tool_meta
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn get_tool_returns_404_when_not_found() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = get_tool_meta(State(app), Path("no-such-tool".to_string())).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_tool_returns_400_for_invalid_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = get_tool_meta(State(app), Path("bad name with spaces".to_string())).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------
    // run_tool
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn run_tool_returns_404_when_not_found() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = run_tool(State(app), Path("no-such-tool".to_string()), None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn run_tool_returns_400_for_invalid_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = run_tool(State(app), Path("bad/name".to_string()), None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------
    // setup_tool
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn setup_tool_returns_404_when_not_found() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = setup_tool(State(app), Path("no-such-tool".to_string())).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn setup_tool_returns_400_for_invalid_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = setup_tool(State(app), Path("../traversal".to_string())).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------
    // create_tool
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn create_tool_returns_scaffolded_on_success() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = CreateToolBody {
            name: "my-new-tool".to_string(),
            description: "A test tool".to_string(),
        };
        let result = create_tool(State(app), Json(body)).await.unwrap();
        assert_eq!(result.0["name"], "my-new-tool");
        assert_eq!(result.0["status"], "scaffolded");
        // Verify files were created
        assert!(sdlc_core::paths::tool_script(dir.path(), "my-new-tool").exists());
    }

    #[tokio::test]
    async fn create_tool_returns_409_for_existing_tool() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        // Scaffold once
        let body1 = CreateToolBody {
            name: "existing-tool".to_string(),
            description: "First".to_string(),
        };
        let _ = create_tool(State(app.clone()), Json(body1)).await.unwrap();
        // Try again
        let body2 = CreateToolBody {
            name: "existing-tool".to_string(),
            description: "Second".to_string(),
        };
        let result = create_tool(State(app), Json(body2)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn create_tool_returns_400_for_invalid_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = CreateToolBody {
            name: "INVALID NAME".to_string(),
            description: "desc".to_string(),
        };
        let result = create_tool(State(app), Json(body)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let response = axum::response::IntoResponse::into_response(err);
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------
    // agent_dispatch
    // -----------------------------------------------------------------

    fn make_auth_header(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    #[tokio::test]
    async fn agent_dispatch_rejects_missing_bearer_token() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AgentDispatchRequest {
            prompt: "/sdlc-next my-feature".to_string(),
            run_key: "dev-driver:feature:my-feature".to_string(),
            label: "test label".to_string(),
            max_turns: None,
        };
        let result = agent_dispatch(State(app), HeaderMap::new(), Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn agent_dispatch_rejects_empty_prompt() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let token = (*app.agent_token).clone();
        let body = AgentDispatchRequest {
            prompt: "".to_string(),
            run_key: "dev-driver:feature:my-feature".to_string(),
            label: "test label".to_string(),
            max_turns: None,
        };
        let result = agent_dispatch(State(app), make_auth_header(&token), Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn agent_dispatch_rejects_empty_run_key() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let token = (*app.agent_token).clone();
        let body = AgentDispatchRequest {
            prompt: "/sdlc-next my-feature".to_string(),
            run_key: "".to_string(),
            label: "test label".to_string(),
            max_turns: None,
        };
        let result = agent_dispatch(State(app), make_auth_header(&token), Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn agent_dispatch_rejects_empty_label() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let token = (*app.agent_token).clone();
        let body = AgentDispatchRequest {
            prompt: "/sdlc-next my-feature".to_string(),
            run_key: "dev-driver:feature:my-feature".to_string(),
            label: "".to_string(),
            max_turns: None,
        };
        let result = agent_dispatch(State(app), make_auth_header(&token), Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------
    // validate_tool_name
    // -----------------------------------------------------------------

    #[test]
    fn validate_tool_name_accepts_valid_names() {
        assert!(validate_tool_name("ama").is_ok());
        assert!(validate_tool_name("quality-check").is_ok());
        assert!(validate_tool_name("my_tool_v2").is_ok());
        assert!(validate_tool_name("Tool123").is_ok());
    }

    #[test]
    fn validate_tool_name_rejects_invalid_names() {
        assert!(validate_tool_name("").is_err());
        assert!(validate_tool_name("bad name").is_err());
        assert!(validate_tool_name("../traversal").is_err());
        assert!(validate_tool_name("tool/path").is_err());
        assert!(validate_tool_name("tool.ts").is_err());
    }

    // -----------------------------------------------------------------
    // extract_bearer_token
    // -----------------------------------------------------------------

    #[test]
    fn extract_bearer_token_parses_valid_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer my-secret-token".parse().unwrap(),
        );
        let token = extract_bearer_token(&headers);
        assert_eq!(token.as_deref(), Some("my-secret-token"));
    }

    #[test]
    fn extract_bearer_token_returns_none_for_missing_header() {
        let headers = HeaderMap::new();
        assert!(extract_bearer_token(&headers).is_none());
    }

    // -----------------------------------------------------------------
    // agent_call (token validation)
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn agent_call_returns_401_for_missing_token() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AgentCallRequest {
            prompt: "do something".to_string(),
            agent_file: None,
            max_turns: None,
        };
        let result = agent_call(State(app), HeaderMap::new(), Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn agent_call_returns_401_for_wrong_token() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let body = AgentCallRequest {
            prompt: "do something".to_string(),
            agent_file: None,
            max_turns: None,
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer wrong-token-value".parse().unwrap(),
        );
        let result = agent_call(State(app), headers, Json(body)).await;
        assert!(result.is_err());
        let response = axum::response::IntoResponse::into_response(result.unwrap_err());
        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
