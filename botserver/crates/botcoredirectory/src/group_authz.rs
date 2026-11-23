//! #2847 — handler-level authorization for directory group mutations.
//!
//! Membership in a group whose name is an exact administrator name
//! (`admin`, `superadmin`, …) resolves to `ROLE_ADMIN` in the live session.
//! The directory group endpoints used to write `rbac_groups` /
//! `rbac_user_groups` with no check of the caller's role or tenant, so any
//! principal that passed the coarse `/api/directory/**` route gate could grant
//! itself (or anyone) admin. This module keeps the decision pure — easy to
//! unit-test — and resolves the caller from the verified request principal
//! plus server-side database state only.

use axum::http::StatusCode;
use botsecurity::auth::AuthenticatedUser;
use botsecurity::rbac_middleware::is_admin_group_name;
use diesel::{OptionalExtension, RunQueryDsl};
use uuid::Uuid;

/// Caller facts gathered from the verified principal and the database.
/// Nothing here comes from a client-supplied request body.
pub(crate) struct CallerContext {
    pub is_admin: bool,
    pub org_ids: Vec<Uuid>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CallerRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_admin: bool,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct GroupNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct OrgRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
}

/// Why a group mutation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupAuthDenial {
    /// The caller is not an administrator.
    AdminRequired,
    /// The target group is built-in/admin-named (or would become one).
    AdminGroupProtected,
    /// The prospective member belongs to a different organization.
    CrossOrgMember,
}

impl GroupAuthDenial {
    pub(crate) fn response(self) -> (StatusCode, String) {
        match self {
            Self::AdminRequired => (
                StatusCode::FORBIDDEN,
                "Admin role required for group management".to_string(),
            ),
            Self::AdminGroupProtected => (
                StatusCode::FORBIDDEN,
                "Built-in administrator groups cannot be created, renamed or modified".to_string(),
            ),
            Self::CrossOrgMember => (
                StatusCode::FORBIDDEN,
                "Target user is outside your organization".to_string(),
            ),
        }
    }
}

/// Inputs to the pure authorization decision.
pub(crate) struct GroupMutation<'a> {
    pub caller_is_admin: bool,
    pub caller_org_ids: &'a [Uuid],
    /// Persisted name of the target group (`None` on create).
    pub group_name: Option<&'a str>,
    /// Requested new name (`Some` on create/rename).
    pub proposed_name: Option<&'a str>,
    /// Tenant memberships of the user being added/removed (`None` otherwise).
    pub member_target_org_ids: Option<&'a [Uuid]>,
}

/// The pure #2847 decision. Every group mutation must pass this before it
/// touches `rbac_groups` or `rbac_user_groups`.
///
/// Rules:
/// 1. Only administrators may mutate groups. This also covers self-assignment
///    of any group by a non-admin: the caller is rejected before membership is
///    written.
/// 2. Admin-named groups cannot be created, renamed into, or modified — their
///    name and `is_active` flag feed role resolution directly.
/// 3. A member add/remove must stay inside the caller's verified tenants. A
///    nil org id in the caller's set denotes global scope; absent bindings on
///    either side mean there is no tenant boundary to cross.
pub(crate) fn authorize_group_mutation(m: &GroupMutation<'_>) -> Result<(), GroupAuthDenial> {
    if !m.caller_is_admin {
        return Err(GroupAuthDenial::AdminRequired);
    }
    if matches!(m.group_name, Some(name) if is_admin_group_name(name))
        || matches!(m.proposed_name, Some(name) if is_admin_group_name(name))
    {
        return Err(GroupAuthDenial::AdminGroupProtected);
    }
    if let Some(target_orgs) = m.member_target_org_ids {
        let caller_global = m.caller_org_ids.iter().any(|id| id.is_nil());
        let unscoped = m.caller_org_ids.is_empty() || target_orgs.is_empty();
        let shares_org = target_orgs.iter().any(|id| m.caller_org_ids.contains(id));
        if !caller_global && !unscoped && !shares_org {
            return Err(GroupAuthDenial::CrossOrgMember);
        }
    }
    Ok(())
}

/// Resolves the caller's admin status and org memberships from the principal
/// inserted by the auth middleware (verified JWT/session) and the database.
/// Both `users.is_admin` and active admin-named group membership are honoured
/// so a stale role on the token alone cannot drive this decision.
pub(crate) fn resolve_caller(
    conn: &mut diesel::PgConnection,
    user: &AuthenticatedUser,
) -> CallerContext {
    let by_id: Option<CallerRow> = if user.user_id == Uuid::nil() {
        None
    } else {
        match diesel::sql_query("SELECT id, is_admin FROM users WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(user.user_id)
            .get_result::<CallerRow>(conn)
            .optional()
        {
            Ok(row) => row,
            Err(e) => {
                log::error!("group authz: caller lookup by id failed: {e}");
                None
            }
        }
    };

    let row: Option<CallerRow> = match by_id {
        Some(row) => Some(row),
        None => match user.email.as_deref().filter(|email| !email.is_empty()) {
            Some(email) => {
                match diesel::sql_query(
                    "SELECT id, is_admin FROM users \
                     WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
                )
                .bind::<diesel::sql_types::Text, _>(email)
                .get_result::<CallerRow>(conn)
                .optional()
                {
                    Ok(row) => row,
                    Err(e) => {
                        log::error!("group authz: caller lookup by email failed: {e}");
                        None
                    }
                }
            }
            None => None,
        },
    };

    let canonical_id = row.as_ref().map(|r| r.id);
    let db_admin = matches!(&row, Some(r) if r.is_admin);

    let group_admin = match canonical_id {
        Some(id) => {
            let names: Vec<GroupNameRow> = match diesel::sql_query(
                "SELECT g.name FROM rbac_user_groups ug \
                 JOIN rbac_groups g ON g.id = ug.group_id \
                 WHERE ug.user_id = $1 AND g.is_active = true",
            )
            .bind::<diesel::sql_types::Uuid, _>(id)
            .load::<GroupNameRow>(conn)
            {
                Ok(names) => names,
                Err(e) => {
                    log::error!("group authz: admin group membership lookup failed: {e}");
                    Vec::new()
                }
            };
            names.iter().any(|r| is_admin_group_name(&r.name))
        }
        None => false,
    };

    let mut org_ids: Vec<Uuid> = match canonical_id {
        Some(id) => {
            match diesel::sql_query("SELECT org_id FROM user_organizations WHERE user_id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .load::<OrgRow>(conn)
            {
                Ok(rows) => rows.into_iter().map(|r| r.org_id).collect(),
                Err(e) => {
                    log::error!("group authz: caller org lookup failed: {e}");
                    Vec::new()
                }
            }
        }
        None => Vec::new(),
    };
    if let Some(id) = user.organization_id {
        if !org_ids.contains(&id) {
            org_ids.push(id);
        }
    }

    CallerContext {
        is_admin: user.is_admin() || db_admin || group_admin,
        org_ids,
    }
}

/// Current persisted name of a group, if it exists.
pub(crate) fn load_group_name(conn: &mut diesel::PgConnection, group_id: Uuid) -> Option<String> {
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct NameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    match diesel::sql_query("SELECT name FROM rbac_groups WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(group_id)
        .get_result::<NameRow>(conn)
        .optional()
    {
        Ok(row) => row.map(|r| r.name),
        Err(e) => {
            log::error!("group authz: group lookup failed for {group_id}: {e}");
            None
        }
    }
}

/// Org memberships of a user, used to reject cross-tenant member adds.
pub(crate) fn load_user_org_ids(conn: &mut diesel::PgConnection, user_id: Uuid) -> Vec<Uuid> {
    match diesel::sql_query("SELECT org_id FROM user_organizations WHERE user_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<OrgRow>(conn)
    {
        Ok(rows) => rows.into_iter().map(|r| r.org_id).collect(),
        Err(e) => {
            log::error!("group authz: target org lookup failed for {user_id}: {e}");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{authorize_group_mutation, GroupAuthDenial, GroupMutation};
    use uuid::Uuid;

    fn admin_mutation<'a>() -> GroupMutation<'a> {
        GroupMutation {
            caller_is_admin: true,
            caller_org_ids: &[],
            group_name: None,
            proposed_name: None,
            member_target_org_ids: None,
        }
    }

    #[test]
    fn non_admin_cannot_mutate_any_group() {
        let mut m = admin_mutation();
        m.caller_is_admin = false;
        assert_eq!(
            authorize_group_mutation(&m),
            Err(GroupAuthDenial::AdminRequired)
        );
    }

    #[test]
    fn admin_named_group_cannot_be_created_or_renamed_into() {
        for proposed in ["admin", "Admin", "superadmin", " SUPER-ADMIN "] {
            let mut m = admin_mutation();
            m.proposed_name = Some(proposed);
            assert_eq!(
                authorize_group_mutation(&m),
                Err(GroupAuthDenial::AdminGroupProtected),
                "creating '{proposed}' must be refused"
            );
        }
        let mut m = admin_mutation();
        m.group_name = Some("Administrator");
        m.proposed_name = Some("support");
        assert_eq!(
            authorize_group_mutation(&m),
            Err(GroupAuthDenial::AdminGroupProtected),
            "an existing admin-named group must not be renamed either"
        );
    }

    #[test]
    fn ordinary_admin_group_update_is_allowed() {
        let mut m = admin_mutation();
        m.group_name = Some("support-team");
        m.proposed_name = Some("helpdesk");
        assert!(authorize_group_mutation(&m).is_ok());
    }

    #[test]
    fn foreign_org_member_add_is_rejected() {
        let caller_org = Uuid::new_v4();
        let foreign_org = Uuid::new_v4();
        let target_orgs = [foreign_org];
        let caller_orgs = [caller_org];
        let mut m = admin_mutation();
        m.caller_org_ids = &caller_orgs;
        m.member_target_org_ids = Some(&target_orgs);
        assert_eq!(
            authorize_group_mutation(&m),
            Err(GroupAuthDenial::CrossOrgMember)
        );
    }

    #[test]
    fn same_org_member_add_is_allowed() {
        let org = Uuid::new_v4();
        let caller_orgs = [org];
        let target_orgs = [org];
        let mut m = admin_mutation();
        m.caller_org_ids = &caller_orgs;
        m.member_target_org_ids = Some(&target_orgs);
        assert!(authorize_group_mutation(&m).is_ok());
    }

    #[test]
    fn global_admin_can_add_cross_org_member() {
        let nil = Uuid::nil();
        let caller_orgs = [nil];
        let target_orgs = [Uuid::new_v4()];
        let mut m = admin_mutation();
        m.caller_org_ids = &caller_orgs;
        m.member_target_org_ids = Some(&target_orgs);
        assert!(authorize_group_mutation(&m).is_ok());
    }
}
