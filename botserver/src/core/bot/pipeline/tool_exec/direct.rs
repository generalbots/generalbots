//! Direct tool execution: a `TOOL_EXEC` (type 6) message runs the tool script
//! without any LLM turn.

use uuid::Uuid;

use super::super::types::SessionIdentity;

pub async fn run_tool_exec(identity: &SessionIdentity<'_>, tool_name: &str) {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    let channel = sink.channel_type();
    let work_path = botcore::shared::utils::get_work_path();

    // Resolve through the shared work-tree resolver: the org directory is
    // `<org-id>.gborg`, so the old `{bot}.gborg` assumption matched nothing
    // and the tool was skipped without any log line (the button did nothing).
    // Tools reach the work tree as `.bas` (compiled to `.ast` only after the
    // compiler pass), so both spellings must be attempted: resolving only the
    // `.ast` returned None on a fresh install and the button did nothing.
    let ast_content = ["ast", "bas"]
        .iter()
        .find_map(|ext| {
            crate::core::bot::pipeline::tool_paths::resolve_bot_file(
                &work_path,
                bot_name,
                &format!("{tool_name}.{ext}"),
            )
            .and_then(|path| std::fs::read_to_string(path).ok())
            .filter(|c| !c.is_empty())
        })
        .unwrap_or(Default::default());

    if ast_content.is_empty() {
        botlog::safe_info!(
            "TOOL_EXEC: no compiled script for tool '{tool_name}' of bot '{bot_name}'"
        );
    }

    if !ast_content.is_empty() {
        // Declarative gate: only execute tools the bot script associated
        // with this session via USE TOOL (e.g. inside IF role = "admin").
        if !crate::core::bot::tool_context::is_tool_associated_with_session(
            &state.conn,
            &session_id,
            tool_name,
        ) {
            botlog::safe_info!(
                "TOOL_EXEC: tool '{tool_name}' not associated with session {session_id}, skipping"
            );
            // The user pressed a button; silence would look like a broken bot.
            let message = format!("A ação '{tool_name}' não está disponível para esta conversa.");
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &message,
                channel,
            );
            if let Err(e) = sink.send_bot_response(&resp).await {
                log::warn!("tool exec: availability notice not delivered: {e}");
            }
            return;
        }
        let state_for_tool = state.clone();
        let tool_name_clone = tool_name.to_string();
        let channel_name = channel.to_string();
        // #2095 — a tool that publishes chips resolves their keys for the
        // language the user picked.
        let locale_for_tool = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
        let session_for_tool = botlib::models::UserSession {
            id: session_id,
            user_id,
            branch_id: Uuid::nil(),
            bot_id: bot_uuid,
            org_id: botcoresession::org_id_for_bot(&state_for_tool.conn, bot_uuid),
            title: String::new(),
            context_data: serde_json::json!({"channel": channel_name, "locale": locale_for_tool.to_string()}),
            current_tool: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            locked: false,
            lock_pin_hash: None,
        };
        // #2562 — tool_run activity.
        let tool_ctx = crate::core::bot::pipeline::event_sink::RunContext::new(
            &session_id.to_string(),
            "chat",
        )
        .with_state(state.clone())
        .with_scope(bot_uuid, user_id, None);
        let tool_act = crate::core::bot::pipeline::activity::Activity::started(
            crate::core::bot::pipeline::activity::ActivityKind::ToolRun,
            crate::core::bot::pipeline::activity::tool_run_summary(tool_name, "running"),
        )
        .with_run_id(&tool_ctx.run_id);
        crate::core::bot::pipeline::activity::emit_activity(sink, &tool_ctx, &tool_act).await;

        let tool_run = tokio::task::spawn_blocking(move || {
            let mut svc =
                crate::basic::ScriptService::new(state_for_tool.clone(), session_for_tool);
            svc.load_bot_config_params(&state_for_tool, bot_uuid);
            svc.run(&ast_content).map_err(|e| e.to_string())
        });
        match tool_run.await {
            Ok(Ok(_)) => {
                let done = tool_act.completed(
                    crate::core::bot::pipeline::activity::tool_run_summary(tool_name, "ok"),
                );
                crate::core::bot::pipeline::activity::emit_activity(sink, &tool_ctx, &done).await;
            }
            Ok(Err(detail)) => {
                let fail = tool_act.failed(&detail);
                crate::core::bot::pipeline::activity::emit_activity(sink, &tool_ctx, &fail).await;

                log::warn!("Tool '{tool_name_clone}' execution error: {detail}");
                let message = format!(
                    "Nao consegui executar '{tool_name_clone}'. Tente novamente em instantes."
                );
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    &message,
                    channel,
                );
                if let Err(e) = sink.send_bot_response(&resp).await {
                    log::warn!("tool exec: failure notice not delivered: {e}");
                }
            }
            Err(e) => {
                let fail = tool_act.failed(&e.to_string());
                crate::core::bot::pipeline::activity::emit_activity(sink, &tool_ctx, &fail).await;
                log::warn!("Tool '{tool_name_clone}' join error: {e}");
            }
        }
    }
}
