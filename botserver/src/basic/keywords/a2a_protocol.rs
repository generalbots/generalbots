use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{info, trace, warn};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum A2AMessageType {
    Request,

    Response,

    Broadcast,

    Delegate,

    Collaborate,

    Ack,

    Error,
}

impl Default for A2AMessageType {
    fn default() -> Self {
        Self::Request
    }
}

impl std::fmt::Display for A2AMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request => write!(f, "request"),
            Self::Response => write!(f, "response"),
            Self::Broadcast => write!(f, "broadcast"),
            Self::Delegate => write!(f, "delegate"),
            Self::Collaborate => write!(f, "collaborate"),
            Self::Ack => write!(f, "ack"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for A2AMessageType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "response" => Self::Response,
            "broadcast" => Self::Broadcast,
            "delegate" => Self::Delegate,
            "collaborate" => Self::Collaborate,
            "ack" => Self::Ack,
            "error" => Self::Error,
            _ => Self::Request,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub id: Uuid,

    pub from_agent: String,

    pub to_agent: Option<String>,

    pub message_type: A2AMessageType,

    pub payload: serde_json::Value,

    pub correlation_id: Uuid,

    pub session_id: Uuid,

    pub timestamp: chrono::DateTime<chrono::Utc>,

    pub metadata: HashMap<String, String>,

    pub ttl_seconds: u32,

    pub hop_count: u32,
}

impl A2AMessage {
    pub fn new(
        from_agent: &str,
        to_agent: Option<&str>,
        message_type: A2AMessageType,
        payload: serde_json::Value,
        session_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.map(|s| s.to_string()),
            message_type,
            payload,
            correlation_id: Uuid::new_v4(),
            session_id,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            ttl_seconds: 30,
            hop_count: 0,
        }
    }

    pub fn create_response(&self, from_agent: &str, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_agent: from_agent.to_string(),
            to_agent: Some(self.from_agent.clone()),
            message_type: A2AMessageType::Response,
            payload,
            correlation_id: self.correlation_id,
            session_id: self.session_id,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            ttl_seconds: 30,
            hop_count: self.hop_count + 1,
        }
    }

    pub fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return false;
        }
        let now = chrono::Utc::now();
        let expiry = self.timestamp + chrono::Duration::seconds(i64::from(self.ttl_seconds));
        now > expiry
    }

    pub fn max_hops_exceeded(&self, max_hops: u32) -> bool {
        self.hop_count >= max_hops
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AConfig {
    pub enabled: bool,

    pub timeout_seconds: u32,

    pub max_hops: u32,

    pub protocol_version: String,

    pub persist_messages: bool,

    pub retry_count: u32,

    pub queue_size: u32,
}

impl Default for A2AConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 30,
            max_hops: 5,
            protocol_version: "1.0".to_string(),
            persist_messages: true,
            retry_count: 3,
            queue_size: 100,
        }
    }
}

pub fn load_a2a_config(state: &AppState, bot_id: Uuid) -> A2AConfig {
    let mut config = A2AConfig::default();

    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key LIKE 'a2a-%'",
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        for row in configs {
            match row.config_key.as_str() {
                "a2a-enabled" => {
                    config.enabled = row.config_value.to_lowercase() == "true";
                }
                "a2a-timeout" => {
                    config.timeout_seconds = row.config_value.parse().unwrap_or(30);
                }
                "a2a-max-hops" => {
                    config.max_hops = row.config_value.parse().unwrap_or(5);
                }
                "a2a-protocol-version" => {
                    config.protocol_version = row.config_value;
                }
                "a2a-persist-messages" => {
                    config.persist_messages = row.config_value.to_lowercase() == "true";
                }
                "a2a-retry-count" => {
                    config.retry_count = row.config_value.parse().unwrap_or(3);
                }
                "a2a-queue-size" => {
                    config.queue_size = row.config_value.parse().unwrap_or(100);
                }
                _ => {}
            }
        }
    }

    config
}

pub fn register_a2a_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    send_to_bot_keyword(Arc::clone(&state), user.clone(), engine);
    broadcast_message_keyword(Arc::clone(&state), user.clone(), engine);
    collaborate_with_keyword(Arc::clone(&state), user.clone(), engine);
    wait_for_bot_keyword(Arc::clone(&state), user.clone(), engine);
    delegate_conversation_keyword(Arc::clone(&state), user.clone(), engine);
    get_a2a_messages_keyword(state, user, engine);
}

pub fn send_to_bot_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["SEND", "TO", "BOT", "$expr$", "MESSAGE", "$expr$"],
            false,
            move |context, inputs| {
                let target_bot = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();
                let message_content = context
                    .eval_expression_tree(&inputs[1])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!(
                    "SEND TO BOT '{}' MESSAGE for session: {}",
                    target_bot,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;
                let from_bot = format!("bot_{}", bot_id);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                            return;
                        }
                    };
                    let result = rt.block_on(async {
                        send_a2a_message(
                            &state_for_task,
                            session_id,
                            &from_bot,
                            Some(&target_bot),
                            A2AMessageType::Request,
                            serde_json::json!({ "content": message_content }),
                        )
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(msg_id)) => Ok(Dynamic::from(msg_id.to_string())),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "SEND TO BOT timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register SEND TO BOT syntax");
}

pub fn broadcast_message_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["BROADCAST", "MESSAGE", "$expr$"],
            false,
            move |context, inputs| {
                let message_content = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!("BROADCAST MESSAGE for session: {}", user_clone.id);

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;
                let from_bot = format!("bot_{}", bot_id);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                            return;
                        }
                    };
                    let result = rt.block_on(async {
                        send_a2a_message(
                            &state_for_task,
                            session_id,
                            &from_bot,
                            None,
                            A2AMessageType::Broadcast,
                            serde_json::json!({ "content": message_content }),
                        )
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(msg_id)) => Ok(Dynamic::from(msg_id.to_string())),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "BROADCAST MESSAGE timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register BROADCAST MESSAGE syntax");
}

pub fn collaborate_with_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["COLLABORATE", "WITH", "$expr$", "ON", "$expr$"],
            false,
            move |context, inputs| {
                let bots_str = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();
                let task = context
                    .eval_expression_tree(&inputs[1])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                let bots: Vec<String> = bots_str
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                trace!(
                    "COLLABORATE WITH {:?} ON '{}' for session: {}",
                    bots,
                    task,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;
                let from_bot = format!("bot_{}", bot_id);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                            return;
                        }
                    };
                    let result = rt.block_on(async {
                        let mut message_ids = Vec::new();
                        for target_bot in &bots {
                            match send_a2a_message(
                                &state_for_task,
                                session_id,
                                &from_bot,
                                Some(target_bot),
                                A2AMessageType::Collaborate,
                                serde_json::json!({
                                    "task": task,
                                    "collaborators": bots.clone()
                                }),
                            ) {
                                Ok(id) => message_ids.push(id.to_string()),
                                Err(e) => {
                                    warn!(
                                        "Failed to send collaboration request to {}: {}",
                                        target_bot, e
                                    );
                                }
                            }
                        }
                        Ok::<Vec<String>, String>(message_ids)
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(ids)) => {
                        let array: rhai::Array = ids.into_iter().map(Dynamic::from).collect();
                        Ok(Dynamic::from(array))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "COLLABORATE WITH timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register COLLABORATE WITH syntax");
}

pub fn wait_for_bot_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["WAIT", "FOR", "BOT", "$expr$", "TIMEOUT", "$expr$"],
            false,
            move |context, inputs| {
                let target_bot = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();
                let timeout_secs: i64 = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(30);

                trace!(
                    "WAIT FOR BOT '{}' TIMEOUT {} for session: {}",
                    target_bot,
                    timeout_secs,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;
                let current_bot = format!("bot_{}", bot_id);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                            return;
                        }
                    };
                    let result = rt.block_on(async {
                        wait_for_bot_response(
                            &state_for_task,
                            session_id,
                            &target_bot,
                            &current_bot,
                            timeout_secs as u64,
                        )
                        .await
                    });
                    let _ = tx.send(result);
                });

                let timeout_duration = std::time::Duration::from_secs(timeout_secs as u64 + 5);
                match rx.recv_timeout(timeout_duration) {
                    Ok(Ok(response)) => Ok(Dynamic::from(response)),
                    Ok(Err(e)) => Ok(Dynamic::from(format!("Error: {}", e))),
                    Err(_) => Ok(Dynamic::from("Timeout waiting for response".to_string())),
                }
            },
        )
        .expect("Failed to register WAIT FOR BOT syntax");
}

pub fn delegate_conversation_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["DELEGATE", "CONVERSATION", "TO", "$expr$"],
            false,
            move |context, inputs| {
                let target_bot = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!(
                    "DELEGATE CONVERSATION TO '{}' for session: {}",
                    target_bot,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let bot_id = user_clone.bot_id;
                let from_bot = format!("bot_{}", bot_id);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to create runtime: {}", e)));
                            return;
                        }
                    };
                    let result = rt.block_on(async {
                        send_a2a_message(
                            &state_for_task,
                            session_id,
                            &from_bot,
                            Some(&target_bot),
                            A2AMessageType::Delegate,
                            serde_json::json!({
                                "action": "delegate",
                                "from_bot": from_bot
                            }),
                        )?;

                        set_session_active_bot(&state_for_task, session_id, &target_bot)
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "DELEGATE CONVERSATION timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register DELEGATE CONVERSATION syntax");
}

pub fn get_a2a_messages_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine.register_fn("GET A2A MESSAGES", move || -> rhai::Array {
        let state = Arc::clone(&state_clone);
        let session_id = user_clone.id;
        let bot_id = user_clone.bot_id;
        let current_bot = format!("bot_{}", bot_id);

        if let Ok(mut conn) = state.conn.get() {
            get_pending_messages_sync(&mut conn, session_id, &current_bot)
                .unwrap_or_default()
                .into_iter()
                .map(|msg| Dynamic::from(serde_json::to_string(&msg).unwrap_or_default()))
                .collect()
        } else {
            rhai::Array::new()
        }
    });
}

fn send_a2a_message(
    state: &AppState,
    session_id: Uuid,
    from_agent: &str,
    to_agent: Option<&str>,
    message_type: A2AMessageType,
    payload: serde_json::Value,
) -> Result<Uuid, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let message = A2AMessage::new(from_agent, to_agent, message_type, payload, session_id);
    let message_id = message.id;

    let payload_str = serde_json::to_string(&message.payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;

    let metadata_str = serde_json::to_string(&message.metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    diesel::sql_query(
        "INSERT INTO a2a_messages \
         (id, session_id, from_agent, to_agent, message_type, payload, correlation_id, \
          timestamp, metadata, ttl_seconds, hop_count, processed) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(message.id)
    .bind::<diesel::sql_types::Uuid, _>(message.session_id)
    .bind::<diesel::sql_types::Text, _>(&message.from_agent)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(message.to_agent.as_deref())
    .bind::<diesel::sql_types::Text, _>(message.message_type.to_string())
    .bind::<diesel::sql_types::Text, _>(&payload_str)
    .bind::<diesel::sql_types::Uuid, _>(message.correlation_id)
    .bind::<diesel::sql_types::Timestamptz, _>(message.timestamp)
    .bind::<diesel::sql_types::Text, _>(&metadata_str)
    .bind::<diesel::sql_types::Integer, _>(message.ttl_seconds as i32)
    .bind::<diesel::sql_types::Integer, _>(message.hop_count as i32)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to insert A2A message: {}", e))?;

    info!(
        "A2A message sent: {} -> {:?} (type: {})",
        from_agent, to_agent, message.message_type
    );

    Ok(message_id)
}

async fn wait_for_bot_response(
    state: &AppState,
    session_id: Uuid,
    from_bot: &str,
    to_bot: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Err("Timeout waiting for bot response".to_string());
        }

        if let Ok(mut conn) = state.conn.get() {
            #[derive(QueryableByName)]
            struct MessageRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                payload: String,
            }

            let result: Option<MessageRow> = diesel::sql_query(
                "SELECT id, payload FROM a2a_messages \
                 WHERE session_id = $1 AND from_agent = $2 AND to_agent = $3 \
                 AND message_type = 'response' AND processed = false \
                 ORDER BY timestamp DESC LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(session_id)
            .bind::<diesel::sql_types::Text, _>(from_bot)
            .bind::<diesel::sql_types::Text, _>(to_bot)
            .get_result(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to query messages: {}", e))?;

            if let Some(msg) = result {
                let _ = diesel::sql_query("UPDATE a2a_messages SET processed = true WHERE id = $1")
                    .bind::<diesel::sql_types::Uuid, _>(msg.id)
                    .execute(&mut conn);

                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    if let Some(content) = payload.get("content").and_then(|c| c.as_str()) {
                        return Ok(content.to_string());
                    }
                }
                return Ok(msg.payload);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

fn set_session_active_bot(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
) -> Result<String, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();

    diesel::sql_query(
        "INSERT INTO session_preferences (session_id, preference_key, preference_value, updated_at) \
         VALUES ($1, 'active_bot', $2, $3) \
         ON CONFLICT (session_id, preference_key) DO UPDATE SET \
         preference_value = EXCLUDED.preference_value, \
         updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to set active bot: {}", e))?;

    info!("Session {} delegated to bot: {}", session_id, bot_name);

    Ok(format!("Conversation delegated to {}", bot_name))
}

fn get_pending_messages_sync(
    conn: &mut diesel::PgConnection,
    session_id: Uuid,
    to_agent: &str,
) -> Result<Vec<A2AMessage>, String> {
    #[derive(QueryableByName)]
    struct MessageRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        session_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        from_agent: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        to_agent: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        message_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        payload: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        correlation_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        timestamp: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        ttl_seconds: i32,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        hop_count: i32,
    }

    let rows: Vec<MessageRow> = diesel::sql_query(
        "SELECT id, session_id, from_agent, to_agent, message_type, payload, \
         correlation_id, timestamp, ttl_seconds, hop_count \
         FROM a2a_messages \
         WHERE session_id = $1 AND (to_agent = $2 OR to_agent IS NULL) AND processed = false \
         ORDER BY timestamp ASC",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(to_agent)
    .load(conn)
    .map_err(|e| format!("Failed to get pending messages: {}", e))?;

    let messages: Vec<A2AMessage> = rows
        .into_iter()
        .map(|row| A2AMessage {
            id: row.id,
            session_id: row.session_id,
            from_agent: row.from_agent,
            to_agent: row.to_agent,
            message_type: A2AMessageType::from(row.message_type.as_str()),
            payload: serde_json::from_str(&row.payload).unwrap_or_else(|_| serde_json::json!({})),
            correlation_id: row.correlation_id,
            timestamp: row.timestamp,
            metadata: HashMap::new(),
            ttl_seconds: row.ttl_seconds as u32,
            hop_count: row.hop_count as u32,
        })
        .filter(|msg| !msg.is_expired())
        .collect();

    Ok(messages)
}

pub fn respond_to_a2a_message(
    state: &AppState,
    original_message: &A2AMessage,
    from_agent: &str,
    response_content: &str,
) -> Result<Uuid, String> {
    send_a2a_message(
        state,
        original_message.session_id,
        from_agent,
        Some(&original_message.from_agent),
        A2AMessageType::Response,
        serde_json::json!({ "content": response_content }),
    )
}
