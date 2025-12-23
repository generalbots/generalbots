




use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub channel: String,
    pub user_name: String,
    pub user_email: Option<String>,
    pub last_message: String,
    pub last_message_time: String,
    pub waiting_time_seconds: i64,
    pub priority: i32,
    pub status: QueueStatus,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueStatus {
    Waiting,
    Assigned,
    Active,
    Resolved,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendantStats {
    pub attendant_id: String,
    pub attendant_name: String,
    pub channel: String,
    pub preferences: String,
    pub active_conversations: i32,
    pub total_handled_today: i32,
    pub avg_response_time_seconds: i32,
    pub status: AttendantStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendantCSV {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub preferences: String,
    pub department: Option<String>,
    pub aliases: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub teams: Option<String>,
    pub google: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttendantStatus {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRequest {
    pub session_id: Uuid,
    pub attendant_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub session_id: Uuid,
    pub from_attendant_id: Uuid,
    pub to_attendant_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueFilters {
    pub channel: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
}



async fn is_transfer_enabled(bot_id: Uuid, work_path: &str) -> bool {
    let config_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("config.csv");

    if !config_path.exists() {

        let alt_path = PathBuf::from(work_path).join("config.csv");
        if alt_path.exists() {
            return check_config_for_crm_enabled(&alt_path);
        }
        warn!("Config file not found: {:?}", config_path);
        return false;
    }

    check_config_for_crm_enabled(&config_path)
}


fn check_config_for_crm_enabled(config_path: &PathBuf) -> bool {
    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            for line in content.lines() {
                let line_lower = line.to_lowercase();

                if (line_lower.contains("crm-enabled") || line_lower.contains("crm_enabled"))
                    && line_lower.contains("true")
                {
                    info!("CRM enabled via crm-enabled setting");
                    return true;
                }

                if line_lower.contains("transfer") && line_lower.contains("true") {
                    info!("CRM enabled via legacy transfer setting");
                    return true;
                }
            }
            false
        }
        Err(e) => {
            error!("Failed to read config file: {}", e);
            false
        }
    }
}


async fn read_attendants_csv(bot_id: Uuid, work_path: &str) -> Vec<AttendantCSV> {
    let attendant_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join("attendant.csv");

    if !attendant_path.exists() {
        warn!("Attendant file not found: {:?}", attendant_path);
        return Vec::new();
    }

    match std::fs::read_to_string(&attendant_path) {
        Ok(content) => {
            let mut attendants = Vec::new();
            let mut lines = content.lines();


            lines.next();

            for line in lines {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    attendants.push(AttendantCSV {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        channel: parts[2].to_string(),
                        preferences: parts[3].to_string(),
                        department: parts
                            .get(4)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                        aliases: parts
                            .get(5)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                        phone: parts
                            .get(6)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                        email: parts
                            .get(7)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                        teams: parts
                            .get(8)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                        google: parts
                            .get(9)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string()),
                    });
                }
            }
            attendants
        }
        Err(e) => {
            error!("Failed to read attendant file: {}", e);
            Vec::new()
        }
    }
}



pub async fn find_attendant_by_identifier(
    bot_id: Uuid,
    work_path: &str,
    identifier: &str,
) -> Option<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path).await;
    let identifier_lower = identifier.to_lowercase().trim().to_string();

    for att in attendants {

        if att.id.to_lowercase() == identifier_lower {
            return Some(att);
        }
        if att.name.to_lowercase() == identifier_lower {
            return Some(att);
        }
        if let Some(ref phone) = att.phone {

            let phone_normalized = phone
                .chars()
                .filter(|c| c.is_numeric() || *c == '+')
                .collect::<String>();
            let id_normalized = identifier
                .chars()
                .filter(|c| c.is_numeric() || *c == '+')
                .collect::<String>();
            if phone_normalized == id_normalized || phone.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref email) = att.email {
            if email.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref teams) = att.teams {
            if teams.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }
        if let Some(ref google) = att.google {
            if google.to_lowercase() == identifier_lower {
                return Some(att);
            }
        }

        if let Some(ref aliases) = att.aliases {
            for alias in aliases.split(';') {
                if alias.trim().to_lowercase() == identifier_lower {
                    return Some(att);
                }
            }
        }
    }

    None
}


pub async fn find_attendants_by_channel(
    bot_id: Uuid,
    work_path: &str,
    channel: &str,
) -> Vec<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path).await;
    let channel_lower = channel.to_lowercase();

    attendants
        .into_iter()
        .filter(|att| {
            att.channel.to_lowercase() == "all" || att.channel.to_lowercase() == channel_lower
        })
        .collect()
}


pub async fn find_attendants_by_department(
    bot_id: Uuid,
    work_path: &str,
    department: &str,
) -> Vec<AttendantCSV> {
    let attendants = read_attendants_csv(bot_id, work_path).await;
    let dept_lower = department.to_lowercase();

    attendants
        .into_iter()
        .filter(|att| {
            att.department
                .as_ref()
                .map(|d| d.to_lowercase() == dept_lower)
                .unwrap_or(false)
        })
        .collect()
}



pub async fn list_queue(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<QueueFilters>,
) -> impl IntoResponse {
    info!("Listing queue items with filters: {:?}", filters);

    let result = tokio::task::spawn_blocking({
        let conn = state.conn.clone();
        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;

            use crate::shared::models::schema::user_sessions;
            use crate::shared::models::schema::users;


            let sessions_data: Vec<UserSession> = user_sessions::table
                .order(user_sessions::created_at.desc())
                .limit(50)
                .load(&mut db_conn)
                .map_err(|e| format!("Failed to load sessions: {}", e))?;

            let mut queue_items = Vec::new();

            for session_data in sessions_data {

                let user_info: Option<(String, String)> = users::table
                    .filter(users::id.eq(session_data.user_id))
                    .select((users::username, users::email))
                    .first(&mut db_conn)
                    .optional()
                    .map_err(|e| format!("Failed to load user: {}", e))?;

                let (uname, uemail) = user_info.unwrap_or_else(|| {
                    (
                        format!("user_{}", session_data.user_id),
                        format!("{}@unknown.local", session_data.user_id),
                    )
                });

                let channel = session_data
                    .context_data
                    .get("channel")
                    .and_then(|c| c.as_str())
                    .unwrap_or("web")
                    .to_string();

                let waiting_time = (Utc::now() - session_data.updated_at).num_seconds();

                queue_items.push(QueueItem {
                    session_id: session_data.id,
                    user_id: session_data.user_id,
                    bot_id: session_data.bot_id,
                    channel,
                    user_name: uname,
                    user_email: Some(uemail),
                    last_message: session_data.title.clone(),
                    last_message_time: session_data.updated_at.to_rfc3339(),
                    waiting_time_seconds: waiting_time,
                    priority: if waiting_time > 300 { 2 } else { 1 },
                    status: QueueStatus::Waiting,
                    assigned_to: None,
                    assigned_to_name: None,
                });
            }

            Ok::<Vec<QueueItem>, String>(queue_items)
        }
    })
    .await;

    match result {
        Ok(Ok(queue_items)) => {
            info!("Found {} queue items", queue_items.len());
            (StatusCode::OK, Json(queue_items))
        }
        Ok(Err(e)) => {
            error!("Queue list error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![] as Vec<QueueItem>),
            )
        }
        Err(e) => {
            error!("Task error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![] as Vec<QueueItem>),
            )
        }
    }
}



pub async fn list_attendants(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Listing attendants");

    let bot_id_str = params.get("bot_id").cloned().unwrap_or_default();
    let bot_id = match Uuid::parse_str(&bot_id_str) {
        Ok(id) => id,
        Err(_) => {

            let conn = state.conn.clone();
            let result = tokio::task::spawn_blocking(move || {
                let mut db_conn = conn.get().ok()?;
                use crate::shared::models::schema::bots;
                bots::table
                    .filter(bots::is_active.eq(true))
                    .select(bots::id)
                    .first::<Uuid>(&mut db_conn)
                    .ok()
            })
            .await;

            match result {
                Ok(Some(id)) => id,
                _ => {
                    error!("No valid bot_id provided and no default bot found");
                    return (StatusCode::BAD_REQUEST, Json(vec![] as Vec<AttendantStats>));
                }
            }
        }
    };


    let work_path = "./work";
    if !is_transfer_enabled(bot_id, work_path).await {
        warn!("Transfer not enabled for bot {}", bot_id);
        return (StatusCode::OK, Json(vec![] as Vec<AttendantStats>));
    }


    let attendant_csvs = read_attendants_csv(bot_id, work_path).await;

    let attendants: Vec<AttendantStats> = attendant_csvs
        .into_iter()
        .map(|att| AttendantStats {
            attendant_id: att.id,
            attendant_name: att.name,
            channel: att.channel,
            preferences: att.preferences,
            active_conversations: 0,
            total_handled_today: 0,
            avg_response_time_seconds: 0,
            status: AttendantStatus::Online,
        })
        .collect();

    info!("Found {} attendants from CSV", attendants.len());
    (StatusCode::OK, Json(attendants))
}



pub async fn assign_conversation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AssignRequest>,
) -> impl IntoResponse {
    info!(
        "Assigning session {} to attendant {}",
        request.session_id, request.attendant_id
    );


    let result = tokio::task::spawn_blocking({
        let conn = state.conn.clone();
        let session_id = request.session_id;
        let attendant_id = request.attendant_id;

        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;

            use crate::shared::models::schema::user_sessions;


            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {}", e))?;


            let mut ctx = session.context_data.clone();
            ctx["assigned_to"] = serde_json::json!(attendant_id.to_string());
            ctx["assigned_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["status"] = serde_json::json!("assigned");

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {}", e))?;

            Ok::<(), String>(())
        }
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "session_id": request.session_id,
                "attendant_id": request.attendant_id,
                "assigned_at": Utc::now().to_rfc3339()
            })),
        ),
        Ok(Err(e)) => {
            error!("Assignment error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e
                })),
            )
        }
        Err(e) => {
            error!("Assignment error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("{:?}", e)
                })),
            )
        }
    }
}



pub async fn transfer_conversation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TransferRequest>,
) -> impl IntoResponse {
    info!(
        "Transferring session {} from {} to {}",
        request.session_id, request.from_attendant_id, request.to_attendant_id
    );

    let result = tokio::task::spawn_blocking({
        let conn = state.conn.clone();
        let session_id = request.session_id;
        let to_attendant = request.to_attendant_id;
        let reason = request.reason.clone();

        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;

            use crate::shared::models::schema::user_sessions;


            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {}", e))?;


            let mut ctx = session.context_data.clone();
            ctx["assigned_to"] = serde_json::json!(to_attendant.to_string());
            ctx["transferred_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["transfer_reason"] = serde_json::json!(reason.unwrap_or_default());
            ctx["status"] = serde_json::json!("transferred");

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set((
                    user_sessions::context_data.eq(&ctx),
                    user_sessions::updated_at.eq(Utc::now()),
                ))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {}", e))?;

            Ok::<(), String>(())
        }
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "session_id": request.session_id,
                "from_attendant": request.from_attendant_id,
                "to_attendant": request.to_attendant_id,
                "transferred_at": Utc::now().to_rfc3339()
            })),
        ),
        Ok(Err(e)) => {
            error!("Transfer error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e
                })),
            )
        }
        Err(e) => {
            error!("Transfer error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("{:?}", e)
                })),
            )
        }
    }
}



pub async fn resolve_conversation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::nil);

    info!("Resolving session {}", session_id);

    let result = tokio::task::spawn_blocking({
        let conn = state.conn.clone();

        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;

            use crate::shared::models::schema::user_sessions;


            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {}", e))?;


            let mut ctx = session.context_data.clone();
            ctx["status"] = serde_json::json!("resolved");
            ctx["resolved_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["resolved"] = serde_json::json!(true);

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set((
                    user_sessions::context_data.eq(&ctx),
                    user_sessions::updated_at.eq(Utc::now()),
                ))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {}", e))?;

            Ok::<(), String>(())
        }
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "session_id": session_id,
                "resolved_at": Utc::now().to_rfc3339()
            })),
        ),
        Ok(Err(e)) => {
            error!("Resolve error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e
                })),
            )
        }
        Err(e) => {
            error!("Resolve error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("{:?}", e)
                })),
            )
        }
    }
}



pub async fn get_insights(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Getting insights for session {}", session_id);

    let result = tokio::task::spawn_blocking({
        let conn = state.conn.clone();
        move || {
            let mut db_conn = conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;

            use crate::shared::models::schema::message_history;


            let messages: Vec<(String, i32)> = message_history::table
                .filter(message_history::session_id.eq(session_id))
                .select((message_history::content_encrypted, message_history::role))
                .order(message_history::created_at.desc())
                .limit(10)
                .load(&mut db_conn)
                .map_err(|e| format!("Failed to load messages: {}", e))?;


            let user_messages: Vec<String> = messages
                .iter()
                .filter(|(_, r)| *r == 0)
                .map(|(c, _)| c.clone())
                .collect();

            let sentiment = if user_messages.iter().any(|m| {
                m.to_lowercase().contains("urgent")
                    || m.to_lowercase().contains("problem")
                    || m.to_lowercase().contains("issue")
            }) {
                "negative"
            } else if user_messages
                .iter()
                .any(|m| m.to_lowercase().contains("thanks") || m.to_lowercase().contains("great"))
            {
                "positive"
            } else {
                "neutral"
            };

            let suggested_reply = if sentiment == "negative" {
                "I understand this is frustrating. Let me help you resolve this immediately."
            } else {
                "How can I assist you further?"
            };

            Ok::<serde_json::Value, String>(serde_json::json!({
                "session_id": session_id,
                "sentiment": sentiment,
                "message_count": messages.len(),
                "suggested_reply": suggested_reply,
                "key_topics": ["support", "technical"],
                "priority": if sentiment == "negative" { "high" } else { "normal" },
                "language": "en"
            }))
        }
    })
    .await;

    match result {
        Ok(Ok(insights)) => (StatusCode::OK, Json(insights)),
        Ok(Err(e)) => {
            error!("Insights error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e
                })),
            )
        }
        Err(e) => {
            error!("Task error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Task error: {}", e)
                })),
            )
        }
    }
}
