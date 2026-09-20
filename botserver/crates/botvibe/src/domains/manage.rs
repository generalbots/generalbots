//! `domains::manage` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub const DNS_VERIFY_PREFIX: &str = "_gb-verify";

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
    pub(crate) pool: DbPool,
}

impl ProjectDomains {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub(crate) fn conn(
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

    /// DNS verification name for a domain (TXT `_gb-verify.<domain>` with
    /// the token as value — #774).
    pub fn verify_name(domain: &str) -> String {
        format!("{DNS_VERIFY_PREFIX}.{domain}")
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
        self.apply_route(&bind).await.map_err(|e| {
            log::warn!("Caddy route re-apply for {} failed: {e}", bind.domain);
            e
        })?;
        Ok(bind)
    }
}

/// `Some("proxy-websites")` when the project's most recent deployment went
/// through the proxy-sites pipeline — such projects must bind domains with
/// `container = "proxy"` so routes are never dialed at a VM IP.
pub(crate) fn project_last_deploy_target(p: &ProjectRow) -> Option<&str> {
    p.deploy_target.as_deref()
}

#[derive(diesel::QueryableByName)]
pub(crate) struct ProjectRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) deploy_target: Option<String>,
}
