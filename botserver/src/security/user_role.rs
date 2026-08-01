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
