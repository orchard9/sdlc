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

    #[error("ponder entry not found: {0}")]
    PonderNotFound(String),

    #[error("ponder entry already exists: {0}")]
    PonderExists(String),

    #[error("tool already exists: {0}")]
    ToolExists(String),

    #[error("invalid ponder status: {0}")]
    InvalidPonderStatus(String),

    #[error("investigation not found: {0}")]
    InvestigationNotFound(String),

    #[error("investigation already exists: {0}")]
    InvestigationExists(String),

    #[error("invalid investigation kind '{0}': must be root_cause, evolve, or guideline")]
    InvalidInvestigationKind(String),

    #[error("invalid investigation status '{0}': must be in_progress, complete, or parked")]
    InvalidInvestigationStatus(String),

    #[error("invalid artifact filename '{0}': must not contain path separators or '..'")]
    InvalidArtifactFilename(String),

    #[error("duplicate team member: {0}")]
    DuplicateTeamMember(String),

    #[error("session {0} not found")]
    SessionNotFound(u32),

    #[error("search error: {0}")]
    Search(String),

    #[error("home directory not found: set HOME environment variable")]
    HomeNotFound,

    #[error(
        "age encryption tool not found — install with: brew install age  (or: apt install age)"
    )]
    AgeNotInstalled,

    #[error("age decryption failed: {0}")]
    AgeDecryptFailed(String),

    #[error("age encryption failed: {0}")]
    AgeEncryptFailed(String),

    #[error("secret env not found: {0}")]
    SecretEnvNotFound(String),

    #[error("secret env key not found: '{0}' in env '{1}'")]
    SecretEnvKeyNotFound(String, String),

    #[error("secret key not found: {0}")]
    SecretKeyNotFound(String),

    #[error("secret key already exists: {0}")]
    SecretKeyExists(String),

    #[error("escalation not found: {0}")]
    EscalationNotFound(String),

    #[error("invalid key type '{0}': must be 'ssh' or 'age'")]
    InvalidSecretKeyType(String),

    #[error("No supported JavaScript runtime found. Install bun to use SDLC tools:\n  curl -fsSL https://bun.sh/install | bash\n  Or install deno: https://deno.land or node: https://nodejs.org")]
    NoToolRuntime,

    #[error("Failed to spawn tool subprocess: {0}")]
    ToolSpawnFailed(String),

    #[error("Tool exited with error: {0}")]
    ToolFailed(String),

    #[error("orchestrator DB error: {0}")]
    OrchestratorDb(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SdlcError>;
