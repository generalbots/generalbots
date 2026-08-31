//! #757 — `publish_project` Vibe tool.
//!
//! Puts a project onto a live environment: ensures the target env VM
//! (#744), calls the deployment API (`/api/deployment/deploy`) so an Incus
//! container (or Caddy route) is raised, optionally binds a custom domain,
//! and records the deployment in the project payload (deployment history —
//! #772 reads the same records).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Extension, Path as AxumPath};
use axum::http::{header, StatusCode};
use axum::routing::get;
use axum::Router;
use diesel::RunQueryDsl;
use serde_json::Value;
use uuid::Uuid;

use crate::types::DbPool;

use crate::domains::{BindDomainRequest, ProjectDomains};
use crate::harness;
use crate::projects::{Project, ProjectRegistry};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult, VibeUseCase};
use crate::vm_lifecycle::{CreateVmRequest, VmLifecycle};

/// Published-app site domain: `{appname}.{published_domain()}` (#1261).
/// Honors `GB_PLATFORM_DOMAIN` (e.g. `generalbots.org`) so published vibe
/// apps sit on the same wildcard zone as bots; falls back to `SITE_DOMAIN`
/// then `gb.solutions` for legacy self-hosted deployments.
pub fn published_domain() -> String {
    std::env::var("GB_PLATFORM_DOMAIN")
        .ok()
        .filter(|d| !d.trim().is_empty())
        .or_else(|| std::env::var("SITE_DOMAIN").ok().filter(|d| !d.trim().is_empty()))
        .unwrap_or_else(|| "gb.solutions".to_string())
}

/// #1180 — public deliverable URLs: `https://host/r/{slug}` 302-redirects to
/// the project's published route (`{slug}.{domain}` or a bound custom
/// domain), so Vibe artifacts are shareable without any auth.
pub fn publish_router(pool: DbPool) -> Router {
    Router::new()
        .route("/r/:slug", get(resolve_slug).layer(Extension(pool)))
}

#[derive(diesel::QueryableByName)]
struct SlugRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    domain: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    access: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    verified: bool,
}

async fn resolve_slug(
    Extension(pool): Extension<DbPool>,
    AxumPath(slug): AxumPath<String>,
) -> axum::response::Response {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Publish /r/ resolve: db pool {e}");
            return response_with_headers(
                StatusCode::INTERNAL_SERVER_ERROR,
                &[],
                "internal error",
            );
        }
    };
    let candidates = [
        format!("{slug}.{}", published_domain()),
        slug.clone(),
    ];
    let mut found: Option<(String, String)> = None;
    for domain in candidates {
        let row = diesel::sql_query(
            "SELECT domain, access, verified FROM project_domains WHERE domain = $1 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&domain)
        .get_result::<SlugRow>(&mut conn);
        match row {
            Ok(r) if r.access != "private" && r.verified => {
                found = Some((r.domain, r.access));
                break;
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    match found {
        Some((domain, _access)) => {
            let url = format!("https://{domain}");
            response_with_headers(
                StatusCode::FOUND,
                &[
                    (header::LOCATION, url),
                    (header::CACHE_CONTROL, "no-store".to_string()),
                ],
                "redirecting",
            )
        }
        None => response_with_headers(StatusCode::NOT_FOUND, &[], "artifact not found"),
    }
}

/// Builds a response with headers, keeping the error path panic-free:
/// builder failures (invalid header name/value) are logged and fall back to
/// a bare 500 response.
fn response_with_headers(
    status: StatusCode,
    headers: &[(header::HeaderName, String)],
    body: &str,
) -> axum::response::Response {
    let mut builder = axum::response::Response::builder().status(status);
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    match builder.body(axum::body::Body::from(body.to_string())) {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Publish response build failed: {e}");
            axum::response::Response::new(axum::body::Body::from("internal error"))
        }
    }
}

const PUBLISH_DEFAULT_ENV: &str = "production";

/// Env var overriding the maximum total bytes the publish path will read
/// into memory for a single project archive (#934).
const PUBLISH_MAX_BYTES_ENV: &str = "VIBE_PUBLISH_MAX_BYTES";
const PUBLISH_DEFAULT_MAX_BYTES: u64 = 20 * 1024 * 1024;

fn publish_max_bytes() -> u64 {
    std::env::var(PUBLISH_MAX_BYTES_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(PUBLISH_DEFAULT_MAX_BYTES)
}

pub fn publish_project_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        let args = args.clone();
        Box::pin(publish_project(args, pool))
    })
}

pub fn publish_project_schema() -> ToolSchema {
    ToolSchema::new("publish/project", "Publish a project to an environment: ensures the env VM (Incus container), deploys via the deployment API, optionally binds a custom domain, and records the deployment for history/rollback.")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "project_id": { "type": "string", "description": "UUID of the project to publish" },
                "env": { "type": "string", "enum": ["development", "staging", "production"], "default": "production" },
                "domain": { "type": "string", "description": "Optional custom domain to bind" },
                "launcher": { "type": "boolean", "description": "When true, the published app auto-pins to the desktop launcher (desktop category) for workspace users (#1160)." },
                "widget": { "type": "boolean", "description": "When true, the published app is registered as a desktop widget (always-visible tile) instead of a windowed app (#1160)." }
            },
            "required": ["project_id"]
        }))
        .with_approval()
        .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

#[cfg(not(target_os = "windows"))]
fn api_base() -> String {
    let (_, vibe) = botcoresecrets::app_runtime();
    if vibe.is_empty() {
        std::env::var("VIBE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
    } else {
        vibe
    }
}

/// Collect the project's source files from its workspace (the agent's actual
/// output) so the deployment API can push them to the ALM repo instead of an
/// empty app. The workspace dir is keyed by the ALM repo slug, falling back
/// to the raw project id.
pub(crate) fn collect_workspace_files(project: &Project) -> Result<Vec<Value>, String> {
    let candidates = [VmLifecycle::alm_repo(&project.name), project.id.to_string()];
    for key in candidates {
        let dir = harness::workspace_root().join(&key);
        if !dir.is_dir() {
            continue;
        }
        let mut out = Vec::new();
        let mut total_bytes = 0u64;
        match walk_workspace(&dir, &dir, &mut out, &mut total_bytes) {
            Ok(()) => {
                if !out.is_empty() {
                    log::info!(
                        "Vibe publish: packaged {} files ({total_bytes} bytes) from workspace '{key}'",
                        out.len()
                    );
                    return Ok(out);
                }
            }
            // A size-budget violation is an actionable error, not a reason to
            // fall through to an empty archive (#934).
            Err(e) => return Err(e),
        }
    }
    log::warn!(
        "Vibe publish: no workspace files found for project '{}' — deploying empty repo",
        project.name
    );
    Ok(Vec::new())
}

fn walk_workspace(
    dir: &Path,
    root: &Path,
    out: &mut Vec<Value>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {dir:?}: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Only source ships — skip VCS and heavy build/artifact directories.
        if matches!(
            name.as_str(),
            ".git" | ".forgejo" | "node_modules" | "target" | "dist" | ".next" | "build"
        ) {
            continue;
        }
        if path.is_dir() {
            walk_workspace(&path, root, out, total_bytes)?;
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("strip prefix: {e}"))?;
        let rel = rel.to_string_lossy().replace('\\', "/");
        match std::fs::read(&path) {
            Ok(bytes) => {
                *total_bytes += bytes.len() as u64;
                if *total_bytes > publish_max_bytes() {
                    return Err(format!(
                        "workspace exceeds publish size budget ({} bytes)",
                        publish_max_bytes()
                    ));
                }
                out.push(serde_json::json!({ "path": rel, "content": bytes }));
            }
            Err(e) => log::warn!("Vibe publish: skip unreadable file {rel}: {e}"),
        }
    }
    Ok(())
}

async fn publish_project(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_publish(args, pool).await {
        Ok(data) => VibeToolResult {
            success: true,
            data,
            error: None,
            latency_ms: started.elapsed().as_millis() as u64,
        },
        Err(e) => VibeToolResult {
            success: false,
            data: Value::Null,
            error: Some(e),
            latency_ms: started.elapsed().as_millis() as u64,
        },
    }
}

pub(crate) async fn do_publish(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "publish/project requires 'project_id' (uuid string)".to_string())?;
    let env = args
        .get("env")
        .and_then(|v| v.as_str())
        .unwrap_or(PUBLISH_DEFAULT_ENV)
        .to_lowercase();
    if !crate::vm_lifecycle::VALID_ENVS.contains(&env.as_str()) {
        return Err(format!("invalid env '{env}'"));
    }
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    // #1160 — desktop launch surface: `launcher` auto-pins the app to the
    // launcher; `widget` marks the app as an always-visible desktop widget.
    let launcher_requested = args
        .get("launcher")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let widget_requested = args
        .get("widget")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let registry = ProjectRegistry::new(pool.clone());
    let project = registry
        .get(project_id)?
        .ok_or_else(|| format!("project {project_id} not found"))?;

    let metering = crate::metering::VMetering::new(pool.clone());
    metering.enforce_for_project(project_id, crate::metering::MeterKind::BuildMinutes)?;
    let _ = metering.add_for_project(
        project_id,
        &env,
        crate::metering::MeterKind::BuildMinutes,
        1.0,
    );

    #[cfg(not(target_os = "windows"))]
    let repo_name = VmLifecycle::alm_repo(&project.name);
    #[cfg(not(target_os = "windows"))]
    let org = VmLifecycle::alm_org(project.branch_id);

    let vm_req = CreateVmRequest {
        env: env.clone(),
        tier: "small".to_string(),
        runner_enabled: false,
    };
    let vm = VmLifecycle::new(pool.clone()).create_project_vm(
        project_id,
        project.branch_id,
        &project.name,
        &vm_req,
    )?;

    // Self-hosted ALM (Forgejo): Vault `secret/gbo/alm` → env → localhost.
    #[cfg(not(target_os = "windows"))]
    let (alm_base, _, _) = botcoresecrets::alm_config();

    #[cfg(not(target_os = "windows"))]
    let target = match &domain {
        Some(d) => serde_json::json!({
            "External": {
                "repo_url": format!("{}/{}/{}", alm_base.trim_end_matches('/'), org, repo_name),
                "custom_domain": d,
                "ci_cd_enabled": true
            }
        }),
        None => serde_json::json!({
            "Internal": {
                "route": format!("{repo_name}-{env}.{}", published_domain()),
                "shared_resources": true
            }
        }),
    };

    let files = collect_workspace_files(&project)?;
    // The Vibe project registry uses the user-facing kinds `custom` and
    // `website`, while the deployment API accepts `app-*` and `site`.
    // Translate at the boundary so a calculator/custom app is deployable.
    let deployment_type = match project.project_type.as_str() {
        "website" => "site".to_string(),
        "custom" => format!("app-{}", project.framework.as_deref().unwrap_or("node")),
        other if other == "site" || other == "bot" || other.starts_with("app-") => {
            other.to_string()
        }
        _ => "app-node".to_string(),
    };
    #[cfg(target_os = "windows")]
    let deployed: Value = {
        let host_port = std::env::var("VIBE_WSL_APP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(80);
        let url = VmLifecycle::new(pool.clone()).deploy_node_files(
            &vm.container_name,
            &files,
            host_port,
        )?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| format!("local deploy probe client: {e}"))?;
        let health_url = format!("{}/health", url.trim_end_matches('/'));
        let mut healthy = false;
        for _ in 0..20 {
            if client
                .get(&health_url)
                .send()
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false)
            {
                healthy = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        if !healthy {
            return Err(format!(
                "local WSL deployment did not become healthy at {health_url}"
            ));
        }
        serde_json::json!({
            "success": true,
            "url": url,
            "repository": "local-wsl",
            "project_type": deployment_type,
            "deploy_target": "incus-container",
            "status": "Deployed"
        })
    };

    #[cfg(not(target_os = "windows"))]
    let deployed: Value = {
        let body = serde_json::json!({
            "app_name": repo_name,
            "organization": org,
            "project_type": deployment_type,
            "environment": env,
            "target": target,
            "project_id": project_id,
            "files": files,
        });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let mut req = client
            .post(format!("{}/api/deployment/deploy", api_base()))
            .header("Content-Type", "application/json");
        // The deployment API is an internal endpoint guarded by the
        // INTERNAL_API_TOKEN (X-Internal-Token), same as the other internal
        // callers (invoke_action, vibe_agent keywords).
        if let Ok(token) = std::env::var("INTERNAL_API_TOKEN") {
            if !token.is_empty() {
                req = req.header("X-Internal-Token", token);
            }
        }
        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("deployment API unreachable: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("deployment API returned {status}: {text}"));
        }
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }))
    };

    // #1271 — prod must keep the framework running always-on, not just raise
    // the container. Deploying to the prod VM reuses the same app-start as
    // dev Run (run_dev_app): workspace pushed into /opt/vibe/app and the node
    // (or python) service started with Restart=always, so the published URL
    // serves a live process instead of an empty VM. Only meaningful for
    // production (dev Run already goes through run_dev_app directly).
    #[cfg(not(target_os = "windows"))]
    if env == "production" {
        let host_port = std::env::var("VIBE_PROD_APP_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(80);
        match VmLifecycle::new(pool.clone()).run_dev_app(
            &vm.container_name,
            &files,
            host_port,
        ) {
            Ok(_) => log::info!(
                "Vibe publish {}: started app always-on in prod container {}",
                project.name,
                vm.container_name
            ),
            Err(e) => {
                log::warn!(
                    "Vibe publish {}: could not start prod app in {}: {e}",
                    project.name,
                    vm.container_name
                );
            }
        }
    }

    let deployment = serde_json::json!({
        "env": env,
        "at": chrono::Utc::now().to_rfc3339(),
        "url": deployed.get("url").and_then(|v| v.as_str()).unwrap_or(""),
        "container": vm.container_name,
        "domain": domain.clone().unwrap_or_default(),
        "track": "ok",
    });
    registry
        .append_deployment(project_id, &deployment)
        .map_err(|e| format!("record deployment: {e}"))?;

    let launch_info = if launcher_requested || widget_requested {
        let launch = serde_json::json!({
            "enabled": true,
            "kind": if widget_requested { "widget" } else { "app" },
            "at": chrono::Utc::now().to_rfc3339(),
            "env": env,
        });
        registry
            .set_launcher(project_id, &launch)
            .map_err(|e| format!("record launcher flag: {e}"))?;
        Some(launch)
    } else {
        None
    };

    let binding = match &domain {
        Some(d) => {
            let bind_req = BindDomainRequest {
                domain: d.clone(),
                env: env.clone(),
                access: None,
                allowed_emails: None,
            };
            match ProjectDomains::new(pool).bind(project_id, &bind_req).await {
                Ok(b) => {
                    serde_json::json!({ "bound": true, "id": b.id, "domain": b.domain, "env": b.env, "container": b.container, "verified": b.verified, "tls_status": b.tls_status })
                }
                Err(e) => serde_json::json!({ "bound": false, "error": e }),
            }
        }
        None => serde_json::json!({ "bound": false, "error": "no domain provided" }),
    };

    Ok(serde_json::json!({
        "published": true,
        "project": project.name,
        "env": env,
        "container": vm.container_name,
        "domain": domain,
        "domain_bind": binding,
        "deployment": deployed,
        "launcher": launch_info,
        "history_key": "deployments"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_project(name: &str) -> Project {
        let now = chrono::Utc::now();
        Project {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            branch_id: Uuid::nil(),
            name: name.to_string(),
            project_type: "app-htmx".to_string(),
            repository: String::new(),
            framework: None,
            custom_domain: None,
            source_control: "native".to_string(),
            status: "active".to_string(),
            environment: "development".to_string(),
            payload: Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn collect_workspace_files_packages_source_and_skips_vcs() {
        let _guard = harness::WORKSPACE_ENV_LOCK
            .lock()
            .expect("workspace env lock");
        let previous = std::env::var_os("VIBE_WORKSPACE_ROOT");
        let tmp = std::env::temp_dir().join(format!("vibe-publish-test-{}", Uuid::new_v4()));
        std::env::set_var("VIBE_WORKSPACE_ROOT", &tmp);
        let project = test_project("My Web App");
        let slug = VmLifecycle::alm_repo(&project.name);
        let dir = harness::workspace_root().join(&slug);
        std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
        std::fs::create_dir_all(dir.join(".git")).expect("mkdir .git");
        std::fs::write(dir.join("README.md"), b"# demo").expect("write README");
        std::fs::write(dir.join("src/main.rs"), b"fn main() {}").expect("write main");
        std::fs::write(dir.join(".git/config"), b"[core]").expect("write git config");

        let files = collect_workspace_files(&project).expect("collect files");
        let paths: Vec<String> = files
            .iter()
            .map(|f| f["path"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(paths.contains(&"README.md".to_string()), "paths: {paths:?}");
        assert!(
            paths.contains(&"src/main.rs".to_string()),
            "paths: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.starts_with(".git")),
            "VCS dirs must be excluded: {paths:?}"
        );
        let main = files
            .iter()
            .find(|f| f["path"] == "src/main.rs")
            .expect("main.rs");
        let content: Vec<u8> =
            serde_json::from_value(main["content"].clone()).expect("content bytes");
        assert_eq!(content, b"fn main() {}");

        let _ = std::fs::remove_dir_all(&tmp);
        harness::restore_workspace_root(previous);
    }
}
