use uuid::Uuid;

use log::info;
use super::types::{PipelineResult, SessionIdentity};

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

    // #1641 — store the attachment and turn it into prompt content.
    let attachment =
        super::turn_hooks::ingest_attachment(state, bot_uuid, bot_name, &parsed, user_text).await;
    let user_text = attachment.user_text;
    let mut attachment_note = attachment.note;

    // #1642 — the assistant finally has a writer. An explicit "remember …"
    // turn is persisted to chat_preferences; the prompt already reads that
    // table, so the next session starts knowing the preference.
    if msg_type == 1 {
        crate::core::user_memory::capture_from_message(
            &state.conn,
            user_id,
            Some(bot_uuid),
            &user_text,
        );
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

    let channel = sink.channel_type();
    let system_prompt = super::system_prompt::with_structured_tools(
        super::system_prompt::build(
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            channel,
            &active_switchers,
            parsed.get("client_capabilities"),
        ),
        channel,
    );

    let session_context = {
        let sm = state.session_manager.lock().await;
        sm.get_session_context_data(&session_id, &user_id)
            .ok()
            .unwrap_or_default()
    };

    let mut messages =
        vec![serde_json::json!({"role": "system", "content": system_prompt.clone()})];

    // #1670 — structured answers travel with the user turn. Record them so a
    // field is never asked twice, and make the model treat them as the
    // authoritative value for this conversation.
    if let Some(values) = parsed.get("question_answers").and_then(|value| value.as_object()) {
        super::surface_tools::record_answers(&session_id.to_string(), values);
        let rendered = serde_json::to_string(values).unwrap_or_default();
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!(
                "The user answered the structured question with {rendered}. These values are authoritative; do not ask for the same fields again."
            )
        }));
    }
    if let Some(plan) = parsed.get("plan_approval") {
        let rendered = serde_json::to_string(plan).unwrap_or_default();
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!(
                "The user approved the plan with {rendered}. Execute it; do not propose it again."
            )
        }));
    }


    // #1549/#1637: the tier now selects a real model in
    // `resolve_bot_llm_provider` (`llm-model-<tier>`); this sentence is only a
    // secondary shaping hint, not the routing mechanism.
    super::turn_context::apply_model_preference(&mut messages, &model_preference);
    super::turn_context::apply_session_context(&mut messages, &session_context);

    let routing = super::turn_context::TurnRouting {
        state,
        sink,
        bot_uuid,
        session_id,
        user_id,
    };
    if super::turn_context::route_project(&routing, project_context.as_ref(), &user_text, &mut messages)
        .await?
    {
        return Ok(());
    }
    super::turn_context::apply_drive_listing(&mut messages, &parsed);

    // #1580: Always-on agent VM — auto-provision and inject execution context.
    super::agent_vm_context::inject(session_id, user_id, bot_uuid, &mut messages).await;

    // #1584: files named in the message are copied from Drive into data/
    // before the model writes the program that reads them.
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    super::native_drive::fetch_referenced(state, bot_uuid, user_id, session_id, &user_text, &mut messages).await;

    super::history::append_conversation_history(state, bot_uuid, session_id, user_id, &mut messages).await;

    // #1641 — attachment content is a separate system block so it never gets
    // persisted as if the user had typed the whole file.
    if let Some(note) = attachment_note.take() {
        messages.push(serde_json::json!({"role": "system", "content": note}));
    }

    // #1644 — results of approved browser/terminal actions are fed back into
    // the conversation before the model answers, so the next reply can cite
    // the page title (or say the action was denied/failed).
    let completed =
        crate::core::computer_use::take_completed_for_prompt(&session_id.to_string(), user_id).await;
    if !completed.is_empty() {
        let mut block = String::from(
            "Results of computer actions the user approved in this conversation:\n",
        );
        for action in &completed {
            let name = action
                .action
                .get("action")
                .and_then(|value| value.as_str())
                .unwrap_or(&action.kind);
            block.push_str(&format!(
                "- {name}: {}\n",
                serde_json::to_string(&action.result).unwrap_or_default()
            ));
        }
        block.push_str(
            "Answer from these results; quote the page title/URL when the user asked about a page. If a result says denied or error, say so plainly.",
        );
        messages.push(serde_json::json!({"role": "system", "content": block}));
    }

    // #1545/#1547: grounding context goes LAST, immediately before the model
    // call, so it is the freshest instruction in the prompt.
    #[cfg(feature = "drive")]
    super::drive_context::inject(state, &bot_uuid, user_id, &user_text, &mut messages).await;

    // #1632 item 3 — the composer's Web switch is the decision. When it is on
    // the turn is searched as written; when it is off nothing is searched and
    // the model can still call the `web.search` command itself. No routing
    // completion asks whether the turn "needs" the web.
    #[cfg(any(feature = "research", feature = "web-search"))]
    {
        let web_search_enabled = parsed
            .get("web_search")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if web_search_enabled {
            let citations =
                super::web_context::search_and_inject(identity, &user_text, &mut messages).await;
            if !citations.is_empty() {
                state
                    .pending_citations
                    .lock()
                    .await
                    .insert(session_id.to_string(), citations);
            }
        }
    }

    super::history::send_stream_start_marker(sink, bot_uuid, session_id, user_id).await;

    if tokio::time::timeout(
        std::time::Duration::from_secs(300),
        super::enriched_llm::build_and_stream(
            identity,
            rx,
            &messages,
            &user_text,
            #[cfg(feature = "integrations")]
            &mentions,
            is_switcher_replay,
            &model_preference,
        ),
    )
    .await
    .is_err()
    {
        log::warn!("stream_llm_response TIMEOUT after 300s for session {session_id}");
    }

    // #1635 — the pipeline streams typed BotResponse frames, which have no
    // citations field. Send the sources as one explicit frame after the answer
    // so the renderer can attach the chips to the message that just completed.
    let citations = {
        let mut pending = state.pending_citations.lock().await;
        pending.remove(&session_id.to_string()).unwrap_or_default()
    };
    if !citations.is_empty() {
        let frame = serde_json::json!({
            "type": "citations",
            "message_type": "citations",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "citations": citations,
        });
        let _ = sink.send_raw_json(&frame).await;
    }

    Ok(())
}
