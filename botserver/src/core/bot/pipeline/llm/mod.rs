//! Streaming LLM turn: session tools, retry budget and the bounded tool loop.
//!
//! Split so the chunk pump (`attempt`) and the final markdown conversion
//! (`markdown`) stay separate from the turn orchestration (`stream`).

use uuid::Uuid;

use super::sink::ChannelSink;

mod attempt;
mod markdown;
mod stream;

pub use stream::stream_llm_response;

/// Loads the session's tool set with a 10 s timeout so a slow DB cannot stall
/// the streaming LLM loop; any failure yields an empty tool list.
async fn get_session_tools_with_timeout(
    state: &botcore::shared::state::AppState,
    bot_name: &str,
    session_id_s: &str,
) -> Vec<serde_json::Value> {
    let sid: Uuid = match session_id_s.parse() {
        Ok(id) => id,
        Err(_) => Uuid::new_v4(),
    };
    let pool = state.conn.clone();
    let name = bot_name.to_string();
    let sid_for_block = sid;
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::task::spawn_blocking(move || {
            crate::core::bot::tool_context::get_session_tools(&pool, &name, &sid_for_block)
        }),
    )
    .await
    {
        Ok(Ok(Ok(tools))) => tools,
        Ok(Ok(Err(e))) => {
            botlog::safe_warn!("get_session_tools error for {session_id_s}: {e}");
            Vec::new()
        }
        Ok(Err(e)) => {
            botlog::safe_warn!(
                "get_session_tools spawn_blocking join error for {session_id_s}: {e}"
            );
            Vec::new()
        }
        Err(_) => {
            botlog::safe_warn!("get_session_tools TIMEOUT after 10s for session {session_id_s}");
            Vec::new()
        }
    }
}

fn is_web_channel(sink: &dyn ChannelSink) -> bool {
    sink.channel_type() == "web"
}

const LLM_RETRY_MAX: usize = 3;
const LLM_RETRY_DELAY: u64 = 3;

fn llm_empty_message() -> String {
    let locale = botcore::i18n::Locale::new("pt-BR").unwrap_or_default();
    let msg = botcore::i18n::t(&locale, "error-llm-empty-response");
    if msg.starts_with('[') && msg.ends_with(']') {
        "Sorry, I could not process your message right now. Please try again in a few seconds."
            .to_string()
    } else {
        msg
    }
}
