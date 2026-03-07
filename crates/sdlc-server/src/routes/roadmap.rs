use axum::extract::{Multipart, Path, Query, State};
use axum::response::Response;
use axum::{http::header, Json};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Query parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
pub struct ListPondersQuery {
    /// When true, include merged entries in the list (hidden by default).
    #[serde(default)]
    pub all: Option<bool>,
}

// ---------------------------------------------------------------------------
// Session route parameter types
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct SessionPath {
    pub slug: String,
    pub n: u32,
}

/// GET /api/roadmap — list all ponder entries with artifact counts and team size.
pub async fn list_ponders(
    State(app): State<AppState>,
    Query(params): Query<ListPondersQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let show_all = params.all.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::ponder::PonderEntry::list(&root)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .filter(|e| show_all || e.merged_into.is_none())
            .map(|e| {
                let artifact_count = sdlc_core::ponder::list_artifacts(&root, &e.slug)
                    .map(|a| a.len())
                    .unwrap_or(0);
                let team_size = sdlc_core::ponder::load_team(&root, &e.slug)
                    .map(|t| t.partners.len())
                    .unwrap_or(0);
                // Best-effort: read last session and extract preview text.
                // Any I/O failure silently produces null.
                let last_session_preview: Option<String> =
                    sdlc_core::ponder::list_sessions(&root, &e.slug)
                        .ok()
                        .and_then(|sessions| sessions.into_iter().last())
                        .and_then(|last| {
                            sdlc_core::ponder::read_session(&root, &e.slug, last.session).ok()
                        })
                        .and_then(|content| {
                            sdlc_core::workspace::extract_session_preview(&content)
                        });
                serde_json::json!({
                    "slug": e.slug,
                    "title": e.title,
                    "status": e.status.to_string(),
                    "tags": e.tags,
                    "sessions": e.sessions,
                    "artifact_count": artifact_count,
                    "team_size": team_size,
                    "created_at": e.created_at,
                    "updated_at": e.updated_at,
                    "committed_at": e.committed_at,
                    "committed_to": e.committed_to,
                    "merged_into": e.merged_into,
                    "merged_from": e.merged_from,
                    "last_session_preview": last_session_preview,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug — full detail: manifest + team + artifacts.
pub async fn get_ponder(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::ponder::PonderEntry::load(&root, &slug)?;
        let team = sdlc_core::ponder::load_team(&root, &slug)?;
        let artifacts = sdlc_core::ponder::list_artifacts(&root, &slug)?;

        let artifact_list: Vec<serde_json::Value> = artifacts
            .iter()
            .map(|a| {
                let content = sdlc_core::ponder::read_artifact(&root, &slug, &a.filename).ok();
                serde_json::json!({
                    "filename": a.filename,
                    "size_bytes": a.size_bytes,
                    "modified_at": a.modified_at,
                    "content": content,
                })
            })
            .collect();

        let orientation = entry.orientation.as_ref().map(|o| {
            serde_json::json!({
                "current": o.current,
                "next": o.next,
                "commit": o.commit,
            })
        });

        let redirect_banner = entry
            .merged_into
            .as_ref()
            .map(|target| format!("This entry was merged into '{target}'"));

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
            "sessions": entry.sessions,
            "orientation": orientation,
            "committed_at": entry.committed_at,
            "committed_to": entry.committed_to,
            "merged_into": entry.merged_into,
            "merged_from": entry.merged_from,
            "redirect_banner": redirect_banner,
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "team": team.partners,
            "artifacts": artifact_list,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CreatePonderBody {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub brief: Option<String>,
}

/// POST /api/roadmap — create a new ponder entry.
pub async fn create_ponder(
    State(app): State<AppState>,
    Json(body): Json<CreatePonderBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let entry = sdlc_core::ponder::PonderEntry::create(&root, body.slug, body.title)?;

        if let Some(brief) = body.brief {
            sdlc_core::ponder::capture_content(&root, &entry.slug, "brief.md", &brief)?;
        }

        if let Ok(mut state) = sdlc_core::state::State::load(&root) {
            state.add_ponder(&entry.slug);
            let _ = state.save(&root);
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct CaptureArtifactBody {
    pub filename: String,
    pub content: String,
}

/// POST /api/roadmap/:slug/capture — capture content into scrapbook.
pub async fn capture_artifact(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<CaptureArtifactBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        sdlc_core::ponder::capture_content(&root, &slug, &body.filename, &body.content)?;
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

#[derive(serde::Deserialize)]
pub struct UpdatePonderBody {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub committed_to: Option<Vec<String>>,
}

/// PUT /api/roadmap/:slug — update status/title/tags.
pub async fn update_ponder(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<UpdatePonderBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut entry = sdlc_core::ponder::PonderEntry::load(&root, &slug)?;

        if let Some(status_str) = body.status {
            let status: sdlc_core::ponder::PonderStatus = status_str.parse()?;
            entry.update_status(status);
        }
        if let Some(title) = body.title {
            entry.update_title(title);
        }
        if let Some(tags) = body.tags {
            entry.set_tags(tags);
        }
        if let Some(milestones) = body.committed_to {
            entry.set_committed_to(milestones);
        }

        entry.save(&root)?;

        // Sync active_ponders when committed or parked
        if matches!(
            entry.status,
            sdlc_core::ponder::PonderStatus::Parked | sdlc_core::ponder::PonderStatus::Committed
        ) {
            if let Ok(mut state) = sdlc_core::state::State::load(&root) {
                state.remove_ponder(&slug);
                let _ = state.save(&root);
            }
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": entry.slug,
            "title": entry.title,
            "status": entry.status.to_string(),
            "tags": entry.tags,
            "committed_to": entry.committed_to,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// DELETE /api/roadmap/:slug — permanently delete a ponder entry and all its artifacts.
pub async fn delete_ponder(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        // Verify entry exists
        let _entry = sdlc_core::ponder::PonderEntry::load(&root, &slug)?;

        let dir = sdlc_core::paths::ponder_dir(&root, &slug);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }

        if let Ok(mut state) = sdlc_core::state::State::load(&root) {
            state.remove_ponder(&slug);
            let _ = state.save(&root);
        }

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "deleted": true,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug/sessions — list session metadata, sorted by session number.
pub async fn list_ponder_sessions(
    State(app): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let sessions = sdlc_core::ponder::list_sessions(&root, &slug)?;
        let list: Vec<serde_json::Value> = sessions
            .iter()
            .map(|s| {
                let orientation = s.orientation.as_ref().map(|o| {
                    serde_json::json!({
                        "current": o.current,
                        "next": o.next,
                        "commit": o.commit,
                    })
                });
                serde_json::json!({
                    "session": s.session,
                    "timestamp": s.timestamp,
                    "orientation": orientation,
                })
            })
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

/// GET /api/roadmap/:slug/sessions/:n — full content of a single session.
pub async fn get_ponder_session(
    State(app): State<AppState>,
    Path(SessionPath { slug, n }): Path<SessionPath>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let content = sdlc_core::ponder::read_session(&root, &slug, n)?;
        let meta = sdlc_core::ponder::parse_session_meta(&content);
        let orientation = meta.as_ref().and_then(|m| m.orientation.as_ref()).map(|o| {
            serde_json::json!({
                "current": o.current,
                "next": o.next,
                "commit": o.commit,
            })
        });
        let timestamp = meta.as_ref().map(|m| m.timestamp);
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "session": n,
            "timestamp": timestamp,
            "orientation": orientation,
            "content": content,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Binary media upload and serve
// ---------------------------------------------------------------------------

/// Returns the MIME type for an allowed ponder image extension, or `None` if
/// the file type is not permitted.  Only PNG, JPEG, GIF, and WebP are accepted.
fn ponder_mime_for_ext(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

/// Validate that a media filename is safe (no path traversal, no separators).
fn validate_media_filename(filename: &str) -> Result<(), AppError> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::bad_request(
            "invalid filename: must not contain path separators or '..'",
        ));
    }
    Ok(())
}

const MAX_MEDIA_BYTES: usize = 10 * 1024 * 1024; // 10 MB

/// POST /api/roadmap/:slug/media — upload a binary image into the ponder workspace.
///
/// Accepts `multipart/form-data` with a field named `file`.
/// Allowed types: PNG, JPEG, GIF, WebP.  Maximum size: 10 MB.
pub async fn upload_ponder_media(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    // Find the "file" field in the multipart body.
    let mut found_filename: Option<String> = None;
    let mut found_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError(anyhow::anyhow!("multipart error: {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::bad_request("missing filename in multipart field"))?;

        // Security: reject path-traversal filenames early.
        validate_media_filename(&filename)?;

        // Type guard: only allowed image extensions.
        if ponder_mime_for_ext(&filename).is_none() {
            return Err(AppError::bad_request(
                "unsupported file type: only PNG, JPEG, GIF, WebP are allowed",
            ));
        }

        // Accumulate bytes with a size cap.
        let raw = field
            .bytes()
            .await
            .map_err(|e| AppError(anyhow::anyhow!("failed to read field bytes: {e}")))?;

        if raw.len() > MAX_MEDIA_BYTES {
            return Err(AppError::payload_too_large("file too large: maximum 10 MB"));
        }

        found_filename = Some(filename);
        found_bytes = Some(raw.to_vec());
        break;
    }

    let filename = found_filename
        .ok_or_else(|| AppError::bad_request("no 'file' field found in multipart body"))?;
    let bytes = found_bytes.unwrap();

    // Write to disk.
    let media_dir = sdlc_core::paths::ponder_media_dir(&app.root, &slug);
    tokio::fs::create_dir_all(&media_dir)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("failed to create media dir: {e}")))?;

    let dest = media_dir.join(&filename);
    // Atomic write: write to a .tmp file, then rename into place.
    let tmp = dest.with_extension(format!(
        "{}.tmp",
        dest.extension().unwrap_or_default().to_string_lossy()
    ));
    tokio::fs::write(&tmp, &bytes)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("failed to write temp file: {e}")))?;
    tokio::fs::rename(&tmp, &dest)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("failed to rename media file: {e}")))?;

    // Notify the frontend via SSE.
    let _ = app.event_tx.send(crate::state::SseMessage::Update);

    let url = format!("/api/roadmap/{slug}/media/{filename}");
    Ok(Json(serde_json::json!({
        "slug": slug,
        "filename": filename,
        "url": url,
    })))
}

/// GET /api/roadmap/:slug/media/:filename — serve a binary media file.
pub async fn serve_ponder_media(
    State(app): State<AppState>,
    Path((slug, filename)): Path<(String, String)>,
) -> Result<Response, AppError> {
    validate_media_filename(&filename)?;

    let content_type = ponder_mime_for_ext(&filename)
        .ok_or_else(|| AppError::bad_request("unsupported file type"))?;

    let path = sdlc_core::paths::ponder_media_dir(&app.root, &slug).join(&filename);

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| AppError::not_found("media file not found"))?;

    let response = Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError(anyhow::anyhow!("failed to build response: {e}")))?;

    Ok(response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_png() {
        assert_eq!(ponder_mime_for_ext("photo.png"), Some("image/png"));
    }

    #[test]
    fn mime_jpeg() {
        assert_eq!(ponder_mime_for_ext("photo.jpg"), Some("image/jpeg"));
        assert_eq!(ponder_mime_for_ext("photo.jpeg"), Some("image/jpeg"));
    }

    #[test]
    fn mime_gif() {
        assert_eq!(ponder_mime_for_ext("anim.gif"), Some("image/gif"));
    }

    #[test]
    fn mime_webp() {
        assert_eq!(ponder_mime_for_ext("diagram.webp"), Some("image/webp"));
    }

    #[test]
    fn mime_case_insensitive() {
        assert_eq!(ponder_mime_for_ext("photo.PNG"), Some("image/png"));
        assert_eq!(ponder_mime_for_ext("photo.JPEG"), Some("image/jpeg"));
    }

    #[test]
    fn mime_rejected_types() {
        assert_eq!(ponder_mime_for_ext("malware.exe"), None);
        assert_eq!(ponder_mime_for_ext("notes.txt"), None);
        assert_eq!(ponder_mime_for_ext("noextension"), None);
        assert_eq!(ponder_mime_for_ext("doc.pdf"), None);
    }

    #[test]
    fn filename_traversal_rejected() {
        assert!(validate_media_filename("../manifest.yaml").is_err());
        assert!(validate_media_filename("subdir/evil.png").is_err());
        assert!(validate_media_filename("..\\etc\\passwd").is_err());
    }

    #[test]
    fn filename_valid_accepted() {
        assert!(validate_media_filename("screenshot.png").is_ok());
        assert!(validate_media_filename("diagram-v2.webp").is_ok());
        assert!(validate_media_filename("photo_01.jpeg").is_ok());
    }
}
