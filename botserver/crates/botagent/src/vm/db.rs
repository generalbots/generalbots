//! `agent_sessions` row helpers shared by the VM lifecycle and workspace code.

use axum::http::StatusCode;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::AgentSessionRow;
use crate::schema::agent_sessions;

use super::{internal, STATUS_STOPPED};

// ─── DB helpers ────────────────────────────────────────────────────────────

pub(super) fn load_session_by_user(
    conn: &mut diesel::PgConnection,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::user_id.eq(user_id))
        .filter(dsl::status.ne(STATUS_STOPPED))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select by user: {e}")))
}

pub(super) fn load_session(
    conn: &mut diesel::PgConnection,
    session_id: &str,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::session_id.eq(session_id))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))
}

pub(super) fn touch_last_active(conn: &mut diesel::PgConnection, row_id: Uuid) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::last_active_at.eq(Utc::now()))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions touch: {e}")))?;
    Ok(())
}

pub(super) fn set_status(conn: &mut diesel::PgConnection, row_id: Uuid, status_value: &str) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::status.eq(status_value))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set status: {e}")))?;
    Ok(())
}

pub(super) fn set_active_conversation(
    conn: &mut diesel::PgConnection,
    row_id: Uuid,
    conversation_id: &str,
) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set((
            dsl::active_conversation_id.eq(Some(conversation_id)),
            dsl::last_active_at.eq(Utc::now()),
        ))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set conversation: {e}")))?;
    Ok(())
}
