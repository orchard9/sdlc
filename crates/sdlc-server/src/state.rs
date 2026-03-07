use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::{broadcast, Mutex, RwLock};

use sdlc_core::orchestrator::OrchestratorBackend;
use sdlc_core::TelemetryBackend;

use crate::hub::{HubRegistry, HubSseMessage, ProjectStatus};

use crate::auth::TunnelConfig;
use crate::tunnel::Tunnel;

/// Entry in the active-runs map: the broadcast sender for SSE subscribers
/// and an abort handle to cancel the spawned task.
pub type AgentRunEntry = (broadcast::Sender<String>, tokio::task::AbortHandle);

/// Owns a set of background watcher task abort handles.
/// Calls `.abort()` on every handle when dropped, ensuring watcher tasks
/// are cancelled when `AppState` is dropped — including in integration tests
/// where the Tokio runtime shuts down after each `#[tokio::test]`.
///
/// This prevents JoinHandle's detach-on-drop semantics from leaking the 7
/// watcher loops spawned by `AppState::new_with_port` into the runtime's
/// blocking thread pool.
pub(crate) struct WatcherGuard(Vec<tokio::task::AbortHandle>);

impl Drop for WatcherGuard {
    fn drop(&mut self) {
        for handle in &self.0 {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// RunRecord — persistent agent run metadata
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub key: String,
    pub run_type: String,
    pub target: String,
    pub label: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Generate a timestamp-based run ID: "20260227-143022-abc"
pub fn generate_run_id() -> String {
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d-%H%M%S").to_string();
    let suffix: String = (0..3).map(|_| (b'a' + (rand_u8() % 26)) as char).collect();
    format!("{ts}-{suffix}")
}

fn rand_u8() -> u8 {
    // Simple random byte from system time nanos
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as u8)
        .wrapping_mul(37)
        .wrapping_add(std::process::id() as u8)
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

fn runs_dir(root: &Path) -> PathBuf {
    root.join(".sdlc").join(".runs")
}

/// Load all RunRecords from `.sdlc/.runs/*.json`, marking any `running` as `failed`
/// (orphaned by a server restart).
pub fn load_run_history(root: &Path) -> Vec<RunRecord> {
    let dir = runs_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut records: Vec<RunRecord> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "json")
                && !e.file_name().to_string_lossy().ends_with(".events.json")
        })
        .filter_map(|e| {
            let data = std::fs::read_to_string(e.path()).ok()?;
            let mut rec: RunRecord = serde_json::from_str(&data).ok()?;
            // Mark stale running records as failed — they were orphaned by a crash
            // or restart, not stopped intentionally by the user.
            if rec.status == "running" {
                rec.status = "failed".to_string();
                rec.completed_at = Some(chrono::Utc::now().to_rfc3339());
                rec.error = Some("server restarted".to_string());
                // Best-effort persist the update
                let _ = std::fs::write(
                    e.path(),
                    serde_json::to_string_pretty(&rec).unwrap_or_default(),
                );
            }
            Some(rec)
        })
        .collect();

    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    records
}

/// Write a RunRecord to `.sdlc/.runs/{id}.json`.
pub fn persist_run(root: &Path, record: &RunRecord) {
    let dir = runs_dir(root);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.json", record.id));
    let _ = std::fs::write(
        path,
        serde_json::to_string_pretty(record).unwrap_or_default(),
    );
}

/// Write events sidecar to `.sdlc/.runs/{id}.events.json`.
pub fn persist_run_events(root: &Path, id: &str, events: &[serde_json::Value]) {
    let dir = runs_dir(root);
    let path = dir.join(format!("{id}.events.json"));
    let _ = std::fs::write(path, serde_json::to_string(events).unwrap_or_default());
}

/// Load events sidecar from `.sdlc/.runs/{id}.events.json`.
pub fn load_run_events(root: &Path, id: &str) -> Vec<serde_json::Value> {
    let path = runs_dir(root).join(format!("{id}.events.json"));
    match std::fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Delete oldest files if count > max.
pub fn enforce_retention(root: &Path, max: usize) {
    let dir = runs_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut record_files: Vec<(PathBuf, String)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".json") && !name.ends_with(".events.json")
        })
        .map(|e| {
            let id = e
                .path()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (e.path(), id)
        })
        .collect();

    if record_files.len() <= max {
        return;
    }

    // Sort oldest first (by filename = timestamp-based ID)
    record_files.sort_by(|a, b| a.1.cmp(&b.1));

    let to_remove = record_files.len() - max;
    for (path, id) in record_files.into_iter().take(to_remove) {
        let _ = std::fs::remove_file(&path);
        let events_path = dir.join(format!("{id}.events.json"));
        let _ = std::fs::remove_file(events_path);
    }
}

// ---------------------------------------------------------------------------
// SSE messages
// ---------------------------------------------------------------------------

/// Typed SSE messages broadcast to all connected clients.
#[derive(Clone, Debug)]
pub enum SseMessage {
    /// Generic state-changed ping (file watcher, CLI mutations).
    Update,
    /// A ponder agent session has started.
    PonderRunStarted { slug: String, session: u32 },
    /// A ponder agent session completed (session file landed).
    PonderRunCompleted { slug: String, session: u32 },
    /// A ponder agent session was stopped by the user.
    PonderRunStopped { slug: String },
    /// An investigation agent session has started.
    InvestigationRunStarted { slug: String, session: u32 },
    /// An investigation agent session completed (session file landed).
    InvestigationRunCompleted { slug: String, session: u32 },
    /// An investigation agent session was stopped by the user.
    InvestigationRunStopped { slug: String },
    /// An agent run started (feature, milestone_uat, ponder).
    RunStarted {
        id: String,
        key: String,
        label: String,
    },
    /// An agent run finished (completed, failed, stopped).
    RunFinished {
        id: String,
        key: String,
        status: String,
        session_id: Option<String>,
        stop_reason: Option<String>,
    },
    /// A vision alignment agent run completed.
    VisionAlignCompleted,
    /// An architecture alignment agent run completed.
    ArchitectureAlignCompleted,
    /// Team recruitment completed — perspective agents written to ~/.claude/agents/.
    TeamRecruitCompleted,
    /// A new tool was scaffolded or an existing tool changed.
    ToolsChanged,
    /// Tool plan agent completed — schema and approach designed.
    ToolPlanCompleted { name: String },
    /// Tool build agent completed — tool fully implemented and tested.
    ToolBuildCompleted { name: String },
    /// An advisory analysis agent run completed — advisory.yaml updated.
    AdvisoryRunCompleted,
    /// An advisory analysis agent run was stopped.
    AdvisoryRunStopped,
    /// Tool evolve agent completed — tool.ts updated.
    ToolEvolveCompleted { name: String },
    /// A result-action agent run completed for a tool.
    ToolActCompleted { name: String, action_index: usize },
    /// A milestone UAT agent run completed — UatRun record may have been written.
    MilestoneUatCompleted { slug: String },
    /// A milestone UAT agent run completed with a failing verdict — no state change,
    /// but the frontend can react (refresh runs list, show failure badge).
    MilestoneUatFailed { slug: String },
    /// The orchestrator daemon completed a tick — action states may have changed.
    /// Frontend should re-fetch the orchestrator actions list.
    ActionStateChanged,
    /// A knowledge research agent run has started.
    KnowledgeResearchStarted { slug: String },
    /// A knowledge research agent run completed — content and session written.
    KnowledgeResearchCompleted { slug: String },
    /// A knowledge base maintenance agent run has started.
    KnowledgeMaintenanceStarted,
    /// A knowledge base maintenance agent run completed.
    KnowledgeMaintenanceCompleted { actions_taken: usize },
    /// A knowledge query (ask) agent run has started.
    KnowledgeQueryStarted { question: String },
    /// A knowledge query (ask) agent run completed — answer ready.
    KnowledgeQueryCompleted {
        answer: String,
        cited_entries: Vec<CitedEntry>,
        gap_detected: bool,
        gap_suggestion: Option<String>,
    },
    /// The changelog event log was updated — clients can re-fetch via the API.
    ChangelogUpdated,
    /// A streaming tool run has started — interaction record created with status "streaming".
    ToolRunStarted {
        name: String,
        interaction_id: String,
    },
    /// A single NDJSON progress line emitted by a streaming tool's stdout.
    ToolRunProgress {
        name: String,
        interaction_id: String,
        line: serde_json::Value,
    },
    /// A streaming tool run completed successfully — record updated to "completed".
    ToolRunCompleted {
        name: String,
        interaction_id: String,
    },
    /// A streaming tool run failed — record updated to "failed".
    ToolRunFailed {
        name: String,
        interaction_id: String,
        error: String,
    },
}

/// A knowledge entry cited in a librarian answer.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CitedEntry {
    pub slug: String,
    pub code: String,
    pub title: String,
}

// ---------------------------------------------------------------------------
// Tunnel snapshot types — written atomically on tunnel start/stop
// ---------------------------------------------------------------------------

/// Read-only view of the main SDLC tunnel state (auth config + URL).
/// Both fields are updated together under a single RwLock so readers never
/// observe a partially-updated state (e.g. a new URL with the old token).
#[derive(Clone, Debug)]
pub struct TunnelSnapshot {
    pub config: TunnelConfig,
    pub url: Option<String>,
    /// When `true`, the auth middleware redirects unauthenticated browser requests
    /// to `/auth/login` instead of showing the QR-code page.
    pub oauth_enabled: bool,
}

impl Default for TunnelSnapshot {
    fn default() -> Self {
        Self {
            config: TunnelConfig::none(),
            url: None,
            oauth_enabled: false,
        }
    }
}

/// Read-only view of the app tunnel state (user's dev-server port + URL).
/// Both fields are updated together under a single RwLock.
#[derive(Clone, Debug, Default)]
pub struct AppTunnelSnapshot {
    pub port: Option<u16>,
    pub url: Option<String>,
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    /// Local port the server is listening on (0 until known).
    pub port: u16,
    pub event_tx: broadcast::Sender<SseMessage>,
    /// Active agent runs keyed by feature slug. Each entry holds the broadcast
    /// sender (for SSE subscribers) and an abort handle to cancel the task.
    pub agent_runs: Arc<Mutex<HashMap<String, AgentRunEntry>>>,
    /// Persistent run history (active + completed).
    pub run_history: Arc<Mutex<Vec<RunRecord>>>,
    /// Atomic snapshot of tunnel auth config + URL.
    /// Written once on tunnel start and once on stop — never partially updated.
    pub tunnel_snapshot: Arc<RwLock<TunnelSnapshot>>,
    /// Running orch-tunnel process, if any.
    pub tunnel_handle: Arc<Mutex<Option<Tunnel>>>,
    /// Atomic snapshot of app tunnel port + URL.
    /// Written once on start and once on stop — never partially updated.
    pub app_tunnel_snapshot: Arc<RwLock<AppTunnelSnapshot>>,
    /// App tunnel: running orch-tunnel process, if any.
    pub app_tunnel_handle: Arc<Mutex<Option<Tunnel>>>,
    /// HTTP client for reverse-proxying app tunnel requests.
    pub http_client: reqwest::Client,
    /// Telemetry backend for persisting raw agent events across restarts.
    /// Populated asynchronously at startup via a background task.
    /// Uses redb (local) or PostgreSQL (cluster) depending on `DATABASE_URL`.
    /// `None` until the background open completes; graceful degradation if unavailable.
    pub telemetry: Arc<OnceLock<Arc<dyn TelemetryBackend>>>,
    /// Orchestrator backend for action, webhook, and route storage.
    /// Populated asynchronously at startup alongside `telemetry`.
    /// Uses redb (local) or PostgreSQL (cluster) depending on `DATABASE_URL`.
    pub orchestrator: Arc<OnceLock<Arc<dyn OrchestratorBackend>>>,
    /// Abort guards for background file-watcher tasks.
    /// `WatcherGuard` calls `.abort()` on every handle when dropped, so all
    /// watcher loops are cancelled when `AppState` goes out of scope.
    pub(crate) _watcher_handles: Arc<WatcherGuard>,
    /// Per-instance token for tool-to-server agent calls via POST /api/tools/agent-call.
    /// Generated at startup from OS CSPRNG, never persisted to disk.
    /// Injected into every tool subprocess as SDLC_AGENT_TOKEN.
    pub agent_token: Arc<String>,
    /// Hub mode project registry. `None` in normal project mode, `Some` when the
    /// `--hub` flag is active; `None` in normal project mode.
    pub hub_registry: Option<Arc<Mutex<HubRegistry>>>,
    /// k8s client for fleet discovery. `None` when running outside k8s.
    pub kube_client: Option<kube::Client>,
    /// Gitea API base URL (from GITEA_URL env).
    pub gitea_url: Option<String>,
    /// Gitea API token (from GITEA_API_TOKEN env).
    pub gitea_token: Option<String>,
    /// Woodpecker CI API base URL (from WOODPECKER_URL env).
    pub woodpecker_url: Option<String>,
    /// Woodpecker CI API token (from WOODPECKER_API_TOKEN env).
    pub woodpecker_token: Option<String>,
    /// Comma-separated service tokens for machine-to-machine API access (from HUB_SERVICE_TOKENS env).
    pub hub_service_tokens: Vec<String>,
    /// Ingress domain for fleet URLs (from INGRESS_DOMAIN or default "sdlc.threesix.ai").
    pub ingress_domain: String,
    /// Google OAuth2 config for hub mode. `None` when OAuth env vars are not set.
    pub oauth_config: Option<Arc<crate::oauth::OAuthConfig>>,
    /// PostgreSQL-backed Claude OAuth credential pool.
    /// Initialized asynchronously at startup via a background task.
    /// `None` (i.e. OnceLock not yet set) until initialization completes;
    /// remains `Disabled` if `DATABASE_URL` is absent or the DB is unreachable.
    pub credential_pool: Arc<std::sync::OnceLock<crate::credential_pool::OptionalCredentialPool>>,
    /// OTP invite store for hub mode. Initialized asynchronously at startup.
    /// `None` (OnceLock not set) until initialization completes.
    pub invite_store: Arc<OnceLock<crate::invite::InviteStore>>,
    /// Notify email client for hub mode. `None` when `NOTIFY_URL` env var is unset
    /// (dev mode — OTP codes are only returned in the API response body).
    pub notify_client: Option<Arc<crate::notify::NotifyClient>>,
    /// Agent provider — controls which CLI backend (Claude, Codex) is used
    /// for all `spawn_agent_run` calls. Defaults to `ClaudeProvider`.
    /// Set via `AGENT_PROVIDER=codex` env var.
    pub agent_provider: Arc<dyn claude_agent::AgentProvider>,
}

/// Generate a 32-char hex token (128-bit entropy) from the OS CSPRNG.
/// Falls back to a timestamp+pid based value if the OS source is unavailable.
fn generate_agent_token() -> String {
    // Read exactly 16 random bytes from OS CSPRNG.
    // Must use read_exact — /dev/urandom never returns EOF, so std::fs::read()
    // would loop forever trying to read to end-of-file.
    let bytes: Option<[u8; 16]> = (|| {
        use std::io::Read;
        let mut buf = [0u8; 16];
        std::fs::File::open("/dev/urandom")
            .ok()?
            .read_exact(&mut buf)
            .ok()?;
        Some(buf)
    })();

    match bytes {
        Some(b) => b.iter().map(|byte| format!("{byte:02x}")).collect(),
        None => {
            // Fallback: mix nanos + pid for environments without /dev/urandom (Windows, some CI)
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            let pid = std::process::id();
            format!("{nanos:08x}{pid:08x}{nanos:08x}{pid:08x}")
        }
    }
}

/// Select the agent provider based on the `AGENT_PROVIDER` env var.
/// Defaults to Claude if unset or unrecognized.
fn select_agent_provider() -> Arc<dyn claude_agent::AgentProvider> {
    match std::env::var("AGENT_PROVIDER")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "codex" => {
            tracing::debug!("Agent provider: codex");
            Arc::new(claude_agent::CodexProvider)
        }
        "opencode" => {
            tracing::debug!("Agent provider: opencode");
            Arc::new(claude_agent::OpenCodeProvider)
        }
        _ => {
            tracing::debug!("Agent provider: claude (default)");
            Arc::new(claude_agent::ClaudeProvider)
        }
    }
}

impl AppState {
    /// Return the orchestrator backend, or a 503 error if not yet initialized.
    pub fn orchestrator_backend(
        &self,
    ) -> Result<
        std::sync::Arc<dyn sdlc_core::orchestrator::OrchestratorBackend>,
        crate::error::AppError,
    > {
        self.orchestrator
            .get()
            .cloned()
            .ok_or_else(|| crate::error::AppError(anyhow::anyhow!("orchestrator not ready")))
    }

    /// Construct AppState without spawning watcher tasks.
    /// All test code uses this path — watcher tasks are only needed in the
    /// production server process (via `new_with_port` → `build_router`).
    pub fn new(root: PathBuf) -> Self {
        Self::build_base_state(root, 0)
    }

    /// Build an AppState with no watcher tasks spawned — for integration tests
    /// that use `#[tokio::test]` and don't need file-change notifications.
    /// Avoids creating and immediately aborting the 7 watcher tasks that
    /// `new_with_port` would spawn when a Tokio runtime is present.
    pub fn new_for_test(root: PathBuf) -> Self {
        Self::build_base_state(root, 0)
    }

    /// Shared construction path: allocates all AppState fields with an empty
    /// WatcherGuard. Called by both `new_with_port` (which then spawns watchers)
    /// and `new_for_test` (which deliberately skips watcher spawning).
    fn build_base_state(root: PathBuf, port: u16) -> Self {
        let (tx, _) = broadcast::channel(64);
        tracing::debug!(root = %root.display(), "loading run history");
        let history = load_run_history(&root);
        tracing::debug!(count = history.len(), "run history loaded");
        // Seed the app tunnel port from config.yaml so it survives restarts.
        tracing::debug!("loading config");
        let saved_app_port = sdlc_core::config::Config::load(&root)
            .ok()
            .and_then(|c| c.app_port);
        tracing::debug!("building http client");
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("infallible: reqwest client construction");
        // Telemetry is opened in a background task (can have large WAL on big files).
        let telemetry: Arc<OnceLock<Arc<dyn TelemetryBackend>>> = Arc::new(OnceLock::new());
        // Orchestrator is opened synchronously when not using postgres (it's always small).
        // When DATABASE_URL is set, new_with_port's background task sets it instead.
        let orchestrator: Arc<OnceLock<Arc<dyn OrchestratorBackend>>> = Arc::new(OnceLock::new());
        if std::env::var("DATABASE_URL").is_err() {
            let db_path = sdlc_core::paths::orchestrator_db_path(&root);
            if let Ok(db) = sdlc_core::orchestrator::ActionDb::open(&db_path) {
                let _ = orchestrator.set(Arc::new(db) as Arc<dyn OrchestratorBackend>);
            }
        }
        // Pre-populate named tokens from .sdlc/auth.yaml so the auth gate is
        // active immediately on startup — before the first watcher tick fires.
        tracing::debug!("loading auth config");
        let initial_tokens: Vec<(String, String)> = sdlc_core::auth_config::load(&root)
            .map(|c| c.tokens.into_iter().map(|t| (t.name, t.token)).collect())
            .unwrap_or_default();
        // Also load hub service tokens (HUB_SERVICE_TOKENS env, comma-separated)
        // into the auth token list so they work with the existing Bearer auth flow.
        let hub_svc_tokens: Vec<(String, String)> = std::env::var("HUB_SERVICE_TOKENS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .enumerate()
                    .map(|(i, t)| (format!("_hub_service_{i}"), t))
                    .collect()
            })
            .unwrap_or_default();
        let mut all_tokens = initial_tokens;
        all_tokens.extend(hub_svc_tokens);

        let initial_tunnel_snapshot = if all_tokens.is_empty() {
            TunnelSnapshot::default()
        } else {
            TunnelSnapshot {
                config: crate::auth::TunnelConfig::with_tokens(all_tokens),
                url: None,
                oauth_enabled: false,
            }
        };
        Self {
            port,
            event_tx: tx,
            agent_runs: Arc::new(Mutex::new(HashMap::new())),
            run_history: Arc::new(Mutex::new(history)),
            tunnel_snapshot: Arc::new(RwLock::new(initial_tunnel_snapshot)),
            tunnel_handle: Arc::new(Mutex::new(None)),
            app_tunnel_snapshot: Arc::new(RwLock::new(AppTunnelSnapshot {
                port: saved_app_port,
                url: None,
            })),
            app_tunnel_handle: Arc::new(Mutex::new(None)),
            http_client,
            telemetry,
            orchestrator,
            _watcher_handles: Arc::new(WatcherGuard(Vec::new())),
            agent_token: Arc::new(generate_agent_token()),
            hub_registry: None,
            kube_client: None,
            gitea_url: std::env::var("GITEA_URL").ok(),
            gitea_token: std::env::var("GITEA_API_TOKEN").ok(),
            woodpecker_url: std::env::var("WOODPECKER_URL").ok(),
            woodpecker_token: std::env::var("WOODPECKER_API_TOKEN").ok(),
            hub_service_tokens: std::env::var("HUB_SERVICE_TOKENS")
                .ok()
                .map(|s| {
                    s.split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            ingress_domain: std::env::var("INGRESS_DOMAIN")
                .unwrap_or_else(|_| "sdlc.threesix.ai".to_string()),
            oauth_config: None,
            credential_pool: Arc::new(std::sync::OnceLock::new()),
            invite_store: Arc::new(OnceLock::new()),
            notify_client: None,
            agent_provider: select_agent_provider(),
            root,
        }
    }

    /// Build hub-mode AppState (with HubRegistry) and spawn watcher tasks.
    /// Used by `build_hub_router` for the project navigator hub server.
    pub fn new_with_port_hub(root: PathBuf, port: u16) -> Self {
        let persist_path = crate::hub::default_persist_path();
        tracing::debug!(path = %persist_path.display(), "initializing hub registry");
        let hub = Arc::new(Mutex::new(HubRegistry::new(persist_path)));
        let mut state = Self::new_with_port(root, port);
        state.hub_registry = Some(hub.clone());

        // Initialize OAuth config from env vars (hub mode only).
        if let Some(oauth_cfg) = crate::oauth::OAuthConfig::from_env() {
            tracing::info!("OAuth2 configured for hub mode");
            state.oauth_config = Some(Arc::new(oauth_cfg));
            // Tell auth middleware to redirect to /auth/login instead of showing QR page.
            state
                .tunnel_snapshot
                .try_write()
                .expect("no contention during startup")
                .oauth_enabled = true;
        } else {
            tracing::warn!("OAuth2 env vars not set — hub will run without Google auth");
        }

        // Initialize kube client for fleet discovery (in-cluster only).
        match kube::Config::incluster() {
            Ok(config) => match kube::Client::try_from(config) {
                Ok(client) => {
                    tracing::info!("k8s client initialized (in-cluster)");
                    state.kube_client = Some(client);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "k8s client creation failed — fleet endpoints will return empty results");
                }
            },
            Err(_) => {
                tracing::info!("not running in k8s — fleet discovery disabled");
            }
        }

        // Initialize invite store (memory or postgres depending on DATABASE_URL).
        if tokio::runtime::Handle::try_current().is_ok() {
            let invite_cell = state.invite_store.clone();
            tokio::spawn(async move {
                let store = crate::invite::InviteStore::from_env().await;
                let _ = invite_cell.set(store);
                tracing::info!("invite store ready");
            });
        }

        // Initialize notify client for OTP email delivery (hub mode only).
        // Absent in dev (NOTIFY_URL unset) — OTP codes are returned in API response only.
        if let Some(client) = crate::notify::NotifyClient::from_env() {
            tracing::info!("notify client configured for OTP delivery");
            state.notify_client = Some(Arc::new(client));
        } else {
            tracing::info!("NOTIFY_URL not set — OTP emails disabled (dev mode)");
        }

        // Spawn the sweep task now that the registry is populated.
        if tokio::runtime::Handle::try_current().is_ok() {
            let hub_for_sweep = hub.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                    let mut reg = hub_for_sweep.lock().await;
                    reg.sweep();
                }
            });
            tracing::debug!("hub sweep task spawned");
        }

        if tokio::runtime::Handle::try_current().is_ok() {
            let hub_for_fleet = hub.clone();
            let kube_client = state.kube_client.clone();
            let ingress_domain = state.ingress_domain.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                    let instances = match crate::fleet::list_fleet_instances(
                        kube_client.as_ref(),
                        Some(hub_for_fleet.as_ref()),
                        &ingress_domain,
                    )
                    .await
                    {
                        Ok(instances) => instances,
                        Err(err) => {
                            tracing::warn!(error = %err, "hub fleet poll failed");
                            continue;
                        }
                    };

                    let (tx, projects_with_agents) = {
                        let mut registry = hub_for_fleet.lock().await;
                        registry.reconcile_fleet(&instances);
                        let projects_with_agents = registry
                            .projects
                            .values()
                            .filter(|project| {
                                project.agent_running == Some(true)
                                    && project.status != ProjectStatus::Offline
                            })
                            .count();
                        (registry.event_tx.clone(), projects_with_agents)
                    };

                    for instance in instances {
                        let _ = tx.send(HubSseMessage::FleetUpdated(instance));
                    }
                    let _ = tx.send(HubSseMessage::FleetAgentStatus {
                        total_active_runs: projects_with_agents,
                        projects_with_agents,
                    });
                }
            });
            tracing::debug!("hub fleet poll task spawned");
        }

        state
    }

    pub fn new_with_port(root: PathBuf, port: u16) -> Self {
        let state = Self::build_base_state(root, port);
        let tx = state.event_tx.clone();

        // Open storage backends in a background task.
        // Guard: only spawn if inside a Tokio runtime (skipped in sync unit tests).
        if tokio::runtime::Handle::try_current().is_ok() {
            let telemetry_path = state.root.join(".sdlc").join("telemetry.redb");

            if let Ok(url) = std::env::var("DATABASE_URL") {
                // Postgres path: connect, migrate, set both backends.
                // Orchestrator redb was set synchronously in build_base_state; postgres
                // overrides it via OnceLock (will silently fail if already set — postgres
                // wins only in fresh cluster environments where no prior set happened).
                let telemetry_cell = state.telemetry.clone();
                let orchestrator_cell = state.orchestrator.clone();
                tracing::debug!("DATABASE_URL detected — connecting postgres backends");
                tokio::spawn(async move {
                    match sqlx::PgPool::connect(&url).await {
                        Ok(pool) => {
                            if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
                                tracing::warn!(error = %e, "postgres migration failed — using redb fallback");
                                open_redb_telemetry(telemetry_cell, telemetry_path).await;
                                return;
                            }
                            use crate::{
                                pg_orchestrator::PgOrchestratorBackend,
                                pg_telemetry::PgTelemetryBackend,
                            };
                            let _ = telemetry_cell
                                .set(Arc::new(PgTelemetryBackend::new(pool.clone()))
                                    as Arc<dyn TelemetryBackend>);
                            let _ = orchestrator_cell
                                .set(Arc::new(PgOrchestratorBackend::new(pool))
                                    as Arc<dyn OrchestratorBackend>);
                            tracing::info!("postgres backends ready");
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "postgres connect failed — using redb");
                            open_redb_telemetry(telemetry_cell, telemetry_path).await;
                        }
                    }
                });
            } else {
                // Redb path: orchestrator already set synchronously in build_base_state.
                // Open telemetry in background (can be slow on large files with WAL recovery).
                let telemetry_cell = state.telemetry.clone();
                tokio::spawn(async move {
                    open_redb_telemetry(telemetry_cell, telemetry_path).await;
                });
            }
        }

        // Initialize the Claude credential pool asynchronously.
        // Guard: only spawn if inside a Tokio runtime (skipped in sync unit tests).
        if tokio::runtime::Handle::try_current().is_ok() {
            let pool_cell = state.credential_pool.clone();
            tokio::spawn(async move {
                let pool = crate::credential_pool::OptionalCredentialPool::from_env().await;
                let _ = pool_cell.set(pool);
            });
        }

        // Watch .sdlc/state.yaml mtime and broadcast when it changes.
        // This catches both web-UI mutations and external CLI updates.
        // Guard: only spawn if inside a Tokio runtime (skipped in sync unit tests).
        if tokio::runtime::Handle::try_current().is_ok() {
            tracing::debug!("spawning 7 file-watcher tasks");
            let mut handles: Vec<tokio::task::AbortHandle> = Vec::new();

            let state_file = state.root.join(".sdlc").join("state.yaml");
            let tx2 = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        if let Ok(meta) = tokio::fs::metadata(&state_file).await {
                            if let Ok(mtime) = meta.modified() {
                                if last_mtime != Some(mtime) {
                                    last_mtime = Some(mtime);
                                    let _ = tx2.send(SseMessage::Update);
                                }
                            }
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch roadmap manifests for ponder space changes.
            // atomic_write uses rename, which updates the parent slug dir's mtime
            // but NOT the top-level roadmap/ dir mtime — so we scan each
            // manifest file directly instead of watching the directory.
            let roadmap_dir = state.root.join(".sdlc").join("roadmap");
            let tx_roadmap = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        let latest = scan_dir_mtime(&roadmap_dir).await;
                        if latest != last_mtime {
                            last_mtime = latest;
                            let _ = tx_roadmap.send(SseMessage::Update);
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch investigations dir for investigation workspace changes.
            let investigations_dir = state.root.join(".sdlc").join("investigations");
            let tx_inv = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        let latest = scan_dir_mtime(&investigations_dir).await;
                        if latest != last_mtime {
                            last_mtime = latest;
                            let _ = tx_inv.send(SseMessage::Update);
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch escalations.yaml for create/resolve mutations (CLI or API).
            let escalations_file = state.root.join(".sdlc").join("escalations.yaml");
            let tx_esc = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        if let Ok(meta) = tokio::fs::metadata(&escalations_file).await {
                            if let Ok(mtime) = meta.modified() {
                                if last_mtime != Some(mtime) {
                                    last_mtime = Some(mtime);
                                    let _ = tx_esc.send(SseMessage::Update);
                                }
                            }
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch .sdlc/tools/ for new tool directories (scaffolding via POST /api/tools or CLI).
            // We scan for subdirectories that contain a tool.ts file and watch their count/mtime.
            let tools_dir = state.root.join(".sdlc").join("tools");
            let tx_tools = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_snapshot: Option<(usize, std::time::SystemTime)> = None;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        let snapshot = scan_tools_snapshot(&tools_dir).await;
                        if snapshot != last_snapshot {
                            last_snapshot = snapshot;
                            let _ = tx_tools.send(SseMessage::ToolsChanged);
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch .sdlc/.orchestrator.state — written by the orchestrator daemon
            // after each tick. Fires ActionStateChanged so the frontend can
            // refresh the actions list without polling.
            let sentinel = state.root.join(".sdlc").join(".orchestrator.state");
            let tx_orch = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        if let Ok(meta) = tokio::fs::metadata(&sentinel).await {
                            if let Ok(mtime) = meta.modified() {
                                if last_mtime != Some(mtime) {
                                    last_mtime = Some(mtime);
                                    let _ = tx_orch.send(SseMessage::ActionStateChanged);
                                }
                            }
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch .sdlc/changelog.yaml — fires ChangelogUpdated whenever a
            // new event is appended, so the dashboard can re-fetch without polling.
            let changelog_file = state.root.join(".sdlc").join("changelog.yaml");
            let tx_changelog = tx.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        if let Ok(meta) = tokio::fs::metadata(&changelog_file).await {
                            if let Ok(mtime) = meta.modified() {
                                if last_mtime != Some(mtime) {
                                    last_mtime = Some(mtime);
                                    let _ = tx_changelog.send(SseMessage::ChangelogUpdated);
                                }
                            }
                        }
                    }
                })
                .abort_handle(),
            );

            // Watch .sdlc/auth.yaml — hot-reload named tokens into tunnel_snapshot.
            // When the file is added, removed, or updated, the in-memory token
            // list is updated atomically so the next request uses the new set.
            let auth_file = state.root.join(".sdlc").join("auth.yaml");
            let snap_auth = state.tunnel_snapshot.clone();
            let root_auth = state.root.clone();
            handles.push(
                tokio::spawn(async move {
                    let mut last_mtime = None::<std::time::SystemTime>;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        // Check current mtime (file may not exist yet).
                        let current = tokio::fs::metadata(&auth_file)
                            .await
                            .ok()
                            .and_then(|m| m.modified().ok());
                        if current != last_mtime {
                            last_mtime = current;
                            // Reload token list from disk.
                            let tokens: Vec<(String, String)> =
                                sdlc_core::auth_config::load(&root_auth)
                                    .map(|c| {
                                        c.tokens.into_iter().map(|t| (t.name, t.token)).collect()
                                    })
                                    .unwrap_or_default();
                            let mut snap = snap_auth.write().await;
                            snap.config.tokens = tokens;
                        }
                    }
                })
                .abort_handle(),
            );

            // Spawn hub heartbeat task (no-op if SDLC_HUB_URL is not set).
            if let Some(hb_handle) = crate::heartbeat::spawn_heartbeat_task(&state) {
                handles.push(hb_handle);
                tracing::debug!("hub heartbeat task spawned");
            }

            tracing::debug!("all watcher tasks spawned");
            // WatcherGuard aborts all tasks when AppState is dropped — including
            // in integration tests where the runtime shuts down after each test.
            return Self {
                _watcher_handles: Arc::new(WatcherGuard(handles)),
                ..state
            };
        }

        state
    }
}

/// Scan `<dir>/<slug>/manifest.yaml` and `<dir>/<slug>/sessions/`
/// across all slug subdirectories and return the most recent mtime found.
/// This is needed because atomic_write uses rename, which updates the slug
/// subdirectory mtime but not the top-level directory mtime.
async fn scan_dir_mtime(roadmap_dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut latest: Option<std::time::SystemTime> = None;

    let mut dir = match tokio::fs::read_dir(roadmap_dir).await {
        Ok(d) => d,
        Err(_) => return None,
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Check manifest.yaml
        let manifest = path.join("manifest.yaml");
        if let Ok(meta) = tokio::fs::metadata(&manifest).await {
            if let Ok(mtime) = meta.modified() {
                if latest.is_none_or(|l| mtime > l) {
                    latest = Some(mtime);
                }
            }
        }
        // Check sessions directory mtime
        let sessions_dir = path.join("sessions");
        if let Ok(meta) = tokio::fs::metadata(&sessions_dir).await {
            if let Ok(mtime) = meta.modified() {
                if latest.is_none_or(|l| mtime > l) {
                    latest = Some(mtime);
                }
            }
        }
    }

    latest
}

/// Scan `.sdlc/tools/` for non-underscore subdirectories that contain `tool.ts`.
/// Returns a snapshot of (count, latest_mtime) so the watcher can detect changes.
async fn scan_tools_snapshot(
    tools_dir: &std::path::Path,
) -> Option<(usize, std::time::SystemTime)> {
    let mut dir = match tokio::fs::read_dir(tools_dir).await {
        Ok(d) => d,
        Err(_) => return None,
    };

    let mut count = 0usize;
    let mut latest: Option<std::time::SystemTime> = None;

    while let Ok(Some(entry)) = dir.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('_') || !entry.path().is_dir() {
            continue;
        }
        let script = entry.path().join("tool.ts");
        if let Ok(meta) = tokio::fs::metadata(&script).await {
            count += 1;
            if let Ok(mtime) = meta.modified() {
                if latest.is_none_or(|l| mtime > l) {
                    latest = Some(mtime);
                }
            }
        }
    }

    if count == 0 {
        None
    } else {
        latest.map(|mtime| (count, mtime))
    }
}

/// Open the redb telemetry backend in a background blocking task and set the OnceLock.
async fn open_redb_telemetry(
    cell: Arc<OnceLock<Arc<dyn TelemetryBackend>>>,
    path: std::path::PathBuf,
) {
    tracing::debug!(path = %path.display(), "opening redb telemetry backend");
    let result = tokio::task::spawn_blocking(move || {
        crate::telemetry::TelemetryStore::open(&path)
            .ok()
            .map(|s| Arc::new(s) as Arc<dyn TelemetryBackend>)
    })
    .await
    .unwrap_or(None);
    if let Some(t) = result {
        let _ = cell.set(t);
        tracing::debug!("redb telemetry backend ready");
    } else {
        tracing::warn!("telemetry backend unavailable — events will not be persisted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_stores_root() {
        let state = AppState::new(std::path::PathBuf::from("/tmp/test"));
        assert_eq!(state.root, std::path::PathBuf::from("/tmp/test"));
    }

    #[test]
    fn orphaned_runs_marked_failed_on_startup() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let runs_dir = tmp.path().join(".sdlc").join(".runs");
        std::fs::create_dir_all(&runs_dir).expect("create runs dir");

        // Helper to build a minimal RunRecord JSON
        fn make_record(id: &str, status: &str) -> serde_json::Value {
            serde_json::json!({
                "id": id,
                "key": format!("sdlc-run:{id}"),
                "run_type": "sdlc-run",
                "target": id,
                "label": format!("Run {id}"),
                "status": status,
                "started_at": "2026-01-01T00:00:00Z",
                "completed_at": null,
                "cost_usd": null,
                "turns": null,
                "error": null
            })
        }

        // Write an orphaned (running) record
        let orphan_id = "20260101-000001-aaa";
        std::fs::write(
            runs_dir.join(format!("{orphan_id}.json")),
            serde_json::to_string_pretty(&make_record(orphan_id, "running")).unwrap(),
        )
        .expect("write orphan");

        // Write a completed record — must remain unchanged
        let completed_id = "20260101-000002-bbb";
        std::fs::write(
            runs_dir.join(format!("{completed_id}.json")),
            serde_json::to_string_pretty(&make_record(completed_id, "completed")).unwrap(),
        )
        .expect("write completed");

        let history = load_run_history(tmp.path());

        // Locate records by id
        let orphan = history
            .iter()
            .find(|r| r.id == orphan_id)
            .expect("orphan record");
        let completed = history
            .iter()
            .find(|r| r.id == completed_id)
            .expect("completed record");

        // Orphaned run must be marked failed with the crash reason
        assert_eq!(orphan.status, "failed", "orphaned run must be 'failed'");
        assert_eq!(
            orphan.error.as_deref(),
            Some("server restarted"),
            "orphaned run must carry 'server restarted' error"
        );
        assert!(
            orphan.completed_at.is_some(),
            "orphaned run must have completed_at set"
        );

        // Completed run must be unchanged
        assert_eq!(
            completed.status, "completed",
            "completed run must remain 'completed'"
        );
        assert!(
            completed.error.is_none(),
            "completed run must not have error set"
        );

        // Verify the on-disk JSON was updated for the orphaned run
        let disk_data =
            std::fs::read_to_string(runs_dir.join(format!("{orphan_id}.json"))).expect("read back");
        let disk_rec: serde_json::Value = serde_json::from_str(&disk_data).expect("parse back");
        assert_eq!(
            disk_rec["status"], "failed",
            "on-disk status must be 'failed'"
        );
        assert_eq!(
            disk_rec["error"], "server restarted",
            "on-disk error must be 'server restarted'"
        );
    }
}
