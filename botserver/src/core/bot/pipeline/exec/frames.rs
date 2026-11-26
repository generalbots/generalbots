//! Control frames a chat message can carry instead of a question: they are
//! consumed here and never reach the model.

use uuid::Uuid;

use super::super::types::SessionIdentity;

/// Returns true when `parsed` was a control frame and has been answered.
pub(super) async fn consume_control_frame(identity: &SessionIdentity<'_>, parsed: &serde_json::Value) -> bool {
    // Agent-mode control frame (legacy toggle, #1167). Always-on since #1580
    // but kept for backward compat — the frame is consumed so it never
    // reaches the LLM.
    #[cfg(feature = "agent-vm")]
    if parsed.get("type").and_then(|v| v.as_str()) == Some("agent_mode") {
        if let Some(status) =
            crate::core::bot::agent_vm_hook::handle_frame(parsed, identity.session_id, identity.user_id, identity.bot_uuid)
                .await
        {
            let resp = botlib::models::BotResponse::new(
                identity.bot_uuid.to_string(),
                identity.session_id.to_string(),
                identity.user_id.to_string(),
                &status,
                identity.sink.channel_type(),
            );
            if let Err(e) = identity.sink.send_bot_response(&resp).await {
                log::debug!("computer-use status response failed: {e}");
            }
            return true;
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
            let result = parsed.get("result").cloned().unwrap_or_else(|| { log::debug!("using default for tool result"); Default::default() });
            if crate::core::computer_use::complete(id, identity.user_id, result).await.is_none() {
                log::debug!("computer-use action returned no result");
            }
        }
        return true;
    }
    false
}

/// The legacy `__TOOL__:` text and a `TOOL_EXEC` (message type 6) run a tool
/// directly. Returns true when the message was one of them.
pub(super) async fn consume_tool_frame(identity: &SessionIdentity<'_>, msg_type: i64, user_text: &str) -> bool {
    if user_text.starts_with("__TOOL__:") {
        let tool_name = user_text.trim_start_matches("__TOOL__:").trim().to_string();
        if !tool_name.is_empty() {
            let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, identity.user_id);
            let resp = botlib::models::BotResponse::new(
                identity.bot_uuid.to_string(),
                identity.session_id.to_string(),
                identity.user_id.to_string(),
                botcore::i18n::t_args(&locale, "error-tool-not-available", &[("name", &tool_name)]),
                "web",
            );
            if let Err(e) = identity.sink.send_bot_response(&resp).await {
                log::debug!("tool error response failed: {e}");
            }
        }
        return true;
    }

    if msg_type == 6 {
        let raw_tool_name = user_text.trim().to_string();
        let tool_name = match crate::core::bot::ws::handler::validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("TOOL_EXEC: invalid tool name '{}': {}", raw_tool_name, e);
                let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, identity.user_id);
                let text = botcore::i18n::t_args(&locale, "error-tool-invalid-name", &[("name", &raw_tool_name)]);
                let resp = botlib::models::BotResponse::new(
                    identity.bot_uuid.to_string(),
                    identity.session_id.to_string(),
                    identity.user_id.to_string(),
                    format!("<p>{text}</p>"),
                    "web",
                );
            if let Err(e) = identity.sink.send_bot_response(&resp).await {
                log::debug!("tool error response failed: {e}");
            }
                return true;
            }
        };

        if !tool_name.is_empty() {
            super::super::turn_watch::long_stage("tool exec");
            super::super::tool_exec::run_tool_exec(identity, &tool_name).await;
        }
        return true;
    }
    false
}
