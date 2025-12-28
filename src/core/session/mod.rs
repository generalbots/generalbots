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
        Self {
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
        let sess = if let Some(existing) = self.sessions.get(&session_id) {
            let mut sess = existing.clone();
            sess.data = input;
            sess
        } else {
            SessionData {
                id: session_id,
                user_id: None,
                data: input,
            }
        };
        self.sessions.insert(session_id, sess);
        self.waiting_for_input.remove(&session_id);
        Ok(Some("user_input".to_string()))
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

    pub fn update_session_context(
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

    pub fn get_session_context_data(
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
                9 => "episodic".to_string(),
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

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn total_count(&mut self) -> usize {
        use crate::shared::models::user_sessions::dsl::*;
        user_sessions
            .count()
            .first::<i64>(&mut self.conn)
            .unwrap_or(0) as usize
    }

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

pub async fn create_session(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let temp_session_id = Uuid::new_v4();

    if state.conn.get().is_ok() {
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let bot_id = Uuid::nil();

        {
            let mut sm = state.session_manager.lock().await;

            if let Ok(Some(session)) =
                sm.get_or_create_user_session(user_id, bot_id, "New Conversation")
            {
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "session_id": session.id,
                        "title": "New Conversation",
                        "created_at": Utc::now()
                    })),
                );
            }
        };
    }

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

pub async fn get_sessions(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let conn_result = state.conn.get();
    if conn_result.is_err() {
        return (StatusCode::OK, Json(serde_json::json!([])));
    }

    let orchestrator = BotOrchestrator::new(state.clone());
    match orchestrator.get_user_sessions(user_id).await {
        Ok(sessions) => (StatusCode::OK, Json(serde_json::json!(sessions))),
        Err(_) => (StatusCode::OK, Json(serde_json::json!([]))),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::time::Duration;

    // Test fixtures from bottest/fixtures/mod.rs

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Role {
        Admin,
        Attendant,
        User,
        Guest,
    }

    impl Default for Role {
        fn default() -> Self {
            Self::User
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: Uuid,
        pub email: String,
        pub name: String,
        pub role: Role,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
        pub metadata: HashMap<String, String>,
    }

    impl Default for User {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                email: "user@example.com".to_string(),
                name: "Test User".to_string(),
                role: Role::User,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Channel {
        WhatsApp,
        Teams,
        Web,
        Sms,
        Email,
        Api,
    }

    impl Default for Channel {
        fn default() -> Self {
            Self::WhatsApp
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Customer {
        pub id: Uuid,
        pub external_id: String,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub name: Option<String>,
        pub channel: Channel,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
        pub metadata: HashMap<String, String>,
    }

    impl Default for Customer {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                external_id: format!("ext_{}", Uuid::new_v4()),
                phone: Some("+15551234567".to_string()),
                email: None,
                name: Some("Test Customer".to_string()),
                channel: Channel::WhatsApp,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Bot {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub kb_enabled: bool,
        pub llm_enabled: bool,
        pub llm_model: Option<String>,
        pub active: bool,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
        pub config: HashMap<String, serde_json::Value>,
    }

    impl Default for Bot {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                name: "test-bot".to_string(),
                description: Some("Test bot for automated testing".to_string()),
                kb_enabled: false,
                llm_enabled: true,
                llm_model: Some("gpt-4".to_string()),
                active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                config: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SessionState {
        Active,
        Waiting,
        Transferred,
        Ended,
    }

    impl Default for SessionState {
        fn default() -> Self {
            Self::Active
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Session {
        pub id: Uuid,
        pub bot_id: Uuid,
        pub customer_id: Uuid,
        pub channel: Channel,
        pub state: SessionState,
        pub context: HashMap<String, serde_json::Value>,
        pub started_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
        pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl Default for Session {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                bot_id: Uuid::new_v4(),
                customer_id: Uuid::new_v4(),
                channel: Channel::WhatsApp,
                state: SessionState::Active,
                context: HashMap::new(),
                started_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                ended_at: None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum MessageDirection {
        Incoming,
        Outgoing,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ContentType {
        Text,
        Image,
        Audio,
        Video,
        Document,
        Location,
        Contact,
        Interactive,
    }

    impl Default for ContentType {
        fn default() -> Self {
            Self::Text
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub id: Uuid,
        pub session_id: Uuid,
        pub direction: MessageDirection,
        pub content: String,
        pub content_type: ContentType,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Default for Message {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                direction: MessageDirection::Incoming,
                content: "Hello".to_string(),
                content_type: ContentType::Text,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Priority {
        Low = 0,
        Normal = 1,
        High = 2,
        Urgent = 3,
    }

    impl Default for Priority {
        fn default() -> Self {
            Self::Normal
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum QueueStatus {
        Waiting,
        Assigned,
        InProgress,
        Completed,
        Cancelled,
    }

    impl Default for QueueStatus {
        fn default() -> Self {
            Self::Waiting
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct QueueEntry {
        pub id: Uuid,
        pub customer_id: Uuid,
        pub session_id: Uuid,
        pub priority: Priority,
        pub status: QueueStatus,
        pub entered_at: chrono::DateTime<chrono::Utc>,
        pub assigned_at: Option<chrono::DateTime<chrono::Utc>>,
        pub attendant_id: Option<Uuid>,
    }

    impl Default for QueueEntry {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                customer_id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                priority: Priority::Normal,
                status: QueueStatus::Waiting,
                entered_at: chrono::Utc::now(),
                assigned_at: None,
                attendant_id: None,
            }
        }
    }

    // Conversation test types from bottest/bot/mod.rs

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConversationState {
        Initial,
        WaitingForUser,
        WaitingForBot,
        Transferred,
        Ended,
        Error,
    }

    impl Default for ConversationState {
        fn default() -> Self {
            Self::Initial
        }
    }

    impl ConversationState {
        pub const fn is_terminal(self) -> bool {
            matches!(self, Self::Ended | Self::Error | Self::Transferred)
        }

        pub const fn is_waiting(self) -> bool {
            matches!(self, Self::WaitingForUser | Self::WaitingForBot)
        }
    }

    #[derive(Debug, Clone)]
    pub struct ConversationConfig {
        pub response_timeout: Duration,
        pub record: bool,
        pub use_mock_llm: bool,
        variables: HashMap<String, String>,
    }

    impl ConversationConfig {
        pub fn get_variable(&self, key: &str) -> Option<&String> {
            self.variables.get(key)
        }

        pub fn set_variable(&mut self, key: String, value: String) {
            self.variables.insert(key, value);
        }
    }

    impl Default for ConversationConfig {
        fn default() -> Self {
            Self {
                response_timeout: Duration::from_secs(30),
                record: true,
                use_mock_llm: true,
                variables: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct AssertionResult {
        pub passed: bool,
        pub message: String,
        pub expected: Option<String>,
        pub actual: Option<String>,
    }

    impl AssertionResult {
        pub fn pass(message: &str) -> Self {
            Self {
                passed: true,
                message: message.to_string(),
                expected: None,
                actual: None,
            }
        }

        pub fn fail(message: &str, expected: &str, actual: &str) -> Self {
            Self {
                passed: false,
                message: message.to_string(),
                expected: Some(expected.to_string()),
                actual: Some(actual.to_string()),
            }
        }
    }

    // Fixture factory functions

    fn admin_user() -> User {
        User {
            email: "admin@test.com".to_string(),
            name: "Test Admin".to_string(),
            role: Role::Admin,
            ..Default::default()
        }
    }

    fn attendant_user() -> User {
        User {
            email: "attendant@test.com".to_string(),
            name: "Test Attendant".to_string(),
            role: Role::Attendant,
            ..Default::default()
        }
    }

    fn regular_user() -> User {
        User {
            email: "user@test.com".to_string(),
            name: "Test User".to_string(),
            role: Role::User,
            ..Default::default()
        }
    }

    fn customer(phone: &str) -> Customer {
        Customer {
            phone: Some(phone.to_string()),
            channel: Channel::WhatsApp,
            ..Default::default()
        }
    }

    fn basic_bot(name: &str) -> Bot {
        Bot {
            name: name.to_string(),
            kb_enabled: false,
            llm_enabled: true,
            ..Default::default()
        }
    }

    fn bot_with_kb(name: &str) -> Bot {
        Bot {
            name: name.to_string(),
            kb_enabled: true,
            llm_enabled: true,
            ..Default::default()
        }
    }

    fn session_for(bot: &Bot, customer: &Customer) -> Session {
        Session {
            bot_id: bot.id,
            customer_id: customer.id,
            channel: customer.channel,
            ..Default::default()
        }
    }

    fn incoming_message(content: &str) -> Message {
        Message {
            direction: MessageDirection::Incoming,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn outgoing_message(content: &str) -> Message {
        Message {
            direction: MessageDirection::Outgoing,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn high_priority_queue_entry() -> QueueEntry {
        QueueEntry {
            priority: Priority::High,
            ..Default::default()
        }
    }

    fn urgent_queue_entry() -> QueueEntry {
        QueueEntry {
            priority: Priority::Urgent,
            ..Default::default()
        }
    }

    // Tests

    #[test]
    fn test_admin_user() {
        let user = admin_user();
        assert_eq!(user.role, Role::Admin);
        assert_eq!(user.email, "admin@test.com");
    }

    #[test]
    fn test_customer_factory() {
        let c = customer("+15559876543");
        assert_eq!(c.phone, Some("+15559876543".to_string()));
        assert_eq!(c.channel, Channel::WhatsApp);
    }

    #[test]
    fn test_bot_with_kb() {
        let bot = bot_with_kb("kb-bot");
        assert!(bot.kb_enabled);
        assert!(bot.llm_enabled);
    }

    #[test]
    fn test_session_for() {
        let bot = basic_bot("test");
        let customer = customer("+15551234567");
        let session = session_for(&bot, &customer);

        assert_eq!(session.bot_id, bot.id);
        assert_eq!(session.customer_id, customer.id);
        assert_eq!(session.channel, customer.channel);
    }

    #[test]
    fn test_message_factories() {
        let incoming = incoming_message("Hello");
        assert_eq!(incoming.direction, MessageDirection::Incoming);
        assert_eq!(incoming.content, "Hello");

        let outgoing = outgoing_message("Hi there!");
        assert_eq!(outgoing.direction, MessageDirection::Outgoing);
        assert_eq!(outgoing.content, "Hi there!");
    }

    #[test]
    fn test_queue_entry_priority() {
        let normal = QueueEntry::default();
        let high = high_priority_queue_entry();
        let urgent = urgent_queue_entry();

        assert!(urgent.priority > high.priority);
        assert!(high.priority > normal.priority);
    }

    #[test]
    fn test_default_implementations() {
        let _user = User::default();
        let _customer = Customer::default();
        let _bot = Bot::default();
        let _session = Session::default();
        let _message = Message::default();
        let _queue = QueueEntry::default();
    }

    #[test]
    fn test_assertion_result_pass() {
        let result = AssertionResult::pass("Test passed");
        assert!(result.passed);
        assert_eq!(result.message, "Test passed");
    }

    #[test]
    fn test_assertion_result_fail() {
        let result = AssertionResult::fail("Test failed", "expected", "actual");
        assert!(!result.passed);
        assert_eq!(result.expected, Some("expected".to_string()));
        assert_eq!(result.actual, Some("actual".to_string()));
    }

    #[test]
    fn test_conversation_config_default() {
        let config = ConversationConfig::default();
        assert_eq!(config.response_timeout, Duration::from_secs(30));
        assert!(config.record);
        assert!(config.use_mock_llm);
    }

    #[test]
    fn test_conversation_config_variables() {
        let mut config = ConversationConfig::default();
        config.set_variable("key1".to_string(), "value1".to_string());
        assert_eq!(config.get_variable("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get_variable("nonexistent"), None);
    }

    #[test]
    fn test_conversation_state_default() {
        let state = ConversationState::default();
        assert_eq!(state, ConversationState::Initial);
    }

    #[test]
    fn test_conversation_state_is_terminal() {
        assert!(!ConversationState::Initial.is_terminal());
        assert!(!ConversationState::WaitingForUser.is_terminal());
        assert!(!ConversationState::WaitingForBot.is_terminal());
        assert!(ConversationState::Transferred.is_terminal());
        assert!(ConversationState::Ended.is_terminal());
        assert!(ConversationState::Error.is_terminal());
    }

    #[test]
    fn test_conversation_state_is_waiting() {
        assert!(!ConversationState::Initial.is_waiting());
        assert!(ConversationState::WaitingForUser.is_waiting());
        assert!(ConversationState::WaitingForBot.is_waiting());
        assert!(!ConversationState::Transferred.is_waiting());
        assert!(!ConversationState::Ended.is_waiting());
        assert!(!ConversationState::Error.is_waiting());
    }

    #[test]
    fn test_channel_sms_and_api() {
        let sms_customer = Customer {
            channel: Channel::Sms,
            ..Default::default()
        };
        let api_customer = Customer {
            channel: Channel::Api,
            ..Default::default()
        };
        assert_eq!(sms_customer.channel, Channel::Sms);
        assert_eq!(api_customer.channel, Channel::Api);
    }

    #[test]
    fn test_session_state_transitions() {
        let mut session = Session::default();
        assert_eq!(session.state, SessionState::Active);

        session.state = SessionState::Waiting;
        assert_eq!(session.state, SessionState::Waiting);

        session.state = SessionState::Transferred;
        assert_eq!(session.state, SessionState::Transferred);

        session.state = SessionState::Ended;
        session.ended_at = Some(chrono::Utc::now());
        assert_eq!(session.state, SessionState::Ended);
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_user_roles() {
        let admin = admin_user();
        let attendant = attendant_user();
        let user = regular_user();

        assert_eq!(admin.role, Role::Admin);
        assert_eq!(attendant.role, Role::Attendant);
        assert_eq!(user.role, Role::User);
    }

    #[test]
    fn test_channel_types() {
        let wa_customer = Customer {
            channel: Channel::WhatsApp,
            ..Default::default()
        };
        let teams_customer = Customer {
            channel: Channel::Teams,
            ..Default::default()
        };
        let web_customer = Customer {
            channel: Channel::Web,
            ..Default::default()
        };

        assert_eq!(wa_customer.channel, Channel::WhatsApp);
        assert_eq!(teams_customer.channel, Channel::Teams);
        assert_eq!(web_customer.channel, Channel::Web);
    }

    #[test]
    fn test_bot_configuration() {
        let mut bot = basic_bot("configurable-bot");
        bot.config
            .insert("max_tokens".to_string(), serde_json::json!(1000));
        bot.config
            .insert("temperature".to_string(), serde_json::json!(0.7));

        assert_eq!(bot.config.get("max_tokens"), Some(&serde_json::json!(1000)));
        assert_eq!(bot.config.get("temperature"), Some(&serde_json::json!(0.7)));
    }

    #[test]
    fn test_message_content_types() {
        let text_msg = Message {
            content_type: ContentType::Text,
            content: "Hello".to_string(),
            ..Default::default()
        };
        let image_msg = Message {
            content_type: ContentType::Image,
            content: "[image]".to_string(),
            ..Default::default()
        };

        assert_eq!(text_msg.content_type, ContentType::Text);
        assert_eq!(image_msg.content_type, ContentType::Image);
    }

    #[test]
    fn test_queue_status_transitions() {
        let mut entry = QueueEntry::default();
        assert_eq!(entry.status, QueueStatus::Waiting);

        entry.status = QueueStatus::Assigned;
        entry.assigned_at = Some(chrono::Utc::now());
        entry.attendant_id = Some(Uuid::new_v4());
        assert_eq!(entry.status, QueueStatus::Assigned);
        assert!(entry.assigned_at.is_some());
        assert!(entry.attendant_id.is_some());

        entry.status = QueueStatus::InProgress;
        assert_eq!(entry.status, QueueStatus::InProgress);

        entry.status = QueueStatus::Completed;
        assert_eq!(entry.status, QueueStatus::Completed);
    }

    #[test]
    fn test_customer_metadata() {
        let mut customer = Customer::default();
        customer
            .metadata
            .insert("vip".to_string(), "true".to_string());
        customer
            .metadata
            .insert("segment".to_string(), "enterprise".to_string());

        assert_eq!(customer.metadata.get("vip"), Some(&"true".to_string()));
        assert_eq!(
            customer.metadata.get("segment"),
            Some(&"enterprise".to_string())
        );
    }

    #[test]
    fn test_session_context() {
        let mut session = Session::default();
        session
            .context
            .insert("last_intent".to_string(), serde_json::json!("greeting"));
        session
            .context
            .insert("turn_count".to_string(), serde_json::json!(5));

        assert_eq!(
            session.context.get("last_intent"),
            Some(&serde_json::json!("greeting"))
        );
        assert_eq!(
            session.context.get("turn_count"),
            Some(&serde_json::json!(5))
        );
    }

    #[test]
    fn test_message_metadata() {
        let mut message = Message::default();
        message
            .metadata
            .insert("sentiment".to_string(), serde_json::json!("positive"));
        message
            .metadata
            .insert("confidence".to_string(), serde_json::json!(0.95));

        assert_eq!(
            message.metadata.get("sentiment"),
            Some(&serde_json::json!("positive"))
        );
        assert_eq!(
            message.metadata.get("confidence"),
            Some(&serde_json::json!(0.95))
        );
    }
}
