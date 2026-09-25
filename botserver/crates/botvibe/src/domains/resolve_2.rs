//! `domains::resolve_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

impl ProjectDomains {
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
        let route_applied = self.apply_route(bind).await;
        let route_state = match &route_applied {
            Ok(CaddyResult { route_id, .. }) => Ok(route_id.clone()),
            Err(e) if e.contains("proxy-published site") => {
                // TLS for proxy sites is owned by the Caddyfile block
                // (`tls internal` on dev, automatic ACME on prod) — not a
                // failure of issuance.
                Ok("caddyfile-managed".to_string())
            }
            Err(e) => Err(e.clone()),
        };
        let status = match route_state {
            Ok(_) => "pending".to_string(),
            Err(_) => "failed".to_string(),
        };
        let error_col: Option<&str> = match &route_applied {
            Ok(_) => bind.tls_error.as_deref(),
            Err(e) if !e.contains("proxy-published site") => Some(e.as_str()),
            Err(_) => bind.tls_error.as_deref(),
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

    pub(crate) fn select_by_domain_env(
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

    pub(crate) fn project_row(&self, project_id: Uuid) -> Result<ProjectRow, String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "SELECT org_id, branch_id, name, \
             payload->'deployments'->-1->>'deploy_target' AS deploy_target \
             FROM vibe_projects WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .get_result::<ProjectRow>(&mut conn)
        .map_err(|e| format!("project lookup: {e}"))
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct DomainRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) project_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) domain: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) env: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) container: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub(crate) verified: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) verify_token: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) tls_status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) tls_error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) access: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) allowed_emails: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) updated_at: DateTime<Utc>,
}

impl DomainRow {
    pub(crate) fn into_bind(self) -> DomainBind {
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
