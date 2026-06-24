use std::sync::Arc;
use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::{error, info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::bot::ws::handler::verify_path_within_workdir;

pub fn load_system_prompt(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let rel_path = format!("{}.gbai/{}.gbot/", bot_name, bot_name);
    if !verify_path_within_workdir(&rel_path) {
        error!("Path traversal detected in load_system_prompt for bot: {}", bot_name);
        let now = chrono::Utc::now().format("%B %d, %Y").to_string();
        return format!("Today is {now}.\n\nYou are a helpful assistant. Respond only with valid HTML fragments. Do not use markdown. Do not use code blocks. Use only: <p>, <h3>, <ul>, <li>, <strong>, <em>. Every tag you open MUST be properly closed. Start your response directly with an HTML tag, never with plain text.");
    }

    let gbot_dir = format!("{}/{}.gbai/{}.gbot/", work_dir, bot_name, bot_name);
    let prompt_from_file = std::fs::read_to_string(format!("{}PROMPT.md", gbot_dir))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.md", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}PROMPT.txt", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.txt", gbot_dir)));

    if let Ok(p) = prompt_from_file {
        return p;
    }

    let now = chrono::Utc::now().format("%B %d, %Y").to_string();
    format!("Today is {now}.\n\nYou are a helpful assistant. Respond only with valid HTML fragments. Do not use markdown. Do not use code blocks. Use only: <p>, <h3>, <ul>, <li>, <strong>, <em>. Every tag you open MUST be properly closed. Start your response directly with an HTML tag, never with plain text.\n\nWhen asked for a ramal (extension), answer ONLY the number. Do not mention name, job title, department or any other information. Just the number.")
}

pub fn load_bot_styles_css(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let rel_path = format!("{}.gbai/{}.gbot/", bot_name, bot_name);
    if !verify_path_within_workdir(&rel_path) {
        error!("Path traversal detected in load_bot_styles_css for bot: {}", bot_name);
        return String::new();
    }

    let gbot_dir = format!("{}/{}.gbai/{}.gbot/", work_dir, bot_name, bot_name);

    let global_css_path = format!("{}global.css", gbot_dir);
    let mut combined_css = match std::fs::read_to_string(&global_css_path) {
        Ok(c) => {
            info!("global.css loaded from {} ({} bytes)", global_css_path, c.len());
            c
        }
        Err(_) => String::new(),
    };

    let css_path = format!("{}styles.css", gbot_dir);
    let local_css = match std::fs::read_to_string(&css_path) {
        Ok(c) => {
            info!("styles.css loaded from {} ({} bytes)", css_path, c.len());
            c
        }
        Err(e1) => {
            let alt_path = format!("{}style.css", gbot_dir);
            match std::fs::read_to_string(&alt_path) {
                Ok(c) => {
                    info!("style.css loaded from {} ({} bytes)", alt_path, c.len());
                    c
                }
                Err(e2) => {
                    warn!("No styles.css/ style.css found at {} or {}: {}, {}", css_path, alt_path, e1, e2);
                    String::new()
                }
            }
        }
    };

    if !local_css.is_empty() {
        if !combined_css.is_empty() {
            combined_css.push('\n');
        }
        combined_css.push_str(&local_css);
    }

    combined_css
}

pub async fn send_start_suggestions(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
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
        info!("ws_handler: sending {} suggestions and {} switchers on reconnect", suggestions.len(), switchers.len());
        let _ = ws_sender.send(Message::Text(serde_json::json!({
            "bot_id": bot_uuid.to_string(),
            "user_id": user_id.to_string(),
            "session_id": session_id.to_string(),
            "channel": "web",
            "content": "",
            "message_type": 2,
            "is_complete": true,
            "suggestions": suggestions,
            "switchers": switchers,
            "context_length": 0,
            "context_max_length": 0,
        }).to_string())).await;
    }
}

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
        let guards = state.start_bas_guards.lock().await;
        if guards.contains_key(&session_id) {
            info!("start.bas execution skipped: session already initialized (in-memory guard)");
            send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id).await;
            return false;
        }
        info!("start.bas execution proceeding: session not found in guard");
    }

    let session_init_key = botlib::key_utils::build_key("", &["start_bas_executed", &bot_uuid.to_string(), &session_id.to_string()]);
    if let Some(ref cache) = state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("DEL").arg(&session_init_key).query_async(&mut conn).await;
        }
    }

    let work_path = botcore::shared::utils::get_work_path();
    let rel_ast_path = format!("{}.gbai/{}.gbdialog/start.ast", bot_name, bot_name);
    if !verify_path_within_workdir(&rel_ast_path) {
        error!("Path traversal detected in run_start_bas_on_connect for bot: {}", bot_name);
        return false;
    }

    let ast_path = format!("{}/{}.gbai/{}.gbdialog/start.ast", work_path, bot_name, bot_name);
    let ast_content = match tokio::fs::read_to_string(&ast_path).await {
        Ok(c) if !c.is_empty() => c,
        _ => {
            let bas_path = ast_path.replace(".ast", ".bas");
            tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
        }
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
    let _bot_name_owned = bot_name.to_string();
    let session_for_bas = botlib::models::UserSession {
        id: session_id, user_id, bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::Value::Null,
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    info!("start.bas: DEBUG BEFORE execute_script");
    let exec_result = crate::basic::ScriptService::execute_script(
        state_for_bas.clone(),
        session_for_bas.clone(),
        &ast_content,
    ).await;
    info!("start.bas: DEBUG AFTER execute_script");
    match exec_result {
        Ok(result) => info!("start.bas: execution result (len={}): {}", result.to_string().len(), result),
        Err(e) => warn!("start.bas: execution error: {}", e),
    }

    for i in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        match rx.try_recv() {
            Ok(response) => {
                info!("start.bas: drained BotResponse: content={}", response.content.chars().take(80).collect::<String>());
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = ws_sender.send(Message::Text(json)).await;
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                if i == 0 { info!("start.bas: rx empty, waiting..."); }
                continue;
            }
            Err(e) => {
                info!("start.bas: rx done: {:?}", e);
                break;
            }
        }
    }

    send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id).await;
    true
}
