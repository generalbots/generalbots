use crate::schema::{message_history, user_sessions, users};
use crate::session_data::{SessionData, UserSession};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use log::{error, trace, warn};
#[cfg(feature = "cache")]
use redis::Client;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

pub struct SessionManager {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    sessions: HashMap<Uuid, SessionData>,
    waiting_for_input: HashSet<Uuid>,
    #[cfg(feature = "cache")]
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
        #[cfg(feature = "cache")]
        redis_client: Option<Arc<Client>>,
    ) -> Self {
        Self {
            conn,
            sessions: HashMap::new(),
            waiting_for_input: HashSet::new(),
            #[cfg(feature = "cache")]
            redis: redis_client,
        }
    }

    pub fn provide_input(
        &mut self,
        session_id: Uuid,
        input: String,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        trace!("SessionManager.provide_input called for session {}", session_id);
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
        let result = user_sessions::table
            .filter(user_sessions::id.eq(session_id))
            .first::<UserSession>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_user_session(
        &mut self,
        uid: Uuid,
        bid: Uuid,
    ) -> Result<Option<UserSession>, Box<dyn Error + Send + Sync>> {
        let result = user_sessions::table
            .filter(user_sessions::user_id.eq(uid))
            .filter(user_sessions::bot_id.eq(bid))
            .order(user_sessions::created_at.desc())
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
        let user_id = uid.unwrap_or_else(Uuid::new_v4);
        let user_exists: Option<Uuid> = users::table
            .filter(users::id.eq(user_id))
            .select(users::id)
            .first(&mut self.conn)
            .optional()?;
        if user_exists.is_none() {
            let now = Utc::now();
            diesel::insert_into(users::table)
                .values((
                    users::id.eq(user_id),
                    users::username.eq(format!("guest_{}", &user_id.to_string()[..8])),
                    users::email.eq(format!(
                        "guest_{}@anonymous.local",
                        &user_id.to_string()[..8]
                    )),
                    users::password_hash.eq(""),
                    users::is_active.eq(true),
                    users::created_at.eq(now),
                    users::updated_at.eq(now),
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
        self.create_session_with_id(Uuid::new_v4(), uid, bid, session_title)
    }

    pub fn create_session_with_id(
        &mut self,
        session_id: Uuid,
        uid: Uuid,
        bid: Uuid,
        session_title: &str,
    ) -> Result<UserSession, Box<dyn Error + Send + Sync>> {
        let verified_uid = self.get_or_create_anonymous_user(Some(uid))?;
        let now = Utc::now();
        let inserted: UserSession = diesel::insert_into(user_sessions::table)
            .values((
                user_sessions::id.eq(session_id),
                user_sessions::user_id.eq(verified_uid),
                user_sessions::bot_id.eq(bid),
                user_sessions::title.eq(session_title),
                user_sessions::context_data.eq(serde_json::json!({})),
                user_sessions::current_tool.eq(None::<String>),
                user_sessions::created_at.eq(now),
                user_sessions::updated_at.eq(now),
            ))
            .returning(UserSession::as_returning())
            .get_result(&mut self.conn)
            .map_err(|e| {
                error!("Failed to create session in database: {}", e);
                e
            })?;

        log::info!("User {} created resource: session {}", verified_uid, inserted.id);
        Ok(inserted)
    }

    pub fn get_or_create_session_by_id(
        &mut self,
        session_id: Uuid,
        uid: Uuid,
        bid: Uuid,
        session_title: &str,
    ) -> Result<UserSession, Box<dyn Error + Send + Sync>> {
        if let Ok(Some(existing)) = self.get_session_by_id(session_id) {
            return Ok(existing);
        }
        self.create_session_with_id(session_id, uid, bid, session_title)
    }

    pub fn save_message(
        &mut self,
        sess_id: Uuid,
        uid: Uuid,
        ro: i32,
        content: &str,
        msg_type: i32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let next_index: i32 = message_history::table
            .filter(message_history::session_id.eq(sess_id))
            .count()
            .get_result::<i64>(&mut self.conn)
            .unwrap_or(0) as i32;
        diesel::insert_into(message_history::table)
            .values((
                message_history::id.eq(Uuid::new_v4()),
                message_history::session_id.eq(sess_id),
                message_history::user_id.eq(uid),
                message_history::role.eq(ro),
                message_history::content_encrypted.eq(content),
                message_history::message_type.eq(msg_type),
                message_history::message_index.eq(next_index),
                message_history::created_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut self.conn)?;
        trace!("Message saved for session {} with index {}", sess_id, next_index);
        Ok(())
    }

    pub fn update_session_context(
        &mut self,
        session_id: &Uuid,
        user_id: &Uuid,
        context_data: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        #[cfg(feature = "cache")]
        {
            use redis::Commands;
            let redis_key = format!("context:{}:{}", user_id, session_id);
            if let Some(redis_client) = &self.redis {
                let mut conn = redis_client.get_connection()?;
                conn.set::<_, _, ()>(&redis_key, &context_data)?;
            } else {
                warn!("No Redis client configured, context not persisted");
            }
        }
        Ok(())
    }

    pub fn get_session_context_data(
        &self,
        session_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        #[cfg(feature = "cache")]
        {
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
                            let full_key = format!("context:{}:{}:{}", user_id, session_id, context_name);
                            match connection.get::<_, Option<String>>(&full_key) {
                                Ok(Some(context_value)) => {
                                    trace!("Retrieved context value from Cache for key {}: {} chars", full_key, context_value.len());
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
        }
        Ok(String::new())
    }

    pub fn get_conversation_history(
        &mut self,
        sess_id: Uuid,
        _uid: Uuid,
        history_limit: Option<i64>,
    ) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
        let limit_val = history_limit.unwrap_or(50);
        let messages = message_history::table
            .filter(message_history::session_id.eq(sess_id))
            .order(message_history::message_index.asc())
            .select((message_history::role, message_history::content_encrypted, message_history::message_index))
            .load::<(i32, String, i32)>(&mut self.conn)?;
        let total_messages_needed = (limit_val * 2) as usize;
        let start_idx = messages.len().saturating_sub(total_messages_needed);
        let recent_messages: Vec<_> = messages.into_iter().skip(start_idx).collect();
        let mut history: Vec<(String, String)> = Vec::new();
        for (other_role, content, _idx) in recent_messages {
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
        let sessions = if uid == Uuid::nil() {
            user_sessions::table
                .order(user_sessions::created_at.desc())
                .load::<UserSession>(&mut self.conn)
                .unwrap_or_else(|_| Vec::new())
        } else {
            user_sessions::table
                .filter(user_sessions::user_id.eq(uid))
                .order(user_sessions::created_at.desc())
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
        let updated_count = diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set((user_sessions::user_id.eq(new_user_id), user_sessions::updated_at.eq(chrono::Utc::now())))
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
        user_sessions::table
            .count()
            .first::<i64>(&mut self.conn)
            .unwrap_or(0) as usize
    }

    pub fn recent_sessions(
        &mut self,
        hours: i64,
    ) -> Result<Vec<UserSession>, Box<dyn Error + Send + Sync>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(hours);
        let sessions = user_sessions::table
            .filter(user_sessions::created_at.gt(since))
            .order(user_sessions::created_at.desc())
            .load::<UserSession>(&mut self.conn)?;
        Ok(sessions)
    }

    pub fn get_statistics(&mut self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        let total = user_sessions::table.count().first::<i64>(&mut self.conn)?;
        let active = self.sessions.len() as i64;
        let today = chrono::Utc::now().date_naive();
        let today_start = today
            .and_hms_opt(0, 0, 0)
            .unwrap_or_else(|| today.and_hms_opt(0, 0, 1).unwrap_or_default())
            .and_utc();
        let today_count = user_sessions::table
            .filter(user_sessions::created_at.ge(today_start))
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
