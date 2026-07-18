use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::PipelineResult;

pub async fn run_start_bas(
    sink: &dyn ChannelSink,
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
) -> PipelineResult<bool> {
    {
        let guards = state.start_bas_guards.lock().await;
        if guards.contains_key(&session_id) {
            log::info!("start.bas execution skipped: session already initialized (in-memory guard)");
            send_start_suggestions(sink, state, bot_uuid, session_id, user_id).await;
            return Ok(false);
        }
        log::info!("start.bas execution proceeding: session not found in guard");
    }

    let session_init_key = botlib::key_utils::build_key("", &["start_bas_executed", &bot_uuid.to_string(), &session_id.to_string()]);
    if let Some(ref cache) = state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SET")
                .arg(session_init_key.as_str())
                .arg("1")
                .arg("EX")
                .arg("259200")
                .query_async(&mut conn).await;
        }
    }

    let work_path = botcore::shared::utils::get_work_path();
    let ast_content = {
        let paths = [
            format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/start.ast"),
            format!("{bot_name}.gbai/{bot_name}.gbdialog/start.ast"),
        ];
        let mut found = String::new();
        for rel_path in &paths {
            let safe = crate::core::bot::ws::handler::verify_path_within_workdir(rel_path);
            if !safe { continue; }
            let full_path = format!("{work_path}/{rel_path}");
            match tokio::fs::read_to_string(&full_path).await {
                Ok(c) if !c.is_empty() => { found = c; break; }
                _ => {
                    let bas_path = full_path.replace(".ast", ".bas");
                    if let Ok(c) = tokio::fs::read_to_string(&bas_path).await {
                        if !c.is_empty() { found = c; break; }
                    }
                }
            }
        }
        found
    };

    if ast_content.is_empty() {
        return Ok(false);
    }

    {
        let mut guards = state.start_bas_guards.lock().await;
        guards.insert(session_id, true);
    }

    let state_for_bas = state.clone();
    let bot_id_for_bas = bot_uuid;
    let channel_name = sink.channel_type().to_string();
    let session_for_bas = botlib::models::UserSession {
        id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::json!({"channel": channel_name}),
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let exec_result = crate::basic::ScriptService::execute_script(
        state_for_bas.clone(),
        session_for_bas.clone(),
        &ast_content,
    ).await;
    match exec_result {
        Ok(result) => log::info!("start.bas: execution result (len={}): {}", result.to_string().len(), result),
        Err(e) => log::warn!("start.bas: execution error: {e}"),
    }

    let mut drained = 0usize;
    loop {
        match rx.try_recv() {
            Ok(response) => {
                drained += 1;
                let _ = sink.send_bot_response(&response).await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(_) => break,
        }
    }
    log::info!("start.bas: drained {drained} responses immediately");

    send_start_suggestions(sink, state, bot_uuid, session_id, user_id).await;
    Ok(true)
}

async fn send_start_suggestions(
    sink: &dyn ChannelSink,
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
) {
    let suggestions = {
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
    let switchers = {
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
    if !suggestions.is_empty() || !switchers.is_empty() {
        let resp = botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: suggestions.into_iter().map(|s| botlib::models::Suggestion::new(s.text)).collect(),
            switchers,
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        };
        let _ = sink.send_bot_response(&resp).await;
    }
}