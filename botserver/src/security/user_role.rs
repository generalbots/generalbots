use botcore::shared::schema::rbac_groups;
use botcore::shared::schema::rbac_user_groups;
use botcore::shared::utils::DbPool;
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
        .unwrap_or_default();

    if group_ids.is_empty() {
        debug!("resolve_user_role: user {user_id} has no groups, role=user");
        return ROLE_USER.to_string();
    }

    let group_names: Vec<String> = rbac_groups::table
        .filter(rbac_groups::id.eq_any(&group_ids))
        .filter(rbac_groups::is_active.eq(true))
        .select(rbac_groups::name)
        .load::<String>(&mut conn)
        .unwrap_or_default();

    for name in &group_names {
        if name.to_lowercase().contains("admin") {
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

/// Whether a (method, path) endpoint is denied to non-admin users, per the
/// `rbac_api_permissions` matrix. Admins are never denied.
pub fn is_admin_only_endpoint(pool: &DbPool, user_id: Uuid, method: &str, path: &str) -> bool {
    if resolve_user_role(pool, user_id) == ROLE_ADMIN {
        return false;
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            debug!("is_admin_only_endpoint: DB pool error: {e}");
            return false;
        }
    };
    let patterns: Vec<String> = match diesel::sql_query(
        "SELECT path_pattern FROM rbac_api_permissions \
         WHERE group_name = 'admin' AND (method = '*' OR method = $1)",
    )
    .bind::<diesel::sql_types::Text, _>(method.to_uppercase())
    .load::<PatternRow>(&mut conn)
    {
        Ok(rows) => rows.into_iter().map(|r| r.path_pattern).collect(),
        Err(e) => {
            debug!("is_admin_only_endpoint: query error: {e}");
            return false;
        }
    };
    for pattern in &patterns {
        let base = pattern.trim_end_matches('%');
        if path.starts_with(base) {
            return true;
        }
    }
    false
}

#[derive(diesel::QueryableByName)]
struct PatternRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    path_pattern: String,
}
