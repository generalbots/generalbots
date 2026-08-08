//! #757 — `publish_project` Vibe tool.
//!
//! Puts a project onto a live environment: ensures the target env VM
//! (#744), calls the deployment API (`/api/deployment/deploy`) so an Incus
//! container (or Caddy route) is raised, optionally binds a custom domain,
//! and records the deployment in the project payload (deployment history —
//! #772 reads the same records).

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use uuid::Uuid;

use crate::domains::{BindDomainRequest, ProjectDomains};
use crate::projects::ProjectRegistry;
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

    let repo_name = VmLifecycle::alm_repo(&project.name);
    let org = VmLifecycle::alm_org(project.branch_id);

    let vm_req = CreateVmRequest { env: env.clone(), tier: "small".to_string(), runner_enabled: false };
    let vm = VmLifecycle::new(pool.clone())
        .create_project_vm(project_id, project.branch_id, &project.name, &vm_req)?;

    let target = match &domain {
        Some(d) => serde_json::json!({
            "External": {
                "repo_url": format!("https://alm.pragmatismo.com.br/{org}/{repo_name}"),
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

    let body = serde_json::json!({
        "app_name": repo_name,
        "org": org,
        "project_type": project.project_type,
        "environment": env,
        "target": target,
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