use crate::bot::BotOrchestrator;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use log::{error, trace, warn};
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionData {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub data: String,
}

pub struct SessionManager {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    sessions: HashMap<Uuid, SessionData>,
    waiting_for_input: HashSet<Uuid>,
    redis: Option<Arc<Client>>,
}

impl std::fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("conn", &"PooledConnection<PgConnection>")
            .field("sessions", &self.sessions)
            .field("waiting_for_input", &self.waiting_for_input)
            .field("redis", &self.redis.is_some())
            .finish()
    }
}

impl SessionManager {
    pub fn new(
        conn: PooledConnection<ConnectionManager<PgConnection>>,
        redis_client: Option<Arc<Client>>,
    ) -> Self {
        SessionManager {
            conn,
            sessions: HashMap::new(),
            waiting_for_input: HashSet::new(),
            redis: redis_client,
        }
    }

    pub fn provide_input(
        &mut self,
        session_id: Uuid,
        input: String,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        trace!(
            "SessionManager.provide_input called for session {}",
            session_id
        );
        if let Some(sess) = self.sessions.get_mut(&session_id) {
            sess.data = input;
            self.waiting_for_input.remove(&session_id);
            Ok(Some("user_input".to_string()))
        } else {
            let sess = SessionData {
                id: session_id,
                user_id: None,
                data: input,
            };
            self.sessions.insert(session_id, sess);
            self.waiting_for_input.remove(&session_id);
            Ok(Some("user_input".to_string()))
        }
    }

    pub fn mark_waiting(&mut self, session_id: Uuid) {
        self.waiting_for_input.insert(session_id);
    }

    pub fn get_session_by_id(
        &mut self,
        session_id: Uuid,
    ) -> Result<Option<UserSession>, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;
        let result = user_sessions
            .filter(id.eq(session_id))
            .first::<UserSession>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_user_session(
        &mut self,
        uid: Uuid,
        bid: Uuid,
    ) -> Result<Option<UserSession>, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;
        let result = user_sessions
            .filter(user_id.eq(uid))
            .filter(bot_id.eq(bid))
            .order(created_at.desc())
            .first::<UserSession>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_or_create_user_session(
        &mut self,
        uid: Uuid,
        bid: Uuid,
        session_title: &str,
    ) -> Result<Option<UserSession>, Box<dyn Error + Send + Sync>> {
        if let Some(existing) = self.get_user_session(uid, bid)? {
            return Ok(Some(existing));
        }
        self.create_session(uid, bid, session_title).map(Some)
    }

    pub fn get_or_create_anonymous_user(
        &mut self,
        uid: Option<Uuid>,
    ) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::users::dsl as users_dsl;
        let user_id = uid.unwrap_or_else(Uuid::new_v4);
        let user_exists: Option<Uuid> = users_dsl::users
            .filter(users_dsl::id.eq(user_id))
            .select(users_dsl::id)
            .first(&mut self.conn)
            .optional()?;
        if user_exists.is_none() {
            let now = Utc::now();
            diesel::insert_into(users_dsl::users)
                .values((
                    users_dsl::id.eq(user_id),
                    users_dsl::username.eq(format!("guest_{}", &user_id.to_string()[..8])),
                    users_dsl::email.eq(format!(
                        "guest_{}@anonymous.local",
                        &user_id.to_string()[..8]
                    )),
                    users_dsl::password_hash.eq(""),
                    users_dsl::is_active.eq(true),
                    users_dsl::created_at.eq(now),
                    users_dsl::updated_at.eq(now),
                ))
                .execute(&mut self.conn)?;
        }
        Ok(user_id)
    }

    pub fn create_session(
        &mut self,
        uid: Uuid,
        bid: Uuid,
        session_title: &str,
    ) -> Result<UserSession, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;
        let verified_uid = self.get_or_create_anonymous_user(Some(uid))?;
        let now = Utc::now();
        let inserted: UserSession = diesel::insert_into(user_sessions)
            .values((
                id.eq(Uuid::new_v4()),
                user_id.eq(verified_uid),
                bot_id.eq(bid),
                title.eq(session_title),
                context_data.eq(serde_json::json!({})),
                current_tool.eq(None::<String>),
                created_at.eq(now),
                updated_at.eq(now),
            ))
            .returning(UserSession::as_returning())
            .get_result(&mut self.conn)
            .map_err(|e| {
                error!("Failed to create session in database: {}", e);
                e
            })?;
        Ok(inserted)
    }

    fn _clear_messages(&mut self, _session_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        use crate::shared::models::message_history::dsl::*;
        diesel::delete(message_history.filter(session_id.eq(session_id)))
            .execute(&mut self.conn)?;
        Ok(())
    }

    pub fn save_message(
        &mut self,
        sess_id: Uuid,
        uid: Uuid,
        ro: i32,
        content: &str,
        msg_type: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use crate::shared::models::message_history::dsl::*;
        let next_index = message_history
            .filter(session_id.eq(sess_id))
            .count()
            .get_result::<i64>(&mut self.conn)
            .unwrap_or(0);
        diesel::insert_into(message_history)
            .values((
                id.eq(Uuid::new_v4()),
                session_id.eq(sess_id),
                user_id.eq(uid),
                role.eq(ro),
                content_encrypted.eq(content),
                message_type.eq(msg_type),
                message_index.eq(next_index),
                created_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut self.conn)?;
        trace!(
            "Message saved for session {} with index {}",
            sess_id,
            next_index
        );
        Ok(())
    }

    pub async fn update_session_context(
        &mut self,
        session_id: &Uuid,
        user_id: &Uuid,
        context_data: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use redis::Commands;
        let redis_key = format!("context:{}:{}", user_id, session_id);
        if let Some(redis_client) = &self.redis {
            let mut conn = redis_client.get_connection()?;
            conn.set::<_, _, ()>(&redis_key, &context_data)?;
        } else {
            warn!("No Redis client configured, context not persisted");
        }
        Ok(())
    }

    pub async fn get_session_context_data(
        &self,
        session_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        use redis::Commands;
        let base_key = format!("context:{}:{}", user_id, session_id);
        if let Some(redis_client) = &self.redis {
            let conn_option = redis_client
                .get_connection()
                .map_err(|e| {
                    warn!("Failed to get Cache connection: {}", e);
                    e
                })
                .ok();
            if let Some(mut connection) = conn_option {
                match connection.get::<_, Option<String>>(&base_key) {
                    Ok(Some(context_name)) => {
                        let full_key =
                            format!("context:{}:{}:{}", user_id, session_id, context_name);
                        match connection.get::<_, Option<String>>(&full_key) {
                            Ok(Some(context_value)) => {
                                trace!(
                                    "Retrieved context value from Cache for key {}: {} chars",
                                    full_key,
                                    context_value.len()
                                );
                                return Ok(context_value);
                            }
                            Ok(None) => {
                                trace!("No context value found for key: {}", full_key);
                            }
                            Err(e) => {
                                warn!("Failed to retrieve context value from Cache: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        trace!("No context name found for key: {}", base_key);
                    }
                    Err(e) => {
                        warn!("Failed to retrieve context name from Cache: {}", e);
                    }
                }
            }
        }
        Ok(String::new())
    }

    pub fn get_conversation_history(
        &mut self,
        sess_id: Uuid,
        _uid: Uuid,
    ) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::message_history::dsl::*;
        let messages = message_history
            .filter(session_id.eq(sess_id))
            .order(message_index.asc())
            .select((role, content_encrypted))
            .load::<(i32, String)>(&mut self.conn)?;
        let mut history: Vec<(String, String)> = Vec::new();
        for (other_role, content) in messages {
            let role_str = match other_role {
                1 => "user".to_string(),
                2 => "assistant".to_string(),
                3 => "system".to_string(),
                9 => "compact".to_string(),
                _ => "unknown".to_string(),
            };
            history.push((role_str, content));
        }
        Ok(history)
    }

    pub fn get_user_sessions(
        &mut self,
        uid: Uuid,
    ) -> Result<Vec<UserSession>, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;

        // Try to query sessions, return empty vec if database error
        let sessions = if uid == Uuid::nil() {
            user_sessions
                .order(created_at.desc())
                .load::<UserSession>(&mut self.conn)
                .unwrap_or_else(|_| Vec::new())
        } else {
            user_sessions
                .filter(user_id.eq(uid))
                .order(created_at.desc())
                .load::<UserSession>(&mut self.conn)
                .unwrap_or_else(|_| Vec::new())
        };
        Ok(sessions)
    }

    pub fn update_user_id(
        &mut self,
        session_id: Uuid,
        new_user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;
        let updated_count = diesel::update(user_sessions.filter(id.eq(session_id)))
            .set((user_id.eq(new_user_id), updated_at.eq(chrono::Utc::now())))
            .execute(&mut self.conn)?;
        if updated_count == 0 {
            warn!("No session found with ID: {}", session_id);
        } else {
            trace!("Updated user ID for session: {}", session_id);
        }
        Ok(())
    }

    /// Get count of active sessions (for analytics)
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get total count of sessions from database
    pub fn total_count(&mut self) -> usize {
        use crate::shared::models::user_sessions::dsl::*;
        user_sessions
            .count()
            .first::<i64>(&mut self.conn)
            .unwrap_or(0) as usize
    }

    /// Get sessions created in the last N hours
    pub fn recent_sessions(
        &mut self,
        hours: i64,
    ) -> Result<Vec<UserSession>, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;
        let since = chrono::Utc::now() - chrono::Duration::hours(hours);
        let sessions = user_sessions
            .filter(created_at.gt(since))
            .order(created_at.desc())
            .load::<UserSession>(&mut self.conn)?;
        Ok(sessions)
    }

    /// Get session statistics for analytics
    pub fn get_statistics(&mut self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        use crate::shared::models::user_sessions::dsl::*;

        let total = user_sessions.count().first::<i64>(&mut self.conn)?;

        let active = self.sessions.len() as i64;

        let today = chrono::Utc::now().date_naive();
        let today_start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();

        let today_count = user_sessions
            .filter(created_at.ge(today_start))
            .count()
            .first::<i64>(&mut self.conn)?;

        Ok(serde_json::json!({
            "total_sessions": total,
            "active_sessions": active,
            "today_sessions": today_count,
            "waiting_for_input": self.waiting_for_input.len()
        }))
    }
}

/* Axum handlers */

/// Create a new session (anonymous user)
pub async fn create_session(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    // Always create a session, even without database
    let temp_session_id = Uuid::new_v4();

    // Try to create in database if available
    if state.conn.get().is_ok() {
        // Using a fixed anonymous user ID for simplicity
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let bot_id = Uuid::nil();

        let _session_result = {
            let mut sm = state.session_manager.lock().await;
            // Try to create, but don't fail if database has issues
            match sm.get_or_create_user_session(user_id, bot_id, "New Conversation") {
                Ok(Some(session)) => {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "session_id": session.id,
                            "title": "New Conversation",
                            "created_at": Utc::now()
                        })),
                    );
                }
                _ => {
                    // Fall through to temporary session
                }
            }
        };
    }

    // Return temporary session if database is unavailable or has errors
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "session_id": temp_session_id,
            "title": "New Conversation",
            "created_at": Utc::now(),
            "temporary": true
        })),
    )
}

/// Get list of sessions for the anonymous user
pub async fn get_sessions(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    // Return empty array if database is not ready or has issues
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    // Try to get a fresh connection from the pool
    let conn_result = state.conn.get();
    if conn_result.is_err() {
        // Database not available, return empty sessions array
        return (StatusCode::OK, Json(serde_json::json!([])));
    }

    let orchestrator = BotOrchestrator::new(state.clone());
    match orchestrator.get_user_sessions(user_id).await {
        Ok(sessions) => (StatusCode::OK, Json(serde_json::json!(sessions))),
        Err(_) => {
            // On any error, return empty array instead of error message
            // This allows the UI to continue functioning
            (StatusCode::OK, Json(serde_json::json!([])))
        }
    }
}

/// Start a session (mark as waiting for input)
pub async fn start_session(
    Extension(state): Extension<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match Uuid::parse_str(&session_id) {
        Ok(session_uuid) => {
            let mut sm = state.session_manager.lock().await;
            match sm.get_session_by_id(session_uuid) {
                Ok(Some(_)) => {
                    sm.mark_waiting(session_uuid);
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({ "status": "started", "session_id": session_id })),
                    )
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Session not found" })),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                ),
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid session ID" })),
        ),
    }
}

/// Get conversation history for a session
pub async fn get_session_history(
    Extension(state): Extension<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    match Uuid::parse_str(&session_id) {
        Ok(session_uuid) => {
            let orchestrator = BotOrchestrator::new(state.clone());
            match orchestrator
                .get_conversation_history(session_uuid, user_id)
                .await
            {
                Ok(history) => (StatusCode::OK, Json(serde_json::json!(history))),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                ),
            }
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid session ID" })),
        ),
    }
}
