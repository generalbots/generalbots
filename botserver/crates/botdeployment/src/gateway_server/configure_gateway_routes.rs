use super::*;

pub fn configure_gateway_routes(state: Arc<GatewayState>) -> Router {
    let registry = Arc::new(ContainerRegistry::default());
    // IMP-07 — this router is served by its own listener, so it must carry the
    // limiter itself (see `rate_limit.rs`). The extension is applied last so it
    // is outermost and therefore visible to the middleware.
    Router::new()
        .route("/deploy", post(gateway_deploy))
        .route("/deploy/stop", post(gateway_stop))
        .route("/deploy/start", post(gateway_start))
        .route("/deploy/status/:org/:app_name", get(gateway_status))
        .route("/deploy/keys", post(gateway_rotate_key))
        .layer(axum::middleware::from_fn(
            botsecurity_auth::rate_limiter::rate_limit_middleware,
        ))
        .layer(gateway_rate_limit_extension())
        .with_state((state, registry))
}
pub(super) fn validate_deploy_key(headers: &HeaderMap, state: &GatewayState) -> Result<(), StatusCode> {
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
/// Only `file://` artifacts inside the configured sites root may be pushed
/// into a container (#2613). Canonicalisation defeats `..`/symlink escapes.
pub(super) fn allowed_local_artifact(sites_root: &str, artifact_url: &str) -> Option<std::path::PathBuf> {
    let raw = artifact_url.strip_prefix("file://")?;
    if raw.is_empty() || raw.contains('\0') || raw.contains("..") {
        return None;
    }
    let root = std::path::Path::new(sites_root).canonicalize().ok()?;
    let candidate = std::path::Path::new(raw).canonicalize().ok()?;
    if candidate.starts_with(&root) && candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

/// #1297 — org/app identifiers flow from HTTP bodies straight into
/// `incus` argv. Restrict them to a safe charset so no shell/control
/// characters or option-looking tokens can reach the command line.
pub(super) fn sanitize_identifier_part(part: &str) -> Result<String, StatusCode> {
    let cleaned: String = part
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if cleaned.is_empty() || cleaned.starts_with('-') {
        log::warn!("Rejected deploy identifier: empty or option-like");
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(cleaned)
}
pub(super) fn container_name_from_parts(org: &str, app_name: &str) -> Result<String, StatusCode> {
    let org = sanitize_identifier_part(org)?;
    let app_name = sanitize_identifier_part(app_name)?;
    Ok(format!("{org}-{app_name}"))
}
/// Strict validation for a per-deployment PostgreSQL URL (#2652). The value
/// is later base64-encoded into a root shell script, so no metacharacter or
/// heredoc terminator may survive this boundary.
pub(super) fn validate_database_url(url: Option<&str>) -> Result<(), StatusCode> {
    let Some(url) = url else {
        return Ok(());
    };
    if url.len() > 512
        || (!url.starts_with("postgres://") && !url.starts_with("postgresql://"))
        || url.contains("UNIT")
        || url.chars().any(|c| {
            !(c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '@' | '?' | '=' | '.' | '_' | '+' | '~' | '%' | '-'))
        })
    {
        log::warn!("Rejected invalid database_url in deploy request");
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

#[cfg(test)]
mod database_url_tests {
    use super::validate_database_url;

    #[test]
    fn rejects_heredoc_and_shell_injection() {
        assert!(validate_database_url(Some("postgresql://u:p@db.example:5432/app")).is_ok());
        assert!(validate_database_url(None).is_ok());
        assert!(validate_database_url(Some("postgresql://u:p@db:5432/app\nUNIT\nid")).is_err());
        assert!(validate_database_url(Some("postgresql://u:p@db:5432/app$HOME")).is_err());
        assert!(validate_database_url(Some("postgresql://u:p@db:5432/app;id")).is_err());
        assert!(validate_database_url(Some("mysql://db/app")).is_err());
    }
}

pub async fn gateway_deploy(
    State((state, registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
    Json(request): Json<DeployGatewayRequest>,
) -> Result<Json<DeployGatewayResponse>, StatusCode> {
    validate_deploy_key(&headers, &state)?;
    validate_database_url(request.database_url.as_deref())?;

    log::info!(
        "Gateway deploy: {}/{} type={:?} env={}",
        request.org,
        request.app_name,
        request.project_type,
        request.environment
    );

    let container_name = container_name_from_parts(&request.org, &request.app_name)?;
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
    let container_name = container_name_from_parts(org, app_name)?;

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
    let container_name = container_name_from_parts(org, app_name)?;

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

    let container_name = container_name_from_parts(&org, &app_name)?;

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
/// Key rotation is not implemented. The old handler returned `success: true`
/// while leaving `state.deploy_keys` untouched (#2613); report the truth.
pub async fn gateway_rotate_key(
    State((state, _registry)): State<(Arc<GatewayState>, Arc<ContainerRegistry>)>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    validate_deploy_key(&headers, &state)?;
    log::warn!("Deploy key rotation rejected: not implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}
pub(super) async fn deploy_to_incus(
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
        let artifact_path = allowed_local_artifact(&state.sites_root, &request.artifact_url)
            .ok_or_else(|| "artifact_path must resolve inside the configured sites root".to_string())?;
        let _ = run_incus_command(state, &[
            "file", "push",
            artifact_path.to_string_lossy().as_ref(),
            &format!("{container_name}/opt/app/"),
            "--recursive",
        ]).await;
    }

    run_incus_command(state, &["start", container_name]).await
        .map_err(|e| format!("Failed to start container: {e}"))?;

    // Auto-manage the app runtime inside the container: detect node/python/
    // rust/static, install the runtime when missing, and run the app as a
    // systemd service bound to port 80 (auto-restart + start on boot).
    // #1386 — the per-environment database URL rides on the deploy request
    // and lands in the unit as DATABASE_URL.
    crate::gateway_runtime::bootstrap_app_runtime(
        state,
        container_name,
        request.database_url.as_deref(),
    )
    .await?;

    let domain = published_domain();
    let env_suffix = match request.environment {
        DeploymentEnvironment::Production => String::new(),
        DeploymentEnvironment::Staging => "-staging".to_string(),
        DeploymentEnvironment::Development => "-dev".to_string(),
    };
    let app_name = request.app_name.clone();
    Ok(format!("https://{app_name}{env_suffix}.{domain}/"))
}

#[cfg(test)]
mod artifact_boundary_tests {
    use super::allowed_local_artifact;
    use std::fs;

    #[test]
    fn only_files_inside_sites_root_are_allowed() {
        let root = std::env::temp_dir().join(format!("gb-gateway-artifact-{}", std::process::id()));
        let nested = root.join("sites");
        fs::create_dir_all(&nested).expect("create sites dir");
        let inside = nested.join("app.zip");
        fs::write(&inside, b"payload").expect("write artifact");
        let outside = std::env::temp_dir().join(format!("gb-gateway-outside-{}", std::process::id()));
        fs::write(&outside, b"secret").expect("write outside artifact");

        let root_text = root.to_string_lossy().to_string();
        let allowed = allowed_local_artifact(&root_text, &format!("file://{}", inside.display()));
        assert_eq!(allowed.as_deref(), Some(inside.as_path()));
        assert!(allowed_local_artifact(&root_text, &format!("file://{}", outside.display())).is_none());
        assert!(allowed_local_artifact(&root_text, "file:///etc/passwd").is_none());
        assert!(allowed_local_artifact(&root_text, &format!("file://{}/../outside", nested.display())).is_none());
        // A directory is not a pushable artifact.
        assert!(allowed_local_artifact(&root_text, &format!("file://{}", nested.display())).is_none());
        // A symlink pointing outside the root must resolve to its target and be refused.
        let link = nested.join("escape.link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("create symlink");
        assert!(allowed_local_artifact(&root_text, &format!("file://{}", link.display())).is_none());

        fs::remove_dir_all(&root).ok();
        fs::remove_file(outside).ok();
    }
}
