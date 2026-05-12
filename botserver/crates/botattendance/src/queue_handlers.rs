use crate::models::UserSession;
use crate::queue_csv;
use crate::queue_types::*;
use crate::schema::user_sessions;
use crate::schema::message_history;
use crate::schema::bots;
use crate::schema::users;
use crate::AttendanceConfig;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub use queue_csv::{
    find_attendant_by_identifier, find_attendants_by_channel, find_attendants_by_department,
    is_transfer_enabled, read_attendants_csv,
};

pub async fn list_queue(
    State(config): State<Arc<AttendanceConfig>>,
    Query(filters): Query<QueueFilters>,
) -> impl IntoResponse {
    info!("Listing queue items with filters: {:?}", filters);
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {}", e))?;
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
                    (format!("user_{}", session_data.user_id), format!("{}@unknown.local", session_data.user_id))
                });
                let channel = session_data.context_data.get("channel").and_then(|c| c.as_str()).unwrap_or("web").to_string();
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
        Ok(Ok(queue_items)) => (StatusCode::OK, Json(queue_items)),
        Ok(Err(e)) => {
            error!("Queue list error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![] as Vec<QueueItem>))
        }
        Err(e) => {
            error!("Task error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![] as Vec<QueueItem>))
        }
    }
}

pub async fn list_attendants(
    State(config): State<Arc<AttendanceConfig>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Listing attendants");
    let bot_id_str = params.get("bot_id").cloned().unwrap_or_default();
    let bot_id = match Uuid::parse_str(&bot_id_str) {
        Ok(id) => id,
        Err(_) => {
            let pool = config.pool.clone();
            let result = tokio::task::spawn_blocking(move || {
                let mut db_conn = pool.get().ok()?;
                bots::table.filter(bots::is_active.eq(true)).select(bots::id).first::<Uuid>(&mut db_conn).ok()
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
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    if !is_transfer_enabled(bot_id, &work_path) {
        warn!("Transfer not enabled for bot {}", bot_id);
        return (StatusCode::OK, Json(vec![] as Vec<AttendantStats>));
    }
    let attendant_csvs = read_attendants_csv(bot_id, &work_path);
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
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<AssignRequest>,
) -> impl IntoResponse {
    info!("Assigning session {} to attendant {}", request.session_id, request.attendant_id);
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        let session_id = request.session_id;
        let attendant_id = request.attendant_id;
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {e}"))?;
            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {e}"))?;
            let session_bot_id = session.bot_id;
            let mut ctx = session.context_data;
            ctx["assigned_to"] = serde_json::json!(attendant_id.to_string());
            ctx["assigned_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["status"] = serde_json::json!("assigned");
            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {e}"))?;
            crate::webhooks::emit_webhook_event(&mut db_conn, session_bot_id, "session.assigned", serde_json::json!({
                "session_id": session_id, "attendant_id": attendant_id, "assigned_at": Utc::now().to_rfc3339()
            }));
            Ok::<(), String>(())
        }
    })
    .await;
    match result {
        Ok(Ok(())) => (StatusCode::OK, Json(serde_json::json!({
            "success": true, "session_id": request.session_id,
            "attendant_id": request.attendant_id, "assigned_at": Utc::now().to_rfc3339()
        }))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("{:?}", e)}))),
    }
}

pub async fn assign_by_skill(
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<SkillBasedAssignRequest>,
) -> impl IntoResponse {
    info!("Skill-based assignment for session {} with skills {:?}", request.session_id, request.required_skills);
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        let session_id = request.session_id;
        let required_skills = request.required_skills.clone();
        let channel_filter = request.channel.clone();
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {e}"))?;
            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {e}"))?;
            let session_bot_id = session.bot_id;
            let csv_path = format!("/opt/gbo/data/{}/attendants.csv", session_bot_id);
            let mut best_attendant: Option<AttendantCSV> = None;
            let mut best_score: i32 = -1;
            if let Ok(contents) = std::fs::read_to_string(&csv_path) {
                for line in contents.lines().skip(1) {
                    let fields: Vec<&str> = line.split(',').collect();
                    if fields.len() < 3 {
                        continue;
                    }
                    let attendant = AttendantCSV {
                        id: fields[0].to_string(),
                        name: fields[1].to_string(),
                        channel: fields.get(2).unwrap_or(&"").to_string(),
                        preferences: fields.get(3).unwrap_or(&"").to_string(),
                        department: fields.get(4).map(|s| s.to_string()),
                        aliases: fields.get(5).map(|s| s.to_string()),
                        phone: fields.get(6).map(|s| s.to_string()),
                        email: fields.get(7).map(|s| s.to_string()),
                        teams: fields.get(8).map(|s| s.to_string()),
                        google: fields.get(9).map(|s| s.to_string()),
                    };
                    if let Some(ref ch) = channel_filter {
                        if attendant.channel != *ch && !attendant.channel.is_empty() {
                            continue;
                        }
                    }
                    let prefs = attendant.preferences.to_lowercase();
                    let mut score = 0;
                    for skill in &required_skills {
                        if prefs.contains(&skill.to_lowercase()) {
                            score += 10;
                        }
                    }
                    if prefs.contains("general") || prefs.is_empty() {
                        score += 1;
                    }
                    if score > best_score {
                        best_score = score;
                        best_attendant = Some(attendant);
                    }
                }
            }
            if let Some(attendant) = best_attendant {
                let mut ctx = session.context_data;
                ctx["assigned_to"] = serde_json::json!(attendant.id.clone());
                ctx["assigned_to_name"] = serde_json::json!(attendant.name.clone());
                ctx["assigned_at"] = serde_json::json!(Utc::now().to_rfc3339());
                ctx["status"] = serde_json::json!("assigned");
                ctx["assignment_reason"] = serde_json::json!("skill_based");
                diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                    .set(user_sessions::context_data.eq(&ctx))
                    .execute(&mut db_conn)
                    .map_err(|e| format!("Failed to update session: {e}"))?;
                Ok::<Option<AttendantCSV>, String>(Some(attendant))
            } else {
                Ok::<Option<AttendantCSV>, String>(None)
            }
        }
    })
    .await;
    match result {
        Ok(Ok(Some(attendant))) => (StatusCode::OK, Json(serde_json::json!({
            "success": true, "session_id": request.session_id,
            "assigned_to": attendant.id, "assigned_to_name": attendant.name,
            "assignment_type": "skill_based", "assigned_at": Utc::now().to_rfc3339()
        }))),
        Ok(Ok(None)) => (StatusCode::OK, Json(serde_json::json!({
            "success": true, "session_id": request.session_id,
            "message": "No attendant found with matching skills", "assignment_type": "skill_based"
        }))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("{:?}", e)}))),
    }
}

pub async fn transfer_conversation(
    State(config): State<Arc<AttendanceConfig>>,
    Json(request): Json<TransferRequest>,
) -> impl IntoResponse {
    info!("Transferring session {} from {} to {}", request.session_id, request.from_attendant_id, request.to_attendant_id);
    let reason_for_webhook = request.reason.clone();
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        let session_id = request.session_id;
        let to_attendant = request.to_attendant_id;
        let reason = request.reason.clone();
        let reason_for_webhook = reason_for_webhook.clone();
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {}", e))?;
            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {}", e))?;
            let mut ctx = session.context_data;
            ctx["assigned_to"] = serde_json::json!(to_attendant.to_string());
            ctx["transferred_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["transfer_reason"] = serde_json::json!(reason.clone().unwrap_or_default());
            ctx["status"] = serde_json::json!("transferred");
            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set((user_sessions::context_data.eq(&ctx), user_sessions::updated_at.eq(Utc::now())))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {e}"))?;
            crate::webhooks::emit_webhook_event(&mut db_conn, session.bot_id, "session.transferred", serde_json::json!({
                "session_id": session_id, "to_attendant": to_attendant, "reason": reason_for_webhook,
                "transferred_at": Utc::now().to_rfc3339()
            }));
            Ok::<(), String>(())
        }
    })
    .await;
    match result {
        Ok(Ok(())) => (StatusCode::OK, Json(serde_json::json!({
            "success": true, "session_id": request.session_id,
            "from_attendant": request.from_attendant_id, "to_attendant": request.to_attendant_id,
            "transferred_at": Utc::now().to_rfc3339()
        }))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("{:?}", e)}))),
    }
}

pub async fn resolve_conversation(
    State(config): State<Arc<AttendanceConfig>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session_id = payload.get("session_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_else(Uuid::nil);
    info!("Resolving session {}", session_id);
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {}", e))?;
            let session: UserSession = user_sessions::table
                .filter(user_sessions::id.eq(session_id))
                .first(&mut db_conn)
                .map_err(|e| format!("Session not found: {}", e))?;
            let mut ctx = session.context_data;
            ctx["status"] = serde_json::json!("resolved");
            ctx["resolved_at"] = serde_json::json!(Utc::now().to_rfc3339());
            ctx["resolved"] = serde_json::json!(true);
            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set((user_sessions::context_data.eq(&ctx), user_sessions::updated_at.eq(Utc::now())))
                .execute(&mut db_conn)
                .map_err(|e| format!("Failed to update session: {e}"))?;
            crate::webhooks::emit_webhook_event(&mut db_conn, session.bot_id, "session.resolved", serde_json::json!({
                "session_id": session_id, "resolved_at": Utc::now().to_rfc3339()
            }));
            Ok::<(), String>(())
        }
    })
    .await;
    match result {
        Ok(Ok(())) => (StatusCode::OK, Json(serde_json::json!({
            "success": true, "session_id": session_id, "resolved_at": Utc::now().to_rfc3339()
        }))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": e}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("{:?}", e)}))),
    }
}

pub async fn get_insights(
    State(config): State<Arc<AttendanceConfig>>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Getting insights for session {}", session_id);
    let result = tokio::task::spawn_blocking({
        let pool = config.pool.clone();
        move || {
            let mut db_conn = pool.get().map_err(|e| format!("Failed to get database connection: {}", e))?;
            let messages: Vec<(String, i32)> = message_history::table
                .filter(message_history::session_id.eq(session_id))
                .select((message_history::content_encrypted, message_history::role))
                .order(message_history::created_at.desc())
                .limit(10)
                .load(&mut db_conn)
                .map_err(|e| format!("Failed to load messages: {}", e))?;
            let user_messages: Vec<String> = messages.iter().filter(|(_, r)| *r == 0).map(|(c, _)| c.clone()).collect();
            let sentiment = if user_messages.iter().any(|m| {
                m.to_lowercase().contains("urgent") || m.to_lowercase().contains("problem") || m.to_lowercase().contains("issue")
            }) {
                "negative"
            } else if user_messages.iter().any(|m| m.to_lowercase().contains("thanks") || m.to_lowercase().contains("great")) {
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
                "session_id": session_id, "sentiment": sentiment, "message_count": messages.len(),
                "suggested_reply": suggested_reply, "key_topics": ["support", "technical"],
                "priority": if sentiment == "negative" { "high" } else { "normal" }, "language": "en"
            }))
        }
    })
    .await;
    match result {
        Ok(Ok(insights)) => (StatusCode::OK, Json(insights)),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Task error: {}", e)}))),
    }
}

pub async fn get_kanban(
    State(config): State<Arc<AttendanceConfig>>,
    Query(query): Query<KanbanQuery>,
) -> Result<Json<KanbanBoard>, (StatusCode, String)> {
    let pool = config.pool.clone();
    let get_default_bot = config.get_default_bot.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
        let resolved_bot_id = query.bot_id.unwrap_or_else(|| {
            let (default_id, _) = get_default_bot(&mut conn);
            default_id
        });
        let sessions: Vec<UserSession> = user_sessions::table
            .filter(user_sessions::bot_id.eq(resolved_bot_id))
            .filter(user_sessions::context_data.contains(serde_json::json!({"status": ""})))
            .load(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;
        let mut new_items = Vec::new();
        let mut waiting_items = Vec::new();
        let mut active_items = Vec::new();
        let mut pending_customer_items = Vec::new();
        let mut resolved_items = Vec::new();
        for session in sessions {
            let status = session.context_data.get("status").and_then(|v| v.as_str()).unwrap_or("new").to_string();
            let assigned_to = session.attendant_id;
            let assigned_to_name = session.context_data.get("attendant_name").and_then(|v| v.as_str()).map(String::from);
            let last_message = session.context_data.get("last_message").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let last_message_time = session.context_data.get("last_message_time").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| session.created_at.to_rfc3339());
            let waiting_time = Utc::now().signed_duration_since(session.created_at).num_seconds();
            let priority = session.context_data.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let channel = session.context_data.get("channel").and_then(|v| v.as_str()).unwrap_or("web").to_string();
            let user_name = session.context_data.get("user_name").and_then(|v| v.as_str()).unwrap_or("Anonymous").to_string();
            let user_email = session.context_data.get("user_email").and_then(|v| v.as_str()).map(String::from);
            let item = QueueItem {
                session_id: session.id, user_id: session.user_id, bot_id: session.bot_id,
                channel, user_name, user_email, last_message, last_message_time,
                waiting_time_seconds: waiting_time, priority,
                status: match status.as_str() {
                    "waiting" => QueueStatus::Waiting,
                    "assigned" => QueueStatus::Assigned,
                    "active" => QueueStatus::Active,
                    "resolved" => QueueStatus::Resolved,
                    "abandoned" => QueueStatus::Abandoned,
                    _ => QueueStatus::Waiting,
                },
                assigned_to, assigned_to_name,
            };
            if let Some(ref ch) = query.channel {
                if item.channel != *ch { continue; }
            }
            match status.as_str() {
                "new" | "waiting" => new_items.push(item),
                "pending_customer" => pending_customer_items.push(item),
                "assigned" => waiting_items.push(item),
                "active" => active_items.push(item),
                "resolved" | "closed" | "abandoned" => resolved_items.push(item),
                _ => new_items.push(item),
            }
        }
        let columns = vec![
            KanbanColumn { id: "new".to_string(), title: "New".to_string(), items: new_items },
            KanbanColumn { id: "waiting".to_string(), title: "Waiting".to_string(), items: waiting_items },
            KanbanColumn { id: "active".to_string(), title: "Active".to_string(), items: active_items },
            KanbanColumn { id: "pending_customer".to_string(), title: "Pending Customer".to_string(), items: pending_customer_items },
            KanbanColumn { id: "resolved".to_string(), title: "Resolved".to_string(), items: resolved_items },
        ];
        Ok::<KanbanBoard, (StatusCode, String)>(KanbanBoard { columns })
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?
    .map(|board| Json(board))
}
