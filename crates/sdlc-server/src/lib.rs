pub mod embed;
pub mod error;
pub mod routes;
pub mod state;
pub mod subprocess;

use axum::routing::{get, post, put};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};

/// Start the SDLC web UI server.
///
/// In dev mode, run the Vite dev server on :5173 (which proxies /api requests
/// to this server on :3141 via vite.config.ts). In release mode, frontend
/// assets are embedded in the binary via rust-embed.
pub async fn serve(root: PathBuf, port: u16, open_browser: bool) -> anyhow::Result<()> {
    let app_state = state::AppState::new(root);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        // State
        .route("/api/state", get(routes::state::get_state))
        // Features
        .route("/api/features", get(routes::features::list_features))
        .route(
            "/api/features",
            post(routes::features::create_feature),
        )
        .route(
            "/api/features/{slug}",
            get(routes::features::get_feature),
        )
        .route(
            "/api/features/{slug}/next",
            get(routes::features::get_feature_next),
        )
        .route(
            "/api/features/{slug}/transition",
            post(routes::features::transition_feature),
        )
        // Milestones
        .route(
            "/api/milestones",
            get(routes::milestones::list_milestones),
        )
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
            "/api/milestones/{slug}/run",
            post(routes::milestones::run_milestone),
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
        // Tasks
        .route(
            "/api/features/{slug}/tasks",
            post(routes::tasks::add_task),
        )
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
        // Config
        .route(
            "/api/config/agents",
            get(routes::config::get_agents_config).put(routes::config::put_agents_config),
        )
        // Vision
        .route("/api/vision", get(routes::vision::get_vision))
        .route("/api/vision", put(routes::vision::put_vision))
        // Init
        .route("/api/init", post(routes::init::init_project))
        // Runs
        .route("/api/run/{slug}", post(routes::runs::run_feature))
        .route("/api/run-command", post(routes::runs::run_command))
        .route(
            "/api/runs/{run_id}/stream",
            get(routes::runs::stream_run),
        );

    let app = api
        .fallback(embed::static_handler)
        .layer(cors)
        .with_state(app_state);

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
