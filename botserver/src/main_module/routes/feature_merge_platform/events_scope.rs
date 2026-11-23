//! Verified tenant scope for the Office event admin surface (issue #2825).
//!
//! The dead-letter list/replay endpoints must never fall back to a global
//! queue: both scope ids come from a signed JWT or a verified session binding
//! and are resolved server-side, never from a request parameter.

use axum::http::HeaderMap;
use botcore::shared::state::AppState;
use botevents::office::runtime::EventTenantScope;
use diesel::prelude::*;
use uuid::Uuid;

/// Owning organization of a verified branch. Opaque suite tokens carry no
/// `org_id` claim, so the branch is resolved from the signed JWT/session and
/// its org is read from the database.
fn org_for_branch(conn: &mut diesel::PgConnection, branch_id: Uuid) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<OrgRow>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|row| row.org_id)
}

/// Resolves the event tenant strictly from verified credentials. Both ids
/// must be present and non-nil; a missing scope yields `None` and the caller
/// fails closed (no global DLQ/ledger fallback).
pub(super) fn caller_scope(headers: &HeaderMap, state: &AppState) -> Option<EventTenantScope> {
    let Ok(mut conn) = state.conn.get() else {
        return None;
    };
    let branch =
        botsecurity_core::scope::branch_from_jwt(headers, &mut conn).filter(|id| !id.is_nil())?;
    let org = botsecurity_core::tenant::org_from_claims(headers)
        .filter(|id| !id.is_nil())
        .or_else(|| org_for_branch(&mut conn, branch))
        .filter(|id| !id.is_nil())?;
    EventTenantScope::new(org.to_string(), branch.to_string())
}
