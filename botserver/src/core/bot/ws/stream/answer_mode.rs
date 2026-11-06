use axum::extract::ws::Message;
use futures_util::SinkExt;

use super::{LlmTurn, WsSink};

/// Answers a data/chart turn without calling the chat provider. Returns `true`
/// when the turn was handled here and the caller must not stream an answer.
pub(super) async fn try_handle(ws_sender: &mut WsSink, turn: &LlmTurn<'_>) -> bool {
    let LlmTurn {
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        user_text,
        ..
    } = *turn;
    let answer_mode = {
        #[cfg(feature = "chat")]
        {
            crate::core::bot::answer_mode::get_answer_mode(state, &session_id).await
        }
        #[cfg(not(feature = "chat"))]
        {
            crate::core::bot::answer_mode::AnswerMode::Default
        }
    };
    if answer_mode == crate::core::bot::answer_mode::AnswerMode::Default {
        return false;
    }
    let mode_response = match answer_mode {
        crate::core::bot::answer_mode::AnswerMode::Data => {
            crate::core::bot::answer_mode::generate_data_response(
                state, user_text, bot_uuid, bot_name, session_id, user_id,
            )
            .await
        }
        crate::core::bot::answer_mode::AnswerMode::Chart => {
            crate::core::bot::answer_mode::generate_chart_response(
                state, user_text, bot_uuid, bot_name, session_id, user_id,
            )
            .await
        }
        _ => return true,
    };
    match mode_response {
        Ok(resp) => {
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = ws_sender.send(Message::Text(json)).await;
            }
            let mut sm = state.session_manager.lock().await;
            let _ = sm.save_message(session_id, user_id, 2, &resp.content, 2);
        }
        Err(e) => {
            let err_resp = super::frames::error(turn, format!("<p>Error: {}</p>", e));
            let _ = ws_sender.send(Message::Text(err_resp.to_string())).await;
        }
    }
    true
}
