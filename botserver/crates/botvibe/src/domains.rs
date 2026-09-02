//! #756/#774/#770 — Project domain bindings.
//!
//! Binds a custom domain to a project environment (domain → project →
//! container/Caddy route). Each binding is persisted in `project_domains`
//! (scoped by branch_id like the rest of the SaaS surface), keeps a
//! verification state (DNS TXT token — #774) and a TLS state driven through
//! the Caddy admin API (ACME — #770).
//!
//! The driver talks to Caddy's admin API (`CADDY_API_URL`); when the proxy
//! is unreachable (dev/offline) the operations return structured errors
//! instead of panicking.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::ensure_schema_sql;
use crate::caddy::{self, CaddyResult};
use crate::types::DbPool;
use crate::vm_lifecycle::VmLifecycle;

pub const PROJECT_DOMAINS_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS project_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    domain VARCHAR(255) NOT NULL,
    env VARCHAR(16) NOT NULL DEFAULT 'production',
    container VARCHAR(255) NOT NULL DEFAULT '',
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    verify_token VARCHAR(255),
    tls_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    tls_error TEXT,
    access VARCHAR(16) NOT NULL DEFAULT 'public',
    allowed_emails TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_project_domains_domain_env ON project_domains(domain, env);
CREATE INDEX IF NOT EXISTS idx_project_domains_project ON project_domains(project_id);

-- Idempotent column additions for tables created by older schema versions:
-- CREATE TABLE IF NOT EXISTS never alters an existing table, so columns
-- added after the first deploy would otherwise be missing and every insert
-- would fail with a missing-column error.
ALTER TABLE project_domains ADD COLUMN IF NOT EXISTS verify_token VARCHAR(255);
ALTER TABLE project_domains ADD COLUMN IF NOT EXISTS tls_status VARCHAR(20) NOT NULL DEFAULT 'pending';
ALTER TABLE project_domains ADD COLUMN IF NOT EXISTS tls_error TEXT;
ALTER TABLE project_domains ADD COLUMN IF NOT EXISTS access VARCHAR(16) NOT NULL DEFAULT 'public';
ALTER TABLE project_domains ADD COLUMN IF NOT EXISTS allowed_emails TEXT;
";

pub const DNS_VERIFY_PREFIX: &str = "_gb-verify";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainBind {
    pub id: Uuid,
    pub project_id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub domain: String,
    pub env: String,
    pub container: String,
    pub verified: bool,
    pub verify_token: Option<String>,
    pub tls_status: String,
    pub tls_error: Option<String>,
    /// Access policy for the served app: `public` (no auth), `authenticated`
    /// (any valid cloud JWT) or `rbac` (email allowlist). Enforced by the
    /// Caddy forward_auth wrapper pointing at `/api/vibe/domain-auth`.
    pub access: String,
    /// Comma-separated email allowlist used when `access == "rbac"`.
    pub allowed_emails: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BindDomainRequest {
    pub domain: String,
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default)]
    pub access: Option<String>,
    #[serde(default)]
    pub allowed_emails: Option<String>,
}

fn default_env() -> String {
    "production".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<DomainBind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binds: Option<Vec<DomainBind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl DomainResult {
    pub fn ok(bind: DomainBind) -> Self {
        Self { success: true, bind: Some(bind), binds: None, verify: None, error: None }
    }
    pub fn ok_list(binds: Vec<DomainBind>) -> Self {
        Self { success: true, bind: None, binds: Some(binds), verify: None, error: None }
    }
    pub fn ok_verify(v: serde_json::Value) -> Self {
        Self { success: true, bind: None, binds: None, verify: Some(v), error: None }
    }
    pub fn err(msg: String) -> Self {
        Self { success: false, bind: None, binds: None, verify: None, error: Some(msg) }
    }
    pub fn deleted() -> Self {
        Self { success: true, bind: None, binds: None, verify: None, error: None }
    }
}

/// Project domain binding registry (DB records + Caddy route driver).
#[derive(Clone)]
pub struct ProjectDomains {
    pool: DbPool,
}

impl ProjectDomains {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, String>
    {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        ensure_schema_sql(&mut conn, PROJECT_DOMAINS_SCHEMA, "project_domains schema")?;
        Ok(())
    }

    /// Hostname validation: lowercase, letters/digits/dots/hyphens, at
    /// least one dot, no spaces or scheme.
    pub fn validate_domain(d: &str) -> Result<String, String> {
        let domain = d.trim().to_lowercase();
        if domain.is_empty() || domain.len() > 253 {
            return Err("invalid domain: empty or too long".to_string());
        }
        if domain.contains("//") || domain.chars().any(|c| c.is_whitespace()) {
            return Err("invalid domain: must be a bare hostname".to_string());
        }
        for ch in domain.chars() {
            if !(ch.is_ascii_alphanumeric() || ch == '.' || ch == '-') {
                return Err(format!("invalid domain '{domain}': bad character '{ch}'"));
            }
        }
        if !domain.contains('.') {
            return Err(format!("invalid domain '{domain}': expected a FQDN"));
        }
        Ok(domain)
    }

    /// Validates an access policy value. `public` serves the app to anyone;
    /// `authenticated` requires a valid cloud JWT; `rbac` restricts to the
    /// binding's email allowlist.
    pub fn validate_access(a: &str) -> Result<String, String> {
        match a.trim().to_lowercase().as_str() {
            "public" | "authenticated" | "rbac" => Ok(a.trim().to_lowercase()),
            other => Err(format!(
                "invalid access '{other}': expected 'public', 'authenticated' or 'rbac'"
            )),
        }
    }

    /// DNS verification name for a domain (TXT `_gb-verify.<domain>` with
    /// the token as value — #774).
    pub fn verify_name(domain: &str) -> String {
        format!("{DNS_VERIFY_PREFIX}.{domain}")
    }

    /// #774 — Parse `dig +short CAA <domain>` output and decide whether the
    /// Let's Encrypt CA (used by Caddy ACME) is allowed to issue for the
    /// domain. No CAA records at all means no policy, hence allowed.
    pub fn caa_allows_acme(records: &str) -> bool {
        let mut saw_issue = false;
        let mut saw_acme_ok = false;
        for line in records.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let is_issue = line.to_ascii_lowercase().contains("issue");
            let value = line
                .split('"')
                .nth(1)
                .map(|v| v.trim().to_ascii_lowercase());
            match value.as_deref() {
                Some(";") | Some("") => {
                    if is_issue {
                        return false;
                    }
                }
                Some(ca) if ca.contains("letsencrypt.org") && is_issue => {
                    saw_issue = true;
                    saw_acme_ok = true;
                }
                _ => {}
            }
            if is_issue {
                saw_issue = true;
            }
        }
        !saw_issue || saw_acme_ok
    }

    pub async fn bind(&self, project_id: Uuid, req: &BindDomainRequest) -> Result<DomainBind, String> {
        let domain = Self::validate_domain(&req.domain)?;
        let env = req.env.trim().to_lowercase();
        if !crate::vm_lifecycle::VALID_ENVS.contains(&env.as_str()) {
            return Err(format!("invalid env '{env}'"));
        }
        let project = self.project_row(project_id)?;
        let branch = project.branch_id;
        let container = VmLifecycle::container_name(&project.name, &env, false);
        let token = format!("{}:{}", branch.simple(), domain);
        let access = match &req.access {
            Some(a) => Self::validate_access(a)?,
            None => "public".to_string(),
        };
        let allowed = req.allowed_emails.clone().unwrap_or_default();

        let mut conn = self.conn()?;
        match self.select_by_domain_env(&mut conn, &domain, &env) {
            Ok(row) => {
                let row_id = row.id;
                diesel::sql_query(
                    "UPDATE project_domains SET project_id = $2, container = $3, verify_token = $4, access = $5, allowed_emails = $6, updated_at = NOW() WHERE id = $1",
                )
                .bind::<diesel::sql_types::Uuid, _>(row_id)
                .bind::<diesel::sql_types::Uuid, _>(project_id)
                .bind::<diesel::sql_types::Text, _>(&container)
                .bind::<diesel::sql_types::Text, _>(&token)
                .bind::<diesel::sql_types::Text, _>(&access)
                .bind::<diesel::sql_types::Text, _>(&allowed)
                .execute(&mut conn)
                .map_err(|e| format!("update bind: {e}"))?;
            }
            Err(_) => {
                let org = project.org_id;
                diesel::sql_query(
                    "INSERT INTO project_domains (project_id, org_id, branch_id, domain, env, container, verify_token, access, allowed_emails, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())",
                )
                .bind::<diesel::sql_types::Uuid, _>(project_id)
                .bind::<diesel::sql_types::Uuid, _>(org)
                .bind::<diesel::sql_types::Uuid, _>(branch)
                .bind::<diesel::sql_types::Text, _>(&domain)
                .bind::<diesel::sql_types::Text, _>(&env)
                .bind::<diesel::sql_types::Text, _>(&container)
                .bind::<diesel::sql_types::Text, _>(&token)
                .bind::<diesel::sql_types::Text, _>(&access)
                .bind::<diesel::sql_types::Text, _>(&allowed)
                .execute(&mut conn)
                .map_err(|e| format!("insert bind: {e}"))?;
            }
        }

        let bind = self.select_by_domain_env(&mut conn, &domain, &env)?.into_bind();
        Ok(bind)
    }

    /// Updates the access policy (public/authenticated/rbac + email
    /// allowlist) of a binding and re-applies the Caddy route so the
    /// forward_auth wrapper is installed/removed accordingly.
    pub async fn update_access(
        &self,
        bind_id: Uuid,
        access: &str,
        allowed_emails: Option<String>,
    ) -> Result<DomainBind, String> {
        let access = Self::validate_access(access)?;
        let allowed = allowed_emails.unwrap_or_default();
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE project_domains SET access = $2, allowed_emails = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(bind_id)
        .bind::<diesel::sql_types::Text, _>(&access)
        .bind::<diesel::sql_types::Text, _>(&allowed)
        .execute(&mut conn)
        .map_err(|e| format!("update access: {e}"))?;
        let bind = self.get(bind_id)?;
        caddy::upsert_route(&bind.domain, &bind.container, &bind.access)
            .await
            .map_err(|e| {
                log::warn!("Caddy route re-apply for {} failed: {e}", bind.domain);
                e
            })?;
        Ok(bind)
    }

    /// Looks up a binding for a domain in a specific environment (#922).
    /// The schema allows the same domain in multiple environments, so any
    /// `ORDER BY env LIMIT 1` lookup is ambiguous and can authorize/proxy the
    /// wrong environment. Callers must pass an explicit env.
    pub fn get_by_domain_env(&self, domain: &str, env: &str) -> Result<DomainBind, String> {
        let mut conn = self.conn()?;
        self.select_by_domain_env(&mut conn, domain, env.trim().to_lowercase().as_str())
            .map(|r| r.into_bind())
    }

    /// Resolves the SaaS JWT secret used to sign cloud-login tokens
    /// (same resolution as `directory_setup::resolve_saas_jwt_secret` in the
    /// botserver crate, replicated here because that fn is `pub(crate)`).
    pub fn saas_jwt_secret() -> String {
        let stack = std::env::var("BOTSERVER_STACK_PATH")
            .ok()
            .filter(|p| !p.trim().is_empty())
            .or_else(|| std::env::var("GBO_STACK_PATH").ok().filter(|p| !p.trim().is_empty()))
            .unwrap_or_else(|| "/opt/gbo".to_string());
        let config_path = format!("{stack}/conf/system/directory_config.json");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(secret) = json
                    .get("saas_jwt_secret")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    return secret.to_string();
                }
            }
        }
        std::env::var("SAAS_JWT_SECRET")
            .or_else(|_| std::env::var("JWT_SECRET"))
            .unwrap_or_else(|_| {
                "dev-secret-key-change-in-production-minimum-32-chars".to_string()
            })
    }

    pub fn list(&self, project_id: Uuid) -> Result<Vec<DomainBind>, String> {
        let mut conn = self.conn()?;
        let rows = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, domain, env, container, verified, verify_token, tls_status, tls_error, access, allowed_emails, created_at, updated_at
             FROM project_domains WHERE project_id = $1 ORDER BY domain",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .load::<DomainRow>(&mut conn)
        .map_err(|e| format!("list binds: {e}"))?;
        Ok(rows.into_iter().map(|r| r.into_bind()).collect())
    }

    pub async fn unbind(&self, bind_id: Uuid) -> Result<(), String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, domain, env, container, verified, verify_token, tls_status, tls_error, access, allowed_emails, created_at, updated_at
             FROM project_domains WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(bind_id)
        .get_result::<DomainRow>(&mut conn)
        .map_err(|e| format!("lookup bind: {e}"))?;
        caddy::remove_route(&row.domain)
            .await
            .map_err(|e| {
                log::warn!("Caddy route removal for {} failed: {e}", row.domain);
                e
            })?;
        diesel::sql_query("DELETE FROM project_domains WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(bind_id)
            .execute(&mut conn)
            .map_err(|e| format!("delete bind: {e}"))?;
        Ok(())
    }

    /// #774 — Verify DNS ownership: `_gb-verify.<domain>` TXT must contain
    /// the recorded token. Uses `dig` through the harness command guard.
    ///
    /// #922 — env-specific: resolves the exact binding for `domain`+`env`
    /// rather than the first row for the domain.
    pub fn verify_dns(&self, domain: &str, env: &str) -> Result<serde_json::Value, String> {
        let domain = Self::validate_domain(domain)?;
        let env = env.trim().to_lowercase();
        let mut conn = self.conn()?;
        let row = self.select_by_domain_env(&mut conn, &domain, &env)?;
        let expected = row.verify_token.clone().unwrap_or_default();

        // #1268 — platform-subdomain hosts ({app}.{published_domain()}) sit
        // on the platform's own wildcard zone: ownership is platform-managed
        // and needs no TXT round-trip. Mark them verified so the deploy
        // pipeline's TLS stage can proceed (custom domains still require the
        // manual TXT token below).
        if domain.ends_with(&format!(".{}", crate::publish::published_domain())) {
            let row_id = row.id;
            diesel::sql_query(
                "UPDATE project_domains SET verified = true, updated_at = NOW() WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(row_id)
            .execute(&mut conn)
            .map_err(|e| format!("update verified: {e}"))?;
            return Ok(serde_json::json!({
                "domain": domain,
                "verified": true,
                "method": "platform_subdomain",
                "message": "host is on the platform-managed wildcard zone; ownership pre-verified"
            }));
        }

        let records = crate::harness::cmd::run(
            "dig",
            &[
                "+short".to_string(),
                "TXT".to_string(),
                Self::verify_name(&domain),
            ],
            std::path::Path::new("."),
            10,
        )
        .map_err(|e| format!("dig failed: {e}"))?
        .stdout;

        let matched = !expected.is_empty() && records.contains(&expected);
        let row_id = row.id;
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE project_domains SET verified = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(row_id)
        .bind::<diesel::sql_types::Bool, _>(matched)
        .execute(&mut conn)
        .map_err(|e| format!("update verified: {e}"))?;

        Ok(serde_json::json!({
            "domain": domain,
            "record": Self::verify_name(&domain),
            "expected_token": expected,
            "found": records,
            "verified": matched,
            "caa": Self::caa_report(&domain),
        }))
    }

    /// #774 — CAA policy report for a domain: records found and whether the
    /// ACME CA used by Caddy (Let's Encrypt) is permitted to issue.
    pub fn caa_report(domain: &str) -> serde_json::Value {
        let records = match crate::harness::cmd::run(
            "dig",
            &[
                "+short".to_string(),
                "CAA".to_string(),
                domain.to_string(),
            ],
            std::path::Path::new("."),
            10,
        ) {
            Ok(out) => out.stdout,
            Err(e) => {
                log::warn!("CAA lookup for {domain} failed: {e}");
                String::new()
            }
        };
        let allowed = Self::caa_allows_acme(&records);
        serde_json::json!({
            "records": records,
            "allows_acme": allowed,
            "check": if allowed {
                "CAA permits Let's Encrypt issuance".to_string()
            } else {
                "CAA policy restricts the CA; TLS issuance may fail".to_string()
            },
        })
    }

    /// #770 — TLS via Caddy ACME: (re)apply the route so the automatic
    /// HTTPS policy issues the certificate on first request; reports the
    /// proxied configuration state.
    pub async fn issue_tls(&self, bind: &DomainBind) -> Result<serde_json::Value, String> {
        // #922 — never activate a public route or issue TLS for an unverified
        // domain; require successful ownership verification first.
        if !bind.verified {
            return Err(format!(
                "domain '{}' is not verified; complete DNS verification before enabling TLS",
                bind.domain
            ));
        }
        let route_applied = caddy::upsert_route(&bind.domain, &bind.container, &bind.access).await;
        let route_state = match &route_applied {
            Ok(CaddyResult { route_id, .. }) => Ok(route_id.clone()),
            Err(e) => Err(e.clone()),
        };
        let status = match route_state {
            Ok(_) => "pending".to_string(),
            Err(_) => "failed".to_string(),
        };
        let error_col: Option<&str> = match &route_applied {
            Ok(_) => bind.tls_error.as_deref(),
            Err(e) => Some(e.as_str()),
        };
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE project_domains SET tls_status = $2, tls_error = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(bind.id)
        .bind::<diesel::sql_types::Text, _>(&status)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(error_col)
        .execute(&mut conn)
        .map_err(|e| format!("update tls: {e}"))?;

        Ok(serde_json::json!({
            "domain": bind.domain,
            "environment": bind.env,
            "container": bind.container,
            "tls_status": status,
            "issuer": "caddy-acme",
            "renewal": "automatic-on-request",
            "route": route_applied.map(|r| r.route_id).unwrap_or_else(|e| format!("error: {e}")),
            "error": bind.tls_error,
        }))
    }

    pub fn get(&self, bind_id: Uuid) -> Result<DomainBind, String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, domain, env, container, verified, verify_token, tls_status, tls_error, access, allowed_emails, created_at, updated_at
             FROM project_domains WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(bind_id)
        .get_result::<DomainRow>(&mut conn)
        .map(|r| r.into_bind())
        .map_err(|e| format!("get bind: {e}"))
    }

    fn select_by_domain_env(
        &self,
        conn: &mut diesel::PgConnection,
        domain: &str,
        env: &str,
    ) -> Result<DomainRow, String> {
        diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, domain, env, container, verified, verify_token, tls_status, tls_error, access, allowed_emails, created_at, updated_at
             FROM project_domains WHERE domain = $1 AND env = $2",
        )
        .bind::<diesel::sql_types::Text, _>(domain)
        .bind::<diesel::sql_types::Text, _>(env)
        .get_result::<DomainRow>(conn)
        .map_err(|e| format!("domain lookup: {e}"))
    }

    fn project_row(&self, project_id: Uuid) -> Result<ProjectRow, String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "SELECT org_id, branch_id, name FROM vibe_projects WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .get_result::<ProjectRow>(&mut conn)
        .map_err(|e| format!("project lookup: {e}"))
    }
}

#[derive(diesel::QueryableByName)]
struct ProjectRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(diesel::QueryableByName)]
struct DomainRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    project_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    domain: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    env: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    container: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    verified: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    verify_token: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    tls_status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    tls_error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    access: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    allowed_emails: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: DateTime<Utc>,
}

impl DomainRow {
    fn into_bind(self) -> DomainBind {
        DomainBind {
            id: self.id,
            project_id: self.project_id,
            org_id: self.org_id,
            branch_id: self.branch_id,
            domain: self.domain,
            env: self.env,
            container: self.container,
            verified: self.verified,
            verify_token: self.verify_token,
            tls_status: self.tls_status,
            tls_error: self.tls_error,
            access: self.access,
            allowed_emails: self.allowed_emails,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_validation_accepts_fqdns() {
        assert_eq!(ProjectDomains::validate_domain("Chat.Example.com"), Ok("chat.example.com".to_string()));
        assert_eq!(ProjectDomains::validate_domain("shop.example.co.uk"), Ok("shop.example.co.uk".to_string()));
    }

    #[test]
    fn domain_validation_rejects_bad_input() {
        assert!(ProjectDomains::validate_domain("").is_err());
        assert!(ProjectDomains::validate_domain("https://x.com").is_err());
        assert!(ProjectDomains::validate_domain("no-dot").is_err());
        assert!(ProjectDomains::validate_domain("bad space.com").is_err());
        assert!(ProjectDomains::validate_domain("bad!chars.com").is_err());
    }

    #[test]
    fn verify_name_prefixes_dns_record() {
        assert_eq!(ProjectDomains::verify_name("app.example.com"), "_gb-verify.app.example.com");
    }
}