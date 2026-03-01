pub mod auth;
pub mod embed;
pub mod error;
pub mod proxy;
pub mod routes;
pub mod state;
pub mod tunnel;

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Build the axum Router with all API routes and middleware.
/// Used by `serve()` and available for integration testing.
///
/// `port` is the local port the server is listening on (0 in tests).
pub fn build_router(root: std::path::PathBuf, port: u16) -> Router {
    build_router_from_state(state::AppState::new_with_port(root, port))
}

/// Build the axum Router with a pre-configured tunnel token and optional app
/// tunnel host. Used by integration tests that need to exercise auth middleware.
pub fn build_router_for_test(
    root: std::path::PathBuf,
    tunnel_token: Option<String>,
    app_tunnel_host: Option<String>,
) -> Router {
    let app_state = state::AppState::new_with_port(root, 0);
    if let Some(token) = tunnel_token {
        let mut cfg = auth::TunnelConfig::with_token(token);
        if let Some(host) = app_tunnel_host {
            cfg = cfg.with_app_tunnel_host(host);
        }
        // SAFETY: we're in test setup, no concurrent access yet.
        *app_state
            .tunnel_config
            .try_write()
            .expect("no contention in test setup") = cfg;
    }
    build_router_from_state(app_state)
}

fn build_router_from_state(app_state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Events (SSE) — GET for local, POST for Cloudflare Quick Tunnels
        // Quick Tunnels intentionally buffer GET streaming responses; POST streaming works.
        .route("/api/events", get(routes::events::sse_events))
        .route("/api/events", post(routes::events::sse_events))
        // State
        .route("/api/state", get(routes::state::get_state))
        // Features
        .route("/api/features", get(routes::features::list_features))
        .route("/api/features", post(routes::features::create_feature))
        .route("/api/features/{slug}", get(routes::features::get_feature))
        .route(
            "/api/features/{slug}/next",
            get(routes::features::get_feature_next),
        )
        .route(
            "/api/features/{slug}/transition",
            post(routes::features::transition_feature),
        )
        // Milestones
        .route("/api/milestones", get(routes::milestones::list_milestones))
        .route(
            "/api/milestones",
            post(routes::milestones::create_milestone),
        )
        .route(
            "/api/milestones/{slug}",
            get(routes::milestones::get_milestone),
        )
        .route(
            "/api/milestones/{slug}/review",
            get(routes::milestones::review_milestone),
        )
        .route(
            "/api/milestones/{slug}/features",
            post(routes::milestones::add_feature_to_milestone),
        )
        .route(
            "/api/milestones/{slug}/features/order",
            put(routes::milestones::reorder_milestone_features),
        )
        // Roadmap (Ponder Space)
        .route("/api/roadmap", get(routes::roadmap::list_ponders))
        .route("/api/roadmap", post(routes::roadmap::create_ponder))
        .route("/api/roadmap/{slug}", get(routes::roadmap::get_ponder))
        .route(
            "/api/roadmap/{slug}/capture",
            post(routes::roadmap::capture_artifact),
        )
        .route("/api/roadmap/{slug}", put(routes::roadmap::update_ponder))
        .route(
            "/api/roadmap/{slug}/sessions",
            get(routes::roadmap::list_ponder_sessions),
        )
        .route(
            "/api/roadmap/{slug}/sessions/{n}",
            get(routes::roadmap::get_ponder_session),
        )
        // Ponder chat (agent-driven sessions)
        .route(
            "/api/ponder/{slug}/chat",
            post(routes::runs::start_ponder_chat),
        )
        .route(
            "/api/ponder/{slug}/chat/current",
            delete(routes::runs::stop_ponder_chat),
        )
        // Ponder commit (headless synthesis + close agent)
        .route(
            "/api/ponder/{slug}/commit",
            post(routes::runs::commit_ponder),
        )
        // Investigations
        .route(
            "/api/investigations",
            get(routes::investigations::list_investigations),
        )
        .route(
            "/api/investigations",
            post(routes::investigations::create_investigation),
        )
        .route(
            "/api/investigations/{slug}",
            get(routes::investigations::get_investigation),
        )
        .route(
            "/api/investigations/{slug}",
            put(routes::investigations::update_investigation),
        )
        .route(
            "/api/investigations/{slug}/capture",
            post(routes::investigations::capture_artifact),
        )
        .route(
            "/api/investigations/{slug}/sessions",
            get(routes::investigations::list_investigation_sessions),
        )
        .route(
            "/api/investigations/{slug}/sessions/{n}",
            get(routes::investigations::get_investigation_session),
        )
        // Investigation chat (agent-driven sessions)
        .route(
            "/api/investigation/{slug}/chat",
            post(routes::runs::start_investigation_chat),
        )
        .route(
            "/api/investigation/{slug}/chat/current",
            delete(routes::runs::stop_investigation_chat),
        )
        // Artifacts
        .route(
            "/api/artifacts/{slug}/{artifact_type}",
            get(routes::artifacts::get_artifact),
        )
        .route(
            "/api/artifacts/{slug}/{artifact_type}/approve",
            post(routes::artifacts::approve_artifact),
        )
        .route(
            "/api/artifacts/{slug}/{artifact_type}/reject",
            post(routes::artifacts::reject_artifact),
        )
        .route(
            "/api/artifacts/{slug}/{artifact_type}/waive",
            post(routes::artifacts::waive_artifact),
        )
        // Tasks
        .route("/api/features/{slug}/tasks", post(routes::tasks::add_task))
        .route(
            "/api/features/{slug}/tasks/{id}/start",
            post(routes::tasks::start_task),
        )
        .route(
            "/api/features/{slug}/tasks/{id}/complete",
            post(routes::tasks::complete_task),
        )
        // Comments
        .route(
            "/api/features/{slug}/comments",
            post(routes::comments::add_comment),
        )
        // Vision
        .route("/api/vision", get(routes::vision::get_vision))
        .route("/api/vision", put(routes::vision::put_vision))
        // Architecture
        .route(
            "/api/architecture",
            get(routes::architecture::get_architecture),
        )
        .route(
            "/api/architecture",
            put(routes::architecture::put_architecture),
        )
        .route("/api/vision/run", post(routes::runs::start_vision_align))
        .route(
            "/api/architecture/run",
            post(routes::runs::start_architecture_align),
        )
        .route("/api/team/recruit", post(routes::runs::start_team_recruit))
        // Run history
        .route("/api/runs", get(routes::runs::list_runs))
        .route("/api/runs/{id}", get(routes::runs::get_run))
        // Run (agent execution via claude-agent + MCP)
        .route("/api/run/{slug}", post(routes::runs::start_run))
        .route("/api/run/{slug}/events", get(routes::runs::run_events))
        .route("/api/run/{slug}/stop", post(routes::runs::stop_run))
        // Milestone UAT (agent execution)
        .route(
            "/api/milestone/{slug}/uat",
            post(routes::runs::start_milestone_uat),
        )
        .route(
            "/api/milestone/{slug}/uat/events",
            get(routes::runs::milestone_uat_events),
        )
        .route(
            "/api/milestone/{slug}/uat/stop",
            post(routes::runs::stop_milestone_uat),
        )
        // Milestone prepare (agent execution)
        .route(
            "/api/milestone/{slug}/prepare",
            post(routes::runs::start_milestone_prepare),
        )
        .route(
            "/api/milestone/{slug}/prepare/events",
            get(routes::runs::milestone_prepare_events),
        )
        .route(
            "/api/milestone/{slug}/prepare/stop",
            post(routes::runs::stop_milestone_prepare),
        )
        // Milestone run-wave (agent execution)
        .route(
            "/api/milestone/{slug}/run-wave",
            post(routes::runs::start_milestone_run_wave),
        )
        .route(
            "/api/milestone/{slug}/run-wave/events",
            get(routes::runs::milestone_run_wave_events),
        )
        .route(
            "/api/milestone/{slug}/run-wave/stop",
            post(routes::runs::stop_milestone_run_wave),
        )
        // Escalations
        .route(
            "/api/escalations",
            get(routes::escalations::list_escalations),
        )
        .route(
            "/api/escalations",
            post(routes::escalations::create_escalation),
        )
        .route(
            "/api/escalations/{id}",
            get(routes::escalations::get_escalation),
        )
        .route(
            "/api/escalations/{id}/resolve",
            post(routes::escalations::resolve_escalation),
        )
        // Secrets (metadata only — no decrypt server-side)
        .route("/api/secrets/status", get(routes::secrets::get_status))
        .route("/api/secrets/keys", get(routes::secrets::list_keys))
        .route("/api/secrets/keys", post(routes::secrets::add_key))
        .route(
            "/api/secrets/keys/{name}",
            delete(routes::secrets::remove_key),
        )
        .route("/api/secrets/envs", get(routes::secrets::list_envs))
        .route(
            "/api/secrets/envs/{name}",
            delete(routes::secrets::delete_env),
        )
        // Tools
        .route("/api/tools/ama/answer", post(routes::runs::answer_ama))
        .route(
            "/api/tools/quality-check/reconfigure",
            post(routes::runs::reconfigure_quality_gates),
        )
        .route(
            "/api/tools/quality-check/fix",
            post(routes::runs::fix_quality_issues),
        )
        .route(
            "/api/tools",
            get(routes::tools::list_tools).post(routes::tools::create_tool),
        )
        .route("/api/tools/{name}", get(routes::tools::get_tool_meta))
        .route("/api/tools/{name}/run", post(routes::tools::run_tool))
        .route("/api/tools/{name}/setup", post(routes::tools::setup_tool))
        // Config
        .route("/api/config", get(routes::config::get_config))
        .route("/api/config", patch(routes::config::update_config))
        // Project (prepare / phase)
        .route(
            "/api/project/phase",
            get(routes::prepare::get_project_phase),
        )
        .route("/api/project/prepare", get(routes::prepare::get_prepare))
        // Query
        .route("/api/query/search", get(routes::query::search))
        .route("/api/query/search-tasks", get(routes::query::search_tasks))
        .route("/api/query/blocked", get(routes::query::blocked))
        .route("/api/query/ready", get(routes::query::ready))
        .route(
            "/api/query/needs-approval",
            get(routes::query::needs_approval),
        )
        // Feedback
        .route("/api/feedback", get(routes::feedback::list_notes))
        .route("/api/feedback", post(routes::feedback::add_note))
        .route("/api/feedback/{id}", delete(routes::feedback::delete_note))
        .route("/api/feedback/to-ponder", post(routes::feedback::to_ponder))
        // Public feedback alias — always reachable through the app tunnel (no auth required).
        .route("/__sdlc/feedback", post(routes::feedback::add_note))
        // Diagnose (pre-feature triage)
        .route("/api/diagnose", post(routes::diagnose::diagnose))
        // Init
        .route("/api/init", post(routes::init::init_project))
        // SDLC tunnel (exposes this UI publicly)
        .route("/api/tunnel", get(routes::tunnel::get_tunnel))
        .route("/api/tunnel", post(routes::tunnel::start_tunnel))
        .route("/api/tunnel", delete(routes::tunnel::stop_tunnel))
        // Agents (Claude agent definitions from ~/.claude/agents/)
        .route("/api/agents", get(routes::agents::list_agents))
        .route("/api/agents/{name}", get(routes::agents::get_agent))
        // App tunnel (exposes the user's project dev server)
        .route("/api/app-tunnel", get(routes::app_tunnel::get_app_tunnel))
        .route(
            "/api/app-tunnel",
            post(routes::app_tunnel::start_app_tunnel),
        )
        .route(
            "/api/app-tunnel",
            delete(routes::app_tunnel::stop_app_tunnel),
        )
        .route(
            "/api/app-tunnel/port",
            put(routes::app_tunnel::set_app_port),
        )
        .fallback(proxy::proxy_handler)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(
            app_state.tunnel_config.clone(),
            auth::auth_middleware,
        ))
        .with_state(app_state)
}

/// Start the SDLC web UI server.
///
/// In dev mode, run the Vite dev server on :5173 (which proxies /api requests
/// to this server on :3141 via vite.config.ts). In release mode, frontend
/// assets are embedded in the binary via rust-embed.
///
/// Pass `None` for `initial_tunnel` when no tunnel is pre-started.
pub async fn serve(
    root: PathBuf,
    port: u16,
    open_browser: bool,
    initial_tunnel: Option<(tunnel::Tunnel, String)>,
) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    serve_on(root, listener, open_browser, initial_tunnel).await
}

/// Start the SDLC web UI server on a pre-bound listener.
///
/// Unlike `serve`, this accepts a `TcpListener` that was already bound so the
/// caller can read the actual port before starting (useful when `port = 0` and
/// the OS picks a free port).
///
/// Pass `Some((tunnel, token))` when a cloudflared tunnel was started before
/// the server (e.g. `sdlc ui --tunnel`). The AppState will be pre-seeded so
/// the tunnel is immediately reflected in the `/api/tunnel` response.
pub async fn serve_on(
    root: PathBuf,
    listener: tokio::net::TcpListener,
    open_browser: bool,
    initial_tunnel: Option<(tunnel::Tunnel, String)>,
) -> anyhow::Result<()> {
    let actual_port = listener.local_addr()?.port();

    tracing::info!("SDLC UI server listening on http://localhost:{actual_port}");

    if open_browser {
        let url = format!("http://localhost:{actual_port}");
        let _ = open::that(&url);
    }

    let app_state = state::AppState::new_with_port(root, actual_port);

    if let Some((tun, token)) = initial_tunnel {
        let url = tun.url.clone();
        *app_state.tunnel_handle.lock().await = Some(tun);
        *app_state.tunnel_url.write().await = Some(url);
        *app_state.tunnel_config.write().await = auth::TunnelConfig::with_token(token);
    }

    let app = build_router_from_state(app_state);

    axum::serve(listener, app).await?;
    Ok(())
}
