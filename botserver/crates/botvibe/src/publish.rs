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

use serde_json::Value;
use uuid::Uuid;

use crate::domains::{BindDomainRequest, ProjectDomains};
use crate::harness;
use crate::projects::{Project, ProjectRegistry};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult, VibeUseCase};
use crate::vm_lifecycle::{CreateVmRequest, VmLifecycle};

const PUBLISH_DEFAULT_ENV: &str = "production";

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
                "domain": { "type": "string", "description": "Optional custom domain to bind" }
            },
            "required": ["project_id"]
        }))
        .with_approval()
        .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

fn api_base() -> String {
    std::env::var("VIBE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Collect the project's source files from its workspace (the agent's actual
/// output) so the deployment API can push them to the ALM repo instead of an
/// empty app. The workspace dir is keyed by the ALM repo slug, falling back
/// to the raw project id.
fn collect_workspace_files(project: &Project) -> Vec<Value> {
    let candidates = [
        VmLifecycle::alm_repo(&project.name),
        project.id.to_string(),
    ];
    for key in candidates {
        let dir = harness::workspace_root().join(&key);
        if !dir.is_dir() {
            continue;
        }
        let mut out = Vec::new();
        if walk_workspace(&dir, &dir, &mut out).is_ok() && !out.is_empty() {
            log::info!(
                "Vibe publish: packaged {} files from workspace '{key}'",
                out.len()
            );
            return out;
        }
    }
    log::warn!(
        "Vibe publish: no workspace files found for project '{}' — deploying empty repo",
        project.name
    );
    Vec::new()
}

fn walk_workspace(dir: &Path, root: &Path, out: &mut Vec<Value>) -> Result<(), String> {
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
            walk_workspace(&path, root, out)?;
            continue;
        }
        let rel = path.strip_prefix(root).map_err(|e| format!("strip prefix: {e}"))?;
        let rel = rel.to_string_lossy().replace('\\', "/");
        match std::fs::read(&path) {
            Ok(bytes) => out.push(serde_json::json!({ "path": rel, "content": bytes })),
            Err(e) => log::warn!("Vibe publish: skip unreadable file {rel}: {e}"),
        }
    }
    Ok(())
}

async fn publish_project(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_publish(args, pool).await {
        Ok(data) => VibeToolResult { success: true, data, error: None, latency_ms: started.elapsed().as_millis() as u64 },
        Err(e) => VibeToolResult { success: false, data: Value::Null, error: Some(e), latency_ms: started.elapsed().as_millis() as u64 },
    }
}

pub(crate) async fn do_publish(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "publish/project requires 'project_id' (uuid string)".to_string())?;
    let env = args.get("env").and_then(|v| v.as_str()).unwrap_or(PUBLISH_DEFAULT_ENV).to_lowercase();
    if !crate::vm_lifecycle::VALID_ENVS.contains(&env.as_str()) {
        return Err(format!("invalid env '{env}'"));
    }
    let domain = args.get("domain").and_then(|v| v.as_str()).map(ToString::to_string);

    let registry = ProjectRegistry::new(pool.clone());
    let project = registry
        .get(project_id)?
        .ok_or_else(|| format!("project {project_id} not found"))?;

    let metering = crate::metering::VMetering::new(pool.clone());
    metering.enforce_for_project(project_id, crate::metering::MeterKind::BuildMinutes)?;
    let _ = metering.add_for_project(project_id, &env, crate::metering::MeterKind::BuildMinutes, 1.0);

    let repo_name = VmLifecycle::alm_repo(&project.name);
    let org = VmLifecycle::alm_org(project.branch_id);

    let vm_req = CreateVmRequest { env: env.clone(), tier: "small".to_string(), runner_enabled: false };
    let vm = VmLifecycle::new(pool.clone())
        .create_project_vm(project_id, project.branch_id, &project.name, &vm_req)?;

    // Self-hosted ALM (Forgejo) base URL: env → localhost default.
    let alm_base = std::env::var("FORGEJO_URL")
        .ok()
        .or_else(|| std::env::var("ALM_URL").ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://localhost:4747".to_string());

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
                "route": format!("{repo_name}-{env}.gb.solutions"),
                "shared_resources": true
            }
        }),
    };

    let files = collect_workspace_files(&project);
    let body = serde_json::json!({
        "app_name": repo_name,
        "org": org,
        "project_type": project.project_type,
        "environment": env,
        "target": target,
        "project_id": project_id,
        "files": files,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let resp = client
        .post(format!("{}/api/deployment/deploy", api_base()))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("deployment API unreachable: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("deployment API returned {status}: {text}"));
    }
    let deployed: Value = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));

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

    let binding = match &domain {
        Some(d) => {
            let bind_req = BindDomainRequest { domain: d.clone(), env: env.clone() };
            match ProjectDomains::new(pool).bind(project_id, &bind_req).await {
                Ok(b) => serde_json::json!({ "bound": true, "id": b.id, "domain": b.domain, "env": b.env, "container": b.container, "verified": b.verified, "tls_status": b.tls_status }),
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
            status: "active".to_string(),
            environment: "development".to_string(),
            payload: Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn collect_workspace_files_packages_source_and_skips_vcs() {
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

        let files = collect_workspace_files(&project);
        let paths: Vec<String> = files
            .iter()
            .map(|f| f["path"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(paths.contains(&"README.md".to_string()), "paths: {paths:?}");
        assert!(paths.contains(&"src/main.rs".to_string()), "paths: {paths:?}");
        assert!(
            !paths.iter().any(|p| p.starts_with(".git")),
            "VCS dirs must be excluded: {paths:?}"
        );
        let main = files.iter().find(|f| f["path"] == "src/main.rs").expect("main.rs");
        let content: Vec<u8> = serde_json::from_value(main["content"].clone()).expect("content bytes");
        assert_eq!(content, b"fn main() {}");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}