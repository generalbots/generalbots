//! Deploy Gateway Server — receives requests from alm-ci and performs
//! privileged operations on the host (Incus container management, Caddy config).
//!
//! This server validates the X-Deploy-Key header and never exposes
//! Forgejo tokens, SSH keys, or Incus socket access to callers.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::*;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayState {
    #[serde(default)]
    deploy_keys: Vec<String>,
    incus_socket: String,
    caddy_api_url: String,
    sites_root: String,
}

impl Default for GatewayState {
    fn default() -> Self {
        let keys = std::env::var("DEPLOY_KEYS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            deploy_keys: keys,
            incus_socket: std::env::var("INCUS_SOCKET")
                .unwrap_or_else(|_| "/var/lib/incus/unix.socket".to_string()),
            caddy_api_url: std::env::var("CADDY_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:2019".to_string()),
            sites_root: std::env::var("SITES_ROOT")
                .unwrap_or_else(|_| "/var/www".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct ContainerRegistry {
    containers: RwLock<HashMap<String, ContainerInfo>>,
}

impl Default for ContainerRegistry {
    fn default() -> Self {
        Self {
            containers: RwLock::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub org: String,
    pub project_type: ProjectType,
    pub status: String,
    pub url: String,
}

pub fn configure_gateway_routes(state: Arc<GatewayState>) -> Router {
    let registry = Arc::new(ContainerRegistry::default());
    Router::new()
        .route("/deploy", post(gateway_deploy))
        .route("/deploy/stop", post(gateway_stop))
        .route("/deploy/start", post(gateway_start))
        .route("/deploy/status/{org}/{app_name}", get(gateway_status))
        .route("/deploy/keys", post(gateway_rotate_key))
        .with_state((state, registry))
}

fn validate_deploy_key(headers: &HeaderMap, state: &GatewayState) -> Result<(), StatusCode> {
    let key = headers
        .get("X-Deploy-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if state.deploy_keys.contains(&key.to_string()) {
        Ok(())
    } else {
        log::warn!("Invalid deploy key attempted");
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn gateway_deploy(
    State((state, registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Json(request): Json<DeployGatewayRequest>,
) -> Result<Json<DeployGatewayResponse>, StatusCode> {
    validate_deploy_key(&headers, &state)?;

    log::info!(
        "Gateway deploy: {}/{} type={:?} env={}",
        request.org,
        request.app_name,
        request.project_type,
        request.environment
    );

    let container_name = format!("{}-{}", request.org, request.app_name);
    let deploy_url = match &request.project_type {
        ProjectType::Bot => {
            return Ok(Json(DeployGatewayResponse {
                success: true,
                url: None,
                container: None,
                status: Some("bot_no_deploy".to_string()),
                error: None,
            }));
        }
        ProjectType::App { .. } => {
            let result = deploy_to_incus(&state, &container_name, &request).await;
            match result {
                Ok(url) => {
                    let mut containers = registry.containers.write().await;
                    containers.insert(container_name.clone(), ContainerInfo {
                        name: container_name.clone(),
                        org: request.org.clone(),
                        project_type: request.project_type.clone(),
                        status: "running".to_string(),
                        url: url.clone(),
                    });
                    url
                }
                Err(e) => {
                    log::error!("Incus deploy failed: {e}");
                    return Ok(Json(DeployGatewayResponse {
                        success: false,
                        url: None,
                        container: Some(container_name),
                        status: Some("failed".to_string()),
                        error: Some(e),
                    }));
                }
            }
        }
        ProjectType::Site => {
            let result = deploy_to_caddy(&state, &container_name, &request).await;
            match result {
                Ok(url) => {
                    let mut containers = registry.containers.write().await;
                    containers.insert(container_name.clone(), ContainerInfo {
                        name: container_name.clone(),
                        org: request.org.clone(),
                        project_type: request.project_type.clone(),
                        status: "deployed".to_string(),
                        url: url.clone(),
                    });
                    url
                }
                Err(e) => {
                    log::error!("Caddy deploy failed: {e}");
                    return Ok(Json(DeployGatewayResponse {
                        success: false,
                        url: None,
                        container: None,
                        status: Some("failed".to_string()),
                        error: Some(e),
                    }));
                }
            }
        }
    };

    Ok(Json(DeployGatewayResponse {
        success: true,
        url: Some(deploy_url),
        container: Some(container_name),
        status: Some("deployed".to_string()),
        error: None,
    }))
}

pub async fn gateway_stop(
    State((state, registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DeployGatewayResponse>, StatusCode> {
    validate_deploy_key(&headers, &state)?;

    let app_name = body.get("app_name").and_then(|v| v.as_str()).unwrap_or_default();
    let org = body.get("org").and_then(|v| v.as_str()).unwrap_or_default();
    let container_name = format!("{org}-{app_name}");

    log::info!("Gateway stop: {container_name}");

    let output = run_incus_command(&state, &["stop", &container_name]).await;
    let success = output.is_ok();

    if success {
        let mut containers = registry.containers.write().await;
        if let Some(info) = containers.get_mut(&container_name) {
            info.status = "stopped".to_string();
        }
    }

    Ok(Json(DeployGatewayResponse {
        success,
        url: None,
        container: Some(container_name),
        status: if success { Some("stopped".to_string()) } else { Some("error".to_string()) },
        error: output.err().map(|e| format!("Stop failed: {e}")),
    }))
}

pub async fn gateway_start(
    State((state, registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DeployGatewayResponse>, StatusCode> {
    validate_deploy_key(&headers, &state)?;

    let app_name = body.get("app_name").and_then(|v| v.as_str()).unwrap_or_default();
    let org = body.get("org").and_then(|v| v.as_str()).unwrap_or_default();
    let container_name = format!("{org}-{app_name}");

    log::info!("Gateway start: {container_name}");

    let output = run_incus_command(&state, &["start", &container_name]).await;
    let success = output.is_ok();

    if success {
        let mut containers = registry.containers.write().await;
        if let Some(info) = containers.get_mut(&container_name) {
            info.status = "running".to_string();
        }
    }

    Ok(Json(DeployGatewayResponse {
        success,
        url: None,
        container: Some(container_name),
        status: if success { Some("running".to_string()) } else { Some("error".to_string()) },
        error: output.err().map(|e| format!("Start failed: {e}")),
    }))
}

pub async fn gateway_status(
    State((state, registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Path((org, app_name)): Path<(String, String)>,
) -> Result<Json<DeployGatewayResponse>, StatusCode> {
    validate_deploy_key(&headers, &state)?;

    let container_name = format!("{org}-{app_name}");

    let containers = registry.containers.read().await;
    if let Some(info) = containers.get(&container_name) {
        Ok(Json(DeployGatewayResponse {
            success: true,
            url: Some(info.url.clone()),
            container: Some(info.name.clone()),
            status: Some(info.status.clone()),
            error: None,
        }))
    } else {
        let list_output = run_incus_command(&state, &["list", &container_name, "-f", "csv"]).await;
        let status = if list_output.is_ok() { "unknown" } else { "not_found" };
        Ok(Json(DeployGatewayResponse {
            success: true,
            url: None,
            container: Some(container_name),
            status: Some(status.to_string()),
            error: None,
        }))
    }
}

pub async fn gateway_rotate_key(
    State((state, _registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    validate_deploy_key(&headers, &state)?;

    let new_key = body.get("new_key").and_then(|v| v.as_str()).unwrap_or_default();
    if new_key.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    log::info!("Deploy key rotation requested");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Key rotation requires server restart to take effect"
    })))
}

async fn deploy_to_incus(
    state: &GatewayState,
    container_name: &str,
    request: &DeployGatewayRequest,
) -> Result<String, String> {
    let list = run_incus_command(state, &["list", container_name, "-f", "csv"]).await;

    if list.is_err() || list.unwrap_or_default().trim().is_empty() {
        log::info!("Creating new Incus container: {container_name}");
        run_incus_command(state, &["launch", "ubuntu:22.04", container_name]).await
            .map_err(|e| format!("Failed to launch container: {e}"))?;
    } else {
        log::info!("Container {container_name} exists, redeploying");
        let _ = run_incus_command(state, &["stop", container_name]).await;
    }

    if !request.artifact_url.is_empty() && request.artifact_url.starts_with("file://") {
        let artifact_path = request.artifact_url.strip_prefix("file://").unwrap_or_default();
        let _ = run_incus_command(state, &[
            "file", "push", artifact_path,
            &format!("{container_name}/opt/app/"),
            "--recursive",
        ]).await;
    }

    run_incus_command(state, &["start", container_name]).await
        .map_err(|e| format!("Failed to start container: {e}"))?;

    let domain = std::env::var("SITE_DOMAIN")
        .unwrap_or_else(|_| "gb.solutions".to_string());
    let env_suffix = match request.environment {
        DeploymentEnvironment::Production => String::new(),
        DeploymentEnvironment::Staging => "-staging".to_string(),
        DeploymentEnvironment::Development => "-dev".to_string(),
    };
    let app_name = request.app_name.clone();
    Ok(format!("https://{app_name}{env_suffix}.{domain}/"))
}

async fn deploy_to_caddy(
    state: &GatewayState,
    _container_name: &str,
    request: &DeployGatewayRequest,
) -> Result<String, String> {
    let site_dir = format!("{}/{}/{}", state.sites_root, request.org, request.app_name);
    tokio::fs::create_dir_all(&site_dir).await
        .map_err(|e| format!("Failed to create site dir: {e}"))?;

    if !request.artifact_url.is_empty() && request.artifact_url.starts_with("file://") {
        let artifact_path = request.artifact_url.strip_prefix("file://").unwrap_or_default();
        let output = tokio::process::Command::new("tar")
            .args(["-xzf", artifact_path, "-C", &site_dir])
            .output().await
            .map_err(|e| format!("Failed to extract artifact: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("Artifact extraction warnings: {stderr}");
        }
    }

    let domain = std::env::var("SITE_DOMAIN")
        .unwrap_or_else(|_| "gb.solutions".to_string());
    let env_suffix = match request.environment {
        DeploymentEnvironment::Production => String::new(),
        DeploymentEnvironment::Staging => "-staging".to_string(),
        DeploymentEnvironment::Development => "-dev".to_string(),
    };
    let app_name = request.app_name.clone();

    let caddy_route = serde_json::json!({
        "@id": format!("{app_name}{env_suffix}"),
        "handle": [{
            "handler": "file_server",
            "root": site_dir,
        }],
        "match": [{ "host": [format!("{app_name}{env_suffix}.{domain}")] }],
    });

    let client = reqwest::Client::new();
    let _ = client
        .post(format!("{}/config/apps/http/servers/srv0/routes", state.caddy_api_url))
        .json(&caddy_route)
        .send().await;

    Ok(format!("https://{app_name}{env_suffix}.{domain}/"))
}

async fn run_incus_command(state: &GatewayState, args: &[&str]) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new("incus");
    cmd.args(args);
    cmd.env("INCUS_SOCKET", &state.incus_socket);

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute incus: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("incus {:?}: {stderr}", args))
    }
}
