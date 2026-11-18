use super::types::{PipelineResult, SessionIdentity};

#[allow(clippy::too_many_arguments)]
pub(super) async fn prepare_messages_and_maybe_finish(
    identity: &SessionIdentity<'_>,
    parsed: &serde_json::Value,
    user_text: &str,
    agent_enabled: bool,
    route: &crate::core::bot::ws::stream::routing::RoutingDecision,
    active_switchers: &[String],
    ui_automation_enabled: bool,
    media_type: &str,
    model_preference: &Option<String>,
    session_context: &std::collections::HashMap<String, String>,
    project_context: Option<&serde_json::Value>,
    attachment_note: &mut Option<String>,
    messages: &mut Vec<serde_json::Value>,
) -> PipelineResult<bool> {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;

    // The composer's app-automation/media switches ride with the client
    // capabilities so the prompt only advertises what this turn may use.
    let client_caps_value = {
        let mut caps = parsed
            .get("client_capabilities")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = caps.as_object_mut() {
            obj.insert(
                "ui_automation".to_string(),
                serde_json::json!(ui_automation_enabled),
            );
            if media_type != "auto" {
                obj.insert(
                    "media_type".to_string(),
                    serde_json::json!(media_type.clone()),
                );
            }
        }
        caps
    };

    let channel = sink.channel_type();
    let system_prompt = super::super::system_prompt::with_structured_tools(
        super::super::system_prompt::build(
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            channel,
            &active_switchers,
            Some(&client_caps_value),
        ),
        channel,
    );
    // #2238 — bias the answer toward the requested artifact/media type.
    let system_prompt = if media_type != "auto" {
        format!(
            "{system_prompt}\n\n---\nRequested output type: {media_type}. Prefer producing this artifact/format when feasible; if not possible, explain briefly and provide the closest alternative."
        )
    } else {
        system_prompt
    };
    // #1975 — selective Canvas/SVG subsystem prompt for drawing turns.
    let canvas_block = super::super::system_prompt::canvas_drawing_block(user_text);
    let system_prompt = if canvas_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}\n\n{canvas_block}")
    };

    let session_context = {
        let sm = state.session_manager.lock().await;
        sm.get_session_context_data(&session_id, &user_id)
            .ok()
            .unwrap_or_default()
    };

    *messages = vec![serde_json::json!({"role": "system", "content": system_prompt.clone()})];

    // #1946 — the selected mode is an instruction for this turn, not a
    // session-wide switch: it tells the model whether to plan execution or to
    // answer/patch through the catalog, while the guards stay in place.
    // #2239 — when Agent/VM is disabled, never offer or encourage workspace tools.
    // #2562 — when `agent-vm` is not compiled, the simpler "answer directly"
    //         block replaces the full VM-disabled message so the model never
    //         sees controls or capabilities that do not exist in this build.
    let turn_instruction = if !agent_enabled {
        if cfg!(feature = "agent-vm") {
            format!(
                "THIS TURN: Agent/VM is disabled for this conversation. Do not use or mention vm_exec, \
                 terminal_run, or terminal.exec, and do not assume a workspace/VM is available. \
                 Never claim a file was created and never emit a vmfile: link. Answer directly or use \
                 non-VM capabilities only. When the request genuinely needs the VM (running code, \
                 generating or attaching a file), say exactly: \"{}\"",
                super::super::agent_turn::DISABLED_MESSAGE
            )
        } else {
            "THIS TURN: Answer directly using your knowledge and the API command catalog. \
             Do not reference terminal commands, code execution, sandboxes or file generation."
                .to_string()
        }
    } else {
        match route.route {
            crate::core::bot::ws::stream::routing::Route::Agentic => {
                "THIS TURN: agentic. Plan and execute the multi-step work the request needs, using tools/VM where they help, and deliver the requested artifact."
            }
            crate::core::bot::ws::stream::routing::Route::NonAgentic => {
                "THIS TURN: non-agentic. Answer directly, or handle app-backed work through the app catalog / UI plan. Do not start a vm_exec or terminal_run loop unless the user explicitly asked to execute something or to produce a file."
            }
        }
        .to_string()
    };
    messages.push(serde_json::json!({
        "role": "system",
        "content": turn_instruction
    }));

    // #1670 — structured answers and approved plans shape this turn's prompt.
    super::super::turn_context::apply_structured_answers(&parsed, session_id, messages);

    // #1549/#1637: the tier now selects a real model in
    // `resolve_bot_llm_provider` (`llm-model-<tier>`); this sentence is only a
    // secondary shaping hint, not the routing mechanism.
    super::super::turn_context::apply_model_preference(messages, &model_preference);
    super::super::turn_context::apply_session_context(messages, &session_context);

    let routing = super::super::turn_context::TurnRouting {
        state,
        sink,
        bot_uuid,
        session_id,
        user_id,
    };
    if super::super::turn_context::route_project(
        &routing,
        project_context,
        user_text,
        messages,
        agent_enabled,
    )
    .await?
    {
        return Ok(true);
    }
    super::super::turn_context::apply_drive_listing(messages, &parsed);

    // #1580: Always-on agent VM — auto-provision and inject execution context.
    // #1835 — news/informational turns belong to web grounding, not to the
    // agent VM's code-execution surface: skip the VM contract for them so the
    // model does not route the request into vm_exec (which never shows refs).
    // #2238 — explicit Agent toggle: skip VM provisioning/inject when off.
    if agent_enabled && !super::super::turn_hooks::is_informational_request(user_text) {
        super::super::agent_vm_context::inject(identity, messages).await;
    }

    // #1584: files named in the message are copied from Drive into data/
    // before the model writes the program that reads them.
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if agent_enabled {
        super::super::native_drive::fetch_referenced(
            state, bot_uuid, user_id, session_id, &user_text, messages,
        )
        .await;
    }

    super::super::history::append_conversation_history(
        state, bot_uuid, session_id, user_id, messages,
    )
    .await;

    // #1641 — attachment content is a separate system block so it never gets
    // persisted as if the user had typed the whole file.
    if let Some(note) = attachment_note.take() {
        messages.push(serde_json::json!({"role": "system", "content": note}));
    }

    // #1644 — results of approved browser/terminal actions are fed back into
    // the conversation before the model answers, so the next reply can cite
    // the page title (or say the action was denied/failed).
    super::super::turn_context::apply_completed_actions(session_id, user_id, messages).await;

    // #1545/#1547: grounding context goes LAST, immediately before the model
    // call, so it is the freshest instruction in the prompt.
    #[cfg(feature = "drive")]
    super::super::drive_context::inject(state, &bot_uuid, user_id, user_text, messages).await;

    // #1632 item 3 / #2238 — the composer's Web switch is the decision.
    // When explicitly off, do not search (even for informational requests).
    // When on (or unspecified legacy), search as before.
    //
    // #2147 — a front-page request is answered from rendered images, not from
    // search snippets, so it never pays for a web search first.
    #[cfg(all(feature = "browser", feature = "drive"))]
    let front_page_turn = super::super::front_page::wants_front_page(user_text);

    #[cfg(any(feature = "research", feature = "web-search"))]
    {
        #[cfg(all(feature = "browser", feature = "drive"))]
        let web_search_allowed = !front_page_turn;
        #[cfg(not(all(feature = "browser", feature = "drive")))]
        let web_search_allowed = true;

        let web_flag = parsed.get("web_search").and_then(|value| value.as_bool());
        let web_search_enabled = web_search_allowed
            && match web_flag {
                Some(false) => false,
                Some(true) => true,
                None => super::super::turn_hooks::is_informational_request(user_text),
            };
        if web_search_enabled {
            let citations =
                super::super::web_context::search_and_inject(identity, user_text, messages).await;
            if !citations.is_empty() {
                state
                    .pending_citations
                    .lock()
                    .await
                    .insert(session_id.to_string(), citations);
            }
        }
    }

    // #2147 — "as capas dos jornais … foto" must arrive as pictures. The
    // capture runs here, after every other grounding block, so the exact
    // markdown image is the freshest instruction the model reads.
    #[cfg(all(feature = "browser", feature = "drive"))]
    if front_page_turn {
        super::super::front_page::capture_and_inject(identity, user_text, messages).await;
    }
    super::super::history::send_stream_start_marker(sink, bot_uuid, session_id, user_id).await;

    Ok(false)
}
