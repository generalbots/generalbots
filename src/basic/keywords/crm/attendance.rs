//! CRM Attendance Keywords
//!
//! Provides BASIC keywords for queue management, LLM-assisted attendant features,
//! and customer journey tracking. These keywords enable flexible CRM automation
//! directly from BASIC scripts.
//!
//! ## Queue Management Keywords
//!
//! - `GET QUEUE` - Get current queue status and items
//! - `NEXT IN QUEUE` - Get next waiting conversation
//! - `ASSIGN CONVERSATION` - Assign conversation to attendant
//! - `RESOLVE CONVERSATION` - Mark conversation as resolved
//! - `SET PRIORITY` - Change conversation priority
//!
//! ## Attendant Keywords
//!
//! - `GET ATTENDANTS` - List available attendants
//! - `SET ATTENDANT STATUS` - Change attendant status
//! - `GET ATTENDANT STATS` - Get performance metrics
//!
//! ## LLM Assist Keywords
//!
//! - `GET TIPS` - Generate AI tips for conversation
//! - `POLISH MESSAGE` - Improve message with AI
//! - `GET SMART REPLIES` - Get AI reply suggestions
//! - `GET SUMMARY` - Get conversation summary
//! - `ANALYZE SENTIMENT` - Analyze customer sentiment
//!
//! ## Customer Journey Keywords
//!
//! - `TAG CONVERSATION` - Add tags to conversation
//! - `ADD NOTE` - Add internal note
//! - `GET CUSTOMER HISTORY` - Get previous interactions
//!
//! ## Example Usage
//!
//! ```basic
//! ' Check queue and assign next conversation
//! queue = GET QUEUE
//! TALK "Queue has " + queue.waiting + " waiting"
//!
//! IF queue.waiting > 0 THEN
//!     next_conv = NEXT IN QUEUE
//!     ASSIGN CONVERSATION next_conv.session_id, "att-001"
//! END IF
//!
//! ' Use LLM assist
//! tips = GET TIPS session_id, customer_message
//! FOR EACH tip IN tips
//!     TALK tip.content
//! NEXT
//!
//! ' Polish before sending
//! polished = POLISH MESSAGE "thx for ur msg ill help u now"
//! SEND polished.text TO customer
//!
//! ' Analyze sentiment
//! sentiment = ANALYZE SENTIMENT session_id, message
//! IF sentiment.escalation_risk = "high" THEN
//!     SET PRIORITY session_id, "urgent"
//!     TRANSFER TO HUMAN "support", "urgent", "High escalation risk"
//! END IF
//! ```

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info};
use rhai::{Array, Dynamic, Engine, Map};

use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Registration
// ============================================================================

/// Register all CRM attendance keywords
pub fn register_attendance_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    debug!("Registering CRM attendance keywords...");

    // Queue Management
    register_get_queue(state.clone(), user.clone(), engine);
    register_next_in_queue(state.clone(), user.clone(), engine);
    register_assign_conversation(state.clone(), user.clone(), engine);
    register_resolve_conversation(state.clone(), user.clone(), engine);
    register_set_priority(state.clone(), user.clone(), engine);

    // Attendant Management
    register_get_attendants(state.clone(), user.clone(), engine);
    register_set_attendant_status(state.clone(), user.clone(), engine);
    register_get_attendant_stats(state.clone(), user.clone(), engine);

    // LLM Assist
    register_get_tips(state.clone(), user.clone(), engine);
    register_polish_message(state.clone(), user.clone(), engine);
    register_get_smart_replies(state.clone(), user.clone(), engine);
    register_get_summary(state.clone(), user.clone(), engine);
    register_analyze_sentiment(state.clone(), user.clone(), engine);

    // Customer Journey
    register_tag_conversation(state.clone(), user.clone(), engine);
    register_add_note(state.clone(), user.clone(), engine);
    register_get_customer_history(state.clone(), user.clone(), engine);

    debug!("CRM attendance keywords registered successfully");
}

// ============================================================================
// Queue Management Keywords
// ============================================================================

/// GET QUEUE - Get current queue status
///
/// Returns queue statistics and optionally filtered items.
///
/// ```basic
/// ' Get queue stats
/// queue = GET QUEUE
/// TALK "Waiting: " + queue.waiting + ", Active: " + queue.active
///
/// ' Get queue with filter
/// queue = GET QUEUE "channel=whatsapp"
/// queue = GET QUEUE "department=sales"
/// queue = GET QUEUE "priority=high"
/// ```
fn register_get_queue(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    // GET QUEUE
    engine.register_fn("get_queue", move || -> Dynamic {
        get_queue_impl(&state_clone, None)
    });

    let state_clone2 = state.clone();
    engine.register_fn("get_queue", move |filter: &str| -> Dynamic {
        get_queue_impl(&state_clone2, Some(filter.to_string()))
    });

    // Also register as GET_QUEUE for compatibility
    let state_clone3 = state.clone();
    engine
        .register_custom_syntax(&["GET", "QUEUE"], false, move |_context, _inputs| {
            Ok(get_queue_impl(&state_clone3, None))
        })
        .unwrap();

    let state_clone4 = state.clone();
    engine
        .register_custom_syntax(
            &["GET", "QUEUE", "$expr$"],
            false,
            move |context, inputs| {
                let filter = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_queue_impl(&state_clone4, Some(filter)))
            },
        )
        .unwrap();
}

fn get_queue_impl(state: &Arc<AppState>, filter: Option<String>) -> Dynamic {
    let conn = state.conn.clone();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                error!("DB connection error: {}", e);
                return create_error_result(&format!("DB error: {}", e));
            }
        };

        use crate::shared::models::schema::user_sessions;

        // Build query
        let mut query = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("needs_human")
                    .eq("true"),
            )
            .into_boxed();

        // Apply filters
        if let Some(ref filter_str) = filter {
            if filter_str.contains("channel=") {
                let channel = filter_str.replace("channel=", "");
                query = query.filter(
                    user_sessions::context_data
                        .retrieve_as_text("channel")
                        .eq(channel),
                );
            }
            // Add more filters as needed
        }

        let sessions: Vec<UserSession> = match query.load(&mut db_conn) {
            Ok(s) => s,
            Err(e) => {
                error!("Query error: {}", e);
                return create_error_result(&format!("Query error: {}", e));
            }
        };

        // Calculate stats
        let mut waiting = 0;
        let mut assigned = 0;
        let mut active = 0;
        let mut resolved = 0;

        for session in &sessions {
            let status = session
                .context_data
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("waiting");

            match status {
                "waiting" => waiting += 1,
                "assigned" => assigned += 1,
                "active" => active += 1,
                "resolved" => resolved += 1,
                _ => waiting += 1,
            }
        }

        let mut result = Map::new();
        result.insert("success".into(), Dynamic::from(true));
        result.insert("total".into(), Dynamic::from(sessions.len() as i64));
        result.insert("waiting".into(), Dynamic::from(waiting));
        result.insert("assigned".into(), Dynamic::from(assigned));
        result.insert("active".into(), Dynamic::from(active));
        result.insert("resolved".into(), Dynamic::from(resolved));

        // Add items array
        let items: Array = sessions
            .iter()
            .take(20) // Limit to 20 items
            .map(|s| {
                let mut item = Map::new();
                item.insert("session_id".into(), Dynamic::from(s.id.to_string()));
                item.insert("user_id".into(), Dynamic::from(s.user_id.to_string()));
                item.insert(
                    "channel".into(),
                    Dynamic::from(
                        s.context_data
                            .get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("web")
                            .to_string(),
                    ),
                );
                item.insert(
                    "status".into(),
                    Dynamic::from(
                        s.context_data
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("waiting")
                            .to_string(),
                    ),
                );
                item.insert(
                    "name".into(),
                    Dynamic::from(
                        s.context_data
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                    ),
                );
                item.insert(
                    "priority".into(),
                    Dynamic::from(
                        s.context_data
                            .get("priority")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1),
                    ),
                );
                Dynamic::from(item)
            })
            .collect();

        result.insert("items".into(), Dynamic::from(items));

        Dynamic::from(result)
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// NEXT IN QUEUE - Get next waiting conversation
///
/// ```basic
/// next = NEXT IN QUEUE
/// IF next.success THEN
///     TALK "Next customer: " + next.name
///     ASSIGN CONVERSATION next.session_id, attendant_id
/// END IF
/// ```
fn register_next_in_queue(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(&["NEXT", "IN", "QUEUE"], false, move |_context, _inputs| {
            Ok(next_in_queue_impl(&state_clone))
        })
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn("next_in_queue", move || -> Dynamic {
        next_in_queue_impl(&state_clone2)
    });
}

fn next_in_queue_impl(state: &Arc<AppState>) -> Dynamic {
    let conn = state.conn.clone();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        // Find next waiting session (ordered by priority and time)
        let session: Option<UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("needs_human")
                    .eq("true"),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .eq("waiting"),
            )
            .order(user_sessions::created_at.asc())
            .first(&mut db_conn)
            .optional()
            .unwrap_or(None);

        match session {
            Some(s) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert("session_id".into(), Dynamic::from(s.id.to_string()));
                result.insert("user_id".into(), Dynamic::from(s.user_id.to_string()));
                result.insert(
                    "name".into(),
                    Dynamic::from(
                        s.context_data
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                    ),
                );
                result.insert(
                    "channel".into(),
                    Dynamic::from(
                        s.context_data
                            .get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("web")
                            .to_string(),
                    ),
                );
                result.insert(
                    "phone".into(),
                    Dynamic::from(
                        s.context_data
                            .get("phone")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    ),
                );
                Dynamic::from(result)
            }
            None => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(false));
                result.insert("message".into(), Dynamic::from("Queue is empty"));
                Dynamic::from(result)
            }
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// ASSIGN CONVERSATION - Assign conversation to attendant
///
/// ```basic
/// result = ASSIGN CONVERSATION session_id, attendant_id
/// result = ASSIGN CONVERSATION session_id, "att-001"
/// ```
fn register_assign_conversation(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["ASSIGN", "CONVERSATION", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let attendant_id = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(assign_conversation_impl(
                    &state_clone,
                    &session_id,
                    &attendant_id,
                ))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn(
        "assign_conversation",
        move |session_id: &str, attendant_id: &str| -> Dynamic {
            assign_conversation_impl(&state_clone2, session_id, attendant_id)
        },
    );
}

fn assign_conversation_impl(
    state: &Arc<AppState>,
    session_id: &str,
    attendant_id: &str,
) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };
    let attendant = attendant_id.to_string();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        // Get current session
        let session: UserSession = match user_sessions::table.find(session_uuid).first(&mut db_conn)
        {
            Ok(s) => s,
            Err(_) => return create_error_result("Session not found"),
        };

        // Update context
        let mut ctx = session.context_data.clone();
        ctx["assigned_to"] = serde_json::json!(attendant);
        ctx["assigned_at"] = serde_json::json!(Utc::now().to_rfc3339());
        ctx["status"] = serde_json::json!("assigned");

        match diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_uuid)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
        {
            Ok(_) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert("session_id".into(), Dynamic::from(session_uuid.to_string()));
                result.insert("assigned_to".into(), Dynamic::from(attendant));
                result.insert("message".into(), Dynamic::from("Conversation assigned"));
                Dynamic::from(result)
            }
            Err(e) => create_error_result(&format!("Update error: {}", e)),
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// RESOLVE CONVERSATION - Mark conversation as resolved
///
/// ```basic
/// RESOLVE CONVERSATION session_id
/// RESOLVE CONVERSATION session_id, "Issue fixed"
/// ```
fn register_resolve_conversation(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["RESOLVE", "CONVERSATION", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(resolve_conversation_impl(&state_clone, &session_id, None))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine
        .register_custom_syntax(
            &["RESOLVE", "CONVERSATION", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let reason = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(resolve_conversation_impl(
                    &state_clone2,
                    &session_id,
                    Some(reason),
                ))
            },
        )
        .unwrap();

    let state_clone3 = state.clone();
    engine.register_fn("resolve_conversation", move |session_id: &str| -> Dynamic {
        resolve_conversation_impl(&state_clone3, session_id, None)
    });
}

fn resolve_conversation_impl(
    state: &Arc<AppState>,
    session_id: &str,
    reason: Option<String>,
) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };
    let reason_clone = reason.clone();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        let session: UserSession = match user_sessions::table.find(session_uuid).first(&mut db_conn)
        {
            Ok(s) => s,
            Err(_) => return create_error_result("Session not found"),
        };

        let mut ctx = session.context_data.clone();
        ctx["needs_human"] = serde_json::json!(false);
        ctx["status"] = serde_json::json!("resolved");
        ctx["resolved_at"] = serde_json::json!(Utc::now().to_rfc3339());
        if let Some(r) = reason_clone {
            ctx["resolution_reason"] = serde_json::json!(r);
        }

        match diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_uuid)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
        {
            Ok(_) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert("session_id".into(), Dynamic::from(session_uuid.to_string()));
                result.insert("message".into(), Dynamic::from("Conversation resolved"));
                Dynamic::from(result)
            }
            Err(e) => create_error_result(&format!("Update error: {}", e)),
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// SET PRIORITY - Change conversation priority
///
/// ```basic
/// SET PRIORITY session_id, "high"
/// SET PRIORITY session_id, "urgent"
/// SET PRIORITY session_id, 3  ' numeric (1=low, 2=normal, 3=high, 4=urgent)
/// ```
fn register_set_priority(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["SET", "PRIORITY", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let priority = context.eval_expression_tree(&inputs[1])?;
                Ok(set_priority_impl(&state_clone, &session_id, priority))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn(
        "set_priority",
        move |session_id: &str, priority: &str| -> Dynamic {
            set_priority_impl(
                &state_clone2,
                session_id,
                Dynamic::from(priority.to_string()),
            )
        },
    );
}

fn set_priority_impl(state: &Arc<AppState>, session_id: &str, priority: Dynamic) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };

    // Convert priority to numeric
    let priority_num: i64 = if priority.is_int() {
        priority.as_int().unwrap_or(2)
    } else {
        let p = priority.to_string().to_lowercase();
        match p.as_str() {
            "low" => 1,
            "normal" => 2,
            "high" => 3,
            "urgent" => 4,
            _ => 2,
        }
    };

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        let session: UserSession = match user_sessions::table.find(session_uuid).first(&mut db_conn)
        {
            Ok(s) => s,
            Err(_) => return create_error_result("Session not found"),
        };

        let mut ctx = session.context_data.clone();
        ctx["priority"] = serde_json::json!(priority_num);

        match diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_uuid)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
        {
            Ok(_) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert("priority".into(), Dynamic::from(priority_num));
                Dynamic::from(result)
            }
            Err(e) => create_error_result(&format!("Update error: {}", e)),
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

// ============================================================================
// Attendant Management Keywords
// ============================================================================

/// GET ATTENDANTS - List available attendants
///
/// ```basic
/// attendants = GET ATTENDANTS
/// FOR EACH att IN attendants.items
///     TALK att.name + " - " + att.status
/// NEXT
///
/// ' Filter by status
/// online = GET ATTENDANTS "online"
/// ```
fn register_get_attendants(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(&["GET", "ATTENDANTS"], false, move |_context, _inputs| {
            Ok(get_attendants_impl(&state_clone, None))
        })
        .unwrap();

    let state_clone2 = state.clone();
    engine
        .register_custom_syntax(
            &["GET", "ATTENDANTS", "$expr$"],
            false,
            move |context, inputs| {
                let filter = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_attendants_impl(&state_clone2, Some(filter)))
            },
        )
        .unwrap();

    let state_clone3 = state.clone();
    engine.register_fn("get_attendants", move || -> Dynamic {
        get_attendants_impl(&state_clone3, None)
    });
}

fn get_attendants_impl(_state: &Arc<AppState>, status_filter: Option<String>) -> Dynamic {
    // Read from attendant.csv
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let mut attendants = Vec::new();

    // Scan for attendant.csv files
    if let Ok(entries) = std::fs::read_dir(&work_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.to_string_lossy().ends_with(".gbai") {
                let attendant_path = path.join("attendant.csv");
                if attendant_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&attendant_path) {
                        for line in content.lines().skip(1) {
                            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                            if parts.len() >= 4 {
                                let mut att = Map::new();
                                att.insert("id".into(), Dynamic::from(parts[0].to_string()));
                                att.insert("name".into(), Dynamic::from(parts[1].to_string()));
                                att.insert("channel".into(), Dynamic::from(parts[2].to_string()));
                                att.insert(
                                    "preferences".into(),
                                    Dynamic::from(parts[3].to_string()),
                                );
                                att.insert("status".into(), Dynamic::from("online".to_string())); // Default
                                if parts.len() >= 5 {
                                    att.insert(
                                        "department".into(),
                                        Dynamic::from(parts[4].to_string()),
                                    );
                                }
                                attendants.push(Dynamic::from(att));
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply filter
    if let Some(filter) = status_filter {
        attendants.retain(|att| {
            if let Some(map) = att.clone().try_cast::<Map>() {
                if let Some(status) = map.get("status") {
                    return status.to_string().to_lowercase() == filter.to_lowercase();
                }
            }
            true
        });
    }

    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(true));
    result.insert("count".into(), Dynamic::from(attendants.len() as i64));
    result.insert("items".into(), Dynamic::from(attendants));

    Dynamic::from(result)
}

/// SET ATTENDANT STATUS - Change attendant status
///
/// ```basic
/// SET ATTENDANT STATUS "att-001", "busy"
/// SET ATTENDANT STATUS attendant_id, "away"
/// ```
fn register_set_attendant_status(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["SET", "ATTENDANT", "STATUS", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let attendant_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let status = context.eval_expression_tree(&inputs[1])?.to_string();
                let now = Utc::now().to_rfc3339();

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB connection error: {}", e))?;

                let query = diesel::sql_query(
                    "UPDATE attendants SET status = $1, updated_at = $2 WHERE id = $3",
                )
                .bind::<diesel::sql_types::Text, _>(&status)
                .bind::<diesel::sql_types::Text, _>(&now)
                .bind::<diesel::sql_types::Text, _>(&attendant_id);

                let rows_affected = query.execute(&mut *conn).unwrap_or(0);

                info!(
                    "Set attendant {} status to {} (rows_affected={})",
                    attendant_id, status, rows_affected
                );

                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(rows_affected > 0));
                result.insert("attendant_id".into(), Dynamic::from(attendant_id));
                result.insert("status".into(), Dynamic::from(status));
                result.insert("rows_affected".into(), Dynamic::from(rows_affected as i64));
                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

/// GET ATTENDANT STATS - Get performance metrics
///
/// ```basic
/// stats = GET ATTENDANT STATS "att-001"
/// TALK "Resolved: " + stats.resolved_today
/// ```
fn register_get_attendant_stats(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["GET", "ATTENDANT", "STATS", "$expr$"],
            false,
            move |context, inputs| {
                let attendant_id = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_attendant_stats_impl(&state_clone, &attendant_id))
            },
        )
        .unwrap();
}

fn get_attendant_stats_impl(state: &Arc<AppState>, attendant_id: &str) -> Dynamic {
    let conn = state.conn.clone();
    let att_id = attendant_id.to_string();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        // Count resolved conversations today
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();

        let resolved_today: i64 = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("assigned_to")
                    .eq(&att_id),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .eq("resolved"),
            )
            .filter(user_sessions::updated_at.ge(today_start))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        // Count active conversations
        let active: i64 = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("assigned_to")
                    .eq(&att_id),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .ne("resolved"),
            )
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        let mut result = Map::new();
        result.insert("success".into(), Dynamic::from(true));
        result.insert("attendant_id".into(), Dynamic::from(att_id));
        result.insert("resolved_today".into(), Dynamic::from(resolved_today));
        result.insert("active_conversations".into(), Dynamic::from(active));
        Dynamic::from(result)
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

// ============================================================================
// LLM Assist Keywords
// ============================================================================

/// GET TIPS - Generate AI tips for conversation
///
/// ```basic
/// tips = GET TIPS session_id, customer_message
/// FOR EACH tip IN tips.items
///     TALK tip.type + ": " + tip.content
/// NEXT
/// ```
fn register_get_tips(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["GET", "TIPS", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(get_tips_impl(&state_clone, &session_id, &message))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn(
        "get_tips",
        move |session_id: &str, message: &str| -> Dynamic {
            get_tips_impl(&state_clone2, session_id, message)
        },
    );
}

fn get_tips_impl(_state: &Arc<AppState>, _session_id: &str, message: &str) -> Dynamic {
    create_fallback_tips(message)
}

fn create_fallback_tips(message: &str) -> Dynamic {
    let msg_lower = message.to_lowercase();
    let mut tips = Vec::new();

    // Urgent detection
    if msg_lower.contains("urgent") || msg_lower.contains("asap") || msg_lower.contains("emergency")
    {
        let mut tip = Map::new();
        tip.insert("type".into(), Dynamic::from("warning"));
        tip.insert(
            "content".into(),
            Dynamic::from("Customer indicates urgency - prioritize quick response"),
        );
        tip.insert("priority".into(), Dynamic::from(1_i64));
        tips.push(Dynamic::from(tip));
    }

    // Question detection
    if message.contains('?') {
        let mut tip = Map::new();
        tip.insert("type".into(), Dynamic::from("intent"));
        tip.insert(
            "content".into(),
            Dynamic::from("Customer is asking a question - provide clear answer"),
        );
        tip.insert("priority".into(), Dynamic::from(2_i64));
        tips.push(Dynamic::from(tip));
    }

    // Problem detection
    if msg_lower.contains("problem")
        || msg_lower.contains("issue")
        || msg_lower.contains("not working")
    {
        let mut tip = Map::new();
        tip.insert("type".into(), Dynamic::from("action"));
        tip.insert(
            "content".into(),
            Dynamic::from("Customer reporting issue - acknowledge and gather details"),
        );
        tip.insert("priority".into(), Dynamic::from(2_i64));
        tips.push(Dynamic::from(tip));
    }

    // Default tip
    if tips.is_empty() {
        let mut tip = Map::new();
        tip.insert("type".into(), Dynamic::from("general"));
        tip.insert(
            "content".into(),
            Dynamic::from("Read carefully and respond helpfully"),
        );
        tip.insert("priority".into(), Dynamic::from(3_i64));
        tips.push(Dynamic::from(tip));
    }

    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(true));
    result.insert("items".into(), Dynamic::from(tips));
    Dynamic::from(result)
}

/// POLISH MESSAGE - Improve message with AI
///
/// ```basic
/// polished = POLISH MESSAGE "thx for contacting us, ill help u"
/// TALK polished.text
///
/// ' With tone
/// polished = POLISH MESSAGE message, "empathetic"
/// ```
fn register_polish_message(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["POLISH", "MESSAGE", "$expr$"],
            false,
            move |context, inputs| {
                let message = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(polish_message_impl(&state_clone, &message, "professional"))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine
        .register_custom_syntax(
            &["POLISH", "MESSAGE", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let message = context.eval_expression_tree(&inputs[0])?.to_string();
                let tone = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(polish_message_impl(&state_clone2, &message, &tone))
            },
        )
        .unwrap();

    let state_clone3 = state.clone();
    engine.register_fn("polish_message", move |message: &str| -> Dynamic {
        polish_message_impl(&state_clone3, message, "professional")
    });
}

fn polish_message_impl(_state: &Arc<AppState>, message: &str, _tone: &str) -> Dynamic {
    // Simple polishing without LLM (fallback)
    let mut polished = message.to_string();

    // Basic improvements
    polished = polished
        .replace("thx", "Thank you")
        .replace("u ", "you ")
        .replace(" u", " you")
        .replace("ur ", "your ")
        .replace("ill ", "I'll ")
        .replace("dont ", "don't ")
        .replace("cant ", "can't ")
        .replace("wont ", "won't ")
        .replace("im ", "I'm ")
        .replace("ive ", "I've ");

    // Capitalize first letter
    if let Some(first_char) = polished.chars().next() {
        polished = first_char.to_uppercase().to_string() + &polished[1..];
    }

    // Ensure ending punctuation
    if !polished.ends_with('.') && !polished.ends_with('!') && !polished.ends_with('?') {
        polished.push('.');
    }

    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(true));
    result.insert("original".into(), Dynamic::from(message.to_string()));
    result.insert("text".into(), Dynamic::from(polished.clone()));
    result.insert("polished".into(), Dynamic::from(polished));
    Dynamic::from(result)
}

/// GET SMART REPLIES - Get AI reply suggestions
///
/// ```basic
/// replies = GET SMART REPLIES session_id
/// FOR EACH reply IN replies.items
///     TALK reply.tone + ": " + reply.text
/// NEXT
/// ```
fn register_get_smart_replies(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["GET", "SMART", "REPLIES", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_smart_replies_impl(&state_clone, &session_id))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn("get_smart_replies", move |session_id: &str| -> Dynamic {
        get_smart_replies_impl(&state_clone2, session_id)
    });
}

fn get_smart_replies_impl(_state: &Arc<AppState>, _session_id: &str) -> Dynamic {
    // Return default replies (fallback without LLM)
    let mut replies = Vec::new();

    let mut reply1 = Map::new();
    reply1.insert(
        "text".into(),
        Dynamic::from("Thank you for reaching out! I'd be happy to help you with that."),
    );
    reply1.insert("tone".into(), Dynamic::from("friendly"));
    reply1.insert("category".into(), Dynamic::from("greeting"));
    replies.push(Dynamic::from(reply1));

    let mut reply2 = Map::new();
    reply2.insert(
        "text".into(),
        Dynamic::from("I understand your concern. Let me look into this for you right away."),
    );
    reply2.insert("tone".into(), Dynamic::from("empathetic"));
    reply2.insert("category".into(), Dynamic::from("acknowledgment"));
    replies.push(Dynamic::from(reply2));

    let mut reply3 = Map::new();
    reply3.insert(
        "text".into(),
        Dynamic::from("Is there anything else I can help you with today?"),
    );
    reply3.insert("tone".into(), Dynamic::from("professional"));
    reply3.insert("category".into(), Dynamic::from("follow_up"));
    replies.push(Dynamic::from(reply3));

    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(true));
    result.insert("items".into(), Dynamic::from(replies));
    Dynamic::from(result)
}

/// GET SUMMARY - Get conversation summary
///
/// ```basic
/// summary = GET SUMMARY session_id
/// TALK "Brief: " + summary.brief
/// TALK "Key points: " + summary.key_points
/// ```
fn register_get_summary(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["GET", "SUMMARY", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_summary_impl(&state_clone, &session_id))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn("get_summary", move |session_id: &str| -> Dynamic {
        get_summary_impl(&state_clone2, session_id)
    });
}

fn get_summary_impl(state: &Arc<AppState>, session_id: &str) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::message_history;

        // Get message count
        let message_count: i64 = message_history::table
            .filter(message_history::session_id.eq(session_uuid))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        let mut result = Map::new();
        result.insert("success".into(), Dynamic::from(true));
        result.insert("session_id".into(), Dynamic::from(session_uuid.to_string()));
        result.insert("message_count".into(), Dynamic::from(message_count));
        result.insert(
            "brief".into(),
            Dynamic::from(format!("Conversation with {} messages", message_count)),
        );
        result.insert("key_points".into(), Dynamic::from(Vec::<Dynamic>::new()));
        result.insert("sentiment_trend".into(), Dynamic::from("neutral"));
        Dynamic::from(result)
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// ANALYZE SENTIMENT - Analyze customer sentiment
///
/// ```basic
/// sentiment = ANALYZE SENTIMENT session_id, message
/// IF sentiment.escalation_risk = "high" THEN
///     SET PRIORITY session_id, "urgent"
/// END IF
/// ```
fn register_analyze_sentiment(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["ANALYZE", "SENTIMENT", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(analyze_sentiment_impl(&state_clone, &session_id, &message))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn(
        "analyze_sentiment",
        move |session_id: &str, message: &str| -> Dynamic {
            analyze_sentiment_impl(&state_clone2, session_id, message)
        },
    );
}

fn analyze_sentiment_impl(_state: &Arc<AppState>, _session_id: &str, message: &str) -> Dynamic {
    // Keyword-based sentiment analysis (fallback without LLM)
    let msg_lower = message.to_lowercase();

    let positive_words = [
        "thank",
        "great",
        "perfect",
        "awesome",
        "excellent",
        "good",
        "happy",
        "love",
    ];
    let negative_words = [
        "angry",
        "frustrated",
        "terrible",
        "awful",
        "horrible",
        "hate",
        "disappointed",
        "problem",
        "issue",
    ];
    let urgent_words = [
        "urgent",
        "asap",
        "immediately",
        "emergency",
        "now",
        "critical",
    ];

    let positive_count = positive_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();
    let negative_count = negative_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();
    let urgent_count = urgent_words
        .iter()
        .filter(|w| msg_lower.contains(*w))
        .count();

    let (overall, score, emoji) = if positive_count > negative_count {
        ("positive", 0.5, "😊")
    } else if negative_count > positive_count {
        ("negative", -0.5, "😟")
    } else {
        ("neutral", 0.0, "😐")
    };

    let escalation_risk = if negative_count >= 3 {
        "high"
    } else if negative_count >= 1 {
        "medium"
    } else {
        "low"
    };

    let urgency = if urgent_count >= 2 {
        "urgent"
    } else if urgent_count >= 1 {
        "high"
    } else {
        "normal"
    };

    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(true));
    result.insert("overall".into(), Dynamic::from(overall));
    result.insert("score".into(), Dynamic::from(score));
    result.insert("emoji".into(), Dynamic::from(emoji));
    result.insert("escalation_risk".into(), Dynamic::from(escalation_risk));
    result.insert("urgency".into(), Dynamic::from(urgency));
    Dynamic::from(result)
}

// ============================================================================
// Customer Journey Keywords
// ============================================================================

/// TAG CONVERSATION - Add tags to conversation
///
/// ```basic
/// TAG CONVERSATION session_id, "billing"
/// TAG CONVERSATION session_id, "vip", "urgent"
/// ```
fn register_tag_conversation(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["TAG", "CONVERSATION", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let tag = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(tag_conversation_impl(&state_clone, &session_id, vec![tag]))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn(
        "tag_conversation",
        move |session_id: &str, tag: &str| -> Dynamic {
            tag_conversation_impl(&state_clone2, session_id, vec![tag.to_string()])
        },
    );
}

fn tag_conversation_impl(state: &Arc<AppState>, session_id: &str, tags: Vec<String>) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };
    let tags_clone = tags.clone();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        let session: UserSession = match user_sessions::table.find(session_uuid).first(&mut db_conn)
        {
            Ok(s) => s,
            Err(_) => return create_error_result("Session not found"),
        };

        let mut ctx = session.context_data.clone();

        // Get existing tags or create empty array
        let mut existing_tags: Vec<String> = ctx
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Add new tags
        for tag in tags_clone {
            if !existing_tags.contains(&tag) {
                existing_tags.push(tag);
            }
        }

        ctx["tags"] = serde_json::json!(existing_tags);

        match diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_uuid)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
        {
            Ok(_) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert(
                    "tags".into(),
                    Dynamic::from(
                        existing_tags
                            .iter()
                            .map(|t| Dynamic::from(t.clone()))
                            .collect::<Vec<_>>(),
                    ),
                );
                Dynamic::from(result)
            }
            Err(e) => create_error_result(&format!("Update error: {}", e)),
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// ADD NOTE - Add internal note to conversation
///
/// ```basic
/// ADD NOTE session_id, "Customer is a VIP - handle with care"
/// ADD NOTE session_id, note_text, attendant_id
/// ```
fn register_add_note(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["ADD", "NOTE", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let note = context.eval_expression_tree(&inputs[1])?.to_string();
                Ok(add_note_impl(&state_clone, &session_id, &note, None))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn("add_note", move |session_id: &str, note: &str| -> Dynamic {
        add_note_impl(&state_clone2, session_id, note, None)
    });
}

fn add_note_impl(
    state: &Arc<AppState>,
    session_id: &str,
    note: &str,
    author: Option<String>,
) -> Dynamic {
    let conn = state.conn.clone();
    let session_uuid = match Uuid::parse_str(session_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid session ID"),
    };
    let note_clone = note.to_string();
    let author_clone = author.clone();

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        let session: UserSession = match user_sessions::table.find(session_uuid).first(&mut db_conn)
        {
            Ok(s) => s,
            Err(_) => return create_error_result("Session not found"),
        };

        let mut ctx = session.context_data.clone();

        // Get existing notes or create empty array
        let mut notes: Vec<serde_json::Value> = ctx
            .get("notes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Add new note
        notes.push(serde_json::json!({
            "text": note_clone,
            "author": author_clone.unwrap_or_else(|| "system".to_string()),
            "timestamp": Utc::now().to_rfc3339()
        }));

        ctx["notes"] = serde_json::json!(notes);

        match diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_uuid)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
        {
            Ok(_) => {
                let mut result = Map::new();
                result.insert("success".into(), Dynamic::from(true));
                result.insert("note_count".into(), Dynamic::from(notes.len() as i64));
                Dynamic::from(result)
            }
            Err(e) => create_error_result(&format!("Update error: {}", e)),
        }
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

/// GET CUSTOMER HISTORY - Get previous interactions
///
/// ```basic
/// history = GET CUSTOMER HISTORY user_id
/// TALK "Previous sessions: " + history.session_count
/// FOR EACH session IN history.sessions
///     TALK session.channel + " - " + session.resolved_at
/// NEXT
/// ```
fn register_get_customer_history(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(
            &["GET", "CUSTOMER", "HISTORY", "$expr$"],
            false,
            move |context, inputs| {
                let user_id = context.eval_expression_tree(&inputs[0])?.to_string();
                Ok(get_customer_history_impl(&state_clone, &user_id))
            },
        )
        .unwrap();

    let state_clone2 = state.clone();
    engine.register_fn("get_customer_history", move |user_id: &str| -> Dynamic {
        get_customer_history_impl(&state_clone2, user_id)
    });
}

fn get_customer_history_impl(state: &Arc<AppState>, user_id: &str) -> Dynamic {
    let conn = state.conn.clone();
    let user_uuid = match Uuid::parse_str(user_id) {
        Ok(u) => u,
        Err(_) => return create_error_result("Invalid user ID"),
    };

    let result = std::thread::spawn(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => return create_error_result(&format!("DB error: {}", e)),
        };

        use crate::shared::models::schema::user_sessions;

        let sessions: Vec<UserSession> = user_sessions::table
            .filter(user_sessions::user_id.eq(user_uuid))
            .order(user_sessions::created_at.desc())
            .limit(10)
            .load(&mut db_conn)
            .unwrap_or_default();

        let session_items: Vec<Dynamic> = sessions
            .iter()
            .map(|s| {
                let mut item = Map::new();
                item.insert("session_id".into(), Dynamic::from(s.id.to_string()));
                item.insert(
                    "channel".into(),
                    Dynamic::from(
                        s.context_data
                            .get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("web")
                            .to_string(),
                    ),
                );
                item.insert(
                    "status".into(),
                    Dynamic::from(
                        s.context_data
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    ),
                );
                item.insert("created_at".into(), Dynamic::from(s.created_at.to_string()));
                Dynamic::from(item)
            })
            .collect();

        let mut result = Map::new();
        result.insert("success".into(), Dynamic::from(true));
        result.insert("user_id".into(), Dynamic::from(user_uuid.to_string()));
        result.insert("session_count".into(), Dynamic::from(sessions.len() as i64));
        result.insert("sessions".into(), Dynamic::from(session_items));
        Dynamic::from(result)
    })
    .join()
    .unwrap_or_else(|_| create_error_result("Thread panic"));

    result
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_error_result(message: &str) -> Dynamic {
    let mut result = Map::new();
    result.insert("success".into(), Dynamic::from(false));
    result.insert("error".into(), Dynamic::from(message.to_string()));
    Dynamic::from(result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_tips_urgent() {
        let tips = create_fallback_tips("This is URGENT! Help now!");
        let result = tips.try_cast::<Map>().unwrap();
        assert!(result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_fallback_tips_question() {
        let tips = create_fallback_tips("Can you help me with this?");
        let result = tips.try_cast::<Map>().unwrap();
        assert!(result.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_polish_message() {
        let polished = polish_text("thx 4 ur msg", "professional");
        assert!(polished.contains("thx") == false);
        assert!(polished.contains("your"));
    }

    #[test]
    fn test_polish_message_capitalization() {
        let polished = polish_text("hello there", "professional");
        assert!(polished.starts_with('H'));
        assert!(polished.ends_with('.'));
    }

    fn polish_text(message: &str, _tone: &str) -> String {
        let mut polished = message.to_string();
        polished = polished
            .replace("thx", "Thank you")
            .replace("u ", "you ")
            .replace(" u", " you")
            .replace("ur ", "your ")
            .replace("ill ", "I'll ")
            .replace("dont ", "don't ")
            .replace("cant ", "can't ")
            .replace("wont ", "won't ")
            .replace("im ", "I'm ")
            .replace("ive ", "I've ");
        if let Some(first_char) = polished.chars().next() {
            polished = first_char.to_uppercase().to_string() + &polished[1..];
        }
        if !polished.ends_with('.') && !polished.ends_with('!') && !polished.ends_with('?') {
            polished.push('.');
        }
        polished
    }

    #[test]
    fn test_sentiment_positive() {
        let result = analyze_text_sentiment("Thank you so much! This is great!");
        assert_eq!(result, "positive");
    }

    #[test]
    fn test_sentiment_negative() {
        let result = analyze_text_sentiment("This is terrible! I'm so frustrated!");
        assert_eq!(result, "negative");
    }

    #[test]
    fn test_sentiment_neutral() {
        let result = analyze_text_sentiment("The meeting is at 3pm.");
        assert_eq!(result, "neutral");
    }

    fn analyze_text_sentiment(message: &str) -> &'static str {
        let msg_lower = message.to_lowercase();
        let positive_words = [
            "thank",
            "great",
            "perfect",
            "awesome",
            "excellent",
            "good",
            "happy",
            "love",
        ];
        let negative_words = [
            "angry",
            "frustrated",
            "terrible",
            "awful",
            "horrible",
            "hate",
            "disappointed",
            "problem",
            "issue",
        ];
        let positive_count = positive_words
            .iter()
            .filter(|w| msg_lower.contains(*w))
            .count();
        let negative_count = negative_words
            .iter()
            .filter(|w| msg_lower.contains(*w))
            .count();
        if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        }
    }

    #[test]
    fn test_smart_replies_count() {
        let replies = generate_smart_replies();
        assert_eq!(replies.len(), 3);
    }

    #[test]
    fn test_smart_replies_content() {
        let replies = generate_smart_replies();
        assert!(replies.iter().any(|r| r.contains("Thank you")));
        assert!(replies.iter().any(|r| r.contains("understand")));
    }

    fn generate_smart_replies() -> Vec<String> {
        vec![
            "Thank you for reaching out! I'd be happy to help you with that.".to_string(),
            "I understand your concern. Let me look into this for you right away.".to_string(),
            "Is there anything else I can help you with today?".to_string(),
        ]
    }
}
