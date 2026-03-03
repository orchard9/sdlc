use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::proxy::extract_host_from_url;
use crate::state::AppState;
use crate::tunnel::Tunnel;

// ---------------------------------------------------------------------------
// Response / request types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct AppTunnelStatus {
    pub active: bool,
    pub url: Option<String>,
    /// The port being tunneled (the user's app dev server).
    pub configured_port: Option<u16>,
}

#[derive(Deserialize)]
pub struct StartAppTunnelBody {
    pub port: u16,
}

#[derive(Deserialize)]
pub struct SetAppPortBody {
    pub port: u16,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Persist `port` to `.sdlc/config.yaml` as `app_port`. Best-effort — the
/// tunnel still starts / the port is still saved in-memory even if the write
/// fails (e.g. first-run before `sdlc init`).
async fn persist_app_port(root: std::path::PathBuf, port: u16) {
    tokio::task::spawn_blocking(move || {
        if let Ok(mut cfg) = sdlc_core::config::Config::load(&root) {
            cfg.app_port = Some(port);
            let _ = cfg.save(&root);
        }
    })
    .await
    .ok();
}

// ---------------------------------------------------------------------------
// GET /api/app-tunnel
// ---------------------------------------------------------------------------

pub async fn get_app_tunnel(State(app): State<AppState>) -> Json<AppTunnelStatus> {
    let snap = app.app_tunnel_snapshot.read().await;
    let url = snap.url.clone();
    let configured_port = snap.port;
    drop(snap);
    Json(AppTunnelStatus {
        active: url.is_some(),
        url,
        configured_port,
    })
}

// ---------------------------------------------------------------------------
// POST /api/app-tunnel  — start (body: { port })
// ---------------------------------------------------------------------------

pub async fn start_app_tunnel(
    State(app): State<AppState>,
    Json(body): Json<StartAppTunnelBody>,
) -> Result<Json<AppTunnelStatus>, AppError> {
    {
        let handle = app.app_tunnel_handle.lock().await;
        if handle.is_some() {
            return Err(AppError(anyhow::anyhow!(
                "an app tunnel is already active; stop it first"
            )));
        }
    }

    let user_port = body.port;
    let sdlc_port = app.port;

    // Tunnel orch-tunnel to sdlc-server (which reverse-proxies to user's app).
    let name = crate::tunnel::derive_tunnel_name(&app.root);
    let tun = Tunnel::start(sdlc_port, &name)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("{e}")))?;

    let url = tun.url.clone();
    let host = extract_host_from_url(&url);

    // Store the handle separately (Mutex, not part of the snapshot).
    *app.app_tunnel_handle.lock().await = Some(tun);

    // Atomically update the app tunnel snapshot (port + url together).
    *app.app_tunnel_snapshot.write().await = crate::state::AppTunnelSnapshot {
        port: Some(user_port),
        url: Some(url.clone()),
    };

    // Register the app tunnel hostname so auth middleware knows to bypass auth
    // for non-API requests arriving with this Host header.
    // We update only app_tunnel_host inside the tunnel_snapshot, preserving
    // the existing token and sdlc tunnel url.
    {
        let mut snap = app.tunnel_snapshot.write().await;
        snap.config.app_tunnel_host = Some(host);
    }

    persist_app_port(app.root.clone(), user_port).await;

    Ok(Json(AppTunnelStatus {
        active: true,
        url: Some(url),
        configured_port: Some(user_port),
    }))
}

// ---------------------------------------------------------------------------
// DELETE /api/app-tunnel  — stop
// ---------------------------------------------------------------------------

pub async fn stop_app_tunnel(
    State(app): State<AppState>,
) -> Result<Json<AppTunnelStatus>, AppError> {
    let tun = app.app_tunnel_handle.lock().await.take();
    match tun {
        None => Err(AppError(anyhow::anyhow!(
            "no app tunnel is currently active"
        ))),
        Some(t) => {
            t.stop().await;
            // Clear app_tunnel_host so auth no longer bypasses for this host.
            {
                let mut snap = app.tunnel_snapshot.write().await;
                snap.config.app_tunnel_host = None;
            }
            // Atomically update the app tunnel snapshot: clear url but preserve port.
            let configured_port = {
                let mut snap = app.app_tunnel_snapshot.write().await;
                snap.url = None;
                snap.port
            };
            Ok(Json(AppTunnelStatus {
                active: false,
                url: None,
                configured_port,
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// PUT /api/app-tunnel/port  — persist preferred port without starting the tunnel
// ---------------------------------------------------------------------------

pub async fn set_app_port(
    State(app): State<AppState>,
    Json(body): Json<SetAppPortBody>,
) -> Result<Json<AppTunnelStatus>, AppError> {
    let port = body.port;
    persist_app_port(app.root.clone(), port).await;
    let url = {
        let mut snap = app.app_tunnel_snapshot.write().await;
        snap.port = Some(port);
        snap.url.clone()
    };
    Ok(Json(AppTunnelStatus {
        active: url.is_some(),
        url,
        configured_port: Some(port),
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn get_app_tunnel_inactive_by_default() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let Json(status) = get_app_tunnel(State(app)).await;
        assert!(!status.active);
        assert!(status.url.is_none());
        assert!(status.configured_port.is_none());
    }

    #[tokio::test]
    async fn stop_app_tunnel_when_none_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let app = AppState::new(dir.path().to_path_buf());
        let result = stop_app_tunnel(State(app)).await;
        assert!(result.is_err());
    }
}
