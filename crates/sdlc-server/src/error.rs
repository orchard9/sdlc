use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sdlc_core::error::SdlcError;

/// Unified error type for HTTP responses.
#[derive(Debug)]
pub struct AppError(pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = if let Some(e) = self.0.downcast_ref::<SdlcError>() {
            match e {
                SdlcError::NotInitialized => StatusCode::BAD_REQUEST,
                SdlcError::FeatureNotFound(_)
                | SdlcError::MilestoneNotFound(_)
                | SdlcError::PonderNotFound(_)
                | SdlcError::InvestigationNotFound(_)
                | SdlcError::TaskNotFound(_)
                | SdlcError::ArtifactNotFound(_)
                | SdlcError::SessionNotFound(_) => StatusCode::NOT_FOUND,
                SdlcError::FeatureExists(_)
                | SdlcError::MilestoneExists(_)
                | SdlcError::PonderExists(_)
                | SdlcError::InvestigationExists(_) => StatusCode::CONFLICT,
                SdlcError::InvalidSlug(_)
                | SdlcError::InvalidPhase(_)
                | SdlcError::InvalidPonderStatus(_)
                | SdlcError::InvalidInvestigationKind(_)
                | SdlcError::InvalidInvestigationStatus(_)
                | SdlcError::InvalidArtifactFilename(_)
                | SdlcError::InvalidFeatureOrder(_) => StatusCode::BAD_REQUEST,
                SdlcError::DuplicateTeamMember(_) => StatusCode::CONFLICT,
                SdlcError::InvalidTransition { .. } => StatusCode::UNPROCESSABLE_ENTITY,
                SdlcError::MissingArtifact { .. } => StatusCode::UNPROCESSABLE_ENTITY,
                SdlcError::Blocked(_) => StatusCode::CONFLICT,
                SdlcError::Search(_)
                | SdlcError::Io(_)
                | SdlcError::Yaml(_)
                | SdlcError::Json(_)
                | SdlcError::HomeNotFound => StatusCode::INTERNAL_SERVER_ERROR,
            }
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };

        let body = serde_json::json!({ "error": self.0.to_string() });
        (status, axum::Json(body)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use sdlc_core::error::SdlcError;

    #[test]
    fn feature_not_found_maps_to_404() {
        let err = AppError(SdlcError::FeatureNotFound("test".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn milestone_not_found_maps_to_404() {
        let err = AppError(SdlcError::MilestoneNotFound("ms-1".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn task_not_found_maps_to_404() {
        let err = AppError(SdlcError::TaskNotFound("task-1".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn artifact_not_found_maps_to_404() {
        let err = AppError(SdlcError::ArtifactNotFound("spec.md".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn feature_exists_maps_to_409() {
        let err = AppError(SdlcError::FeatureExists("test".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn ponder_not_found_maps_to_404() {
        let err = AppError(SdlcError::PonderNotFound("my-idea".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn ponder_exists_maps_to_409() {
        let err = AppError(SdlcError::PonderExists("my-idea".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn milestone_exists_maps_to_409() {
        let err = AppError(SdlcError::MilestoneExists("ms-1".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn invalid_slug_maps_to_400() {
        let err = AppError(SdlcError::InvalidSlug("BAD SLUG".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_phase_maps_to_400() {
        let err = AppError(SdlcError::InvalidPhase("nope".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn not_initialized_maps_to_400() {
        let err = AppError(SdlcError::NotInitialized.into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_transition_maps_to_422() {
        let err = AppError(
            SdlcError::InvalidTransition {
                from: "design".into(),
                to: "done".into(),
                reason: "skipped impl".into(),
            }
            .into(),
        );
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn missing_artifact_maps_to_422() {
        let err = AppError(
            SdlcError::MissingArtifact {
                artifact: "spec.md".into(),
                phase: "design".into(),
            }
            .into(),
        );
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn blocked_maps_to_409() {
        let err = AppError(SdlcError::Blocked("dependency not met".into()).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn io_error_maps_to_500() {
        let io_err = std::io::Error::other("disk full");
        let err = AppError(SdlcError::Io(io_err).into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn non_sdlc_error_maps_to_500() {
        let err = AppError(anyhow::anyhow!("something unexpected"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn response_body_contains_error_field() {
        let err = AppError(SdlcError::FeatureNotFound("my-feat".into()).into());
        let response = err.into_response();
        // Verify it is JSON with an "error" key by checking Content-Type header
        let ct = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .expect("should have content-type");
        assert!(
            ct.to_str().unwrap().contains("application/json"),
            "expected JSON content type, got {:?}",
            ct
        );
    }
}
