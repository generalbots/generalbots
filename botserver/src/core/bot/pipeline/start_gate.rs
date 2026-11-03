//! Session start gate: hearing input, `start.bas` and the two opening frames.
//!
//! Split out of `exec.rs` (#1632 follow-up). Keeping this here leaves the
//! pipeline entry point readable and every pipeline file under the repository's
//! 450-line limit.

use std::sync::Arc;

use botlib::message_types::MessageType;
use botlib::models::BotResponse;

use super::types::{PipelineResult, SessionIdentity};

/// Runs the per-session preamble for one turn.
///
/// Returns `true` when the turn was consumed (a `HEAR` listener took the text
/// and the caller must stop); `false` when the turn continues to the model.
/// The start-bas suggestions/switchers and the "turn started" frame are sent
/// from here exactly as before.
pub(super) async fn gate_turn(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<BotResponse>,
    start_bas_ran: &mut bool,
    user_text: &str,
) -> PipelineResult<bool> {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
    } = *identity;

    let runtime: Arc<dyn botbasic_types::BasicRuntime> =
        Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
    let delivered =
        crate::basic::keywords::hearing::deliver_hear_input(&runtime, session_id, user_text.to_string());
    if delivered {
        return Ok(true);
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
        let resp = BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            // Keep the full suggestion: `Suggestion::new(text)` discards the
            // `action`, so every chip re-sent after start.bas reached the
            // client as a plain message and tool chips (`invoke_tool`) stopped
            // executing the tool (they sent `message_type: 1` instead of 6).
            suggestions: post_start_suggestions,
            switchers: post_start_switchers,
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        };
        let _ = sink.send_bot_response(&resp).await;
    }

    let _ = sink
        .send_bot_response(&BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: MessageType::BOT_RESPONSE,
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

    Ok(false)
}
