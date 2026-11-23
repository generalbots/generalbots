use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;

use botcore::shared::utils::DbPool;
use crate::security::auth_api::types::AuthenticatedUser;
use uuid::Uuid;

/// SECURITY gate for container attachment: the requested container must be a
/// `vm_instances` row for a project the caller can access (direct or group
/// membership with at least the `viewer` role).
pub(super) fn authorize_project_container(
    pool: &DbPool,
    user_id: Uuid,
    container: &str,
) -> Result<(), String> {
    use diesel::prelude::*;
    if container.is_empty()
        || !container
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err("Invalid container name".to_string());
    }
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct UuidCell {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        value: Uuid,
    }
    let project_id: Option<UuidCell> =
        diesel::sql_query("SELECT project_id AS value FROM vm_instances WHERE container_name = $1")
            .bind::<diesel::sql_types::Text, _>(container)
            .get_result::<UuidCell>(&mut conn)
            .optional()
            .map_err(|e| format!("vm lookup: {e}"))?;
    let project_id = match project_id {
        Some(row) => row.value,
        None => {
            return Err(format!(
                "container '{container}' is not a project VM — only project VMs can be attached"
            ));
        }
    };
    if !project_role_at_least_viewer(&mut conn, user_id, project_id)? {
        return Err("you do not have access to this project's VM".to_string());
    }
    Ok(())
}

/// Resolves whether the caller holds at least the `viewer` role on the
/// project that owns the VM (direct membership or via an active group).
fn project_role_at_least_viewer(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<bool, String> {
    use diesel::prelude::*;
    if user_id == Uuid::nil() {
        return Ok(false);
    }
    #[derive(diesel::QueryableByName)]
    struct RoleCell {
        #[diesel(sql_type = diesel::sql_types::Text)]
        value: String,
    }
    let direct: Option<RoleCell> = diesel::sql_query(
        "SELECT role AS value FROM project_members WHERE project_id = $1 AND user_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(project_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<RoleCell>(conn)
    .optional()
    .map_err(|e| format!("project role lookup: {e}"))?;
    if let Some(role) = direct {
        return Ok(is_viewer_or_above(&role.value));
    }
    let groups: Vec<RoleCell> = diesel::sql_query(
        "SELECT g.name AS value FROM rbac_user_groups ug \
         JOIN rbac_groups g ON g.id = ug.group_id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load::<RoleCell>(conn)
    .map_err(|e| format!("user groups lookup: {e}"))?;
    for group in groups {
        let group_role: Option<RoleCell> = diesel::sql_query(
            "SELECT role AS value FROM project_members \
             WHERE project_id = $1 AND group_name = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(&group.value)
        .get_result::<RoleCell>(conn)
        .optional()
        .map_err(|e| format!("group role lookup: {e}"))?;
        if let Some(role) = group_role {
            if is_viewer_or_above(&role.value) {
                return Ok(true);
            }
        }
    }
    // #2730 — fail closed. An authenticated user with no membership (direct or
    // via an active group) on the project that owns the VM gets no terminal
    // access. The previous `Ok(true)` made every project VM in every tenant
    // reachable by any signed-in user.
    Ok(false)
}

fn is_viewer_or_above(role: &str) -> bool {
    matches!(
        role.to_ascii_lowercase().as_str(),
        "viewer" | "developer" | "admin" | "owner"
    )
}

/// Exact user + tenant scope captured when a PTY session is created.
///
/// The registry is process-wide, so scope — not obscurity of the session id —
/// is what keeps one tenant's terminal out of another tenant's hands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionScope {
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
}

impl SessionScope {
    pub(crate) fn new(user_id: Uuid, tenant_id: Option<Uuid>) -> Self {
        Self { user_id, tenant_id }
    }

    pub(crate) fn of(user: &AuthenticatedUser) -> Self {
        Self::new(user.user_id, user.organization_id)
    }

    /// True only for the exact principal the session was created for. A
    /// missing tenant never matches a present one, so a session cannot cross a
    /// tenant boundary. Nil ids match too: single-user desktop installs use
    /// the nil principal on purpose, and the per-session capability still
    /// gates attachment there.
    pub(crate) fn matches(&self, user: &AuthenticatedUser) -> bool {
        self.user_id == user.user_id && self.tenant_id == user.organization_id
    }
}

/// Generate an unguessable per-session capability (256 bits of CSPRNG output,
/// hex encoded). The value lives only in the in-memory session and in the
/// create/list response returned to the owner; knowledge of it is the WS
/// handshake credential.
pub(crate) fn new_capability() -> String {
    hex::encode(rand::random::<[u8; 32]>())
}

/// Constant-time comparison of two capabilities. Equal-length tokens are
/// compared without an early exit so timing cannot reveal a matching prefix.
pub(crate) fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Denied session access (bad capability or wrong user/tenant). Deliberately
/// opaque so callers cannot distinguish "unknown id" from "not yours".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionAccessDenied;

/// Full access check for an existing session: the caller must be the exact
/// owner scope AND present the unguessable capability. Any failure is a
/// cross-user/cross-tenant hijack attempt and maps to 403 / WS close.
pub(crate) fn verify_session_access(
    scope: &SessionScope,
    capability: &str,
    user: Option<&AuthenticatedUser>,
    provided: &str,
) -> Result<(), SessionAccessDenied> {
    if provided.is_empty() || !constant_time_eq(capability, provided) {
        return Err(SessionAccessDenied);
    }
    // When the WS reaches botserver directly the auth middleware supplies the
    // principal; enforce scope as well. The BotUI WS proxy cannot forward
    // headers, so in that case the capability alone is the credential — it was
    // only ever handed to the owner.
    if let Some(user) = user {
        if !scope.matches(user) {
            return Err(SessionAccessDenied);
        }
    }
    Ok(())
}

/// Parses an xterm.js resize control message: `resize <cols> <rows>`.
pub(super) fn parse_resize(text: &str) -> Option<(u16, u16)> {
    let rest = text.strip_prefix("resize ")?;
    let mut parts = rest.split_whitespace();
    let cols: u16 = parts.next()?.parse().ok()?;
    let rows: u16 = parts.next()?.parse().ok()?;
    Some((cols, rows))
}

pub(super) async fn send_history(
    session: &Arc<super::super::TerminalSession>,
    socket: &mut WebSocket,
) {
    for line in session.history() {
        let payload = serde_json::json!({ "type": "output", "data": line.data });
        if socket
            .send(Message::Text(payload.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resize_accepts_valid_and_rejects_garbage() {
        assert_eq!(parse_resize("resize 120 40"), Some((120, 40)));
        assert_eq!(parse_resize("resize 0 0"), Some((0, 0)));
        assert_eq!(parse_resize("resize nope 40"), None);
        assert_eq!(parse_resize("echo hi"), None);
    }

    #[test]
    fn capability_is_256_bits_and_unique() {
        let a = new_capability();
        let b = new_capability();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn constant_time_eq_rejects_wrong_or_empty_capability() {
        let cap = "0123456789abcdef".repeat(4);
        assert!(constant_time_eq(&cap, &cap.clone()));
        assert!(!constant_time_eq(&cap, ""));
        assert!(!constant_time_eq(&cap, &cap[..63]));
        let mut wrong = cap.clone();
        wrong.replace_range(0..1, "f");
        if wrong == cap {
            wrong.replace_range(0..1, "0");
        }
        assert!(!constant_time_eq(&cap, &wrong));
    }

    #[test]
    fn session_scope_matches_only_owner_user_and_tenant() {
        let owner = AuthenticatedUser::new(Uuid::new_v4(), "owner".to_string())
            .with_organization(Uuid::new_v4());
        let scope = SessionScope::of(&owner);
        assert!(scope.matches(&owner));

        let mut other_user = owner.clone();
        other_user.user_id = Uuid::new_v4();
        assert!(!scope.matches(&other_user));

        let mut other_tenant = owner.clone();
        other_tenant.organization_id = Some(Uuid::new_v4());
        assert!(!scope.matches(&other_tenant));

        let mut no_tenant = owner.clone();
        no_tenant.organization_id = None;
        assert!(!scope.matches(&no_tenant));
    }

    #[test]
    fn verify_session_access_checks_capability_and_optional_scope() {
        let owner = AuthenticatedUser::new(Uuid::new_v4(), "owner".to_string())
            .with_organization(Uuid::new_v4());
        let scope = SessionScope::of(&owner);
        let cap = new_capability();

        // Capability alone (the BotUI WS proxy path) is sufficient.
        assert!(verify_session_access(&scope, &cap, None, &cap).is_ok());
        // Direct calls additionally enforce the owner scope.
        assert!(verify_session_access(&scope, &cap, Some(&owner), &cap).is_ok());
        // Wrong capability is always rejected.
        assert!(verify_session_access(&scope, &cap, Some(&owner), "nope").is_err());
        assert!(verify_session_access(&scope, &cap, None, "").is_err());
        // Cross-user / cross-tenant with a valid capability is rejected 403.
        let mut intruder = owner.clone();
        intruder.user_id = Uuid::new_v4();
        assert!(verify_session_access(&scope, &cap, Some(&intruder), &cap).is_err());
        let mut other_tenant = owner.clone();
        other_tenant.organization_id = Some(Uuid::new_v4());
        assert!(verify_session_access(&scope, &cap, Some(&other_tenant), &cap).is_err());
    }
}
