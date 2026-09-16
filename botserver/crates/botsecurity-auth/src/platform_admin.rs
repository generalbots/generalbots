//! Platform (global) admin resolution — the single source of truth for
//! "may this caller cross tenant boundaries?" (#1387).
//!
//! Platform admin = explicit `superadmin` role **or** membership in the
//! reserved **root org** (`organizations.slug = 'default'` — the first /
//! default bot's org, which owns the seeded cloud catalog and the default
//! CRM account). Plain org admins (the global RBAC `admin` group also
//! holds them) are NOT platform admins and stay confined to their tenant.
//!
//! Consumers:
//! - `botdrive` — drive tenant isolation (#1401)
//! - `botcloud` — domains, vouchers, payment cards admin gates
//! - `botbrowserpolicy` — policy admin gates

use axum::http::HeaderMap;
use base64::Engine as _;
use diesel::prelude::*;

/// Reserved root org slug — the first/default account every instance seeds.
pub const ROOT_ORG_SLUG: &str = "default";

/// Dev-mode escape hatch: `admin@localhost` is always a platform admin.
pub const DEV_ADMIN_EMAIL: &str = "admin@localhost";

/// Extract the caller's email claim from a Bearer JWT in the
/// `Authorization` header, without verifying the signature (callers must
/// have authenticated the request already; this only reads the claim).
pub fn email_from_bearer(headers: &HeaderMap) -> Option<String> {
    let token = headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    claims
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Is this caller a PLATFORM admin (may cross tenants)? DB-backed:
/// explicit membership in the root org, or the dev-mode admin email.
pub fn is_platform_admin(
    conn: &mut diesel::PgConnection,
    headers: &HeaderMap,
) -> Result<bool, String> {
    let Some(email) = email_from_bearer(headers) else {
        return Ok(false);
    };
    if email.eq_ignore_ascii_case(DEV_ADMIN_EMAIL) {
        return Ok(true);
    }

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    // Same membership sources as the drive entitlement model: direct
    // user_organizations rows ∪ crm_contacts→branches (signup-derived).
    diesel::sql_query(
        "SELECT COUNT(*) AS count FROM ( \
         SELECT uo.org_id FROM user_organizations uo \
         JOIN organizations o ON o.org_id = uo.org_id \
         JOIN users u ON u.id = uo.user_id \
         WHERE lower(u.email) = lower($1) AND o.slug = $2 \
         UNION \
         SELECT br.org_id FROM crm_contacts c \
         JOIN branches br ON br.id = c.branch_id \
         JOIN organizations o ON o.org_id = br.org_id \
         WHERE lower(c.email) = lower($1) AND o.slug = $2 \
         ) AS roots",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .bind::<diesel::sql_types::Text, _>(ROOT_ORG_SLUG)
    .get_result::<CountRow>(conn)
    .map(|r| r.count > 0)
    .map_err(|e| format!("platform-admin check failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_extraction_handles_missing_header() {
        let headers = HeaderMap::new();
        assert_eq!(email_from_bearer(&headers), None);
    }

    #[test]
    fn test_email_extraction_rejects_non_jwt() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer not-a-jwt".parse().unwrap());
        assert_eq!(email_from_bearer(&headers), None);
    }
}
