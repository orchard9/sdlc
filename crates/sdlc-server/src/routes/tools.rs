use axum::extract::{Path, Query, State};
use axum::Json;
use std::collections::HashMap;
use tracing::warn;

use crate::error::AppError;
use crate::state::{generate_run_id, AppState};

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
/// Returns 400 if the name is invalid.
/// Returns 404 if the tool is not installed.
/// Returns 422 with `{ missing_secrets: [...] }` if required env vars are absent.
/// Returns 503 if no JavaScript runtime is available.
pub async fn run_tool(
    State(app): State<AppState>,
    Path(name): Path<String>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_tool_name(&name)?;

    let input = body.map(|b| b.0).unwrap_or_else(|| serde_json::json!({}));

    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let script = sdlc_core::paths::tool_script(&root, &name);
        if !script.exists() {
            return Err(AppError::not_found(format!("tool '{name}' not found")));
        }

        // Fetch tool meta to resolve secrets declarations
        let extra_env = match sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root, None)
        {
            Ok(meta_stdout) => match sdlc_core::tool_runner::parse_tool_meta(&meta_stdout) {
                Ok(meta) => resolve_secrets(&name, &meta)?,
                Err(_) => HashMap::new(),
            },
            Err(_) => HashMap::new(),
        };

        // Create interaction record (status: running)
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
        // Best-effort — don't fail the run if persistence fails
        let _ = sdlc_core::tool_interaction::save_interaction(&root, &record);

        // Run the tool
        let stdin_json = serde_json::to_string(&input)
            .map_err(|e| AppError(anyhow::anyhow!("failed to serialize tool input: {e}")))?;
        let run_result = sdlc_core::tool_runner::run_tool(
            &script,
            "--run",
            Some(&stdin_json),
            &root,
            Some(&extra_env),
        );

        // Update interaction record with result
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
        let _ = sdlc_core::tool_interaction::save_interaction(&root, &record);
        sdlc_core::tool_interaction::enforce_interaction_retention(&root, &name, 200);

        Ok(output)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
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
    let result = tokio::task::spawn_blocking(move || {
        let script = sdlc_core::paths::tool_script(&root, &name);
        if !script.exists() {
            return Err(AppError::not_found(format!("tool '{name}' not found")));
        }

        let stdout = sdlc_core::tool_runner::run_tool(&script, "--setup", None, &root, None)?;
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
// Internal helpers
// ---------------------------------------------------------------------------

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
/// Checks `std::env::var` for each declared secret. If any required secrets
/// are missing, returns a 422 AppError with `missing_secrets` in the body.
/// Collected values are returned as a HashMap for injection into the subprocess.
fn resolve_secrets(
    tool_name: &str,
    meta: &sdlc_core::tool_runner::ToolMeta,
) -> Result<HashMap<String, String>, AppError> {
    let Some(secrets) = &meta.secrets else {
        return Ok(HashMap::new());
    };

    let mut env_map = HashMap::new();
    let mut missing: Vec<String> = Vec::new();

    for secret in secrets {
        match std::env::var(&secret.env_var) {
            Ok(val) => {
                env_map.insert(secret.env_var.clone(), val);
            }
            Err(_) => {
                if secret.required {
                    missing.push(secret.env_var.clone());
                }
            }
        }
    }

    if !missing.is_empty() {
        let body = serde_json::json!({
            "error": format!(
                "Tool '{}' requires environment variable(s) that are not set: {}. \
                 Export them in your shell before running the tool.",
                tool_name,
                missing.join(", ")
            ),
            "missing_secrets": missing,
        });
        return Err(AppError::unprocessable_json(body));
    }

    Ok(env_map)
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
}
