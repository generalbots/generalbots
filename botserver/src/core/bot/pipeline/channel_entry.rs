//! Channel entry point for the message pipeline (#939 D split).
//!
//! `run_pipeline_for_channel` adapts a structured [`UserMessage`] arriving
//! from any channel into the raw JSON envelope consumed by
//! `process_message_internal`, resolving the bot name to its UUID first.
//! Split out of `exec.rs` to keep every pipeline file under the 450-line
//! repository limit; behavior is unchanged.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::exec::process_message_internal;
use super::sink::ChannelSink;
use super::types::PipelineResult;

pub async fn run_pipeline_for_channel(
    state: &Arc<AppState>,
    msg: &botlib::models::UserMessage,
    sink: &dyn ChannelSink,
) -> PipelineResult<()> {
    // `msg.bot_id` carries a bot NAME on some channels and a bot UUID on
    // others (the Telegram webhook fills it with the UUID). Everything
    // downstream — tool loading (`get_session_tools` looks up
    // `bots.name`), work-dir paths, prompt text — expects the NAME, so
    // normalize here once instead of patching each consumer.
    let mut bot_name = msg.bot_id.clone();
    if Uuid::parse_str(&bot_name).is_ok() {
        if let Some(resolved) = resolve_bot_name(&state.conn, &bot_name).await {
            bot_name = resolved;
        }
    }
    let user_text = msg.content.clone();
    let session_id = Uuid::parse_str(&msg.session_id).unwrap_or_else(|_| Uuid::new_v4());
    let user_id = Uuid::parse_str(&msg.user_id).unwrap_or_else(|_| Uuid::nil());

    let bot_uuid = resolve_bot_uuid(&state.conn, &bot_name).await;

    let response_key = format!("{}_{}", session_id, Uuid::new_v4());
    let (tx_internal, mut rx_internal) =
        tokio::sync::mpsc::channel::<botlib::models::BotResponse>(100);
    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(response_key.clone(), tx_internal);
    }

    let json_msg = serde_json::json!({
        "text": user_text,
        "content": user_text,
        "message_type": i32::from(msg.message_type),
    })
    .to_string();

    let mut start_bas_ran = false;
    let result = process_message_internal(
        sink,
        &mut rx_internal,
        state,
        session_id,
        user_id,
        bot_uuid,
        &bot_name,
        &mut start_bas_ran,
        &json_msg,
    )
    .await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&response_key);
    }

    result
}

/// Reverse lookup for callers that pass a bot UUID where a name is
/// expected. Returns `None` when the value is not a known bot id.
async fn resolve_bot_name(
    pool: &botcore::shared::utils::DbPool,
    bot_uuid: &str,
) -> Option<String> {
    let uuid = Uuid::parse_str(bot_uuid).ok()?;
    use diesel::prelude::*;
    if let Ok(mut conn) = pool.get_timeout(std::time::Duration::from_secs(3)) {
        #[derive(diesel::QueryableByName)]
        struct BotName {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }
        diesel::sql_query("SELECT name FROM bots WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(uuid)
            .get_result::<BotName>(&mut conn)
            .ok()
            .map(|r| r.name)
    } else {
        None
    }
}

async fn resolve_bot_uuid(pool: &botcore::shared::utils::DbPool, bot_name: &str) -> uuid::Uuid {
    if let Ok(uuid) = uuid::Uuid::parse_str(bot_name) {
        return uuid;
    }
    use diesel::RunQueryDsl;
    if let Ok(mut conn) = pool.get_timeout(std::time::Duration::from_secs(3)) {
        #[derive(diesel::QueryableByName)]
        struct BotId {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
        }
        diesel::sql_query("SELECT id FROM bots WHERE name = $1 AND is_active = true LIMIT 1")
            .bind::<diesel::sql_types::Text, _>(bot_name)
            .get_result::<BotId>(&mut conn)
            .ok()
            .map(|r| r.id)
            .unwrap_or_default()
    } else {
        uuid::Uuid::nil()
    }
}
