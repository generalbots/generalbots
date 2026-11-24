//! #756/#774/#770 — `domain/*` Vibe tools.
//!
//! Exposes the project domain surface to the LLM: bind a domain to a
//! project env (`domain/bind`), verify DNS ownership via TXT (#774 —
//! `domain/verify`), and (re)issue TLS via the Caddy ACME automation
//! (#770 — `domain/tls`).

use std::sync::Arc;

use botsecurity_core::validation::validate_url_ssrf_strict;
use serde_json::Value;
use uuid::Uuid;

use crate::domains::{BindDomainRequest, ProjectDomains};
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult, VibeUseCase};

/// #2751 — `on_behalf_of_user` is stamped by every executor call path from the
/// authenticated principal. A missing/nil value means the call did not come
/// through a trusted path and must be refused, never treated as privileged.
fn acting_user(args: &Value) -> Result<Uuid, String> {
    let user_id = args
        .get("on_behalf_of_user")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "forbidden: missing acting user for project-admin tool".to_string())?;
    if user_id.is_nil() {
        return Err("forbidden: anonymous user cannot manage project domains".to_string());
    }
    Ok(user_id)
}

/// Pure authorization decision (#2751), unit-testable with a two-role resolver.
/// Production passes `ProjectRbac::resolve_role` as `resolve_role`.
fn require_domain_admin<F>(args: &Value, project_id: Uuid, resolve_role: F) -> Result<Uuid, String>
where
    F: FnOnce(Uuid, Uuid) -> Result<ProjectRole, String>,
{
    let user_id = acting_user(args)?;
    let role = resolve_role(user_id, project_id)?;
    if role < ProjectRole::Admin {
        return Err(format!(
            "forbidden: role '{role}' requires 'admin' on project {project_id}"
        ));
    }
    Ok(user_id)
}

/// Reject a domain that points at the local node, a cloud metadata service or
/// a non-public address before a Caddy route is written (#2751). The shared
/// SSRF guard is the single source of truth for hostile targets; bindings are
/// bare hostnames, so validate them as `https://` URLs.
fn validate_tool_domain(domain: &str) -> Result<String, String> {
    let host = ProjectDomains::validate_domain(domain)?;
    validate_url_ssrf_strict(&format!("https://{host}"))
        .map_err(|_| format!("invalid domain '{host}': private or metadata target is not allowed"))?;
    Ok(host)
}

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
            "allowed_emails": { "type": "string", "description": "Comma-separated email allowlist (only used when access=rbac)" },
            "on_behalf_of_user": { "type": "string", "description": "Internal — always overwritten by the server with the authenticated acting user for project-Admin RBAC; never supply it yourself." }
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
            "allowed_emails": { "type": "string", "description": "Comma-separated email allowlist (only used when access=rbac)" },
            "on_behalf_of_user": { "type": "string", "description": "Internal — always overwritten by the server with the authenticated acting user for project-Admin RBAC; never supply it yourself." }
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
            "domain": { "type": "string", "description": "Bound domain to verify" },
            "env": { "type": "string", "enum": ["development", "staging", "production"], "description": "Project environment the binding belongs to (defaults to production)" }
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
            "domain": { "type": "string", "description": "Bound domain to secure" },
            "env": { "type": "string", "enum": ["development", "staging", "production"], "description": "Project environment the binding belongs to (defaults to production)" },
            "on_behalf_of_user": { "type": "string", "description": "Internal — always overwritten by the server with the authenticated acting user for project-Admin RBAC; never supply it yourself." }
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
    let domain = validate_tool_domain(
        args.get("domain")
            .and_then(Value::as_str)
            .ok_or_else(|| "domain/bind requires 'domain'".to_string())?,
    )?;
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

    // #2751 — the REST equivalent gates with Admin + metering; the tool path
    // must enforce the same policy before touching the binding table.
    let rbac = ProjectRbac::new(pool.clone());
    require_domain_admin(&args, project_id, |user, project| rbac.resolve_role(user, project))?;
    let metering = crate::metering::VMetering::new(pool.clone());
    metering.enforce_for_project(project_id, crate::metering::MeterKind::DomainBindings)?;

    let domains = ProjectDomains::new(pool);
    let req = BindDomainRequest {
        domain,
        env,
        access,
        allowed_emails,
    };
    let bind = domains.bind(project_id, &req).await?;
    if let Err(e) = metering.add_for_project(
        project_id,
        &bind.env,
        crate::metering::MeterKind::DomainBindings,
        1.0,
    ) {
        log::warn!("Vibe domains_tool: metering domain binding for {project_id} failed: {e}");
    }
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
    let d = validate_tool_domain(domain)?;
    let env = args.get("env").and_then(|v| v.as_str()).unwrap_or("production").to_string();
    let rbac = ProjectRbac::new(pool.clone());
    let domains = ProjectDomains::new(pool);
    let bind = domains.get_by_domain_env(&d, &env)?;
    // #2751 — authorize against the binding's real project, never a caller id.
    require_domain_admin(&args, bind.project_id, |user, project| {
        rbac.resolve_role(user, project)
    })?;
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
    let env = args.get("env").and_then(|v| v.as_str()).unwrap_or("production").to_string();
    let domain = validate_tool_domain(domain)?;
    ProjectDomains::new(pool).verify_dns(&domain, &env)
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
    let d = validate_tool_domain(domain)?;
    let env = args.get("env").and_then(|v| v.as_str()).unwrap_or("production").to_string();
    let rbac = ProjectRbac::new(pool.clone());
    let domains = ProjectDomains::new(pool);
    let bind = domains.get_by_domain_env(&d, &env)
        .map_err(|_| format!("no binding for {d}/{env}; bind it first with domain/bind"))?;
    // #2751 — authorize against the binding's real project, never a caller id.
    require_domain_admin(&args, bind.project_id, |user, project| {
        rbac.resolve_role(user, project)
    })?;
    domains.issue_tls(&bind).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two-role resolver: only `admin` holds Admin on the project, everybody
    /// else is a Viewer (#2751 acceptance: non-admin rejected, admin works).
    fn two_role_resolver(admin: Uuid) -> impl Fn(Uuid, Uuid) -> Result<ProjectRole, String> + Copy {
        move |user, _project| {
            if user == admin {
                Ok(ProjectRole::Admin)
            } else {
                Ok(ProjectRole::Viewer)
            }
        }
    }

    #[test]
    fn domain_admin_tool_rejects_non_admin() {
        let project = Uuid::new_v4();
        let admin = Uuid::new_v4();
        let viewer = Uuid::new_v4();
        let resolve = two_role_resolver(admin);

        let admin_args = serde_json::json!({ "on_behalf_of_user": admin.to_string() });
        assert_eq!(
            require_domain_admin(&admin_args, project, resolve).ok(),
            Some(admin)
        );

        let viewer_args = serde_json::json!({ "on_behalf_of_user": viewer.to_string() });
        let error = require_domain_admin(&viewer_args, project, resolve).unwrap_err();
        assert!(error.contains("forbidden"), "{error}");
        assert!(error.contains("admin"), "{error}");
    }

    #[test]
    fn domain_admin_tool_refuses_missing_or_anonymous_actor() {
        let project = Uuid::new_v4();
        let admin = Uuid::new_v4();
        let resolve = two_role_resolver(admin);
        for args in [
            serde_json::json!({}),
            serde_json::json!({ "on_behalf_of_user": Uuid::nil().to_string() }),
            serde_json::json!({ "on_behalf_of_user": "not-a-uuid" }),
        ] {
            assert!(require_domain_admin(&args, project, resolve).is_err());
        }
    }

    #[test]
    fn hostile_domain_targets_are_rejected() {
        for hostile in [
            "localhost",
            "foo.localhost",
            "127.0.0.1",
            "169.254.169.254",
            "metadata.google.internal",
            "instance-data",
            "service.internal",
            "10.0.0.5",
            "192.168.1.10",
            "[::1]",
            "shop.example.com:5432",
            "https://example.com",
            "user:pass@example.com",
            "example.com/path",
            "bad domain.example.com",
        ] {
            assert!(
                validate_tool_domain(hostile).is_err(),
                "{hostile} should be rejected"
            );
        }
        for safe in ["shop.example.com", "app.example.co.uk", "GB-Shop.Example.COM"] {
            assert!(
                validate_tool_domain(safe).is_ok(),
                "{safe} should be accepted"
            );
        }
    }
}
