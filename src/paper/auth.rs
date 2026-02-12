use crate::core::shared::state::AppState;
use axum::http::HeaderMap;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use super::models::{UserRow, UserIdRow};

pub async fn get_current_user(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<(Uuid, String), String> {
    let session_id = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find(|c| c.trim().starts_with("session_id="))
                        .map(|c| c.trim().trim_start_matches("session_id="))
                })
        });

    if let Some(sid) = session_id {
        if let Ok(session_uuid) = Uuid::parse_str(sid) {
            let conn = state.conn.clone();
            let result = tokio::task::spawn_blocking(move || {
                let mut db_conn = conn.get().map_err(|e| e.to_string())?;

                let user_id: Option<Uuid> =
                    diesel::sql_query("SELECT user_id FROM user_sessions WHERE id = $1")
                        .bind::<diesel::sql_types::Uuid, _>(session_uuid)
                        .get_result::<UserIdRow>(&mut db_conn)
                        .optional()
                        .map_err(|e| e.to_string())?
                        .map(|r| r.user_id);

                if let Some(uid) = user_id {
                    let user: Option<UserRow> =
                        diesel::sql_query("SELECT id, email, username FROM users WHERE id = $1")
                            .bind::<diesel::sql_types::Uuid, _>(uid)
                            .get_result(&mut db_conn)
                            .optional()
                            .map_err(|e| e.to_string())?;

                    if let Some(u) = user {
                        return Ok((u.id, u.email));
                    }
                }
                Err("User not found".to_string())
            })
            .await
            .map_err(|e| e.to_string())?;

            return result;
        }
    }

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;


        let anon_email = "anonymous@local";
        let user: Option<UserRow> = diesel::sql_query(
            "SELECT id, email, username FROM users WHERE email = $1",
        )
        .bind::<diesel::sql_types::Text, _>(anon_email)
        .get_result(&mut db_conn)
        .optional()
        .map_err(|e| e.to_string())?;

        if let Some(u) = user {
            Ok((u.id, u.email))
        } else {
            let new_id = Uuid::new_v4();
            let now = chrono::Utc::now();
            diesel::sql_query(
                "INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
                 VALUES ($1, $2, $3, '', true, $4, $4)"
            )
            .bind::<diesel::sql_types::Uuid, _>(new_id)
            .bind::<diesel::sql_types::Text, _>("anonymous")
            .bind::<diesel::sql_types::Text, _>(anon_email)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut db_conn)
            .map_err(|e| e.to_string())?;

            Ok((new_id, anon_email.to_string()))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
