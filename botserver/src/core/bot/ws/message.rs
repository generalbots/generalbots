//! Session bootstrap: running `start.bas` once per conversation.
//!
//! Prompt loading lives in [`super::prompt`] and suggestion chips in
//! [`super::suggestions`]; both are re-exported here because that is the path
//! callers already use (#2095).

use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub use super::prompt::{
    load_bot_styles_css, load_system_prompt, load_system_prompt_for_channel,
    with_response_language_contract, RESPONSE_LANGUAGE_INSTRUCTION,
};
pub use super::suggestions::send_start_suggestions;

pub async fn run_start_bas_on_connect(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    rx: &mut mpsc::Receiver<botlib::models::BotResponse>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
) -> bool {
    {
        // Reserve the session in the SAME lock scope as the check: two sockets
        // connecting at once both saw an empty guard and ran start.bas twice,
        // which rendered the greeting twice.
        let mut guards = state.start_bas_guards.lock().await;
        if guards.contains_key(&session_id) {
            info!("start.bas execution skipped: session already initialized (in-memory guard)");
            drop(guards);
            send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id, bot_name).await;
            return false;
        }
        guards.insert(session_id, true);
        info!("start.bas execution proceeding: session not found in guard");
    }

    // A conversation that already has messages is being reopened (history tab,
    // sidebar click, reload): running start.bas again appended its greeting to
    // the restored transcript and looked like a duplicated/foreign message.
    {
        let has_history = {
            let mut sm = state.session_manager.lock().await;
            sm.get_conversation_history(session_id, user_id, Some(1))
                .map(|h| !h.is_empty())
                .unwrap_or(false)
        };
        if has_history {
            botlog::safe_info!("start.bas skipped: session {session_id} already has messages");
            send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id, bot_name).await;
            return false;
        }
    }

    let session_init_key = botlib::key_utils::build_key(
        "",
        &[
            "start_bas_executed",
            &bot_uuid.to_string(),
            &session_id.to_string(),
        ],
    );
    if let Some(ref cache) = state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("DEL")
                .arg(&session_init_key)
                .query_async(&mut conn)
                .await;
        }
    }

    let work_path = botcore::shared::utils::get_work_path();

    // Cross-layout lookup: bot-named org bucket, standalone bucket, or org workspace.
    let ast_content = {
        let mut found = String::new();
        let rel = format!("{bot_name}.gbdialog/start.ast");
        match botcore::shared::utils::read_bot_file_across_layouts(&work_path, bot_name, &rel).await
        {
            Some(c) => {
                info!(
                    "start.bas: found start.ast for {bot_name} ({} bytes)",
                    c.len()
                );
                found = c;
            }
            None => {
                let bas_rel = format!("{bot_name}.gbdialog/start.bas");
                if let Some(c) = botcore::shared::utils::read_bot_file_across_layouts(
                    &work_path, bot_name, &bas_rel,
                )
                .await
                {
                    info!("start.bas: found start.bas (uncompiled) for {bot_name}");
                    found = c;
                }
            }
        }
        found
    };

    if ast_content.is_empty() {
        return false;
    }

    {
        let mut guards = state.start_bas_guards.lock().await;
        guards.insert(session_id, true);
    }

    let state_for_bas = state.clone();
    let bot_id_for_bas = bot_uuid;
    // #2095 — keyed suggestion labels resolve for the language the user picked.
    let locale_for_bas = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
    let session_for_bas = botlib::models::UserSession {
        id: session_id,
        user_id,
        branch_id: Uuid::nil(),
        bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::json!({"channel": "web", "locale": locale_for_bas.to_string()}),
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        locked: false,
        lock_pin_hash: None,
    };
    // #1369 — both values are consumed by the call and unused afterwards, so
    // they are moved instead of cloned (`UserSession` owns several Strings).
    let exec_result =
        crate::basic::ScriptService::execute_script(state_for_bas, session_for_bas, &ast_content)
            .await;
    match exec_result {
        Ok(result) => info!(
            "start.bas: execution result (len={}): {}",
            result.len(),
            result
        ),
        Err(e) => warn!("start.bas: execution error: {}", e),
    }

    let mut drained = 0usize;
    loop {
        match rx.try_recv() {
            Ok(response) => {
                drained += 1;
                botlog::safe_info!(
                    "start.bas: drained BotResponse[{}]: mtype={}, session={}, content='{}'",
                    drained,
                    i32::from(response.message_type),
                    response.session_id,
                    response.content.chars().take(80).collect::<String>()
                );
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = ws_sender.send(Message::Text(json)).await;
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(_) => break,
        }
    }
    info!("start.bas: drained {} responses immediately", drained);

    send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id, bot_name).await;
    true
}
