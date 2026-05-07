use axum::http::HeaderMap;
use diesel::prelude::*;
use uuid::Uuid;

use crate::state::TasksState;

pub fn get_user_id_from_headers(
    state: &TasksState,
    headers: &HeaderMap,
) -> Result<Uuid, String> {
    let session_id = headers
        .get("x-session-id")
        .or_else(|| headers.get("cookie"))
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "No session header found".to_string())?;

    let session_id = session_id
        .split(';')
        .find(|s| s.trim().starts_with("session_id="))
        .map(|s| s.trim().strip_prefix("session_id=").unwrap_or(s).trim())
        .unwrap_or(session_id);

    let sid = session_id
        .parse::<Uuid>()
        .map_err(|_| "Invalid session ID format".to_string())?;

    use crate::schema::user_sessions::dsl::*;

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    user_sessions
        .find(sid)
        .select(user_id)
        .first::<Uuid>(&mut conn)
        .map_err(|e| format!("Session lookup error: {}", e))
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
