//! Fleet management: k8s namespace discovery, Gitea repo listing, provisioning.
//!
//! All functions in this module are pure data-fetching / diffing logic.
//! They take client references and return typed results — no axum dependency.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of a k8s deployment in a fleet namespace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Running,
    Pending,
    Failed,
    Unknown,
}

/// One sdlc project instance discovered from the k8s cluster.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetInstance {
    pub slug: String,
    pub namespace: String,
    pub url: String,
    pub deployment_status: DeploymentStatus,
    pub pod_healthy: bool,
    pub created_at: Option<DateTime<Utc>>,
    // Merged from HubRegistry heartbeat (optional)
    pub active_milestone: Option<String>,
    pub feature_count: Option<u32>,
    pub agent_running: Option<bool>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub heartbeat_status: Option<crate::hub::ProjectStatus>,
    pub provision_status: Option<crate::hub::ProvisionState>,
    #[serde(default)]
    pub attention_reasons: Vec<String>,
}

/// A repo from the Gitea org.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GiteaRepo {
    pub slug: String,
    pub full_name: String,
    pub description: Option<String>,
    pub clone_url: String,
    pub created_at: Option<DateTime<Utc>>,
    pub archived: bool,
}

/// A repo that does not have a running instance in the fleet.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailableRepo {
    #[serde(flatten)]
    pub repo: GiteaRepo,
    pub can_provision: bool,
}

// ---------------------------------------------------------------------------
// Namespace exclusion
// ---------------------------------------------------------------------------

/// Namespaces that start with `sdlc-` but are NOT project instances.
const EXCLUDED_NAMESPACES: &[&str] = &["sdlc-tls", "sdlc-hub", "sdlc-system"];

/// Check if a namespace name should be excluded from fleet listing.
pub fn is_excluded_namespace(name: &str) -> bool {
    EXCLUDED_NAMESPACES.contains(&name)
}

// ---------------------------------------------------------------------------
// k8s fleet discovery
// ---------------------------------------------------------------------------

/// Discover fleet instances from the k8s API.
///
/// Returns an empty vec (not an error) if the kube client is `None` (running
/// outside k8s). The caller can check for an empty result and include a
/// warning in the response.
pub async fn list_fleet_instances(
    kube_client: Option<&kube::Client>,
    hub_registry: Option<&tokio::sync::Mutex<crate::hub::HubRegistry>>,
    ingress_domain: &str,
) -> Result<Vec<FleetInstance>, FleetError> {
    let mut heartbeat_by_url: std::collections::HashMap<String, crate::hub::ProjectEntry> =
        std::collections::HashMap::new();
    let mut provision_by_slug: std::collections::HashMap<String, crate::hub::ProvisionEntry> =
        std::collections::HashMap::new();
    if let Some(registry_lock) = hub_registry {
        let registry = registry_lock.lock().await;
        heartbeat_by_url = registry.projects.clone();
        provision_by_slug = registry.provisions.clone();
    }

    let client = match kube_client {
        Some(c) => c,
        None => {
            let mut placeholders = provision_placeholders(&provision_by_slug);
            placeholders.sort_by(|a, b| a.slug.cmp(&b.slug));
            return Ok(placeholders);
        }
    };

    use k8s_openapi::api::apps::v1::Deployment;
    use k8s_openapi::api::core::v1::Namespace;
    use kube::api::{Api, ListParams};

    // List all namespaces
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let ns_list = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| FleetError::K8sUnavailable(e.to_string()))?;

    // Collect sdlc-* namespaces that aren't excluded
    let sdlc_namespaces: Vec<String> = ns_list
        .items
        .iter()
        .filter_map(|ns| {
            let name = ns.metadata.name.as_deref()?;
            if name.starts_with("sdlc-") && !is_excluded_namespace(name) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

    // For each namespace, check for sdlc-server deployment
    let mut instances = Vec::new();
    for ns_name in &sdlc_namespaces {
        let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), ns_name);
        let deploys = match deploy_api
            .list(&ListParams::default().labels("app.kubernetes.io/name=sdlc-server"))
            .await
        {
            Ok(d) => d,
            Err(_) => continue, // Skip namespace if we can't list deployments
        };

        if deploys.items.is_empty() {
            continue; // No sdlc-server deployment — not a project namespace
        }

        let deploy = &deploys.items[0];
        let slug = ns_name.strip_prefix("sdlc-").unwrap_or(ns_name).to_string();
        let url = format!("https://{slug}.{ingress_domain}");

        // Determine deployment status from conditions
        let (status, healthy) = deployment_status(deploy);

        // Extract creation timestamp
        let created_at = deploy.metadata.creation_timestamp.as_ref().map(|ts| ts.0);

        let mut instance = FleetInstance {
            slug: slug.clone(),
            namespace: ns_name.clone(),
            url: url.clone(),
            deployment_status: status,
            pod_healthy: healthy,
            created_at,
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_heartbeat_at: None,
            heartbeat_status: None,
            provision_status: None,
            attention_reasons: Vec::new(),
        };

        if let Some(entry) = heartbeat_by_url.get(&url) {
            instance.active_milestone = entry.active_milestone.clone();
            instance.feature_count = entry.feature_count;
            instance.agent_running = entry.agent_running;
            instance.last_heartbeat_at = Some(entry.last_seen);
            instance.heartbeat_status = Some(entry.status.clone());
        }
        if let Some(provision) = provision_by_slug.get(&slug) {
            instance.provision_status = Some(provision.status.clone());
        }
        decorate_attention(&mut instance);
        instances.push(instance);
    }

    let known_slugs: std::collections::HashSet<String> = instances
        .iter()
        .map(|instance| instance.slug.clone())
        .collect();
    for placeholder in provision_by_slug.values() {
        if known_slugs.contains(&placeholder.slug) {
            continue;
        }
        let mut instance = FleetInstance {
            slug: placeholder.slug.clone(),
            namespace: format!("sdlc-{}", placeholder.slug),
            url: placeholder.url.clone(),
            deployment_status: match placeholder.status {
                crate::hub::ProvisionState::Failed => DeploymentStatus::Failed,
                crate::hub::ProvisionState::Ready => DeploymentStatus::Running,
                crate::hub::ProvisionState::Requested
                | crate::hub::ProvisionState::Provisioning => DeploymentStatus::Pending,
            },
            pod_healthy: matches!(placeholder.status, crate::hub::ProvisionState::Ready),
            created_at: Some(placeholder.created_at),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_heartbeat_at: None,
            heartbeat_status: None,
            provision_status: Some(placeholder.status.clone()),
            attention_reasons: Vec::new(),
        };
        if let Some(entry) = heartbeat_by_url.get(&placeholder.url) {
            instance.active_milestone = entry.active_milestone.clone();
            instance.feature_count = entry.feature_count;
            instance.agent_running = entry.agent_running;
            instance.last_heartbeat_at = Some(entry.last_seen);
            instance.heartbeat_status = Some(entry.status.clone());
        }
        decorate_attention(&mut instance);
        instances.push(instance);
    }

    instances.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(instances)
}

fn provision_placeholders(
    provisions: &std::collections::HashMap<String, crate::hub::ProvisionEntry>,
) -> Vec<FleetInstance> {
    let mut instances = Vec::new();
    for provision in provisions.values() {
        let mut instance = FleetInstance {
            slug: provision.slug.clone(),
            namespace: format!("sdlc-{}", provision.slug),
            url: provision.url.clone(),
            deployment_status: match provision.status {
                crate::hub::ProvisionState::Failed => DeploymentStatus::Failed,
                crate::hub::ProvisionState::Ready => DeploymentStatus::Running,
                crate::hub::ProvisionState::Requested
                | crate::hub::ProvisionState::Provisioning => DeploymentStatus::Pending,
            },
            pod_healthy: matches!(provision.status, crate::hub::ProvisionState::Ready),
            created_at: Some(provision.created_at),
            active_milestone: None,
            feature_count: None,
            agent_running: None,
            last_heartbeat_at: None,
            heartbeat_status: None,
            provision_status: Some(provision.status.clone()),
            attention_reasons: Vec::new(),
        };
        decorate_attention(&mut instance);
        instances.push(instance);
    }
    instances
}

fn decorate_attention(instance: &mut FleetInstance) {
    instance.attention_reasons.clear();
    match instance.deployment_status {
        DeploymentStatus::Failed => {
            instance
                .attention_reasons
                .push("Deployment failed in the cluster".to_string());
        }
        DeploymentStatus::Running if !instance.pod_healthy => {
            instance
                .attention_reasons
                .push("Pod is running but not healthy".to_string());
        }
        DeploymentStatus::Pending => {
            instance
                .attention_reasons
                .push("Provisioning is still in progress".to_string());
        }
        DeploymentStatus::Unknown if instance.provision_status.is_none() => {
            instance
                .attention_reasons
                .push("Deployment state is unknown".to_string());
        }
        _ => {}
    }
    if let Some(status) = &instance.heartbeat_status {
        match status {
            crate::hub::ProjectStatus::Stale => instance
                .attention_reasons
                .push("Heartbeat is stale".to_string()),
            crate::hub::ProjectStatus::Offline => instance
                .attention_reasons
                .push("Heartbeat is offline".to_string()),
            crate::hub::ProjectStatus::Online => {}
        }
    } else if matches!(instance.deployment_status, DeploymentStatus::Running) {
        instance
            .attention_reasons
            .push("No heartbeat has been received yet".to_string());
    }
    if matches!(
        instance.provision_status,
        Some(crate::hub::ProvisionState::Failed)
    ) {
        instance
            .attention_reasons
            .push("Latest provision attempt failed".to_string());
    }
}

/// Extract deployment status and pod health from a k8s Deployment.
fn deployment_status(deploy: &k8s_openapi::api::apps::v1::Deployment) -> (DeploymentStatus, bool) {
    let status = match &deploy.status {
        Some(s) => s,
        None => return (DeploymentStatus::Unknown, false),
    };

    let available = status.available_replicas.unwrap_or(0);
    let desired = status.replicas.unwrap_or(0);

    if available > 0 && available >= desired {
        (DeploymentStatus::Running, true)
    } else if desired > 0 {
        // Check conditions for failure
        let failed = status.conditions.as_ref().is_some_and(|conditions| {
            conditions.iter().any(|c| {
                c.type_ == "Available" && c.status == "False"
                    || c.type_ == "Progressing"
                        && c.status == "False"
                        && c.reason.as_deref() == Some("ProgressDeadlineExceeded")
            })
        });
        if failed {
            (DeploymentStatus::Failed, false)
        } else {
            (DeploymentStatus::Pending, false)
        }
    } else {
        (DeploymentStatus::Unknown, false)
    }
}

// ---------------------------------------------------------------------------
// Gitea repo listing
// ---------------------------------------------------------------------------

/// Gitea API response for a single repo (only fields we need).
#[derive(Debug, Deserialize)]
struct GiteaApiRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    clone_url: Option<String>,
    html_url: Option<String>,
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    archived: bool,
}

/// List all repos in the orchard9 Gitea org.
pub async fn list_gitea_repos(
    http_client: &reqwest::Client,
    gitea_url: &str,
    token: &str,
) -> Result<Vec<GiteaRepo>, FleetError> {
    let mut all_repos = Vec::new();
    let mut page = 1u32;
    let limit = 50u32;

    loop {
        let url = format!("{gitea_url}/api/v1/orgs/orchard9/repos?limit={limit}&page={page}");
        let resp = http_client
            .get(&url)
            .header("Authorization", format!("token {token}"))
            .send()
            .await
            .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(FleetError::GiteaUnavailable(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        let repos: Vec<GiteaApiRepo> = resp
            .json()
            .await
            .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

        let count = repos.len();
        for r in repos {
            all_repos.push(GiteaRepo {
                slug: r.name.clone(),
                full_name: r.full_name,
                description: r.description,
                clone_url: r
                    .clone_url
                    .unwrap_or_else(|| r.html_url.unwrap_or_default()),
                created_at: r.created_at,
                archived: r.archived,
            });
        }

        if count < limit as usize {
            break;
        }
        page += 1;
    }

    all_repos.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(all_repos)
}

// ---------------------------------------------------------------------------
// Available repos (diff)
// ---------------------------------------------------------------------------

/// Return repos that do not have a running fleet instance.
pub fn list_available_repos(
    instances: &[FleetInstance],
    repos: &[GiteaRepo],
) -> Vec<AvailableRepo> {
    let instance_slugs: std::collections::HashSet<&str> =
        instances.iter().map(|i| i.slug.as_str()).collect();

    repos
        .iter()
        .filter(|r| !instance_slugs.contains(r.slug.as_str()))
        .map(|r| AvailableRepo {
            repo: r.clone(),
            can_provision: !r.archived,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Provisioning
// ---------------------------------------------------------------------------

/// Trigger the fleet-reconcile Woodpecker pipeline for a repo slug.
pub async fn trigger_provision(
    http_client: &reqwest::Client,
    woodpecker_url: &str,
    woodpecker_token: &str,
    repo_slug: &str,
) -> Result<(), FleetError> {
    // Woodpecker 2.x API uses numeric repo IDs: POST /api/repos/{repo_id}/pipelines
    // First look up the sdlc repo to get its ID.
    let lookup_url = format!("{woodpecker_url}/api/repos/lookup/orchard9/sdlc");
    let lookup_resp = http_client
        .get(&lookup_url)
        .header("Authorization", format!("Bearer {woodpecker_token}"))
        .send()
        .await
        .map_err(|e| FleetError::WoodpeckerUnavailable(e.to_string()))?;

    if !lookup_resp.status().is_success() {
        let status = lookup_resp.status();
        let detail = lookup_resp.text().await.unwrap_or_default();
        return Err(FleetError::WoodpeckerUnavailable(format!(
            "repo lookup failed HTTP {status}: {detail}"
        )));
    }

    let repo: serde_json::Value = lookup_resp
        .json()
        .await
        .map_err(|e| FleetError::WoodpeckerUnavailable(e.to_string()))?;
    let repo_id = repo["id"]
        .as_i64()
        .ok_or_else(|| FleetError::WoodpeckerUnavailable("repo id not found".into()))?;

    let url = format!("{woodpecker_url}/api/repos/{repo_id}/pipelines");

    let body = serde_json::json!({
        "branch": "main",
        "variables": {
            "PROVISION_SLUG": repo_slug,
        }
    });

    let resp = http_client
        .post(&url)
        .header("Authorization", format!("Bearer {woodpecker_token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| FleetError::WoodpeckerUnavailable(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let detail = resp.text().await.unwrap_or_default();
        return Err(FleetError::WoodpeckerUnavailable(format!(
            "HTTP {status}: {detail}"
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

/// Import an external repo into the orchard9 Gitea org via the migrate API.
pub async fn import_repo(
    http_client: &reqwest::Client,
    gitea_url: &str,
    gitea_token: &str,
    clone_url: &str,
    repo_name: &str,
    auth_token: Option<&str>,
) -> Result<GiteaRepo, FleetError> {
    let url = format!("{gitea_url}/api/v1/repos/migrate");

    let mut body = serde_json::json!({
        "clone_addr": clone_url,
        "repo_name": repo_name,
        "repo_owner": "orchard9",
        "service": "git",
        "mirror": false,
    });

    if let Some(token) = auth_token {
        body["auth_token"] = serde_json::Value::String(token.to_string());
    }

    let resp = http_client
        .post(&url)
        .header("Authorization", format!("token {gitea_token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let detail = resp.text().await.unwrap_or_default();
        return Err(FleetError::GiteaUnavailable(format!(
            "import failed: HTTP {status}: {detail}"
        )));
    }

    let api_repo: GiteaApiRepo = resp
        .json()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    Ok(GiteaRepo {
        slug: api_repo.name,
        full_name: api_repo.full_name,
        description: api_repo.description,
        clone_url: api_repo
            .clone_url
            .unwrap_or_else(|| api_repo.html_url.unwrap_or_default()),
        created_at: api_repo.created_at,
        archived: api_repo.archived,
    })
}

// ---------------------------------------------------------------------------
// Repo creation
// ---------------------------------------------------------------------------

/// Create a new empty repo in the orchard9 Gitea org.
///
/// Returns `FleetError::RepoAlreadyExists` if the name is taken (HTTP 409).
pub async fn create_gitea_repo(
    http_client: &reqwest::Client,
    gitea_url: &str,
    gitea_token: &str,
    name: &str,
) -> Result<GiteaRepo, FleetError> {
    let url = format!("{gitea_url}/api/v1/orgs/orchard9/repos");

    let body = serde_json::json!({
        "name": name,
        "private": false,
        "auto_init": false,
        "description": "",
    });

    let resp = http_client
        .post(&url)
        .header("Authorization", format!("token {gitea_token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    let status = resp.status();
    if status.as_u16() == 409 {
        return Err(FleetError::RepoAlreadyExists(name.to_string()));
    }
    if !status.is_success() {
        let detail = resp.text().await.unwrap_or_default();
        return Err(FleetError::GiteaUnavailable(format!(
            "create repo failed: HTTP {status}: {detail}"
        )));
    }

    let api_repo: GiteaApiRepo = resp
        .json()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    Ok(GiteaRepo {
        slug: api_repo.name,
        full_name: api_repo.full_name,
        description: api_repo.description,
        clone_url: api_repo
            .clone_url
            .unwrap_or_else(|| api_repo.html_url.unwrap_or_default()),
        created_at: api_repo.created_at,
        archived: api_repo.archived,
    })
}

/// Retrieve the login name of the authenticated Gitea user (admin token owner).
pub async fn get_gitea_username(
    http_client: &reqwest::Client,
    gitea_url: &str,
    gitea_token: &str,
) -> Result<String, FleetError> {
    let url = format!("{gitea_url}/api/v1/user");

    let resp = http_client
        .get(&url)
        .header("Authorization", format!("token {gitea_token}"))
        .send()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(FleetError::GiteaUnavailable(format!(
            "get user failed: HTTP {status}"
        )));
    }

    #[derive(serde::Deserialize)]
    struct UserResp {
        login: String,
    }

    let user: UserResp = resp
        .json()
        .await
        .map_err(|e| FleetError::GiteaUnavailable(e.to_string()))?;

    Ok(user.login)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum FleetError {
    #[error("k8s API unavailable: {0}")]
    K8sUnavailable(String),

    #[error("Gitea API unavailable: {0}")]
    GiteaUnavailable(String),

    #[error("Woodpecker API unavailable: {0}")]
    WoodpeckerUnavailable(String),

    #[error("repo not found: {0}")]
    RepoNotFound(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("repo already exists: {0}")]
    RepoAlreadyExists(String),
}

impl FleetError {
    /// Map to an HTTP status code.
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::K8sUnavailable(_) => StatusCode::BAD_GATEWAY,
            Self::GiteaUnavailable(_) => StatusCode::BAD_GATEWAY,
            Self::WoodpeckerUnavailable(_) => StatusCode::BAD_GATEWAY,
            Self::RepoNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::RepoAlreadyExists(_) => StatusCode::CONFLICT,
        }
    }

    /// Error code for JSON response.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::K8sUnavailable(_) => "k8s_unavailable",
            Self::GiteaUnavailable(_) => "gitea_unavailable",
            Self::WoodpeckerUnavailable(_) => "woodpecker_unavailable",
            Self::RepoNotFound(_) => "repo_not_found",
            Self::InvalidRequest(_) => "invalid_request",
            Self::RepoAlreadyExists(_) => "repo_exists",
        }
    }

    /// Convert to an axum response.
    pub fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;
        let status = self.status_code();
        let body = serde_json::json!({
            "error": self.error_code(),
            "detail": self.to_string(),
        });
        (status, axum::Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_namespaces_are_filtered() {
        assert!(is_excluded_namespace("sdlc-tls"));
        assert!(is_excluded_namespace("sdlc-hub"));
        assert!(is_excluded_namespace("sdlc-system"));
        assert!(!is_excluded_namespace("sdlc-payments"));
        assert!(!is_excluded_namespace("sdlc-sdlc"));
        assert!(!is_excluded_namespace("default"));
    }

    #[test]
    fn list_available_repos_diffs_correctly() {
        let instances = vec![
            FleetInstance {
                slug: "sdlc".into(),
                namespace: "sdlc-sdlc".into(),
                url: "https://sdlc.sdlc.threesix.ai".into(),
                deployment_status: DeploymentStatus::Running,
                pod_healthy: true,
                created_at: None,
                active_milestone: None,
                feature_count: None,
                agent_running: None,
                last_heartbeat_at: None,
                heartbeat_status: None,
                provision_status: None,
                attention_reasons: Vec::new(),
            },
            FleetInstance {
                slug: "payments".into(),
                namespace: "sdlc-payments".into(),
                url: "https://payments.sdlc.threesix.ai".into(),
                deployment_status: DeploymentStatus::Running,
                pod_healthy: true,
                created_at: None,
                active_milestone: None,
                feature_count: None,
                agent_running: None,
                last_heartbeat_at: None,
                heartbeat_status: None,
                provision_status: None,
                attention_reasons: Vec::new(),
            },
        ];

        let repos = vec![
            GiteaRepo {
                slug: "sdlc".into(),
                full_name: "orchard9/sdlc".into(),
                description: None,
                clone_url: "https://git.threesix.ai/orchard9/sdlc.git".into(),
                created_at: None,
                archived: false,
            },
            GiteaRepo {
                slug: "payments".into(),
                full_name: "orchard9/payments".into(),
                description: None,
                clone_url: "https://git.threesix.ai/orchard9/payments.git".into(),
                created_at: None,
                archived: false,
            },
            GiteaRepo {
                slug: "new-project".into(),
                full_name: "orchard9/new-project".into(),
                description: Some("A new project".into()),
                clone_url: "https://git.threesix.ai/orchard9/new-project.git".into(),
                created_at: None,
                archived: false,
            },
            GiteaRepo {
                slug: "old-archive".into(),
                full_name: "orchard9/old-archive".into(),
                description: None,
                clone_url: "https://git.threesix.ai/orchard9/old-archive.git".into(),
                created_at: None,
                archived: true,
            },
            GiteaRepo {
                slug: "another-repo".into(),
                full_name: "orchard9/another-repo".into(),
                description: None,
                clone_url: "https://git.threesix.ai/orchard9/another-repo.git".into(),
                created_at: None,
                archived: false,
            },
        ];

        let available = list_available_repos(&instances, &repos);
        assert_eq!(available.len(), 3);

        // Check that instance slugs are excluded
        let available_slugs: Vec<&str> = available.iter().map(|a| a.repo.slug.as_str()).collect();
        assert!(available_slugs.contains(&"new-project"));
        assert!(available_slugs.contains(&"old-archive"));
        assert!(available_slugs.contains(&"another-repo"));
        assert!(!available_slugs.contains(&"sdlc"));
        assert!(!available_slugs.contains(&"payments"));

        // Archived repo should have can_provision = false
        let archived = available
            .iter()
            .find(|a| a.repo.slug == "old-archive")
            .unwrap();
        assert!(!archived.can_provision);

        // Active repo should have can_provision = true
        let active = available
            .iter()
            .find(|a| a.repo.slug == "new-project")
            .unwrap();
        assert!(active.can_provision);
    }

    #[test]
    fn list_available_repos_empty_instances() {
        let repos = vec![GiteaRepo {
            slug: "solo".into(),
            full_name: "orchard9/solo".into(),
            description: None,
            clone_url: "https://git.threesix.ai/orchard9/solo.git".into(),
            created_at: None,
            archived: false,
        }];

        let available = list_available_repos(&[], &repos);
        assert_eq!(available.len(), 1);
        assert!(available[0].can_provision);
    }

    #[test]
    fn fleet_error_status_codes() {
        assert_eq!(
            FleetError::K8sUnavailable("test".into()).status_code(),
            axum::http::StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            FleetError::RepoNotFound("test".into()).status_code(),
            axum::http::StatusCode::NOT_FOUND
        );
        assert_eq!(
            FleetError::InvalidRequest("test".into()).status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
        assert_eq!(
            FleetError::RepoAlreadyExists("test".into()).status_code(),
            axum::http::StatusCode::CONFLICT
        );
        assert_eq!(
            FleetError::RepoAlreadyExists("test".into()).error_code(),
            "repo_exists"
        );
    }

    #[tokio::test]
    async fn create_gitea_repo_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/orgs/orchard9/repos")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "name": "my-project",
                "full_name": "orchard9/my-project",
                "description": null,
                "clone_url": "http://gitea/orchard9/my-project.git",
                "html_url": "http://gitea/orchard9/my-project",
                "created_at": null,
                "archived": false
            }"#,
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let result = create_gitea_repo(&client, &server.url(), "token123", "my-project").await;

        mock.assert_async().await;
        let repo = result.expect("should succeed");
        assert_eq!(repo.slug, "my-project");
        assert_eq!(repo.full_name, "orchard9/my-project");
    }

    #[tokio::test]
    async fn create_gitea_repo_conflict() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/orgs/orchard9/repos")
            .with_status(409)
            .with_body(r#"{"message":"repo already exists"}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let result = create_gitea_repo(&client, &server.url(), "token123", "existing-repo").await;

        mock.assert_async().await;
        assert!(matches!(result, Err(FleetError::RepoAlreadyExists(_))));
    }

    #[tokio::test]
    async fn get_gitea_username_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/user")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"login":"claude-agent","id":1}"#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let result = get_gitea_username(&client, &server.url(), "token123").await;

        mock.assert_async().await;
        assert_eq!(result.expect("should succeed"), "claude-agent");
    }

    #[tokio::test]
    async fn get_gitea_username_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/user")
            .with_status(401)
            .with_body("unauthorized")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let result = get_gitea_username(&client, &server.url(), "bad-token").await;

        mock.assert_async().await;
        assert!(matches!(result, Err(FleetError::GiteaUnavailable(_))));
    }
}
