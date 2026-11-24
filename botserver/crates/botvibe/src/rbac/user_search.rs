//! Scoped users typeahead for the Vibe project members picker (#2769).
//!
//! `users` is a global directory table with no org/branch column, so the
//! tenant boundary is the active RBAC group membership (`rbac_user_groups`),
//! the same signal `org_role` uses to decide org-admin. Every row returned by
//! the search must belong to the caller's tenant; platform admins are the
//! explicit exception. No unscoped user enumeration remains on this surface.

use diesel::prelude::*;
use uuid::Uuid;

use super::ProjectRbac;

/// Hard ceiling for the typeahead so a short or wildcard query can never dump
/// the directory, regardless of what a future caller passes as `limit`.
pub(crate) const MAX_USER_SEARCH_RESULTS: i64 = 50;

/// Clamp a requested result count to `1..=MAX_USER_SEARCH_RESULTS`.
pub(crate) fn cap_search_limit(requested: i64) -> i64 {
    if requested <= 0 {
        return 1;
    }
    requested.min(MAX_USER_SEARCH_RESULTS)
}

/// Pure tenant predicate, re-applied to every row the scoped SQL returns.
///
/// `same_tenant` is computed in SQL as "caller is a platform admin, or caller
/// and candidate share an active RBAC group". A caller always sees themselves.
/// This is the single place the tenant rule is expressed in Rust so the
/// two-tenant behaviour can be unit-tested without a database.
pub(crate) fn tenant_scoped(caller_id: Uuid, candidate_id: Uuid, same_tenant: bool) -> bool {
    candidate_id == caller_id || same_tenant
}

impl ProjectRbac {
    /// Search active users by username/email, restricted to the caller's
    /// tenant. The LIKE pattern is escaped and bound as a parameter, the tenant
    /// scope is evaluated in SQL (so foreign users are never fetched), results
    /// are ordered and capped, and `tenant_scoped` is re-checked on each row.
    pub fn search_users_scoped(
        &self,
        query: &str,
        limit: i64,
        caller_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, String> {
        let mut conn = self.conn()?;
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            username: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            email: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            same_tenant: bool,
        }

        // Escape LIKE wildcards (and the escape character itself) before
        // wrapping in '%' so a typed '%' or '_' cannot broaden the search.
        let escaped = query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let like = format!("%{escaped}%");

        let rows = diesel::sql_query(
            "WITH matched AS ( \
               SELECT u.id, u.username, u.email, \
                      (u.id = $2 \
                       OR EXISTS (SELECT 1 FROM users c \
                                  WHERE c.id = $2 AND c.is_admin = true) \
                       OR EXISTS ( \
                            SELECT 1 FROM rbac_user_groups caller_ug \
                            JOIN rbac_user_groups other_ug \
                              ON other_ug.group_id = caller_ug.group_id \
                            JOIN rbac_groups g ON g.id = other_ug.group_id \
                            WHERE caller_ug.user_id = $2 \
                              AND other_ug.user_id = u.id \
                              AND g.is_active = true \
                       )) AS same_tenant \
               FROM users u \
               WHERE u.is_active = true \
                 AND (u.username ILIKE $1 OR u.email ILIKE $1) \
             ) \
             SELECT id, username, email, same_tenant FROM matched \
             WHERE same_tenant \
             ORDER BY username LIMIT $3",
        )
        .bind::<diesel::sql_types::Text, _>(&like)
        .bind::<diesel::sql_types::Uuid, _>(caller_id)
        .bind::<diesel::sql_types::BigInt, _>(cap_search_limit(limit))
        .load::<Row>(&mut conn)
        .map_err(|e| format!("search users: {e}"))?;

        Ok(rows
            .into_iter()
            .filter(|r| tenant_scoped(caller_id, r.id, r.same_tenant))
            .map(|r| (r.id, r.username, r.email))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant_user(tenant: u128, member: u128) -> Uuid {
        Uuid::from_u128(tenant * 100 + member)
    }

    #[test]
    fn search_limit_is_positive_and_capped() {
        assert_eq!(cap_search_limit(20), 20);
        assert_eq!(cap_search_limit(0), 1);
        assert_eq!(cap_search_limit(-5), 1);
        assert_eq!(cap_search_limit(10_000), MAX_USER_SEARCH_RESULTS);
    }

    #[test]
    fn two_tenant_search_excludes_foreign_user() {
        let caller = tenant_user(1, 1);
        let tenant_a_peer = tenant_user(1, 2);
        let tenant_b_peer = tenant_user(2, 1);

        // SQL marks tenant A's peer as sharing the caller's group.
        assert!(tenant_scoped(caller, tenant_a_peer, true));
        // Tenant B's peer shares no group and is not the caller: hidden.
        assert!(!tenant_scoped(caller, tenant_b_peer, false));
        // Even a caller with no group membership always sees themselves.
        assert!(tenant_scoped(caller, caller, false));
    }

    #[test]
    fn admin_scope_flag_controls_cross_tenant_visibility() {
        let caller = tenant_user(1, 1);
        let foreign = tenant_user(2, 1);
        assert!(!tenant_scoped(caller, foreign, false));
        assert!(tenant_scoped(caller, foreign, true));
    }
}
