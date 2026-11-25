//! Computer-use proposals raised before the model runs.

use super::super::types::SessionIdentity;

/// True when the turn was turned into a consent card (the caller returns
/// without invoking the model).
pub(crate) async fn propose_computer_action(
    identity: &SessionIdentity<'_>,
    msg_type: i64,
    user_text: &str,
) -> bool {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    if msg_type != 1 || !sink.supports_raw_frames() {
        return false;
    }
    if let Some(action) = crate::core::computer_use::propose_from_text(user_text) {
        let proposal = crate::core::computer_use::create(
            &session_id.to_string(),
            user_id,
            "browser",
            action.clone(),
        )
        .await;
        let frame = serde_json::json!({
            "type": "browser_action",
            "action_type": "browser",
            "message_type": "browser_action",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "browser_action": {
                "action_id": proposal.id,
                "action": "open_url",
                "url": action.get("url").cloned().map_or_else(Default::default, |v| v),
            },
        });
        if let Err(e) = sink.send_raw_json(&frame).await {
            log::warn!("computer action: frame not delivered: {e}");
        }
        return true;
    }
    if let Some(action) = crate::core::computer_use::propose_terminal_from_text(user_text) {
        // #2239/#2264 — a terminal proposal runs inside the workspace VM:
        // refuse it (with the switch hint) instead of offering a consent card
        // the user could approve into a VM run.
        if !super::super::agent_turn::workspace_route_allowed(identity.agent_vm_enabled) {
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                super::super::agent_turn::DISABLED_MESSAGE,
                "web",
            );
            if let Err(e) = sink.send_bot_response(&resp).await {
                log::warn!("computer action: disabled notice not delivered: {e}");
            }
            return true;
        }
        let proposal = crate::core::computer_use::create(
            &session_id.to_string(),
            user_id,
            "terminal",
            action.clone(),
        )
        .await;
        let frame = serde_json::json!({
            "type": "terminal_action",
            "action_type": "terminal",
            "message_type": "terminal_action",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "terminal_action": {
                "action_id": proposal.id,
                "command": action.get("command").cloned().map_or_else(Default::default, |v| v),
                "cwd": action.get("cwd").cloned().map_or_else(Default::default, |v| v),
                "reason": action.get("reason").cloned().map_or_else(Default::default, |v| v),
            },
        });
        if let Err(e) = sink.send_raw_json(&frame).await {
            log::warn!("computer action: frame not delivered: {e}");
        }
        return true;
    }
    false
}
