//! Server-side role resolution for suite logins (#2808).
//!
//! The Zitadel session path used to mint every successfully authenticated
//! session with `roles: ["admin"]`. That handed full RBAC (`is_admin()`
//! short-circuits every permission check) to *any* directory user, so the
//! ownership of a real administrator account was only ever checked by a
//! hardcoded string.
//!
//! Roles are now derived from the database exactly like the rest of the
//! server does it: the canonical `users.is_admin` flag, or membership of an
//! explicit administrator group (`rbac_user_groups` → `rbac_groups`). A
//! database failure fails closed to `user`, never to `admin`.

use super::super::*;
use diesel::prelude::*;
use uuid::Uuid;

/// Canonical administrator group names — mirrors
/// `botsecurity::rbac_middleware::ADMIN_GROUP_NAMES` so the SQL predicate and
/// the in-process resolver cannot drift apart.
const ADMIN_GROUP_NAMES: &[&str] = &[
    "admin",
    "administrator",
    "superadmin",
    "super_admin",
    "super-admin",
    "super",
];

/// Roles minted into the session for a directory login.
///
/// Only `admin` and `user` are produced: the session cache maps unknown role
/// strings to `Role::User`, and the RBAC matrix already treats every
/// non-admin role identically for the admin-only surfaces.
pub(super) fn resolve_login_roles(
    state: &Arc<AppState>,
    user_id: &str,
    email: &str,
) -> Vec<String> {
    match db_user_is_admin(state, user_id, email) {
        Some(true) => {
            info!("Role resolved for {}: admin", email);
            vec!["admin".to_string()]
        }
        Some(false) => {
            debug!("Role resolved for {}: user", email);
            vec!["user".to_string()]
        }
        None => {
            // Unknown identity (or database unavailable) is never an admin.
            warn!(
                "Role resolution fell back to 'user' for {} — no matching active users row",
                email
            );
            vec!["user".to_string()]
        }
    }
}

#[derive(diesel::QueryableByName)]
struct AdminRow {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_admin: bool,
}

/// Whether the canonical `users` row for this identity is an administrator.
///
/// The id supplied by the directory is tried first; the row is matched by
/// email as well because Zitadel numeric ids are mapped through a stable
/// UUIDv5 by the rest of the server (`users.id` = `UUIDv5("zitadel:{id}")`).
/// `None` means "no row found or database unavailable".
fn db_user_is_admin(state: &Arc<AppState>, user_id: &str, email: &str) -> Option<bool> {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            warn!("Role resolution: database pool unavailable: {e}");
            return None;
        }
    };

    let uuid = stable_user_uuid(user_id);
    let names: Vec<String> = ADMIN_GROUP_NAMES.iter().map(|n| (*n).to_string()).collect();

    let sql = format!(
        "SELECT (COALESCE(u.is_admin, FALSE) OR EXISTS ( \
             SELECT 1 FROM rbac_user_groups ug \
             JOIN rbac_groups g ON g.id = ug.group_id \
             WHERE ug.user_id = u.id \
               AND g.is_active = TRUE \
               AND lower(btrim(g.name)) = ANY($3) \
         )) AS is_admin \
         FROM users u \
         WHERE u.is_active = TRUE \
           AND (u.id = $1 OR lower(u.email) = lower($2)) \
         ORDER BY (u.id = $1) DESC \
         LIMIT 1"
    );

    match diesel::sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(uuid)
        .bind::<diesel::sql_types::Text, _>(email)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(names)
        .get_result::<AdminRow>(&mut conn)
    {
        Ok(row) => Some(row.is_admin),
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => {
            warn!("Role resolution query failed for {email}: {e}");
            None
        }
    }
}

/// Parses the directory user id into a UUID, deriving the same stable UUIDv5
/// the rest of the server uses for numeric Zitadel ids.
fn stable_user_uuid(user_id: &str) -> Uuid {
    match Uuid::parse_str(user_id.trim()) {
        Ok(uuid) => uuid,
        Err(_) => Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", user_id.trim()).as_bytes(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use botsecurity::auth_api::{AuthenticatedUser, Role};

    #[test]
    fn non_admin_roles_never_grant_admin() {
        let roles = vec!["user".to_string()];
        assert!(!roles
            .iter()
            .any(|r| botsecurity::rbac_middleware::is_admin_group_name(r)));

        let user = AuthenticatedUser::new(Uuid::new_v4(), "user@example.com".to_string())
            .with_roles(vec![Role::User]);
        assert!(!user.is_admin(), "a non-admin login must not be is_admin()");
    }

    #[test]
    fn admin_group_names_match_the_shared_resolver() {
        for name in ADMIN_GROUP_NAMES {
            assert!(
                botsecurity::rbac_middleware::is_admin_group_name(name),
                "SQL predicate name '{name}' must also be an admin group for the shared resolver"
            );
        }
    }

    #[test]
    fn numeric_zitadel_ids_derive_a_stable_uuid() {
        let first = stable_user_uuid("380726576955786242");
        let second = stable_user_uuid("380726576955786242");
        assert_eq!(first, second);
        assert_eq!(
            first,
            Uuid::new_v5(
                &Uuid::NAMESPACE_DNS,
                b"zitadel:380726576955786242"
            )
        );
    }
}
