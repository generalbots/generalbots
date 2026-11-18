use uuid::Uuid;

pub use super::types::PipelineFn;
use super::types::{PipelineResult, SessionIdentity};
use log::info;

mod context;
mod finish;

pub async fn process_message_internal(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    start_bas_ran: &mut bool,
    text: &str,
) -> PipelineResult<()> {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap_or_default();

    // #1635 — citations belong to exactly one turn. Drop anything left from a
    // previous turn (or from a stream that was interrupted before its final
    // frame) so a later answer can never inherit stale sources.
    {
        let mut pending = state.pending_citations.lock().await;
        pending.remove(&session_id.to_string());
    }

    // Agent-mode control frame (legacy toggle, #1167). Always-on since #1580
    // but kept for backward compat — the frame is consumed so it never
    // reaches the LLM.
    #[cfg(feature = "agent-vm")]
    if parsed.get("type").and_then(|v| v.as_str()) == Some("agent_mode") {
        if let Some(status) =
            crate::core::bot::agent_vm_hook::handle_frame(&parsed, session_id, user_id, bot_uuid)
                .await
        {
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &status,
                sink.channel_type(),
            );
            let _ = sink.send_bot_response(&resp).await;
            return Ok(());
        }
    }

    // #1644 — the client reports what happened for the action it executed.
    // This is the frame the old build sent with no server handler.
    if matches!(
        parsed.get("type").and_then(|v| v.as_str()),
        Some("browser_result") | Some("terminal_result")
    ) || matches!(
        parsed.get("message_type").and_then(|v| v.as_str()),
        Some("browser_result") | Some("terminal_result")
    ) {
        let action_id = parsed
            .get("action_id")
            .and_then(|v| v.as_str())
            .and_then(|value| Uuid::parse_str(value).ok());
        if let Some(id) = action_id {
            let result = parsed.get("result").cloned().unwrap_or_default();
            let _ = crate::core::computer_use::complete(id, user_id, result).await;
        }
        return Ok(());
    }

    let mut user_text = parsed
        .get("text")
        .and_then(|v| v.as_str())
        .or_else(|| parsed.get("content").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let mut msg_type = parsed
        .get("message_type")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    // #1549: the composer's model tier is a behavioural preference. The
    // provider is built at boot, so the tier is applied as a response
    // directive instead of swapping the provider per message.
    let model_preference = parsed
        .get("model_preference")
        .and_then(|v| v.as_str())
        .unwrap_or("auto")
        .to_string();
    let active_switchers: Vec<String> = parsed
        .get("active_switchers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    // Project context is supplied by the chat mention picker as
    // {project_id, project_name}. It is kept separate from entity mentions so
    // an ordinary `@calculator` reference can target Vibe without changing
    // the existing CRM/integration mention contract.
    let project_context = parsed.get("project_context").cloned().or_else(|| {
        parsed.get("mentions").and_then(|value| {
            value.as_array()?.iter().find_map(|mention| {
                if mention.get("kind").and_then(|v| v.as_str()) != Some("project") {
                    return None;
                }
                Some(serde_json::json!({
                    "project_id": mention.get("project_id").or_else(|| mention.get("id")),
                    "project_name": mention.get("label").or_else(|| mention.get("name")),
                }))
            })
        })
    });
    // @-mentions selected in the composer (#939 phase D); resolved against
    // the connection control plane only when the integrations feature is
    // compiled in, exactly like the `integrations.invoke` command arm.
    #[cfg(feature = "integrations")]
    let mentions = super::mentions::parse_lenient_mentions(&parsed);

    {
        let mut sm = state.session_manager.lock().await;
        let _ = sm.get_or_create_session_by_id(session_id, user_id, bot_uuid, "");
    }

    info!(
        "WS message: type={msg_type} content_chars={} session={session_id} bot={bot_name}",
        user_text.chars().count()
    );
    if msg_type == 7 || msg_type == 0 {
        info!("WS message type {msg_type} is consumed without a reply");
        return Ok(());
    }

    let mut is_switcher_replay = false;
    if msg_type == 8 {
        let last_user_msg = {
            let mut sm = state.session_manager.lock().await;
            let history = sm
                .get_conversation_history(session_id, user_id, Some(1))
                .ok();
            history.and_then(|h| {
                h.into_iter()
                    .find(|(role, _)| role == "user")
                    .map(|(_, c)| c)
            })
        };
        if let Some(last_content) = last_user_msg {
            user_text = last_content;
            is_switcher_replay = true;
            msg_type = 1;
        } else {
            return Ok(());
        }
    }

    // #2238/#2239 — unified chat controls. agent_enabled is session-scoped
    // state: when the payload carries it we persist it, otherwise we read the
    // stored session value so the toggle is the single source of truth.
    // Resolved BEFORE `start.bas` and TOOL_EXEC so no path (script, tool chip,
    // workspace route) can reach the VM while the switch is off.
    let agent_enabled = super::chat_controls::resolve_agent_enabled(
        state,
        session_id,
        user_id,
        parsed.get("agent_enabled").and_then(|v| v.as_bool()),
    )
    .await;
    // #2239 — make the resolved agent/VM switch visible to every pipeline/tool
    // path in this turn (native tools, provider resolution, workspace bridges).
    let _agent_turn_guard = super::agent_turn::TurnGuard::new(session_id, agent_enabled);
    let ui_automation_enabled = parsed
        .get("ui_automation")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let media_type = parsed
        .get("media_type")
        .and_then(|v| v.as_str())
        .unwrap_or("auto")
        .to_string();
    // Turn-scoped identity: downstream tool dispatch and provisioning must see
    // the resolved per-turn Agent/VM toggle.
    let identity = &SessionIdentity {
        agent_vm_enabled: agent_enabled,
        ..*identity
    };

    // Early run context for activity events that fire before the LLM call.
    let early_run = super::event_sink::RunContext::new(&session_id.to_string(), "chat")
        .with_scope(bot_uuid, user_id, None)
        .with_state(state.clone());

    // #1641 — store the attachment and turn it into prompt content.
    // #2562 — emit attachment_read activities around the ingestion.
    let has_file = parsed.get("file").is_some()
        || parsed
            .get("files")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
    let file_name = parsed
        .get("file")
        .and_then(|f| f.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let file_size = parsed
        .get("file")
        .and_then(|f| f.get("content_base64"))
        .and_then(|b| b.as_str())
        .map(|b| b.len() * 3 / 4)
        .unwrap_or(0);
    let attach_activity = if has_file {
        let summary = super::activity::attachment_summary(&file_name, "", file_size, 0, false, "");
        let act = super::activity::Activity::started(
            super::activity::ActivityKind::AttachmentRead,
            summary,
        )
        .with_run_id(&early_run.run_id);
        super::activity::emit_activity(sink, &early_run, &act).await;
        Some(act)
    } else {
        None
    };

    let attachment =
        super::turn_hooks::ingest_attachment(state, bot_uuid, bot_name, &parsed, user_text).await;
    let user_text = attachment.user_text;
    let mut attachment_note = attachment.note;
    let media_fail_reason = attachment.media_fail_reason;

    // Emit the completed/failed attachment activity.
    if let Some(act) = attach_activity {
        let drive_path = format!("inbox/{file_name}");
        // Count only the actual extracted text (delimited by the BEGIN/END
        // ATTACHMENT markers), not the wrapper instructions sent to the model.
        // Without this, unsupported formats report chars_read = wrapper length
        // and the step card does not show the "format not readable" warning.
        let excerpt_chars = attachment_note
            .as_ref()
            .and_then(|n| {
                let marker = format!("--- BEGIN ATTACHMENT {file_name} ---\n");
                n.find(&marker).map(|start| {
                    let content_start = start + marker.len();
                    let end_marker = format!("\n--- END ATTACHMENT {file_name} ---");
                    let end = n[content_start..]
                        .find(&end_marker)
                        .map(|e| content_start + e)
                        .unwrap_or(n.len());
                    n[content_start..end].chars().count().min(12000)
                })
            })
            .unwrap_or(0);
        let truncated = attachment_note
            .as_ref()
            .is_some_and(|n| n.contains("excerpt truncated"));
        let mime = super::activity::mime_from_name(&file_name);
        if let Some(reason) = media_fail_reason {
            // #2561 — the file was stored but its media content could not be
            // read (e.g. no image reader installed). Emit as failed with the
            // reason code so the interface can show it.
            let fail = act.failed(reason);
            super::activity::emit_activity(sink, &early_run, &fail).await;
        } else if attachment_note.is_some() || user_text.contains(&format!("inbox/{file_name}")) {
            let summary = super::activity::attachment_summary(
                &file_name,
                &mime,
                file_size,
                excerpt_chars,
                truncated,
                &drive_path,
            );
            let done = act.completed(summary);
            super::activity::emit_activity(sink, &early_run, &done).await;
        } else {
            let reason = if file_size > 12 * 1024 * 1024 {
                "File exceeds the 8 MB limit"
            } else {
                "Could not store or read the file"
            };
            let fail = act.failed(reason);
            super::activity::emit_activity(sink, &early_run, &fail).await;
        }
    }

    // #1642 — the assistant finally has a writer. An explicit "remember …"
    // turn is persisted to chat_preferences; the prompt already reads that
    // table, so the next session starts knowing the preference.
    if msg_type == 1 {
        let memory_reply = crate::core::user_memory::capture_from_message(
            &state.conn,
            user_id,
            Some(bot_uuid),
            &user_text,
        );
        // #1790 — a forget turn must answer with the real outcome (including
        // "no stored memory matched") instead of letting the LLM claim it
        // removed something. Remember turns keep the normal LLM path.
        if crate::core::user_memory::extract_forget_directive(&user_text).is_some() {
            let reply = memory_reply.unwrap_or_else(|| {
                "I could not update your stored memory. Please try again.".to_string()
            });
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                format!("<p>{reply}</p>"),
                "web",
            );
            let _ = sink.send_bot_response(&resp).await;
            return Ok(());
        }
    }

    // #1644/#1645 — an explicit URL or command request becomes a consent card.
    if super::turn_hooks::propose_computer_action(identity, msg_type, &user_text).await {
        return Ok(());
    }

    if user_text.starts_with("__TOOL__:") {
        let tool_name = user_text.trim_start_matches("__TOOL__:").trim().to_string();
        if !tool_name.is_empty() {
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                format!("Tool '{tool_name}' not implemented via legacy path"),
                "web",
            );
            let _ = sink.send_bot_response(&resp).await;
        }
        return Ok(());
    }

    if msg_type == 6 {
        let raw_tool_name = user_text.trim().to_string();
        let tool_name = match crate::core::bot::ws::handler::validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("TOOL_EXEC: invalid tool name '{}': {}", raw_tool_name, e);
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    format!("<p>Invalid tool name: {raw_tool_name}</p>"),
                    "web",
                );
                let _ = sink.send_bot_response(&resp).await;
                return Ok(());
            }
        };

        if !tool_name.is_empty() {
            super::tool_exec::run_tool_exec(identity, &tool_name).await;
        }
        return Ok(());
    }

    // #1580 — hearing, start.bas and the opening frames.
    if super::start_gate::gate_turn(identity, rx, start_bas_ran, &user_text).await? {
        return Ok(());
    }

    // #1946 — decide this request's execution mode and tell the model which one
    // applies. The decision is per turn: a question inside a long agentic
    // session must not inherit the previous turn's mode.
    let route = crate::core::bot::ws::stream::routing::decide(
        &user_text,
        parsed
            .get("files")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len),
        parsed.get("client_capabilities"),
    );
    // #3011 — CodeQL rust/cleartext-logging: identifiers go through the botlog
    // redacting facade, never the raw log macros.
    botlog::safe_info!(
        "agentic routing: session={session_id} mode={} reason={}",
        route.route.as_str(),
        route.reason
    );

    let mut messages = Vec::new();
    if context::prepare_messages_and_maybe_finish(
        identity,
        &parsed,
        &user_text,
        agent_enabled,
        &route,
        &active_switchers,
        ui_automation_enabled,
        &media_type,
        &model_preference,
        &session_context,
        project_context.as_ref(),
        &mut attachment_note,
        &mut messages,
    )
    .await?
    {
        return Ok(());
    }

    finish::stream_and_finalize(
        identity,
        rx,
        &messages,
        &user_text,
        #[cfg(feature = "integrations")]
        &mentions,
        is_switcher_replay,
        &model_preference,
        agent_enabled,
    )
    .await
}
