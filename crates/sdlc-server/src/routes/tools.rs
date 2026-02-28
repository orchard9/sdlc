use axum::extract::{Path, State};
use axum::Json;
use tracing::warn;

use crate::error::AppError;
use crate::state::AppState;

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

            match sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root) {
                Ok(stdout) => match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(meta) => metas.push(meta),
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

        let stdout = sdlc_core::tool_runner::run_tool(&script, "--meta", None, &root)?;
        let meta: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| AppError(anyhow::anyhow!("tool --meta returned invalid JSON: {e}")))?;
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
/// Returns 503 if no JavaScript runtime is available.
/// Returns 422 if the tool exits non-zero (tool-level error; inspect `error` field).
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

        let stdin_json = serde_json::to_string(&input)
            .map_err(|e| AppError(anyhow::anyhow!("failed to serialize tool input: {e}")))?;
        let stdout = sdlc_core::tool_runner::run_tool(&script, "--run", Some(&stdin_json), &root)?;
        let output: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| AppError(anyhow::anyhow!("tool --run returned invalid JSON: {e}")))?;
        Ok(output)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
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

        let stdout = sdlc_core::tool_runner::run_tool(&script, "--setup", None, &root)?;
        let output: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| AppError(anyhow::anyhow!("tool --setup returned invalid JSON: {e}")))?;
        Ok(output)
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
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
