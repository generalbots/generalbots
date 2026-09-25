//! `domains::resolve` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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

pub(crate) fn default_env() -> String {
    "production".to_string()
}

impl ProjectDomains {
    /// Applies the Caddy route for a binding, dialing the container's real
    /// IPv4: apps listen on :3000 inside the container and the proxy
    /// container cannot resolve `{container}.incus` names (#1261). Without a
    /// resolvable IP the route would silently point at a dead upstream (the
    /// ERR_SSL_PROTOCOL_ERROR/502 symptom on published apps), so fail
    /// loudly instead.
    pub(crate) async fn apply_route(&self, bind: &DomainBind) -> Result<CaddyResult, String> {
        // #1288 — proxy-published sites (website / python via proxy_sites)
        // already have their Caddyfile site block managed by the publish
        // pipeline; re-dialing a VM IP here would overwrite the file_server
        // / python reverse_proxy route with a dead container dial (502).
        if bind.container == "proxy" {
            return Err(
                "proxy-published site: route is managed by the vibe publish pipeline (Caddyfile section), not the container dial path"
                    .to_string(),
            );
        }
        let dial = match VmLifecycle::new(self.pool.clone()).linux_ip(&bind.container) {
            Ok(Some(ip)) => format!("{ip}:3000"),
            Ok(None) => {
                return Err(format!(
                    "container '{}' has no IPv4 address — cannot route {}",
                    bind.container, bind.domain
                ));
            }
            Err(e) => {
                return Err(format!(
                    "could not resolve IPv4 for '{}': {e}",
                    bind.container
                ));
            }
        };
        caddy::upsert_route_to(&bind.domain, &dial, &bind.access).await
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
        // #1288 — bindings for proxy-published projects must never carry a
        // VM container name: apply_route would then dial the project VM's
        // IP (dead upstream for a site served from the proxy websites tree)
        // and the admin-API route would shadow the Caddyfile file_server
        // block with a 502. The container field is the routing source of
        // truth, so record it as "proxy".
        let container = if project_last_deploy_target(&project) == Some("proxy-websites") {
            "proxy".to_string()
        } else {
            VmLifecycle::container_name(&project.name, &env, false)
        };
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
}
