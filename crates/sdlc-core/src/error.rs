use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdlcError {
    #[error("not initialized: run 'sdlc init'")]
    NotInitialized,

    #[error("feature not found: {0}")]
    FeatureNotFound(String),

    #[error("feature already exists: {0}")]
    FeatureExists(String),

    #[error("milestone not found: {0}")]
    MilestoneNotFound(String),

    #[error("milestone already exists: {0}")]
    MilestoneExists(String),

    #[error("invalid feature order: {0}")]
    InvalidFeatureOrder(String),

    #[error("invalid slug '{0}': must be lowercase alphanumeric with hyphens")]
    InvalidSlug(String),

    #[error("invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },

    #[error("invalid phase: {0}")]
    InvalidPhase(String),

    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("missing required artifact '{artifact}' for phase '{phase}'")]
    MissingArtifact { artifact: String, phase: String },

    #[error("blocked by: {0}")]
    Blocked(String),

    #[error("search error: {0}")]
    Search(String),

    #[error("home directory not found: set HOME environment variable")]
    HomeNotFound,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SdlcError>;
