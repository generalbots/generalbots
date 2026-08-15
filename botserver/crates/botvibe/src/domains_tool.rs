//! #756/#774/#770 — `domain/*` Vibe tools.
//!
//! Exposes the project domain surface to the LLM: bind a domain to a
//! project env (`domain/bind`), verify DNS ownership via TXT (#774 —
//! `domain/verify`), and (re)issue TLS via the Caddy ACME automation
//! (#770 — `domain/tls`).

use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

use crate::domains::{BindDomainRequest, ProjectDomains};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult, VibeUseCase};

pub fn domain_bind_schema() -> ToolSchema {
    ToolSchema::new(
        "domain/bind",
        "Bind a custom domain to a project environment: registers the domain record and the Caddy route so the domain reaches the project's env container.",
    )
    .with_parameters(serde_json::json!({
        "type": "object",
        "properties": {
            "project_id": { "type": "string", "description": "UUID of the project" },
            "domain": { "type": "string", "description": "Bare hostname to bind (e.g. shop.example.com)" },
            "env": { "type": "string", "enum": ["development", "staging", "production"], "default": "production" },
            "access": { "type": "string", "enum": ["public", "authenticated", "rbac"], "default": "public", "description": "Who can open the app: public (anyone), authenticated (any account), rbac (only emails in allowed_emails)" },
            "allowed_emails": { "type": "string", "description": "Comma-separated email allowlist (only used when access=rbac)" }
        },
        "required": ["project_id", "domain"]
    }))
    .with_approval()
    .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

pub fn domain_security_schema() -> ToolSchema {
    ToolSchema::new(
        "domain/security",
        "Change the access policy of a bound domain: public (anyone), authenticated (any account), or rbac (email allowlist). Re-applies the Caddy route with a JWT gate when the domain is not public.",
    )
    .with_parameters(serde_json::json!({
        "type": "object",
        "properties": {
            "domain": { "type": "string", "description": "Bound domain to secure" },
            "access": { "type": "string", "enum": ["public", "authenticated", "rbac"], "description": "Access policy" },
            "allowed_emails": { "type": "string", "description": "Comma-separated email allowlist (only used when access=rbac)" }
        },
        "required": ["domain", "access"]
    }))
    .with_approval()
    .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

pub fn domain_verify_schema() -> ToolSchema {
    ToolSchema::new(
        "domain/verify",
        "Verify DNS ownership of a bound domain: checks that the TXT record _gb-verify.<domain> carries the binding token.",
    )
    .with_parameters(serde_json::json!({
        "type": "object",
        "properties": {
            "domain": { "type": "string", "description": "Bound domain to verify" }
        },
        "required": ["domain"]
    }))
    .with_approval()
    .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

pub fn domain_tls_schema() -> ToolSchema {
    ToolSchema::new(
        "domain/tls",
        "Issue TLS for a bound domain via the Caddy ACME automation (certificate issued on first request).",
    )
    .with_parameters(serde_json::json!({
        "type": "object",
        "properties": {
            "domain": { "type": "string", "description": "Bound domain to secure" }
        },
        "required": ["domain"]
    }))
    .with_approval()
    .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

pub fn domain_bind_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        let args = args.clone();
        Box::pin(domain_bind(args, pool))
    })
}

pub fn domain_security_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        let args = args.clone();
        Box::pin(domain_security(args, pool))
    })
}

pub fn domain_verify_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        Box::pin(async {
            let started = std::time::Instant::now();
            match do_domain_verify(args, pool).await {
                Ok(v) => VibeToolResult { success: true, data: v, error: None, latency_ms: started.elapsed().as_millis() as u64 },
                Err(e) => VibeToolResult { success: false, data: Value::Null, error: Some(e), latency_ms: started.elapsed().as_millis() as u64 },
            }
        })
    })
}

pub fn domain_tls_tool() -> ToolHandler {
    Arc::new(|args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        Box::pin(domain_tls(args, pool))
    })
}

async fn domain_bind(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_domain_bind(args, pool).await {
        Ok(data) => VibeToolResult { success: true, data, error: None, latency_ms: started.elapsed().as_millis() as u64 },
        Err(e) => VibeToolResult { success: false, data: Value::Null, error: Some(e), latency_ms: started.elapsed().as_millis() as u64 },
    }
}

async fn do_domain_bind(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "domain/bind requires 'project_id' (uuid string)".to_string())?;
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "domain/bind requires 'domain'".to_string())?;
    let env = args
        .get("env")
        .and_then(|v| v.as_str())
        .unwrap_or("production")
        .to_lowercase();
    let access = args.get("access").and_then(|v| v.as_str()).map(String::from);
    let allowed_emails = args
        .get("allowed_emails")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let domains = ProjectDomains::new(pool);
    let req = BindDomainRequest {
        domain: domain.to_string(),
        env,
        access,
        allowed_emails,
    };
    let bind = domains.bind(project_id, &req).await?;
    Ok(serde_json::json!({
        "bound": true,
        "id": bind.id,
        "project_id": bind.project_id,
        "domain": bind.domain,
        "env": bind.env,
        "container": bind.container,
        "verified": bind.verified,
        "verify_record": ProjectDomains::verify_name(&bind.domain),
        "verify_token": bind.verify_token,
        "tls_status": bind.tls_status,
        "access": bind.access,
        "allowed_emails": bind.allowed_emails,
    }))
}

async fn domain_security(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_domain_security(args, pool).await {
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

async fn do_domain_security(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "domain/security requires 'domain'".to_string())?;
    let access = args
        .get("access")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "domain/security requires 'access'".to_string())?;
    let allowed_emails = args
        .get("allowed_emails")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let d = ProjectDomains::validate_domain(domain)?;
    let domains = ProjectDomains::new(pool);
    let bind = domains.get_by_domain(&d)?;
    let updated = domains.update_access(bind.id, access, allowed_emails).await?;
    Ok(serde_json::json!({
        "updated": true,
        "id": updated.id,
        "domain": updated.domain,
        "access": updated.access,
        "allowed_emails": updated.allowed_emails,
    }))
}

async fn do_domain_verify(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "domain/verify requires 'domain'".to_string())?;
    ProjectDomains::new(pool).verify_dns(domain)
}

async fn domain_tls(args: Value, pool: crate::types::DbPool) -> VibeToolResult {
    let started = std::time::Instant::now();
    match do_domain_tls(args, pool).await {
        Ok(data) => VibeToolResult { success: true, data, error: None, latency_ms: started.elapsed().as_millis() as u64 },
        Err(e) => VibeToolResult { success: false, data: Value::Null, error: Some(e), latency_ms: started.elapsed().as_millis() as u64 },
    }
}

async fn do_domain_tls(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "domain/tls requires 'domain'".to_string())?;
    let d = ProjectDomains::validate_domain(domain)?;
    let domains = ProjectDomains::new(pool);
    let binds = domains.list_for_domain(&d)?;
    let bind = binds
        .into_iter()
        .next()
        .ok_or_else(|| format!("no binding for {d}; bind it first with domain/bind"))?;
    domains.issue_tls(&bind).await
}