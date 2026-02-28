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
    let url = app.app_tunnel_url.read().await.clone();
    let configured_port = *app.app_tunnel_port.read().await;
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

    // Tunnel cloudflared to sdlc-server (which reverse-proxies to user's app).
    let tun = Tunnel::start(sdlc_port)
        .await
        .map_err(|e| AppError(anyhow::anyhow!("{e}")))?;

    let url = tun.url.clone();
    let host = extract_host_from_url(&url);

    // Record the user's app port for the proxy to use, then store tunnel state.
    *app.app_tunnel_port.write().await = Some(user_port);
    *app.app_tunnel_handle.lock().await = Some(tun);
    *app.app_tunnel_url.write().await = Some(url.clone());

    // Register the app tunnel hostname so auth middleware knows to bypass auth
    // for non-API requests arriving with this Host header.
    {
        let mut cfg = app.tunnel_config.write().await;
        cfg.app_tunnel_host = Some(host);
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
            *app.app_tunnel_url.write().await = None;
            // Clear app_tunnel_host so auth no longer bypasses for this host.
            {
                let mut cfg = app.tunnel_config.write().await;
                cfg.app_tunnel_host = None;
            }
            let configured_port = *app.app_tunnel_port.read().await;
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
    *app.app_tunnel_port.write().await = Some(port);
    let url = app.app_tunnel_url.read().await.clone();
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
