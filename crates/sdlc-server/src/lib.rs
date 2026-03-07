pub mod auth;
pub mod citadel;
pub mod credential_pool;
pub mod email;
pub mod embed;
pub mod error;
pub mod fleet;
pub mod heartbeat;
pub mod hub;
pub mod invite;
pub mod notify;
pub mod oauth;
pub mod pg_common;
pub mod pg_orchestrator;
pub mod pg_telemetry;
pub mod proxy;
pub mod routes;
pub mod state;
pub mod telemetry;
pub mod tunnel;

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

async fn log_request(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();
    let is_events = uri.path() == "/api/events";
    if is_events {
        tracing::debug!(method = %method, path = %uri, "request started");
    } else {
        tracing::info!(method = %method, path = %uri, "request started");
    }
    let resp = next.run(req).await;
    if is_events {
        tracing::debug!(
            method = %method,
            path = %uri,
            status = resp.status().as_u16(),
            latency_ms = start.elapsed().as_millis(),
            "request completed"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %uri,
            status = resp.status().as_u16(),
            latency_ms = start.elapsed().as_millis(),
            "request completed"
        );
    }
    resp
}

/// Build the axum Router with all API routes and middleware.
/// Used by `serve()` and available for integration testing.
///
/// `port` is the local port the server is listening on (0 in tests).
/// `hub_mode` activates hub routes and the hub registry in AppState.
pub fn build_router(root: std::path::PathBuf, port: u16) -> Router {
    build_router_from_state(state::AppState::new_with_port(root, port))
}

/// Build the axum Router in hub mode (project navigator).
pub fn build_hub_router(root: std::path::PathBuf, port: u16) -> Router {
    build_router_from_state(state::AppState::new_with_port_hub(root, port))
}

/// Build the axum Router with a pre-configured tunnel token and optional app
/// tunnel host. Used by integration tests that need to exercise auth middleware.
pub fn build_router_for_test(
    root: std::path::PathBuf,
    tunnel_token: Option<String>,
    app_tunnel_host: Option<String>,
) -> Router {
    let app_state = state::AppState::new_for_test(root);
    // Seed storage backends synchronously for tests (no background task in test mode).
    let db_path = sdlc_core::paths::orchestrator_db_path(&app_state.root);
    if let Ok(db) = sdlc_core::orchestrator::ActionDb::open(&db_path) {
        let _ = app_state.orchestrator.set(std::sync::Arc::new(db)
            as std::sync::Arc<dyn sdlc_core::orchestrator::OrchestratorBackend>);
    }
    let telemetry_path = app_state.root.join(".sdlc").join("telemetry.redb");
    if let Ok(store) = crate::telemetry::TelemetryStore::open(&telemetry_path) {
        let _ = app_state
            .telemetry
            .set(std::sync::Arc::new(store) as std::sync::Arc<dyn sdlc_core::TelemetryBackend>);
    }
    if let Some(token) = tunnel_token {
        let mut cfg = auth::TunnelConfig::with_token(token);
        if let Some(host) = app_tunnel_host {
            cfg = cfg.with_app_tunnel_host(host);
        }
        // SAFETY: we're in test setup, no concurrent access yet.
        app_state
            .tunnel_snapshot
            .try_write()
            .expect("no contention in test setup")
            .config = cfg;
    }
    build_router_from_state(app_state)
}

fn build_router_from_state(app_state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health — used by Playwright webServer health check and load balancers.
        // Must respond 200 before any auth middleware so `reuseExistingServer` works.
        .route(
            "/api/health",
            get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }),
        )
        // Events (SSE) — GET for local, POST for orch-tunnel Quick Tunnels
        // Quick Tunnels intentionally buffer GET streaming responses; POST streaming works.
        .route("/api/events", get(routes::events::sse_events))
        .route("/api/events", post(routes::events::sse_events))
        // State
        .route("/api/state", get(routes::state::get_state))
        // Changelog
        .route("/api/changelog", get(routes::changelog::get_changelog))
        // Backlog
        .route(
            "/api/backlog",
            get(routes::backlog::list_backlog).post(routes::backlog::create_backlog_item),
        )
        .route(
            "/api/backlog/{id}/park",
            post(routes::backlog::park_backlog_item),
        )
        .route(
            "/api/backlog/{id}/promote",
            post(routes::backlog::promote_backlog_item),
        )
        // Features
        .route("/api/features", get(routes::features::list_features))
        .route("/api/features", post(routes::features::create_feature))
        .route("/api/features/{slug}", get(routes::features::get_feature))
        .route(
            "/api/features/{slug}/directive",
            get(routes::features::get_feature_directive),
        )
        .route(
            "/api/features/{slug}/next",
            get(routes::features::get_feature_next),
        )
        .route(
            "/api/features/{slug}/transition",
            post(routes::features::transition_feature),
        )
        .route(
            "/api/features/{slug}/merge",
            post(routes::features::merge_feature),
        )
        .route(
            "/api/features/{slug}/blockers/{idx}",
            delete(routes::features::remove_blocker),
        )
        .route(
            "/api/features/{slug}/human-qa",
            post(routes::features::submit_human_qa),
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
        .route(
            "/api/milestones/{slug}/acceptance-test",
            get(routes::milestones::get_milestone_acceptance_test),
        )
        .route(
            "/api/milestones/{slug}/uat-runs",
            get(routes::milestones::list_milestone_uat_runs),
        )
        .route(
            "/api/milestones/{slug}/uat-runs/latest",
            get(routes::milestones::get_latest_milestone_uat_run),
        )
        .route(
            "/api/milestones/{slug}/uat-runs/{run_id}/artifacts/{filename}",
            get(routes::milestones::get_uat_run_artifact),
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
            "/api/roadmap/{slug}",
            delete(routes::roadmap::delete_ponder),
        )
        .route(
            "/api/roadmap/{slug}/sessions",
            get(routes::roadmap::list_ponder_sessions),
        )
        .route(
            "/api/roadmap/{slug}/sessions/{n}",
            get(routes::roadmap::get_ponder_session),
        )
        .route(
            "/api/roadmap/{slug}/media",
            post(routes::roadmap::upload_ponder_media),
        )
        .route(
            "/api/roadmap/{slug}/media/{filename}",
            get(routes::roadmap::serve_ponder_media),
        )
        // Advisory (maturity-ladder codebase analysis)
        .route("/api/advisory", get(routes::advisory::get_advisory))
        .route(
            "/api/advisory/findings/{id}",
            patch(routes::advisory::update_finding),
        )
        .route(
            "/api/advisory/run",
            post(routes::advisory::start_advisory_run),
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
        // Knowledge base
        .route(
            "/api/knowledge/catalog",
            get(routes::knowledge::get_catalog),
        )
        // NOTE: static /api/knowledge/* segments must appear before /api/knowledge/{slug}
        // so Axum resolves them before the slug wildcard.
        .route("/api/knowledge/ask", post(routes::knowledge::ask_knowledge))
        .route(
            "/api/knowledge/maintain",
            post(routes::knowledge::maintain_knowledge),
        )
        .route(
            "/api/knowledge/harvest",
            post(routes::knowledge::harvest_knowledge_workspace),
        )
        .route(
            "/api/knowledge",
            get(routes::knowledge::list_knowledge).post(routes::knowledge::create_knowledge),
        )
        // NOTE: /api/knowledge/relevant must appear before /api/knowledge/{slug}
        // so Axum resolves the static "relevant" segment before the slug wildcard.
        .route(
            "/api/knowledge/relevant",
            get(routes::knowledge::get_relevant_knowledge),
        )
        .route(
            "/api/knowledge/{slug}",
            get(routes::knowledge::get_knowledge).put(routes::knowledge::update_knowledge),
        )
        .route(
            "/api/knowledge/{slug}/capture",
            post(routes::knowledge::capture_knowledge_artifact),
        )
        .route(
            "/api/knowledge/{slug}/sessions",
            get(routes::knowledge::list_knowledge_sessions),
        )
        .route(
            "/api/knowledge/{slug}/sessions/{n}",
            get(routes::knowledge::get_knowledge_session),
        )
        .route(
            "/api/knowledge/{slug}/research",
            post(routes::knowledge::research_knowledge),
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
        // Spikes
        .route("/api/spikes", get(routes::spikes::list_spikes))
        .route("/api/spikes/{slug}", get(routes::spikes::get_spike))
        .route(
            "/api/spikes/{slug}/promote",
            post(routes::spikes::promote_spike),
        )
        // Artifacts
        .route(
            "/api/artifacts/{slug}/{artifact_type}",
            get(routes::artifacts::get_artifact),
        )
        .route(
            "/api/artifacts/{slug}/{artifact_type}/draft",
            post(routes::artifacts::draft_artifact),
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
        // Run telemetry
        .route(
            "/api/runs/{id}/telemetry",
            get(routes::runs::get_run_telemetry),
        )
        .route(
            "/api/runs/{id}/telemetry/summary",
            get(routes::telemetry::get_run_telemetry_summary),
        )
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
        .route(
            "/api/milestone/{slug}/uat/fail",
            post(routes::runs::fail_milestone_uat),
        )
        .route(
            "/api/milestone/{slug}/uat/human",
            post(routes::runs::submit_milestone_uat_human),
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
        .route(
            "/api/secrets/envs",
            get(routes::secrets::list_envs).post(routes::secrets::create_env),
        )
        .route(
            "/api/secrets/envs/{name}",
            delete(routes::secrets::delete_env),
        )
        // Auth tokens (named tunnel-access tokens stored in .sdlc/auth.yaml)
        .route(
            "/api/auth/tokens",
            get(routes::auth_tokens::list_tokens).post(routes::auth_tokens::create_token),
        )
        .route(
            "/api/auth/tokens/{name}",
            delete(routes::auth_tokens::delete_token),
        )
        // AMA threads (must be before /api/tools/{name} wildcard)
        .route(
            "/api/tools/ama/threads",
            get(routes::ama_threads::list_ama_threads).post(routes::ama_threads::create_ama_thread),
        )
        .route(
            "/api/tools/ama/threads/{id}",
            get(routes::ama_threads::get_ama_thread)
                .patch(routes::ama_threads::update_ama_thread)
                .delete(routes::ama_threads::delete_ama_thread),
        )
        .route(
            "/api/tools/ama/threads/{id}/turns",
            post(routes::ama_threads::add_ama_turn),
        )
        .route(
            "/api/tools/ama/threads/{id}/turns/{n}",
            patch(routes::ama_threads::update_ama_turn_synthesis),
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
        // Plan-Act pattern for tool creation (must be before {name} wildcard)
        .route("/api/tools/plan", post(routes::runs::plan_tool))
        .route("/api/tools/build", post(routes::runs::build_tool))
        // Agent-call endpoint — used by tools to dispatch synchronous blocking agent runs.
        // Validates SDLC_AGENT_TOKEN bearer header and waits for completion before responding.
        // Must be before {name} wildcard.
        .route("/api/tools/agent-call", post(routes::tools::agent_call))
        // Agent-dispatch endpoint — fire-and-forget variant; returns 202 immediately (must be before {name} wildcard)
        .route(
            "/api/tools/agent-dispatch",
            post(routes::tools::agent_dispatch),
        )
        .route(
            "/api/tools",
            get(routes::tools::list_tools).post(routes::tools::create_tool),
        )
        .route("/api/tools/{name}", get(routes::tools::get_tool_meta))
        .route("/api/tools/{name}/clone", post(routes::tools::clone_tool))
        .route("/api/tools/{name}/evolve", post(routes::runs::evolve_tool))
        .route("/api/tools/{name}/act", post(routes::runs::act_tool))
        .route("/api/tools/{name}/run", post(routes::tools::run_tool))
        .route("/api/tools/{name}/setup", post(routes::tools::setup_tool))
        .route(
            "/api/tools/{name}/interactions",
            get(routes::tools::list_tool_interactions),
        )
        .route(
            "/api/tools/{name}/interactions/{id}",
            get(routes::tools::get_tool_interaction).delete(routes::tools::delete_tool_interaction),
        )
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
        .route("/api/feedback/{id}", patch(routes::feedback::update_note))
        .route(
            "/api/feedback/{id}/enrich",
            post(routes::feedback::enrich_note),
        )
        .route("/api/feedback/to-ponder", post(routes::feedback::to_ponder))
        .route(
            "/api/feedback/slack",
            post(routes::feedback::receive_slack_feedback),
        )
        // Public feedback alias — always reachable through the app tunnel (no auth required).
        .route("/__sdlc/feedback", post(routes::feedback::add_note))
        // Feedback threads — contextual, append-only comment logs
        .route(
            "/api/threads",
            get(routes::threads::list_threads).post(routes::threads::create_thread),
        )
        .route(
            "/api/threads/{id}",
            get(routes::threads::get_thread)
                .patch(routes::threads::patch_thread)
                .delete(routes::threads::delete_thread),
        )
        .route("/api/threads/{id}/posts", post(routes::threads::add_post))
        .route(
            "/api/threads/{id}/comments",
            post(routes::threads::add_comment),
        )
        .route(
            "/api/threads/{id}/promote",
            post(routes::threads::promote_thread),
        )
        // Webhook payload inspector — query stored payloads and replay them.
        // Must be before the wildcard ingestion route to avoid conflicts.
        .route(
            "/api/webhooks/{route}/data",
            get(routes::webhooks::query_webhook_payloads),
        )
        .route(
            "/api/webhooks/{route}/replay/{id}",
            post(routes::webhooks::replay_webhook_payload),
        )
        // Webhook ingestion — accepts raw payloads from external senders and stores in redb.
        .route("/webhooks/{route}", post(routes::webhooks::receive_webhook))
        // Orchestrator webhook event history
        .route(
            "/api/orchestrator/webhooks/events",
            get(routes::orchestrator::list_webhook_events),
        )
        // Orchestrator webhook route management (must be before webhook ingestion wildcard)
        .route(
            "/api/orchestrator/webhooks/routes",
            get(routes::orchestrator::list_routes).post(routes::orchestrator::register_route),
        )
        .route(
            "/api/orchestrator/webhooks/routes/{id}",
            delete(routes::orchestrator::delete_route),
        )
        // Orchestrator actions CRUD
        .route(
            "/api/orchestrator/actions",
            get(routes::orchestrator::list_actions).post(routes::orchestrator::create_action),
        )
        .route(
            "/api/orchestrator/actions/{id}",
            delete(routes::orchestrator::delete_action).patch(routes::orchestrator::patch_action),
        )
        // Diagnose (pre-feature triage)
        .route("/api/diagnose", post(routes::diagnose::diagnose))
        // Init
        .route("/api/init", post(routes::init::init_project))
        // SDLC tunnel (exposes this UI publicly)
        .route("/api/tunnel", get(routes::tunnel::get_tunnel))
        .route("/api/tunnel", post(routes::tunnel::start_tunnel))
        .route("/api/tunnel", delete(routes::tunnel::stop_tunnel))
        .route("/api/tunnel/preflight", get(routes::tunnel::tunnel_preflight))
        // Agents (Claude agent definitions from ~/.claude/agents/)
        .route("/api/agents", get(routes::agents::list_agents))
        .route("/api/agents/{name}", get(routes::agents::get_agent))
        // Project agents (from <project_root>/.claude/agents/)
        .route(
            "/api/project/agents",
            get(routes::agents::list_project_agents),
        )
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
        // Credential pool (Claude OAuth token management)
        .route(
            "/api/credential-pool",
            get(routes::credential_pool::get_status),
        )
        .route(
            "/api/credential-pool/credentials",
            get(routes::credential_pool::list_credentials)
                .post(routes::credential_pool::add_credential),
        )
        .route(
            "/api/credential-pool/credentials/{id}",
            patch(routes::credential_pool::patch_credential)
                .delete(routes::credential_pool::delete_credential),
        )
        // Hub mode routes — always registered; handlers return 503 in project mode
        .route("/api/hub/heartbeat", post(routes::hub::heartbeat))
        .route("/api/hub/projects", get(routes::hub::list_projects))
        .route("/api/hub/events", get(routes::hub::hub_sse_events))
        // Fleet management (hub mode only)
        .route("/api/hub/fleet", get(routes::hub::fleet))
        .route("/api/hub/summary", get(routes::hub::summary))
        .route("/api/hub/attention", get(routes::hub::attention))
        .route("/api/hub/activity", get(routes::hub::activity))
        .route("/api/hub/repos", get(routes::hub::repos))
        .route("/api/hub/available", get(routes::hub::available))
        .route("/api/hub/provision", post(routes::hub::provision))
        .route("/api/hub/import", post(routes::hub::import))
        .route("/api/hub/create-repo", post(routes::hub::create_repo))
        .route("/api/hub/agents", get(routes::hub::agents))
        .route(
            "/api/hub/projects/{slug}",
            delete(routes::hub::delete_project),
        )
        // OTP invite management (admin endpoints — behind auth middleware)
        .route(
            "/api/invites",
            get(routes::invites::list_invites).post(routes::invites::create_invite),
        )
        .route("/api/invites/{id}", delete(routes::invites::revoke_invite))
        // OAuth2 routes (hub mode) — login/callback MUST be public (before auth layer)
        .route("/auth/login", get(oauth::login))
        .route("/auth/callback", get(oauth::callback))
        .route("/auth/verify", get(oauth::verify))
        .route("/auth/logout", post(oauth::logout))
        // OTP verify — public (alongside existing /auth/* routes, bypassed by auth middleware)
        .route("/auth/otp", post(routes::invites::verify_otp))
        .fallback(proxy::proxy_handler)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(
            app_state.tunnel_snapshot.clone(),
            auth::auth_middleware,
        ))
        .layer(axum::middleware::from_fn(log_request))
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
/// Pass `Some((tunnel, token))` when an orch-tunnel was started before
/// the server (e.g. `sdlc ui --tunnel`). The AppState will be pre-seeded so
/// the tunnel is immediately reflected in the `/api/tunnel` response.
pub async fn serve_on(
    root: PathBuf,
    listener: tokio::net::TcpListener,
    open_browser: bool,
    initial_tunnel: Option<(tunnel::Tunnel, String)>,
) -> anyhow::Result<()> {
    serve_on_with_mode(root, listener, open_browser, initial_tunnel, false).await
}

/// Start the SDLC server in hub mode on a pre-bound listener.
/// Hub mode activates the project navigator registry instead of project routes.
pub async fn serve_on_hub(
    root: PathBuf,
    listener: tokio::net::TcpListener,
    open_browser: bool,
) -> anyhow::Result<()> {
    serve_on_with_mode(root, listener, open_browser, None, true).await
}

async fn serve_on_with_mode(
    root: PathBuf,
    listener: tokio::net::TcpListener,
    open_browser: bool,
    initial_tunnel: Option<(tunnel::Tunnel, String)>,
    hub_mode: bool,
) -> anyhow::Result<()> {
    let actual_port = listener.local_addr()?.port();

    // Start the Citadel telemetry flush task now that tokio is running.
    // No-op if PONDER_CITADEL_* env vars were not set.
    citadel::start_citadel_flush();

    if hub_mode {
        tracing::info!(port = actual_port, "SDLC hub server started");
    } else {
        tracing::info!(port = actual_port, "SDLC server started");
    }

    if open_browser {
        let url = format!("http://localhost:{actual_port}");
        tracing::debug!("opening browser at {url}");
        let _ = open::that(&url);
        tracing::debug!("browser open returned");
    }

    tracing::debug!("initializing app state");
    let app_state = if hub_mode {
        state::AppState::new_with_port_hub(root, actual_port)
    } else {
        state::AppState::new_with_port(root, actual_port)
    };
    tracing::debug!("app state ready");

    if let Some((tun, token)) = initial_tunnel {
        let url = tun.url.clone();
        tracing::debug!("seeding tunnel state: {url}");
        *app_state.tunnel_handle.lock().await = Some(tun);
        let oauth = app_state.tunnel_snapshot.read().await.oauth_enabled;
        *app_state.tunnel_snapshot.write().await = state::TunnelSnapshot {
            config: auth::TunnelConfig::with_token(token),
            url: Some(url),
            oauth_enabled: oauth,
        };
        tracing::debug!("tunnel state seeded");
    }

    tracing::debug!("building router");
    let app = build_router_from_state(app_state);
    tracing::debug!("router ready — accepting connections");

    axum::serve(listener, app).await?;
    Ok(())
}
