mod answer_mode;
mod caps;
mod frames;
mod tool_call;
mod ttft;
mod turn;

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::info;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::bot::runs::caps::RunBudget;

/// Sender half of the web socket that a turn writes to.
pub(super) type WsSink = futures_util::stream::SplitSink<WebSocket, Message>;

/// One LLM turn of a web-socket session.
pub struct LlmTurn<'a> {
    pub state: &'a Arc<AppState>,
    pub bot_uuid: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub bot_name: &'a str,
    pub messages: &'a serde_json::Value,
    pub user_text: &'a str,
}

pub async fn process_llm_response(
    ws_sender: &mut WsSink,
    rx: &mut mpsc::Receiver<botlib::models::BotResponse>,
    turn: &LlmTurn<'_>,
) {
    let LlmTurn {
        state,
        bot_uuid,
        session_id,
        user_id,
        user_text,
        ..
    } = *turn;
    // #1620 — mark the reply as in flight for the whole function (including
    // tool calls) so the desktop shell never terminates mid-generation.
    let _desktop_activity = crate::core::desktop_activity::ActivityGuard::begin();

    // #1696 — opt-in per-run budget (tokens + wall clock).
    let caps = caps::resolve(state, bot_uuid, user_id);
    let budget = RunBudget::start(caps);

    let bot_llm_provider: Option<(Arc<dyn botlib::traits::LLMProvider>, String, String)> =
        match crate::core::bot::pipeline::enriched_llm::resolve_bot_llm_provider(
            state, bot_uuid, user_id, session_id, "auto",
        )
        .await
        {
            Ok(provider) => provider,
            Err(message) => {
                // #1654 — a subscription provider without a linked account must
                // tell the user what to do instead of silently using another key.
                let err_resp = frames::error(turn, format!("<p>{}</p>", message));
                let _ = ws_sender.send(Message::Text(err_resp.to_string())).await;
                return;
            }
        };

    if answer_mode::try_handle(ws_sender, turn).await {
        return;
    }

    let env_key = std::env::var("LLM_KEY").unwrap_or_default();
    let env_model = std::env::var("LLM_MODEL").unwrap_or_default();
    info!(
        "stream: bot_llm_provider.is_some={}, fallback_key_len={}",
        bot_llm_provider.is_some(),
        env_key.len()
    );
    match bot_llm_provider.or_else(|| {
        state
            .llm_provider
            .clone()
            .map(move |p| (p, env_key.clone(), env_model.clone()))
    }) {
        Some((llm, llm_key, llm_model)) => {
            let result = turn::run(ws_sender, turn, llm, llm_key, llm_model, budget).await;

            // #1635 — drain the sources for this turn. The frame below is the
            // only place the renderer reads `citations` from, so an empty vec
            // here means the answer was produced without live sources.
            let citations = {
                let mut pending = state.pending_citations.lock().await;
                pending.remove(&session_id.to_string()).unwrap_or_default()
            };

            let final_resp = frames::final_frame(
                turn,
                result.final_content.clone(),
                result.reasoning.trim(),
                citations,
            );
            if ws_sender
                .send(Message::Text(final_resp.to_string()))
                .await
                .is_err()
            {
                let mut pending = state.pending_stream_responses.lock().await;
                pending.insert(session_id.to_string(), result.final_content);
            }

            if !result.capped
                && tool_call::maybe_execute(ws_sender, rx, turn, &result.full_response).await
            {
                return;
            }

            {
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &result.full_response, 2);
            }
        }
        None => {
            info!("No LLM provider");
            let fallback = format!("Recebi: \"{}\"", user_text);
            {
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &fallback, 2);
            }
            let resp = frames::fallback(turn, fallback);
            let _ = ws_sender.send(Message::Text(resp.to_string())).await;
        }
    }
}
