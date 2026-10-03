//! #1441 C5 — record-level authorization for destructive CRM operations.
//!
//! `/api/crm/**` is open to every authenticated tenant member, which is right
//! for reading and for capturing records but not for destroying them: any
//! signed-in user could delete another rep's deal. The rule applied here:
//!
//! * an **admin** (group containing "admin", or `users.is_admin`) may act on
//!   any record of the branch;
//! * a **plain user** may act only on records they own (`owner_id` set to
//!   their user id) or on **unowned** records (`owner_id IS NULL`) — the
//!   common case for freshly captured data, so nothing existing regresses;
//! * everything else is `403 Forbidden`.
//!
//! Ownership is resolved from the request token's actor (JWT `email` claim or
//! the suite session cache) against the `users` table, so no client input can
//! influence the decision.

use axum::http::{HeaderMap, StatusCode};
use diesel::prelude::*;
use uuid::Uuid;

use crate::scope::{email_from_jwt, email_from_session};

#[derive(Debug, QueryableByName)]
struct ActorRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_admin: bool,
}

#[derive(Debug, QueryableByName)]
struct GroupRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

/// The caller's `users` row, resolved by the token's email.
fn actor(conn: &mut diesel::PgConnection, headers: &HeaderMap) -> Option<ActorRow> {
    let email = email_from_jwt(headers).or_else(|| email_from_session(headers))?;
    diesel::sql_query(
        "SELECT id, is_admin FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(&email)
    .get_result::<ActorRow>(conn)
    .ok()
}

fn is_group_admin(conn: &mut diesel::PgConnection, user_id: Uuid) -> bool {
    diesel::sql_query(
        "SELECT g.name FROM rbac_user_groups ug \
         JOIN rbac_groups g ON g.id = ug.group_id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load::<GroupRow>(conn)
    .unwrap_or_default()
    .iter()
    .any(|g| g.name.to_lowercase().contains("admin"))
}

/// `Ok(())` when the caller may modify `record_owner`; `403` otherwise.
///
/// `record_owner = None` means the record is unowned: any authenticated member
/// of the branch may act on it (the pre-#1441 behaviour, preserved).
pub fn ensure_can_modify(
    conn: &mut diesel::PgConnection,
    headers: &HeaderMap,
    record_owner: Option<Uuid>,
) -> Result<(), (StatusCode, String)> {
    let Some(row) = actor(conn, headers) else {
        // No resolvable identity (service/loopback callers): keep the
        // branch-scope checks as the only gate.
        return Ok(());
    };
    if row.is_admin || is_group_admin(conn, row.id) {
        return Ok(());
    }
    match record_owner {
        None => Ok(()),
        Some(owner) if owner == row.id => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            "only the record owner or an admin may perform this action".to_string(),
        )),
    }
}
