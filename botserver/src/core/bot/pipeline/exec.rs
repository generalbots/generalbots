use std::sync::Arc;

use base64::Engine;
use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::PipelineResult;

pub type PipelineFn = std::sync::Arc<
    dyn Fn(
            botlib::models::UserMessage,
            tokio::sync::mpsc::Sender<botlib::models::BotResponse>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = PipelineResult<()>> + Send>>
        + Send
        + Sync,
>;

pub async fn process_message_internal(
    sink: &dyn ChannelSink,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: &str,
    start_bas_ran: &mut bool,
    text: &str,
) -> PipelineResult<()> {
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap_or_default();

    // Agent-mode switcher control frame (issue #1167). Consumed before the
    // regular pipeline so it never reaches the LLM; ignored when the
    // agent-vm feature is disabled at boot.
    #[cfg(feature = "agent-vm")]
    if parsed.get("type").and_then(|v| v.as_str()) == Some("agent_mode") {
        if let Some(status) = crate::core::bot::agent_vm_hook::handle_frame(
            &parsed, session_id, user_id, bot_uuid,
        )
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
    let project_context = parsed
        .get("project_context")
        .cloned()
        .or_else(|| {
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

    if msg_type == 7 || msg_type == 0 {
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

    // Optional chat file attachment (base64) -> stored under inbox/ so the
    // catalog `drive.file` command can organize it into the right folder.
    if let Some(file) = parsed.get("file").and_then(|v| v.as_object()) {
        let fname = file
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let b64 = file
            .get("content_base64")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !fname.is_empty() && !b64.is_empty() {
            if let Some(drive) = state.drive.as_ref() {
                if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    let key = format!("{bot_name}.gbdrive/inbox/{fname}");
                    match drive
                        .put_object(&format!("{bot_name}.gbai"), &key, data, None)
                        .await
                    {
                        Ok(()) => {
                            log::info!("stored chat attachment: {key}");
                            user_text = format!(
                                "{user_text}\n[User attached a file stored at inbox/{fname}; \
                                 use the drive.file command to organize it into its folder]"
                            );
                        }
                        Err(e) => log::error!("chat attachment store failed: {e}"),
                    }
                }
            }
        }
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
            super::tool_exec::run_tool_exec(
                state,
                bot_uuid,
                session_id,
                user_id,
                bot_name,
                &tool_name,
                sink.channel_type(),
            )
            .await;
        }
        return Ok(());
    }

    let runtime: Arc<dyn botbasic_types::BasicRuntime> =
        Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
    let delivered = crate::basic::keywords::hearing::deliver_hear_input(
        &runtime,
        session_id,
        user_text.clone(),
    );
    if delivered {
        return Ok(());
    }

    if !*start_bas_ran {
        let guards = state.start_bas_guards.lock().await;
        if !guards.contains_key(&session_id) {
            drop(guards);
            *start_bas_ran = super::start_bas::run_start_bas(
                sink, state, bot_uuid, session_id, user_id, bot_name, rx,
            )
            .await
            .unwrap_or(false);
        }
    }
    let mut guards = state.start_bas_guards.lock().await;
    guards.entry(session_id).or_insert(true);

    let post_start_suggestions = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::add_suggestion::get_suggestions(
                state.cache.as_ref(),
                &bot_uuid.to_string(),
                &session_id.to_string(),
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    let post_start_switchers = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::switcher::get_switchers(
                state.cache.as_ref(),
                &bot_uuid.to_string(),
                &session_id.to_string(),
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    if !post_start_suggestions.is_empty() || !post_start_switchers.is_empty() {
        let resp = botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: post_start_suggestions
                .into_iter()
                .map(|s| botlib::models::Suggestion::new(s.text))
                .collect(),
            switchers: post_start_switchers,
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        };
        let _ = sink.send_bot_response(&resp).await;
    }

    let _ = sink
        .send_bot_response(&botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: false,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        })
        .await;

    let channel = sink.channel_type();
    let base_system_prompt =
        crate::core::bot::ws::message::load_system_prompt_for_channel(bot_name, channel);
    let system_prompt = if !active_switchers.is_empty() {
        let switcher_prompts = crate::basic::keywords::switcher::resolve_active_switchers(
            state.cache.as_ref(),
            &bot_uuid.to_string(),
            &session_id.to_string(),
            &active_switchers,
        );
        if switcher_prompts.is_empty() {
            base_system_prompt
        } else {
            format!("{base_system_prompt}\n\n{switcher_prompts}")
        }
    } else {
        base_system_prompt
    };
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    let system_prompt = if channel == "web" {
        format!(
            "{system_prompt}\n\n{}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
            crate::main_module::ui_plan::ui_automation_instructions(),
        )
    } else if channel == "whatsapp" {
        format!(
            "{system_prompt}\n\n---\nThis conversation is on WhatsApp.\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    } else {
        format!(
            "{system_prompt}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    };

    let session_context = {
        let sm = state.session_manager.lock().await;
        sm.get_session_context_data(&session_id, &user_id)
            .ok()
            .unwrap_or_default()
    };

    let mut messages =
        vec![serde_json::json!({"role": "system", "content": system_prompt.clone()})];

    if !session_context.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!("Contexto da conversa:\n{session_context}")
        }));
    }
    // #1298 — a chat message that explicitly references a Vibe project via the
    // mention picker (`@project` / the vibe Chat toolbar) is a deterministic
    // vibe agent request, NOT a general chat question. Route it straight to the
    // vibe run pipeline instead of relying on the LLM to emit an `__api_call__`
    // block with `vibe.project.change` (smaller models like gpt-oss skip it and
    // answer with a generic clarification, so "@app paint it red" never reaches
    // the agent). The run executor is the same one `vibe.project.change` uses.
    let picked_project_id = project_context.as_ref().and_then(|ctx| {
        ctx.get("project_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    });
    let (routed_project_id, routed_project_name) = if let Some(pid) = picked_project_id {
        let name = project_context
            .as_ref()
            .and_then(|c| c.get("project_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        (Some(pid), name)
    } else {
        // #mention — a typed "@project-name" in the raw text is deterministic
        // too: resolve it by name instead of trusting the LLM, which may route
        // the message to ui_plan automation (app 'designer') instead of the
        // vibe agent.
        match crate::core::bot::api_catalog::mentioned_project(state, &bot_uuid, &user_text) {
            Ok(Some((pid, name))) => (Some(pid), name),
            _ => (None, String::new()),
        }
    };
    if let Some(pid) = routed_project_id {
        let project_name = routed_project_name;
        let mut run_params = serde_json::Map::new();
        run_params.insert("intent".to_string(), serde_json::Value::String(user_text.clone()));
        run_params.insert("project_id".to_string(), serde_json::Value::String(pid.clone()));
        if !project_name.is_empty() {
            run_params.insert("project_name".to_string(), serde_json::Value::String(project_name.clone()));
        }
        run_params.insert(
            "use_case".to_string(),
            serde_json::Value::String("software_development".to_string()),
        );
        let result = crate::core::bot::api_exec::exec_endpoint(
            state,
            user_id,
            Some(bot_uuid),
            "POST",
            "/api/vibe/run",
            &run_params,
        )
        .await;
        match result {
            Ok(value) => {
                // #1286 — a lock conflict (another session is modifying the
                // same project) must be reported honestly, never as success:
                // the API returns success=false + lock_conflict=true.
                if value.get("lock_conflict").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let detail = value
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("another session is modifying this project");
                    let resp = botlib::models::BotResponse::new(
                        bot_uuid.to_string(),
                        session_id.to_string(),
                        user_id.to_string(),
                        &format!("⏳ {detail} — your change is queued and will run automatically."),
                        "web",
                    );
                    let _ = sink.send_bot_response(&resp).await;
                    return Ok(());
                }
                // Surface a concise confirmation; the run itself streams its
                // progress through the vibe websocket/Run Dock, not the chat.
                let run_id = value.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
                let note = if run_id.is_empty() {
                    "Vibe agent started for project '{project_name}'.".to_string()
                } else {
                    format!("Vibe agent started for project '{project_name}' (run #{run_id}).")
                };
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    &note,
                    "web",
                );
                let _ = sink.send_bot_response(&resp).await;

                // #e2e — report the run's outcome back into this chat. The
                // "agent started" note above is the last thing the user sees
                // otherwise, so they never learn whether the edit applied or
                // the deploy published. A poller watches the run state via
                // the same loopback API and pushes one final BotResponse
                // through the live session channel (response_channels), so
                // the message lands in the open Chat window even after this
                // pipeline call returns.
                if !run_id.is_empty() {
                    let state = state.clone();
                    let run_id = run_id.to_string();
                    let project_name = project_name.clone();
                    let bot_uuid = bot_uuid;
                    let session_id = session_id;
                    let user_id = user_id;
                    tokio::spawn(async move {
                        let final_state = loop {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            let value = match crate::core::bot::api_exec::exec_endpoint(
                                &state,
                                user_id,
                                Some(bot_uuid),
                                "GET",
                                &format!("/api/vibe/run/{run_id}"),
                                &serde_json::Map::new(),
                            )
                            .await
                            {
                                Ok(v) => v,
                                Err(_) => return,
                            };
                            let s = value
                                .get("state")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if matches!(s.as_str(), "completed" | "failed" | "cancelled")
                                || s.is_empty()
                            {
                                break s;
                            }
                        };
                        let text = match final_state.as_str() {
                            "completed" => format!(
                                "✅ Vibe agent finished for '{project_name}' — the changes are ready to go."
                            ),
                            "failed" => format!(
                                "❌ Vibe run for '{project_name}' failed — check the Runner Log for details."
                            ),
                            "cancelled" => {
                                format!("⏹ Vibe run for '{project_name}' was cancelled.")
                            }
                            _ => return,
                        };
                        let resp = botlib::models::BotResponse::new(
                            bot_uuid.to_string(),
                            session_id.to_string(),
                            user_id.to_string(),
                            &text,
                            "web",
                        );
                        let channels = state.response_channels.lock().await;
                        if let Some(tx) = channels.get(&session_id.to_string()) {
                            let _ = tx.try_send(resp);
                        }
                    });
                }
            }
            Err(e) => {
                log::warn!("Vibe chat routing failed for project {pid}: {e}");
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    &format!("Vibe agent could not start for '{project_name}': {e}"),
                    "web",
                );
                let _ = sink.send_bot_response(&resp).await;
            }
        }
        return Ok(());
    }

    if let Some(context) = project_context.as_ref() {
        let project_id = context.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
        let project_name = context.get("project_name").and_then(|v| v.as_str()).unwrap_or("");
        if !project_id.is_empty() || !project_name.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!(
                    "The user referenced Vibe project '{project_name}' (id: {project_id}).\n\
                     For code changes to this project, use the vibe.project.change command with \
                     project_id='{project_id}', project_name='{project_name}', and the user's requested intent."
                ),
            }));
        }
    }

    let history_limit: i64 = {
        use botcore::config::ConfigManager;
        let cfg = ConfigManager::new(state.conn.clone());
        cfg.get_config(&bot_uuid, "history-limit", Some("10"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10)
    };

    {
        let mut sm = state.session_manager.lock().await;
        if let Ok(history) = sm.get_conversation_history(session_id, user_id, Some(history_limit)) {
            for (role, content) in history.iter() {
                let api_role = match role.as_str() {
                    "user" => "user",
                    "assistant" | "bot" => "assistant",
                    _ => "system",
                };
                messages.push(serde_json::json!({
                    "role": api_role,
                    "content": content
                }));
            }
        }
    }

    let _ = sink
        .send_bot_response(&botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: false,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        })
        .await;

    let user_query = user_text.clone();
    let mut messages_val = serde_json::Value::Array(messages.clone());
    if tokio::time::timeout(
        std::time::Duration::from_secs(30),
        super::kb::inject_kb(
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            &user_query,
            &mut messages_val,
        ),
    )
    .await
    .is_err()
    {
        log::warn!("ws_handler: inject_kb_context TIMEOUT after 30s for session {session_id}");
    }

    #[cfg(feature = "memory-os")]
    crate::core::bot::memory_hook::inject_recall(
        state,
        user_id,
        &user_query,
        &mut messages_val,
    );

    // Mention system blocks land right before the user turn so the LLM sees
    // the advertised integration surface adjacent to the request it enables.
    #[cfg(feature = "integrations")]
    if !mentions.is_empty() {
        super::mentions::append_integration_mention_blocks(
            state,
            bot_uuid,
            user_id,
            &mentions,
            &mut messages_val,
        )
        .await;
    }

    if !is_switcher_replay {
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 1, &user_text, 1);
    }

    if let Some(arr) = messages_val.as_array_mut() {
        arr.push(serde_json::json!({
            "role": "user",
            "content": user_text
        }));
    }

    if tokio::time::timeout(
        std::time::Duration::from_secs(300),
        super::llm::stream_llm_response(
            sink,
            rx,
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            &messages_val,
            &user_query,
        ),
    )
    .await
    .is_err()
    {
        log::warn!("stream_llm_response TIMEOUT after 300s for session {session_id}");
    }

    Ok(())
}
