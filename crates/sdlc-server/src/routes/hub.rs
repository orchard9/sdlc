use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::fleet;
use crate::hub::{HeartbeatPayload, HubSseMessage};
use crate::state::AppState;

/// POST /api/hub/heartbeat
///
/// Accepts a heartbeat payload from a project instance. First call from a given
/// URL registers the project; subsequent calls update `last_seen`. Returns 503
/// if the server is not running in hub mode.
pub async fn heartbeat(
    State(app): State<AppState>,
    Json(payload): Json<HeartbeatPayload>,
) -> axum::response::Response {
    let Some(hub) = &app.hub_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "not running in hub mode"})),
        )
            .into_response();
    };
    let mut registry = hub.lock().await;
    let _entry = registry.apply_heartbeat(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({"registered": true})),
    )
        .into_response()
}

/// GET /api/hub/projects
///
/// Returns the current project registry sorted by last_seen descending.
/// Returns 503 if not in hub mode.
pub async fn list_projects(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let Some(hub) = &app.hub_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"error": "not running in hub mode"})),
        )
            .into_response();
    };
    let registry = hub.lock().await;
    let projects = registry.projects_sorted();
    axum::Json(serde_json::json!(projects)).into_response()
}

/// GET /api/hub/events
///
/// SSE stream for hub UI clients. Emits `ProjectUpdated` and `ProjectRemoved` events.
/// Returns 503 if not in hub mode.
pub async fn hub_sse_events(State(app): State<AppState>) -> impl axum::response::IntoResponse {
    let Some(hub) = &app.hub_registry else {
        return (StatusCode::SERVICE_UNAVAILABLE, "not running in hub mode").into_response();
    };
    let rx = hub.lock().await.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(HubSseMessage::ProjectUpdated(entry)) => {
            let data = serde_json::json!({
                "type": "project_updated",
                "project": entry,
            })
            .to_string();
            Some(Ok::<Event, Infallible>(
                Event::default().event("hub").data(data),
            ))
        }
        Ok(HubSseMessage::ProjectRemoved { url }) => {
            let data = serde_json::json!({
                "type": "project_removed",
                "url": url,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Err(_) => None,
    });

    // 2KB padding comment so Cloudflare/nginx don't buffer the initial flush.
    let padding = Ok::<Event, Infallible>(Event::default().comment(" ".repeat(2048)));
    let padded = tokio_stream::iter(std::iter::once(padding)).chain(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    headers.insert(
        header::HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    (headers, Sse::new(padded).keep_alive(KeepAlive::default())).into_response()
}

// ---------------------------------------------------------------------------
// Fleet management endpoints
// ---------------------------------------------------------------------------

/// Return 503 JSON if not in hub mode.
fn not_hub_mode() -> axum::response::Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "not running in hub mode"})),
    )
        .into_response()
}

/// Return 503 JSON if Gitea is not configured.
fn gitea_not_configured() -> axum::response::Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "gitea_not_configured",
            "detail": "GITEA_URL and GITEA_API_TOKEN must be set"
        })),
    )
        .into_response()
}

/// GET /api/hub/fleet
pub async fn fleet(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let hub_lock = app.hub_registry.as_ref();
    match fleet::list_fleet_instances(
        app.kube_client.as_ref(),
        hub_lock.map(|h| h.as_ref()),
        &app.ingress_domain,
    )
    .await
    {
        Ok(instances) => {
            let warning = if app.kube_client.is_none() {
                Some("k8s not available — showing heartbeat data only")
            } else {
                None
            };

            // If no k8s client, fall back to heartbeat-only data
            let instances = if app.kube_client.is_none() {
                if let Some(hub) = &app.hub_registry {
                    let registry = hub.lock().await;
                    registry
                        .projects_sorted()
                        .iter()
                        .map(|p| fleet::FleetInstance {
                            slug: p.name.clone(),
                            namespace: format!("sdlc-{}", p.name),
                            url: p.url.clone(),
                            deployment_status: fleet::DeploymentStatus::Unknown,
                            pod_healthy: false,
                            created_at: None,
                            active_milestone: p.active_milestone.clone(),
                            feature_count: p.feature_count,
                            agent_running: p.agent_running,
                        })
                        .collect()
                } else {
                    instances
                }
            } else {
                instances
            };

            if let Some(w) = warning {
                tracing::warn!(warning = w, "fleet listing has degraded data");
            }
            Json(instances).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// GET /api/hub/repos
pub async fn repos(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let (gitea_url, gitea_token) = match (&app.gitea_url, &app.gitea_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => return gitea_not_configured(),
    };

    match fleet::list_gitea_repos(&app.http_client, gitea_url, gitea_token).await {
        Ok(repos) => {
            let total = repos.len();
            Json(serde_json::json!({
                "repos": repos,
                "total": total,
            }))
            .into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// GET /api/hub/available
pub async fn available(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let (gitea_url, gitea_token) = match (&app.gitea_url, &app.gitea_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => return gitea_not_configured(),
    };

    let hub_lock = app.hub_registry.as_ref();
    let instances = match fleet::list_fleet_instances(
        app.kube_client.as_ref(),
        hub_lock.map(|h| h.as_ref()),
        &app.ingress_domain,
    )
    .await
    {
        Ok(i) => i,
        Err(e) => return e.into_response(),
    };

    let repos = match fleet::list_gitea_repos(&app.http_client, gitea_url, gitea_token).await {
        Ok(r) => r,
        Err(e) => return e.into_response(),
    };

    let avail = fleet::list_available_repos(&instances, &repos);
    Json(avail).into_response()
}

/// POST /api/hub/provision
#[derive(serde::Deserialize)]
pub struct ProvisionRequest {
    pub repo_slug: String,
}

pub async fn provision(
    State(app): State<AppState>,
    Json(req): Json<ProvisionRequest>,
) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    if req.repo_slug.is_empty() {
        return fleet::FleetError::InvalidRequest("repo_slug is required".into()).into_response();
    }

    let (woodpecker_url, woodpecker_token) = match (&app.woodpecker_url, &app.woodpecker_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "woodpecker_not_configured",
                    "detail": "WOODPECKER_URL and WOODPECKER_API_TOKEN must be set"
                })),
            )
                .into_response();
        }
    };

    // Optionally validate the repo exists in Gitea
    if let (Some(gitea_url), Some(gitea_token)) = (&app.gitea_url, &app.gitea_token) {
        match fleet::list_gitea_repos(&app.http_client, gitea_url, gitea_token).await {
            Ok(repos) => {
                if !repos.iter().any(|r| r.slug == req.repo_slug) {
                    return fleet::FleetError::RepoNotFound(req.repo_slug.clone()).into_response();
                }
            }
            Err(_) => {
                tracing::warn!("gitea unreachable during provision validation — proceeding");
            }
        }
    }

    match fleet::trigger_provision(
        &app.http_client,
        woodpecker_url,
        woodpecker_token,
        &req.repo_slug,
    )
    .await
    {
        Ok(()) => {
            let _ = app.event_tx.send(crate::state::SseMessage::Update);
            Json(serde_json::json!({
                "status": "provisioning",
                "repo_slug": req.repo_slug,
            }))
            .into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// POST /api/hub/import
#[derive(serde::Deserialize)]
pub struct ImportRequest {
    pub clone_url: String,
    pub repo_name: String,
    pub auth_token: Option<String>,
}

pub async fn import(
    State(app): State<AppState>,
    Json(req): Json<ImportRequest>,
) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    if req.clone_url.is_empty() {
        return fleet::FleetError::InvalidRequest("clone_url is required".into()).into_response();
    }
    if req.repo_name.is_empty() {
        return fleet::FleetError::InvalidRequest("repo_name is required".into()).into_response();
    }
    if !req.clone_url.starts_with("https://") && !req.clone_url.starts_with("http://") {
        return fleet::FleetError::InvalidRequest("clone_url must be an HTTP(S) URL".into())
            .into_response();
    }

    let (gitea_url, gitea_token) = match (&app.gitea_url, &app.gitea_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => return gitea_not_configured(),
    };

    let imported = match fleet::import_repo(
        &app.http_client,
        gitea_url,
        gitea_token,
        &req.clone_url,
        &req.repo_name,
        req.auth_token.as_deref(),
    )
    .await
    {
        Ok(repo) => repo,
        Err(e) => return e.into_response(),
    };

    // Trigger provisioning if Woodpecker is configured
    if let (Some(woodpecker_url), Some(woodpecker_token)) =
        (&app.woodpecker_url, &app.woodpecker_token)
    {
        if let Err(e) = fleet::trigger_provision(
            &app.http_client,
            woodpecker_url,
            woodpecker_token,
            &imported.slug,
        )
        .await
        {
            tracing::warn!(error = %e, "provision trigger failed after import");
        }
    }

    Json(serde_json::json!({
        "status": "importing",
        "repo_name": imported.slug,
        "gitea_url": imported.clone_url,
    }))
    .into_response()
}

/// POST /api/hub/create-repo
///
/// Creates a new empty repo in the orchard9 Gitea org and returns HTTP push credentials.
/// The caller can immediately add the returned `push_url` as a git remote and push.
#[derive(serde::Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
}

pub async fn create_repo(
    State(app): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let (gitea_url, gitea_token) = match (&app.gitea_url, &app.gitea_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => return gitea_not_configured(),
    };

    // Validate: non-empty, lowercase alphanumeric + hyphens
    if req.name.is_empty() {
        return fleet::FleetError::InvalidRequest("name is required".into()).into_response();
    }
    let valid = req.name.len() <= 100
        && req
            .name
            .chars()
            .next()
            .map(|c| c.is_ascii_alphanumeric())
            .unwrap_or(false)
        && req
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid {
        return fleet::FleetError::InvalidRequest(
            "name must be lowercase alphanumeric with hyphens, max 100 chars".into(),
        )
        .into_response();
    }

    // Create the repo in Gitea
    let repo =
        match fleet::create_gitea_repo(&app.http_client, gitea_url, gitea_token, &req.name).await {
            Ok(r) => r,
            Err(e) => return e.into_response(),
        };

    // Get admin username to build the authenticated push URL
    let username = match fleet::get_gitea_username(&app.http_client, gitea_url, gitea_token).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    // Build push URL: http://user:token@host/org/name.git
    // Strip scheme from gitea_url to reconstruct with credentials
    let push_url = {
        let host_path = gitea_url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let scheme = if gitea_url.starts_with("https://") {
            "https"
        } else {
            "http"
        };
        format!(
            "{scheme}://{username}:{gitea_token}@{host_path}/orchard9/{}.git",
            repo.slug
        )
    };

    let gitea_web_url = format!("{gitea_url}/orchard9/{}", repo.slug);

    // Trigger provisioning if Woodpecker is configured (fire-and-forget)
    let provision_triggered = if let (Some(woodpecker_url), Some(woodpecker_token)) =
        (&app.woodpecker_url, &app.woodpecker_token)
    {
        match fleet::trigger_provision(
            &app.http_client,
            woodpecker_url,
            woodpecker_token,
            &repo.slug,
        )
        .await
        {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!(error = %e, "provision trigger failed after create-repo");
                false
            }
        }
    } else {
        false
    };

    Json(serde_json::json!({
        "repo_slug": repo.slug,
        "push_url": push_url,
        "gitea_url": gitea_web_url,
        "provision_triggered": provision_triggered,
    }))
    .into_response()
}

/// GET /api/hub/agents
pub async fn agents(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let Some(hub) = &app.hub_registry else {
        return Json(serde_json::json!({
            "active_count": 0,
            "active_projects": [],
        }))
        .into_response();
    };

    let registry = hub.lock().await;
    let active_projects: Vec<String> = registry
        .projects
        .values()
        .filter(|p| p.agent_running == Some(true))
        .map(|p| p.name.clone())
        .collect();

    let projects_with_agents = active_projects.len();

    Json(serde_json::json!({
        "total_active_runs": projects_with_agents,
        "projects_with_agents": projects_with_agents,
    }))
    .into_response()
}
