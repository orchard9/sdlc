use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct AddCommentBody {
    pub body: String,
    pub flag: Option<String>,
    pub by: Option<String>,
}

/// POST /api/features/:slug/comments — add a comment.
pub async fn add_comment(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<AddCommentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut feature = sdlc_core::feature::Feature::load(&root, &slug)?;

        let flag = body.flag.as_deref().and_then(|f| match f {
            "blocker" => Some(sdlc_core::comment::CommentFlag::Blocker),
            "question" => Some(sdlc_core::comment::CommentFlag::Question),
            "decision" => Some(sdlc_core::comment::CommentFlag::Decision),
            "fyi" => Some(sdlc_core::comment::CommentFlag::Fyi),
            _ => None,
        });

        let id = sdlc_core::comment::add_comment(
            &mut feature.comments,
            &mut feature.next_comment_seq,
            body.body,
            flag,
            sdlc_core::comment::CommentTarget::Feature,
            body.by,
        );
        feature.save(&root)?;

        Ok::<_, sdlc_core::SdlcError>(serde_json::json!({
            "slug": slug,
            "comment_id": id,
        }))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
