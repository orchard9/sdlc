pub mod embed;
pub mod error;
pub mod routes;
pub mod state;

use axum::routing::{get, post, put};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};

/// Build the axum Router with all API routes and middleware.
/// Used by `serve()` and available for integration testing.
pub fn build_router(root: std::path::PathBuf) -> Router {
    let app_state = state::AppState::new(root);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Events (SSE)
        .route("/api/events", get(routes::events::sse_events))
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
        // Run (generate directive)
        .route("/api/run/{slug}", post(routes::runs::run_feature))
        // Config
        .route("/api/config", get(routes::config::get_config))
        // Query
        .route("/api/query/search", get(routes::query::search))
        .route("/api/query/search-tasks", get(routes::query::search_tasks))
        .route("/api/query/blocked", get(routes::query::blocked))
        .route("/api/query/ready", get(routes::query::ready))
        .route(
            "/api/query/needs-approval",
            get(routes::query::needs_approval),
        )
        // Init
        .route("/api/init", post(routes::init::init_project))
        .fallback(embed::static_handler)
        .layer(cors)
        .with_state(app_state)
}

/// Start the SDLC web UI server.
///
/// In dev mode, run the Vite dev server on :5173 (which proxies /api requests
/// to this server on :3141 via vite.config.ts). In release mode, frontend
/// assets are embedded in the binary via rust-embed.
pub async fn serve(root: PathBuf, port: u16, open_browser: bool) -> anyhow::Result<()> {
    let app = build_router(root);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("SDLC UI server listening on http://localhost:{port}");

    if open_browser {
        let url = format!("http://localhost:{port}");
        let _ = open::that(&url);
    }

    axum::serve(listener, app).await?;
    Ok(())
}

/// Start the SDLC web UI server on a pre-bound listener.
///
/// Unlike `serve`, this accepts a `TcpListener` that was already bound so the
/// caller can read the actual port before starting (useful when `port = 0` and
/// the OS picks a free port).
pub async fn serve_on(
    root: PathBuf,
    listener: tokio::net::TcpListener,
    open_browser: bool,
) -> anyhow::Result<()> {
    let actual_port = listener.local_addr()?.port();
    let app = build_router(root);

    tracing::info!("SDLC UI server listening on http://localhost:{actual_port}");

    if open_browser {
        let url = format!("http://localhost:{actual_port}");
        let _ = open::that(&url);
    }

    axum::serve(listener, app).await?;
    Ok(())
}
