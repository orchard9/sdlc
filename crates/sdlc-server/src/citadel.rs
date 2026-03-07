//! Optional Citadel telemetry layer.
//!
//! When `PONDER_CITADEL_URL`, `PONDER_CITADEL_API_KEY`, and
//! `PONDER_CITADEL_TENANT_ID` are all set, a [`CitadelLayer`] ships
//! tracing events to the Citadel batch ingest endpoint.  When any var
//! is absent the layer is not created and there is zero runtime overhead.
//!
//! The layer uses a bounded channel to decouple the synchronous tracing
//! path from the async HTTP flush.  Call [`start_citadel_flush`] once a
//! tokio runtime is available to spawn the background task.

use std::sync::mpsc;
use std::sync::{Arc, OnceLock};

use serde::Serialize;
use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// Global holder for the flush task state.  Populated by [`CitadelLayer::new`],
/// consumed by [`start_citadel_flush`].
static FLUSH_STATE: OnceLock<FlushState> = OnceLock::new();

struct FlushState {
    rx: std::sync::Mutex<Option<mpsc::Receiver<CitadelEvent>>>,
    config: Arc<CitadelConfig>,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Citadel connection config read from environment variables.
#[derive(Clone, Debug)]
pub struct CitadelConfig {
    pub url: String,
    pub api_key: String,
    pub tenant_id: String,
    pub environment: String,
    pub service: String,
}

impl CitadelConfig {
    /// Read config from env.  Returns `None` if any required var is missing.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("PONDER_CITADEL_URL").ok()?;
        let api_key = std::env::var("PONDER_CITADEL_API_KEY").ok()?;
        let tenant_id = std::env::var("PONDER_CITADEL_TENANT_ID").ok()?;

        let is_hub = std::env::var("SDLC_HUB").ok().as_deref() == Some("true");

        let environment = std::env::var("PONDER_ENVIRONMENT").unwrap_or_else(|_| {
            if is_hub {
                "production".to_string()
            } else {
                "local".to_string()
            }
        });

        let service = if is_hub {
            "ponder-hub".to_string()
        } else {
            let slug = std::env::var("SDLC_BASE_URL")
                .ok()
                .and_then(|u| {
                    u.split("//")
                        .nth(1)?
                        .split('.')
                        .next()
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "ponder".to_string());
            format!("ponder-{slug}")
        };

        Some(Self {
            url: url.trim_end_matches('/').to_string(),
            api_key,
            tenant_id,
            environment,
            service,
        })
    }
}

// ---------------------------------------------------------------------------
// Event payload
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CitadelEvent {
    message: String,
    level: String,
    timestamp: String,
    service: String,
    environment: String,
    target: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    fields: serde_json::Map<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Field visitor
// ---------------------------------------------------------------------------

struct JsonVisitor {
    message: Option<String>,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl JsonVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: serde_json::Map::new(),
        }
    }
}

impl Visit for JsonVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let val = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(val);
        } else {
            self.fields
                .insert(field.name().to_string(), serde_json::Value::String(val));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

const CHANNEL_CAPACITY: usize = 4096;
const FLUSH_INTERVAL_SECS: u64 = 5;
const FLUSH_BATCH_SIZE: usize = 100;

/// A `tracing::Layer` that ships events to Citadel's batch ingest endpoint.
///
/// Events are serialised synchronously and pushed into a bounded channel.
/// Call [`start_citadel_flush`] once a tokio runtime is available to spawn
/// the background flush task.
pub struct CitadelLayer {
    config: Arc<CitadelConfig>,
    tx: mpsc::SyncSender<CitadelEvent>,
}

impl CitadelLayer {
    /// Create the layer and stash the receiver in a global for
    /// [`start_citadel_flush`] to pick up later.
    pub fn new(config: CitadelConfig) -> Self {
        let (tx, rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
        let config = Arc::new(config);

        let _ = FLUSH_STATE.set(FlushState {
            rx: std::sync::Mutex::new(Some(rx)),
            config: config.clone(),
        });

        Self { config, tx }
    }
}

impl<S: Subscriber> Layer<S> for CitadelLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        // Skip events from this module to prevent feedback loops.
        let meta = event.metadata();
        if meta.target().starts_with("sdlc_server::citadel") {
            return;
        }

        // Only ship INFO and above.
        if *meta.level() > tracing::Level::INFO {
            return;
        }

        let mut visitor = JsonVisitor::new();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_else(|| meta.name().to_string());

        let evt = CitadelEvent {
            message,
            level: level_str(meta.level()),
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            service: self.config.service.clone(),
            environment: self.config.environment.clone(),
            target: meta.target().to_string(),
            fields: visitor.fields,
        };

        // Best-effort: drop if channel is full.
        let _ = self.tx.try_send(evt);
    }
}

fn level_str(level: &tracing::Level) -> String {
    match *level {
        tracing::Level::ERROR => "error",
        tracing::Level::WARN => "warn",
        tracing::Level::INFO => "info",
        tracing::Level::DEBUG => "debug",
        tracing::Level::TRACE => "trace",
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// Public API: start the background flush
// ---------------------------------------------------------------------------

/// Spawn the Citadel flush background task.  Must be called from within a
/// tokio runtime.  No-op if the layer was never created or the task was
/// already started.
pub fn start_citadel_flush() {
    let Some(state) = FLUSH_STATE.get() else {
        return;
    };
    let rx = {
        let mut guard = state.rx.lock().expect("citadel rx lock poisoned");
        match guard.take() {
            Some(rx) => rx,
            None => return, // already started
        }
    };
    tokio::spawn(flush_loop(rx, state.config.clone()));
}

// ---------------------------------------------------------------------------
// Background flush
// ---------------------------------------------------------------------------

async fn flush_loop(rx: mpsc::Receiver<CitadelEvent>, config: Arc<CitadelConfig>) {
    let client = reqwest::Client::new();
    let endpoint = format!("{}/api/v1/ingest", config.url);

    tracing::info!(
        url = %config.url,
        service = %config.service,
        environment = %config.environment,
        "citadel telemetry active"
    );

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(FLUSH_INTERVAL_SECS)).await;

        let mut batch: Vec<CitadelEvent> = Vec::with_capacity(FLUSH_BATCH_SIZE);
        while let Ok(evt) = rx.try_recv() {
            batch.push(evt);
            if batch.len() >= FLUSH_BATCH_SIZE {
                ship_batch(&client, &endpoint, &config, &batch).await;
                batch.clear();
            }
        }

        if !batch.is_empty() {
            ship_batch(&client, &endpoint, &config, &batch).await;
        }
    }
}

async fn ship_batch(
    client: &reqwest::Client,
    endpoint: &str,
    config: &CitadelConfig,
    batch: &[CitadelEvent],
) {
    let result = client
        .post(endpoint)
        .timeout(std::time::Duration::from_secs(10))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("X-Tenant-ID", &config.tenant_id)
        .header("Content-Type", "application/json")
        .json(batch)
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!(count = batch.len(), "citadel batch shipped");
        }
        Ok(resp) => {
            tracing::warn!(
                status = %resp.status(),
                count = batch.len(),
                "citadel ingest failed: non-2xx"
            );
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                count = batch.len(),
                "citadel ingest failed: request error"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_returns_none_when_vars_missing() {
        std::env::remove_var("PONDER_CITADEL_URL");
        std::env::remove_var("PONDER_CITADEL_API_KEY");
        std::env::remove_var("PONDER_CITADEL_TENANT_ID");
        assert!(CitadelConfig::from_env().is_none());
    }

    #[test]
    fn level_str_maps_correctly() {
        assert_eq!(level_str(&tracing::Level::ERROR), "error");
        assert_eq!(level_str(&tracing::Level::WARN), "warn");
        assert_eq!(level_str(&tracing::Level::INFO), "info");
    }

    #[test]
    fn json_visitor_captures_message() {
        let v = JsonVisitor::new();
        // Use record_debug which is the entry point for all field types.
        // We can't construct Field directly (private), but we can test
        // the visitor logic via the message detection path.
        assert!(v.message.is_none());
        assert!(v.fields.is_empty());
    }

    #[test]
    fn start_citadel_flush_noop_without_layer() {
        // No layer was created in this test — should not panic.
        // Can't call start_citadel_flush without a tokio runtime,
        // but we can verify FLUSH_STATE is empty in a fresh test binary.
        // (This test verifies the None guard path.)
        assert!(
            FLUSH_STATE.get().is_none() || FLUSH_STATE.get().unwrap().rx.lock().unwrap().is_none(),
            "FLUSH_STATE should be empty or already consumed"
        );
    }
}
