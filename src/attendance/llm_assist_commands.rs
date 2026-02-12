use super::llm_assist_types::*;
use super::llm_assist_helpers::*;
use super::llm_assist_handlers::*;
use crate::core::shared::state::AppState;
use log::info;
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

pub async fn process_attendant_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
    command: &str,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let cmd = parts[0].to_lowercase();
    let args: Vec<&str> = parts[1..].to_vec();

    match cmd.as_str() {
        "/queue" | "/fila" => handle_queue_command(state).await,
        "/take" | "/pegar" => handle_take_command(state, attendant_phone).await,
        "/status" => handle_status_command(state, attendant_phone, args).await,
        "/transfer" | "/transferir" => handle_transfer_command(state, current_session, args).await,
        "/resolve" | "/resolver" => handle_resolve_command(state, current_session).await,
        "/tips" | "/dicas" => handle_tips_command(state, current_session).await,
        "/polish" | "/polir" => {
            let message = args.join(" ");
            handle_polish_command(state, current_session, &message).await
        }
        "/replies" | "/respostas" => handle_replies_command(state, current_session).await,
        "/summary" | "/resumo" => handle_summary_command(state, current_session).await,
        "/help" | "/ajuda" => Ok(get_help_text()),
        _ => Err(format!(
            "Unknown command: {}. Type /help for available commands.",
            cmd
        )),
    }
}

async fn handle_queue_command(state: &Arc<AppState>) -> Result<String, String> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::core::shared::models::schema::user_sessions;

        let sessions: Vec<crate::core::shared::models::UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("needs_human")
                    .eq("true"),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .ne("resolved"),
            )
            .order(user_sessions::updated_at.desc())
            .limit(10)
            .load(&mut db_conn)
            .map_err(|e| e.to_string())?;

        Ok::<Vec<crate::core::shared::models::UserSession>, String>(sessions)
    })
    .await
    .map_err(|e| e.to_string())??;

    if result.is_empty() {
        return Ok(" *Queue is empty*\nNo conversations waiting for attention.".to_string());
    }

    let mut response = format!(" *Queue* ({} waiting)\n\n", result.len());

    for (i, session) in result.iter().enumerate() {
        let name = session
            .context_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let channel = session
            .context_data
            .get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("web");
        let status = session
            .context_data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("waiting");

        let _ = write!(
            response,
            "{}. *{}* ({})\n   Status: {} | ID: {}\n\n",
            i + 1,
            name,
            channel,
            status,
            &session.id.to_string()[..8]
        );
    }

    response.push_str("Type `/take` to take the next conversation.");

    Ok(response)
}

async fn handle_take_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
) -> Result<String, String> {
    let conn = state.conn.clone();
    let phone = attendant_phone.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::core::shared::models::schema::user_sessions;

        let session: Option<crate::core::shared::models::UserSession> = user_sessions::table
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
            .order(user_sessions::updated_at.asc())
            .first(&mut db_conn)
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(session) = session {
            let mut ctx = session.context_data.clone();
            ctx["assigned_to_phone"] = serde_json::json!(phone);
            ctx["status"] = serde_json::json!("assigned");
            ctx["assigned_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session.id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| e.to_string())?;

            let name = session
                .context_data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            Ok::<String, String>(format!(
                " *Conversation assigned*\n\nCustomer: *{}*\nSession: {}\n\nYou can now respond to this customer. Their messages will be forwarded to you.",
                name,
                &session.id.to_string()[..8]
            ))
        } else {
            Ok::<String, String>(" No conversations waiting in queue.".to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(result)
}

async fn handle_status_command(
    state: &Arc<AppState>,
    attendant_phone: &str,
    args: Vec<&str>,
) -> Result<String, String> {
    if args.is_empty() {
        return Ok(
            " *Status Options*\n\n`/status online` - Available\n`/status busy` - In conversation\n`/status away` - Temporarily away\n`/status offline` - Not available"
                .to_string(),
        );
    }

    let status = args[0].to_lowercase();
    let (emoji, text, status_value) = match status.as_str() {
        "online" => ("✅", "Online - Available for conversations", "online"),
        "busy" => ("🔵", "Busy - Handling conversations", "busy"),
        "away" => ("🟡", "Away - Temporarily unavailable", "away"),
        "offline" => ("⚫", "Offline - Not available", "offline"),
        _ => {
            return Err(format!(
                "Invalid status: {}. Use online, busy, away, or offline.",
                status
            ))
        }
    };

    let conn = state.conn.clone();
    let phone = attendant_phone.to_string();
    let status_val = status_value.to_string();

    let update_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::core::shared::models::schema::user_sessions;

        let sessions: Vec<crate::core::shared::models::UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("assigned_to_phone")
                    .eq(&phone),
            )
            .load(&mut db_conn)
            .map_err(|e| e.to_string())?;

        let session_count = sessions.len();
        for session in sessions {
            let mut ctx = session.context_data.clone();
            ctx["attendant_status"] = serde_json::json!(status_val);
            ctx["attendant_status_updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());

            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session.id)))
                .set(user_sessions::context_data.eq(&ctx))
                .execute(&mut db_conn)
                .map_err(|e| e.to_string())?;
        }

        Ok::<usize, String>(session_count)
    })
    .await
    .map_err(|e| e.to_string())?;

    match update_result {
        Ok(count) => {
            info!(
                "Attendant {} set status to {} ({} sessions updated)",
                attendant_phone, status_value, count
            );
            Ok(format!("{} Status set to *{}*", emoji, text))
        }
        Err(e) => {
            log::warn!("Failed to persist status for {}: {}", attendant_phone, e);

            Ok(format!("{} Status set to *{}*", emoji, text))
        }
    }
}

async fn handle_transfer_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
    args: Vec<&str>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation to transfer")?;

    if args.is_empty() {
        return Err("Usage: `/transfer @attendant_name` or `/transfer department`".to_string());
    }

    let target = args.join(" ");
    let target_clean = target.trim_start_matches('@').to_string();

    let conn = state.conn.clone();
    let target_attendant = target_clean.clone();

    let transfer_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::core::shared::models::schema::user_sessions;

        let session: crate::core::shared::models::UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| format!("Session not found: {}", e))?;

        let mut ctx = session.context_data;
        let previous_attendant = ctx
            .get("assigned_to_phone")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        ctx["transferred_from"] = serde_json::json!(previous_attendant);
        ctx["transfer_target"] = serde_json::json!(target_attendant);
        ctx["transferred_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
        ctx["status"] = serde_json::json!("pending_transfer");

        ctx["assigned_to_phone"] = serde_json::Value::Null;
        ctx["assigned_to"] = serde_json::Value::Null;

        ctx["needs_human"] = serde_json::json!(true);

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set((
                user_sessions::context_data.eq(&ctx),
                user_sessions::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut db_conn)
            .map_err(|e| format!("Failed to update session: {}", e))?;

        Ok::<String, String>(previous_attendant)
    })
    .await
    .map_err(|e| e.to_string())??;

    info!(
        "Session {} transferred from {} to {}",
        session_id, transfer_result, target_clean
    );

    Ok(format!(
        " *Transfer initiated*\n\nSession {} is being transferred to *{}*.\n\nThe conversation is now in the queue for the target attendant. They will be notified when they check their queue.",
        &session_id.to_string()[..8],
        target_clean
    ))
}

async fn handle_resolve_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation to resolve")?;

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| e.to_string())?;

        use crate::core::shared::models::schema::user_sessions;

        let session: crate::core::shared::models::UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| e.to_string())?;

        let mut ctx = session.context_data;
        ctx["status"] = serde_json::json!("resolved");
        ctx["needs_human"] = serde_json::json!(false);
        ctx["resolved_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set(user_sessions::context_data.eq(&ctx))
            .execute(&mut db_conn)
            .map_err(|e| e.to_string())?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(format!(
        " *Conversation resolved*\n\nSession {} has been marked as resolved. The customer will be returned to bot mode.",
        &session_id.to_string()[..8]
    ))
}

async fn handle_tips_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation. Use /take first.")?;

    let history = load_conversation_history(state, session_id).await;

    if history.is_empty() {
        return Ok(
            " No messages yet. Tips will appear when customer sends a message.".to_string(),
        );
    }

    let last_customer_msg = history
        .iter()
        .rev()
        .find(|m| m.role == "customer")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let request = TipRequest {
        session_id,
        customer_message: last_customer_msg,
        history,
    };

    let (_, Json(tip_response)) = generate_tips(State(state.clone()), Json(request)).await;

    if tip_response.tips.is_empty() {
        return Ok(" No specific tips for this conversation yet.".to_string());
    }

    let mut result = " *Tips for this conversation*\n\n".to_string();

    for tip in tip_response.tips {
        let emoji = match tip.tip_type {
            TipType::Intent
            | TipType::Action
            | TipType::Warning
            | TipType::Knowledge
            | TipType::History
            | TipType::General => "💡",
        };
        let _ = write!(result, "{} {}\n\n", emoji, tip.content);
    }

    Ok(result)
}

async fn handle_polish_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
    message: &str,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    if message.is_empty() {
        return Err("Usage: `/polish Your message here`".to_string());
    }

    let request = PolishRequest {
        session_id,
        message: message.to_string(),
        tone: "professional".to_string(),
    };

    let (_, Json(polish_response)) = polish_message(State(state.clone()), Json(request)).await;

    if !polish_response.success {
        return Err(polish_response
            .error
            .unwrap_or_else(|| "Failed to polish message".to_string()));
    }

    let mut result = " *Polished message*\n\n".to_string();
    {
        let _ = write!(result, "_{}_\n\n", polish_response.polished);
    }

    if !polish_response.changes.is_empty() {
        result.push_str("Changes:\n");
        for change in polish_response.changes {
            let _ = writeln!(result, "• {}", change);
        }
    }

    result.push_str("\n_Copy and send, or edit as needed._");

    Ok(result)
}

async fn handle_replies_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    let history = load_conversation_history(state, session_id).await;

    let request = SmartRepliesRequest {
        session_id,
        history,
    };

    let (_, Json(replies_response)) =
        generate_smart_replies(State(state.clone()), Json(request)).await;

    if replies_response.replies.is_empty() {
        return Ok(" No reply suggestions available.".to_string());
    }

    let mut result = " *Suggested replies*\n\n".to_string();

    for (i, reply) in replies_response.replies.iter().enumerate() {
        let _ = write!(
            result,
            "*{}. {}*\n_{}_\n\n",
            i + 1,
            reply.tone.to_uppercase(),
            reply.text
        );
    }

    result.push_str("_Copy any reply or use as inspiration._");

    Ok(result)
}

async fn handle_summary_command(
    state: &Arc<AppState>,
    current_session: Option<Uuid>,
) -> Result<String, String> {
    let session_id = current_session.ok_or("No active conversation")?;

    let (_, Json(summary_response)) =
        generate_summary(State(state.clone()), Path(session_id)).await;

    if !summary_response.success {
        return Err(summary_response
            .error
            .unwrap_or_else(|| "Failed to generate summary".to_string()));
    }

    let summary = summary_response.summary;

    let mut result = " *Conversation Summary*\n\n".to_string();
    {
        let _ = write!(result, "{}\n\n", summary.brief);
    }

    if !summary.key_points.is_empty() {
        result.push_str("*Key Points:*\n");
        for point in &summary.key_points {
            let _ = writeln!(result, "• {}", point);
        }
        result.push('\n');
    }

    if !summary.customer_needs.is_empty() {
        result.push_str("*Customer Needs:*\n");
        for need in &summary.customer_needs {
            let _ = writeln!(result, "• {}", need);
        }
        result.push('\n');
    }

    if !summary.unresolved_issues.is_empty() {
        result.push_str("*Unresolved:*\n");
        for issue in &summary.unresolved_issues {
            let _ = writeln!(result, "• {}", issue);
        }
        result.push('\n');
    }

    {
        let _ = write!(
            result,
            " {} messages | {} minutes | Sentiment: {}",
            summary.message_count, summary.duration_minutes, summary.sentiment_trend
        );

        if !summary.recommended_action.is_empty() {
            let _ = write!(result, "\n\n *Recommended:* {}", summary.recommended_action);
        }
    }

    Ok(result)
}

pub fn get_help_text() -> String {
    r"*Attendant Commands*

*Queue Management:*
`/queue` - View waiting conversations
`/take` - Take next conversation
`/transfer @name` - Transfer conversation
`/resolve` - Mark as resolved
`/status [online|busy|away|offline]`

*AI Assistance:*
`/tips` - Get tips for current conversation
`/polish <message>` - Improve your message
`/replies` - Get smart reply suggestions
`/summary` - Get conversation summary

*Other:*
`/help` - Show this help

_Portuguese: /fila, /pegar, /transferir, /resolver, /dicas, /polir, /respostas, /resumo, /ajuda_"
        .to_string()
}
