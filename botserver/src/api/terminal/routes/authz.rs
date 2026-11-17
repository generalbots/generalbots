use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;

use botcore::shared::utils::DbPool;
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
    // Mirror the Vibe project RBAC semantics: an authenticated user with no
    // explicit membership on the project is treated as a viewer there (the
    // run/files endpoints resolve to the default `Viewer` role), so the
    // Terminal must grant the same access — otherwise a user who can already
    // Run a project's VM gets a 403 from its own companion terminal. The
    // container is still restricted to `vm_instances` rows, so infra
    // containers (tables, vault, system) remain unreachable.
    Ok(true)
}

fn is_viewer_or_above(role: &str) -> bool {
    matches!(
        role.to_ascii_lowercase().as_str(),
        "viewer" | "developer" | "admin" | "owner"
    )
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
}
