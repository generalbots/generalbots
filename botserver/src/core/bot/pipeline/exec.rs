//! One chat turn, from the incoming frame to the published artifacts. The
//! attachment, the prompt, the turn's context and the control frames live in
//! the `exec/` modules.

use super::types::{PipelineResult, SessionIdentity};
use log::info;
pub use super::types::PipelineFn;

mod attachment;
mod context;
mod frames;
mod prompt;

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

    if frames::consume_control_frame(identity, &parsed).await {
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

    super::turn_watch::bounded("session lookup", super::turn_watch::STEP_LIMIT, async {
        let mut sm = state.session_manager.lock().await;
        let _ = sm.get_or_create_session_by_id(session_id, user_id, bot_uuid, "");
    })
    .await;

    info!(
        "WS message: type={msg_type} content_chars={} session={session_id} bot={bot_name}",
        user_text.chars().count()
    );
    if msg_type == 7 || msg_type == 0 {
        info!("WS message type {msg_type} is consumed without a reply");
        return Ok(());
    }

    // What this turn reads or runs is recorded with its answer (`turn_record`).
    super::turn_record::begin(session_id);

    let mut is_switcher_replay = false;
    if msg_type == 8 {
        let last_user_msg = super::turn_watch::bounded("session history", super::turn_watch::STEP_LIMIT, async {
            let mut sm = state.session_manager.lock().await;
            let history = sm
                .get_conversation_history(session_id, user_id, Some(1))
                .ok();
            history.and_then(|h| {
                h.into_iter()
                    .find(|(role, _)| role == "user")
                    .map(|(_, c)| c)
            })
        })
        .await
        .flatten();
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
    let mut agent_enabled = super::chat_controls::resolve_agent_enabled(
        state,
        session_id,
        user_id,
        parsed.get("agent_enabled").and_then(|v| v.as_bool()),
    )
    .await;
    // An explicit subscription choice (`sub:claude-code`, …) in the model
    // menu means the agent is ON for this turn regardless of the per-
    // conversation switch — do not persist it; just override the resolved
    // value so the workspace starts.
    #[cfg(feature = "agent-vm")]
    if !agent_enabled {
        agent_enabled = super::chat_controls::subscription_implies_agent(
            &model_preference,
        );
    }
    // #2239 — make the resolved agent/VM switch visible to every pipeline/tool
    // path in this turn (native tools, provider resolution, workspace bridges).
    let _agent_turn_guard = super::agent_turn::TurnGuard::new(session_id, agent_enabled);
    super::turn_watch::stage("agent switch resolved");
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

    let attachment = attachment::read(identity, &parsed, user_text, &model_preference).await;
    let user_text = attachment.user_text;
    let mut attachment_note = attachment.note;
    let attachment_media = attachment.media;

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
                let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
                botcore::i18n::t(&locale, "error-memory-not-updated")
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
    super::turn_watch::stage("consent check");
    if super::turn_hooks::propose_computer_action(identity, msg_type, &user_text).await {
        return Ok(());
    }
    if frames::consume_tool_frame(identity, msg_type, &user_text).await {
        return Ok(());
    }

    // #1580 — hearing, start.bas and the opening frames.
    super::turn_watch::stage("start gate");
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
    super::turn_watch::stage("prompt build");
    let system_prompt = prompt::system_prompt(
        identity,
        &parsed,
        &active_switchers,
        ui_automation_enabled,
        &media_type,
        &user_text,
    );

    let session_context = super::turn_watch::bounded("session context", super::turn_watch::STEP_LIMIT, async {
        let sm = state.session_manager.lock().await;
        sm.get_session_context_data(&session_id, &user_id)
            .ok()
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut messages =
        vec![serde_json::json!({"role": "system", "content": system_prompt.clone()})];
    let turn_instruction = prompt::turn_instruction(agent_enabled, route.route);
    messages.push(serde_json::json!({
        "role": "system",
        "content": format!("{turn_instruction} {}", super::turn_record::RECORDS_ARE_FACTS)
    }));

    // #1670 — structured answers and approved plans shape this turn's prompt.
    super::turn_context::apply_structured_answers(&parsed, session_id, &mut messages);

    // #1549/#1637: the tier now selects a real model in
    // `resolve_bot_llm_provider` (`llm-model-<tier>`); this sentence is only a
    // secondary shaping hint, not the routing mechanism.
    let response_style = if parsed.get("fast").and_then(serde_json::Value::as_bool) == Some(true) {
        "fast"
    } else {
        &model_preference
    };
    super::turn_context::apply_model_preference(&mut messages, response_style);
    super::turn_context::apply_session_context(&mut messages, &session_context);

    let routing = super::turn_context::TurnRouting {
        state,
        sink,
        bot_uuid,
        session_id,
        user_id,
    };
    super::turn_watch::long_stage("project route");
    if super::turn_context::route_project(
        &routing,
        project_context.as_ref(),
        &user_text,
        &mut messages,
        agent_enabled,
    )
    .await?
    {
        return Ok(());
    }
    context::workspace_and_history(identity, &parsed, &user_text, &mut messages).await;

    // #1641 — attachment content is a separate system block so it never gets
    // persisted as if the user had typed the whole file.
    if let Some(note) = attachment_note.take() {
        let mut block = serde_json::json!({"role": "system", "content": note});
        // The files themselves, for a model that can see them; each client
        // turns them into its provider's content parts (botllm::media_parts).
        if !attachment_media.is_empty() {
            block["media"] = serde_json::Value::Array(attachment_media);
        }
        messages.push(block);
    }
    context::grounding(identity, &parsed, &user_text, &model_preference, &mut messages).await;
    super::turn_watch::reached_model();
    super::history::send_stream_start_marker(sink, bot_uuid, session_id, user_id).await;

    let turn_timeout_secs = super::enriched_llm::turn_timeout_secs();
    if super::turn_progress::until_quiet(
        &session_id.to_string(),
        std::time::Duration::from_secs(turn_timeout_secs),
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
    .is_none()
    {
        botlog::safe_warn!("turn made no progress for {turn_timeout_secs}s for session {session_id}");
        return Err(super::types::PipelineError::LLM(format!("Turn made no progress for {turn_timeout_secs}s")));
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

    // #2143 gap D — publish this turn's artifacts; a Drive outage never fails it.
    // #2239 — skip workspace artifact publish when Agent/VM is off for this turn.
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if agent_enabled {
        super::output_publish::publish_and_attach(identity).await;
    }

    Ok(())
}
