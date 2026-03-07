use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::fleet;
use crate::hub::{
    ActivitySeverity, HeartbeatPayload, HubActivityEntry, HubSseMessage, ProjectStatus,
    ProvisionState,
};
use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
pub struct HubSummary {
    pub total_projects: usize,
    pub online: usize,
    pub degraded: usize,
    pub provisioning: usize,
    pub failed: usize,
    pub active_agents: usize,
    pub attention_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct HubAttentionItem {
    pub id: String,
    pub severity: ActivitySeverity,
    pub title: String,
    pub detail: String,
    pub slug: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FleetBucket {
    Online,
    Degraded,
    Provisioning,
    Failed,
}

fn classify_instance(instance: &fleet::FleetInstance) -> FleetBucket {
    if matches!(instance.deployment_status, fleet::DeploymentStatus::Failed)
        || matches!(instance.provision_status, Some(ProvisionState::Failed))
    {
        FleetBucket::Failed
    } else if matches!(
        instance.provision_status,
        Some(ProvisionState::Requested | ProvisionState::Provisioning)
    ) || matches!(instance.deployment_status, fleet::DeploymentStatus::Pending)
    {
        FleetBucket::Provisioning
    } else if !instance.attention_reasons.is_empty() {
        FleetBucket::Degraded
    } else {
        FleetBucket::Online
    }
}

fn build_summary(instances: &[fleet::FleetInstance]) -> HubSummary {
    let mut online = 0usize;
    let mut degraded = 0usize;
    let mut provisioning = 0usize;
    let mut failed = 0usize;
    let mut active_agents = 0usize;

    for instance in instances {
        match classify_instance(instance) {
            FleetBucket::Online => online += 1,
            FleetBucket::Degraded => degraded += 1,
            FleetBucket::Provisioning => provisioning += 1,
            FleetBucket::Failed => failed += 1,
        }
        if instance.agent_running == Some(true) {
            active_agents += 1;
        }
    }

    HubSummary {
        total_projects: instances.len(),
        online,
        degraded,
        provisioning,
        failed,
        active_agents,
        attention_count: build_attention_items(instances).len(),
    }
}

fn build_attention_items(instances: &[fleet::FleetInstance]) -> Vec<HubAttentionItem> {
    let mut items = Vec::new();
    for instance in instances {
        let bucket = classify_instance(instance);
        if bucket == FleetBucket::Online {
            continue;
        }
        let severity = match bucket {
            FleetBucket::Failed => ActivitySeverity::Error,
            FleetBucket::Provisioning | FleetBucket::Degraded => ActivitySeverity::Warning,
            FleetBucket::Online => ActivitySeverity::Info,
        };
        let title = match bucket {
            FleetBucket::Failed => format!("{} needs intervention", instance.slug),
            FleetBucket::Provisioning => format!("{} is provisioning", instance.slug),
            FleetBucket::Degraded => format!("{} is degraded", instance.slug),
            FleetBucket::Online => format!("{} is online", instance.slug),
        };
        let detail = if !instance.attention_reasons.is_empty() {
            instance.attention_reasons.join(" • ")
        } else {
            "Requires review".to_string()
        };
        items.push(HubAttentionItem {
            id: format!("attention:{}", instance.slug),
            severity,
            title,
            detail,
            slug: Some(instance.slug.clone()),
            url: Some(instance.url.clone()),
        });
    }
    items
}

fn placeholder_instance(slug: &str, url: String, status: ProvisionState) -> fleet::FleetInstance {
    let mut instance = fleet::FleetInstance {
        slug: slug.to_string(),
        namespace: format!("sdlc-{slug}"),
        url,
        deployment_status: match status {
            ProvisionState::Failed => fleet::DeploymentStatus::Failed,
            ProvisionState::Ready => fleet::DeploymentStatus::Running,
            ProvisionState::Requested | ProvisionState::Provisioning => {
                fleet::DeploymentStatus::Pending
            }
        },
        pod_healthy: matches!(status, ProvisionState::Ready),
        created_at: None,
        active_milestone: None,
        feature_count: None,
        agent_running: None,
        last_heartbeat_at: None,
        heartbeat_status: None,
        provision_status: Some(status),
        attention_reasons: Vec::new(),
    };
    if matches!(instance.deployment_status, fleet::DeploymentStatus::Pending) {
        instance
            .attention_reasons
            .push("Provisioning is still in progress".to_string());
    }
    if matches!(instance.deployment_status, fleet::DeploymentStatus::Failed) {
        instance
            .attention_reasons
            .push("Latest provision attempt failed".to_string());
    }
    instance
}

async fn load_fleet_instances(
    app: &AppState,
) -> Result<Vec<fleet::FleetInstance>, axum::response::Response> {
    let hub_lock = app.hub_registry.as_ref();
    let instances = fleet::list_fleet_instances(
        app.kube_client.as_ref(),
        hub_lock.map(|h| h.as_ref()),
        &app.ingress_domain,
    )
    .await
    .map_err(|e| e.into_response())?;

    if app.kube_client.is_none() {
        if let Some(hub) = &app.hub_registry {
            let registry = hub.lock().await;
            let mut fallback: Vec<fleet::FleetInstance> = registry
                .projects_sorted()
                .iter()
                .map(|p| {
                    let mut instance = fleet::FleetInstance {
                        slug: p.name.clone(),
                        namespace: format!("sdlc-{}", p.name),
                        url: p.url.clone(),
                        deployment_status: fleet::DeploymentStatus::Unknown,
                        pod_healthy: false,
                        created_at: None,
                        active_milestone: p.active_milestone.clone(),
                        feature_count: p.feature_count,
                        agent_running: p.agent_running,
                        last_heartbeat_at: Some(p.last_seen),
                        heartbeat_status: Some(p.status.clone()),
                        provision_status: registry
                            .provisions
                            .get(&p.name)
                            .map(|provision| provision.status.clone()),
                        attention_reasons: Vec::new(),
                    };
                    if p.status != ProjectStatus::Online {
                        instance.attention_reasons.push(match p.status {
                            ProjectStatus::Stale => "Heartbeat is stale".to_string(),
                            ProjectStatus::Offline => "Heartbeat is offline".to_string(),
                            ProjectStatus::Online => String::new(),
                        });
                    }
                    instance
                })
                .collect();
            let existing: std::collections::HashSet<String> = fallback
                .iter()
                .map(|instance| instance.slug.clone())
                .collect();
            fallback.extend(
                instances
                    .into_iter()
                    .filter(|instance| !existing.contains(&instance.slug)),
            );
            fallback.sort_by(|a, b| a.slug.cmp(&b.slug));
            return Ok(fallback);
        }
    } else if let Some(hub) = &app.hub_registry {
        let mut registry = hub.lock().await;
        registry.reconcile_fleet(&instances);
    }

    Ok(instances)
}

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
        Ok(HubSseMessage::FleetUpdated(instance)) => {
            let data = serde_json::json!({
                "type": "fleet_updated",
                "instance": instance,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Ok(HubSseMessage::FleetProvisioned(instance)) => {
            let data = serde_json::json!({
                "type": "fleet_provisioned",
                "instance": instance,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Ok(HubSseMessage::ProvisionUpdated(provision)) => {
            let data = serde_json::json!({
                "type": "provision_updated",
                "provision": provision,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Ok(HubSseMessage::ActivityAppended(activity)) => {
            let data = serde_json::json!({
                "type": "activity_appended",
                "activity": activity,
            })
            .to_string();
            Some(Ok(Event::default().event("hub").data(data)))
        }
        Ok(HubSseMessage::FleetAgentStatus {
            total_active_runs,
            projects_with_agents,
        }) => {
            let data = serde_json::json!({
                "type": "fleet_agent_status",
                "agent_summary": {
                    "total_active_runs": total_active_runs,
                    "projects_with_agents": projects_with_agents,
                },
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

    match load_fleet_instances(&app).await {
        Ok(instances) => {
            if app.kube_client.is_none() {
                tracing::warn!("fleet listing has degraded data");
            }
            Json(instances).into_response()
        }
        Err(e) => e,
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

    let instances = match load_fleet_instances(&app).await {
        Ok(i) => i,
        Err(e) => return e,
    };

    let repos = match fleet::list_gitea_repos(&app.http_client, gitea_url, gitea_token).await {
        Ok(r) => r,
        Err(e) => return e.into_response(),
    };

    let avail = fleet::list_available_repos(&instances, &repos);
    Json(avail).into_response()
}

/// GET /api/hub/summary
pub async fn summary(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    match load_fleet_instances(&app).await {
        Ok(instances) => Json(build_summary(&instances)).into_response(),
        Err(e) => e,
    }
}

/// GET /api/hub/attention
pub async fn attention(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    match load_fleet_instances(&app).await {
        Ok(instances) => Json(build_attention_items(&instances)).into_response(),
        Err(e) => e,
    }
}

/// GET /api/hub/activity
pub async fn activity(State(app): State<AppState>) -> axum::response::Response {
    if app.hub_registry.is_none() {
        return not_hub_mode();
    }

    let Some(hub) = &app.hub_registry else {
        return Json(Vec::<HubActivityEntry>::new()).into_response();
    };
    let registry = hub.lock().await;
    Json(registry.activity_recent(30)).into_response()
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
            // Emit fleet SSE event so the UI spinner clears
            if let Some(hub) = &app.hub_registry {
                let url = format!("https://{}.{}", req.repo_slug, app.ingress_domain);
                let mut registry = hub.lock().await;
                let _ = registry.start_provision(&req.repo_slug, url.clone(), "start", None);
                let _ =
                    registry
                        .event_tx
                        .send(HubSseMessage::FleetProvisioned(placeholder_instance(
                            &req.repo_slug,
                            url,
                            ProvisionState::Requested,
                        )));
            }
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
    /// Optional — derived from the last path segment of `clone_url` if omitted.
    pub repo_name: Option<String>,
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
    if !req.clone_url.starts_with("https://") && !req.clone_url.starts_with("http://") {
        return fleet::FleetError::InvalidRequest("clone_url must be an HTTP(S) URL".into())
            .into_response();
    }

    let repo_name = req
        .repo_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| {
            req.clone_url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("repo")
                .trim_end_matches(".git")
                .to_string()
        });

    let (gitea_url, gitea_token) = match (&app.gitea_url, &app.gitea_token) {
        (Some(url), Some(token)) => (url.as_str(), token.as_str()),
        _ => return gitea_not_configured(),
    };

    let imported = match fleet::import_repo(
        &app.http_client,
        gitea_url,
        gitea_token,
        &req.clone_url,
        &repo_name,
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
        } else if let Some(hub) = &app.hub_registry {
            let url = format!("https://{}.{}", imported.slug, app.ingress_domain);
            let mut registry = hub.lock().await;
            let _ = registry.start_provision(
                &imported.slug,
                url.clone(),
                "import",
                Some(format!("Imported from {}", req.clone_url)),
            );
            let _ = registry
                .event_tx
                .send(HubSseMessage::FleetProvisioned(placeholder_instance(
                    &imported.slug,
                    url,
                    ProvisionState::Requested,
                )));
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

    if let Some(hub) = &app.hub_registry {
        let mut registry = hub.lock().await;
        registry.push_activity(
            "repo_created",
            ActivitySeverity::Success,
            format!("Created repo {}", repo.slug),
            Some("Push your code to start the workspace".to_string()),
            Some(repo.slug.clone()),
            Some(gitea_web_url.clone()),
        );
        if provision_triggered {
            let url = format!("https://{}.{}", repo.slug, app.ingress_domain);
            let _ = registry.start_provision(
                &repo.slug,
                url.clone(),
                "create",
                Some("New project created from hub".to_string()),
            );
            let _ = registry
                .event_tx
                .send(HubSseMessage::FleetProvisioned(placeholder_instance(
                    &repo.slug,
                    url,
                    ProvisionState::Requested,
                )));
        }
    }

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
        .filter(|p| p.agent_running == Some(true) && p.status != ProjectStatus::Offline)
        .map(|p| p.name.clone())
        .collect();

    let projects_with_agents = active_projects.len();

    Json(serde_json::json!({
        "total_active_runs": projects_with_agents,
        "projects_with_agents": projects_with_agents,
    }))
    .into_response()
}
