use botcore::shared::schema::rbac_groups;
use botcore::shared::schema::rbac_user_groups;
use botcore::shared::utils::DbPool;
use botsecurity::rbac_middleware::is_admin_group_name;
use diesel::prelude::*;
use log::{debug, info};
use uuid::Uuid;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

/// Derives a stable UUID from a raw user identifier.
///
/// Zitadel/OIDC numeric user_ids (e.g. `380726576955786242`) are not valid
/// UUIDs; mapping them through `UUIDv5(zitadel:{id})` produces a deterministic
/// UUID so RBAC group membership survives across sessions and channels.
pub fn derive_stable_user_uuid(user_id: &str) -> Uuid {
    match Uuid::parse_str(user_id) {
        Ok(uuid) => uuid,
        Err(_) => Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("zitadel:{user_id}").as_bytes(),
        ),
    }
}

pub fn resolve_user_role(pool: &DbPool, user_id: Uuid) -> String {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            debug!("resolve_user_role: DB pool error for user {user_id}: {e}");
            return ROLE_USER.to_string();
        }
    };

    let group_ids: Vec<Uuid> = rbac_user_groups::table
        .filter(rbac_user_groups::user_id.eq(user_id))
        .select(rbac_user_groups::group_id)
        .load::<Uuid>(&mut conn)
        .unwrap_or(Default::default());

    if group_ids.is_empty() {
        debug!("resolve_user_role: user {user_id} has no groups, role=user");
        return ROLE_USER.to_string();
    }

    let group_names: Vec<String> = rbac_groups::table
        .filter(rbac_groups::id.eq_any(&group_ids))
        .filter(rbac_groups::is_active.eq(true))
        .select(rbac_groups::name)
        .load::<String>(&mut conn)
        .unwrap_or(Default::default());

    for name in &group_names {
        if is_admin_group_name(name) {
            info!("resolve_user_role: user {user_id} is admin via group '{name}'");
            return ROLE_ADMIN.to_string();
        }
    }

    debug!("resolve_user_role: user {user_id} groups={group_names:?}, role=user");
    ROLE_USER.to_string()
}

/// Resolves the effective role for an authenticated user, falling back to the
/// canonical `users` row matched by email when the supplied (derived) user_id
/// has no group memberships.
///
/// Background: cloud JWTs carry numeric Zitadel `sub` claims that are mapped
/// to a deterministic UUIDv5. Older deployments created the `users` row (and
/// its `rbac_user_groups` memberships) under a *different* derivation, so the
/// derived id finds no groups and every role-gated endpoint 403s even for
/// real admins. Matching the canonical row by email keeps role resolution
/// robust to identity-id derivation drift.
pub fn resolve_user_role_with_email(pool: &DbPool, user_id: Uuid, email: Option<&str>) -> String {
    if resolve_user_role(pool, user_id) == ROLE_ADMIN {
        return ROLE_ADMIN.to_string();
    }
    let email = match email {
        Some(e) if !e.is_empty() => e,
        _ => return ROLE_USER.to_string(),
    };
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            debug!("resolve_user_role_with_email: DB pool error: {e}");
            return ROLE_USER.to_string();
        }
    };
    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let canonical: Option<IdRow> = diesel::sql_query(
        "SELECT id FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result(&mut conn)
    .ok();
    match canonical {
        Some(row) if row.id != user_id => resolve_user_role(pool, row.id),
        _ => ROLE_USER.to_string(),
    }
}

/// #2671 — code-level baseline of the default-deny admin matrix. The
/// state-changing billing/store/voucher/organization/product surfaces used to
/// rely on the coarse bearer check only, so the LLM api-command catalog could
/// propose and execute them for ordinary users. The same prefixes are seeded
/// in migration `6.9.4-rbac-billing-store` and enforced by the HTTP middleware
/// route table (`rbac_middleware::routes_user::rest_routes`); this constant
/// keeps the catalog gate effective even when a deployment has not applied the
/// DB row. Keep all three in sync. Reads (GET) are deliberately absent.
const DEFAULT_DENY_ADMIN_ROUTES: &[(&str, &str)] = &[
    ("POST", "/api/billing"),
    ("PUT", "/api/billing"),
    ("DELETE", "/api/billing"),
    ("POST", "/api/products"),
    ("PUT", "/api/products"),
    ("DELETE", "/api/products"),
    ("POST", "/api/cloud/store"),
    ("POST", "/api/cloud/vouchers"),
    ("PUT", "/api/cloud/vouchers"),
    ("DELETE", "/api/cloud/vouchers"),
    ("POST", "/api/cloud/organizations"),
    ("PUT", "/api/cloud/organizations"),
    ("DELETE", "/api/cloud/organizations"),
    ("POST", "/api/ui/billing"),
    ("PUT", "/api/ui/billing"),
];

/// Whether a (method, path) endpoint is denied to non-admin users, per the
/// `rbac_api_permissions` matrix plus the code-level baseline above. Admins are
/// never denied.
pub fn is_admin_only_endpoint(pool: &DbPool, user_id: Uuid, method: &str, path: &str) -> bool {
    let role = resolve_user_role(pool, user_id);
    if role == ROLE_ADMIN {
        return false;
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            // #2632 — fail CLOSED. `true` means "admin-only", so a database
            // outage denies non-admins instead of silently disabling the
            // admin-only gate for the command catalog and every caller.
            log::error!("is_admin_only_endpoint: DB pool error, denying: {e}");
            return true;
        }
    };
    let patterns = diesel::sql_query(
        "SELECT path_pattern FROM rbac_api_permissions \
         WHERE group_name = 'admin' AND (method = '*' OR method = $1)",
    )
    .bind::<diesel::sql_types::Text, _>(method.to_uppercase())
    .load::<PatternRow>(&mut conn)
    .map(|rows| {
        rows.into_iter()
            .map(|r| r.path_pattern)
            .collect::<Vec<String>>()
    })
    .map_err(|e| e.to_string());

    admin_only_decision_for_role(patterns, &role, method, path)
}

/// Pure decision table for [`is_admin_only_endpoint`], combining the DB matrix
/// rows with the code-level baseline. `Err` is the fail-closed case: a matrix
/// lookup that could not be completed denies non-admins instead of silently
/// disabling the gate (#2632). Admins are never denied (#2671).
fn admin_only_decision_for_role(
    patterns: Result<Vec<String>, String>,
    role: &str,
    method: &str,
    path: &str,
) -> bool {
    if role == ROLE_ADMIN {
        return false;
    }
    admin_only_decision(patterns, path) || static_admin_only_route(method, path)
}

/// Pure decision table for the DB rows of the `rbac_api_permissions` matrix.
fn admin_only_decision(patterns: Result<Vec<String>, String>, path: &str) -> bool {
    match patterns {
        Ok(patterns) => patterns.iter().any(|pattern| {
            let base = pattern.trim_end_matches('%');
            path.starts_with(base)
        }),
        Err(e) => {
            log::error!("is_admin_only_endpoint: query error, denying: {e}");
            true
        }
    }
}

/// Matches the code-level default-deny baseline for a (method, path) pair.
fn static_admin_only_route(method: &str, path: &str) -> bool {
    DEFAULT_DENY_ADMIN_ROUTES
        .iter()
        .any(|(m, base)| m.eq_ignore_ascii_case(method) && path.starts_with(base))
}

#[derive(diesel::QueryableByName)]
struct PatternRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    path_pattern: String,
}

#[cfg(test)]
mod tests {
    use super::{
        admin_only_decision, admin_only_decision_for_role, static_admin_only_route,
        DEFAULT_DENY_ADMIN_ROUTES, ROLE_ADMIN, ROLE_USER,
    };

    #[test]
    fn matrix_lookup_failure_denies() {
        assert!(admin_only_decision(
            Err("pool closed".to_string()),
            "/api/admin/x"
        ));
    }

    #[test]
    fn empty_matrix_allows_non_admin_routes() {
        assert!(!admin_only_decision(Ok(Vec::new()), "/api/chat/messages"));
    }

    #[test]
    fn matching_prefix_denies() {
        let patterns = Ok(vec!["/api/billing%".to_string()]);
        assert!(admin_only_decision(patterns, "/api/billing/invoices"));
        assert!(!admin_only_decision(
            Ok(vec!["/api/billing%".to_string()]),
            "/api/chat/messages"
        ));
    }

    /// #2671 — every state-changing billing/store/voucher/organization/product
    /// prefix is denied for a non-admin even with an empty DB matrix, and stays
    /// allowed for an admin. This is the code-level mirror of migration 6.9.4.
    #[test]
    fn default_deny_prefixes_cover_both_roles() {
        const CASES: &[(&str, &str)] = &[
            ("POST", "/api/billing/invoices"),
            ("PUT", "/api/billing/invoices/abc/void"),
            ("DELETE", "/api/billing/invoices/abc"),
            ("POST", "/api/billing/payments"),
            ("POST", "/api/products/items"),
            ("PUT", "/api/products/items/abc"),
            ("DELETE", "/api/products/items/abc"),
            ("POST", "/api/cloud/store/purchase"),
            ("POST", "/api/cloud/vouchers"),
            ("PUT", "/api/cloud/vouchers/redeem"),
            ("DELETE", "/api/cloud/vouchers/redeem"),
            ("POST", "/api/cloud/organizations"),
            ("PUT", "/api/cloud/organizations/org/branches/b"),
            (
                "DELETE",
                "/api/cloud/organizations/org/workspaces/w/resources/r",
            ),
            ("POST", "/api/ui/billing/subscription/upgrade"),
            ("PUT", "/api/ui/billing/subscription/cancel"),
        ];
        for (method, path) in CASES {
            assert!(
                admin_only_decision_for_role(Ok(Vec::new()), ROLE_USER, method, path),
                "{method} {path} must be denied for a non-admin user"
            );
            assert!(
                !admin_only_decision_for_role(Ok(Vec::new()), ROLE_ADMIN, method, path),
                "{method} {path} must stay allowed for an admin"
            );
        }
    }

    /// The baseline must stay write-only: reads and unrelated surfaces remain
    /// reachable by tenant users.
    #[test]
    fn baseline_keeps_reads_and_unrelated_writes_open() {
        for path in [
            "/api/billing/invoices",
            "/api/products/items",
            "/api/cloud/store",
        ] {
            assert!(
                !static_admin_only_route("GET", path),
                "GET {path} must stay readable for tenant users"
            );
        }
        assert!(!static_admin_only_route("POST", "/api/chat/messages"));
        assert!(!admin_only_decision_for_role(
            Ok(Vec::new()),
            ROLE_USER,
            "GET",
            "/api/billing/invoices"
        ));
    }

    /// Guard the matrix itself: only POST/PUT/DELETE prefixes under /api/ may
    /// be added, so a future edit cannot accidentally lock out read traffic.
    #[test]
    fn baseline_is_write_only_api_prefixes() {
        for (method, base) in DEFAULT_DENY_ADMIN_ROUTES {
            assert!(
                matches!(*method, "POST" | "PUT" | "DELETE"),
                "{method} {base} is not a state-changing method"
            );
            assert!(base.starts_with("/api/"), "{base} is not an API prefix");
        }
    }
}
