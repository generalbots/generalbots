use std::sync::Arc;
use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::{info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::bot::ws::handler::verify_path_within_workdir;
use botcore::shared::utils::current_org_id;

pub fn load_system_prompt(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let org_id = current_org_id();
    let nil_uuid = uuid::Uuid::nil();

    // Helper to read prompt from a gbot directory
    let read_prompt = |gbot_dir: &str| -> Option<String> {
        let path = |f: &str| format!("{}{}", gbot_dir, f);
        std::fs::read_to_string(path("PROMPT.md"))
            .or_else(|_| std::fs::read_to_string(path("prompt.md")))
            .or_else(|_| std::fs::read_to_string(path("PROMPT.txt")))
            .or_else(|_| std::fs::read_to_string(path("prompt.txt")))
            .ok()
    };

    // Primary path: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/ (mirrors MinIO drive)
    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
    if verify_path_within_workdir(&primary_rel) {
        let gbot_dir = format!("{work_dir}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    // Fallback: {org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/ (nil UUID legacy)
    let fallback_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
    if org_id != nil_uuid && verify_path_within_workdir(&fallback_rel) {
        let gbot_dir = format!("{work_dir}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    let now = chrono::Utc::now().format("%B %d, %Y").to_string();
    format!("Today is {now}.\n\nYou are a helpful assistant. Be concise. Respond in plain text. No HTML.")
}

pub fn load_bot_styles_css(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let org_id = current_org_id();
    let nil_uuid = uuid::Uuid::nil();

    // Helper: read CSS from a gbot directory
    let read_css = |gbot_dir: &str| -> String {
        let global_css_path = format!("{}global.css", gbot_dir);
        let mut combined = match std::fs::read_to_string(&global_css_path) {
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
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&local_css);
        }
        combined
    };

    // Primary: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
    if verify_path_within_workdir(&primary_rel) {
        let gbot_dir = format!("{work_dir}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
        let css = read_css(&gbot_dir);
        if !css.is_empty() {
            return css;
        }
    }

    // Fallback: {org_id}.gborg path (nil UUID)
    let fallback_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
    if org_id != nil_uuid && verify_path_within_workdir(&fallback_rel) {
        let gbot_dir = format!("{work_dir}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
        let css = read_css(&gbot_dir);
        if !css.is_empty() {
            return css;
        }
    }

    String::new()
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

    // Try {bot_name}.gborg/ path first, then {bot_name}.gbai/ (legacy)
    let ast_content = {
        let paths = [
            format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/start.ast"),
            format!("{bot_name}.gbai/{bot_name}.gbdialog/start.ast"),
        ];
        let mut found = String::new();
        for rel_path in &paths {
            let safe = verify_path_within_workdir(rel_path);
            info!("start.bas: trying path='{}' safe={} work_path='{}'", rel_path, safe, work_path);
            if !safe {
                continue;
            }
            let full_path = format!("{work_path}/{rel_path}");
            match tokio::fs::read_to_string(&full_path).await {
                Ok(c) if !c.is_empty() => { found = c; info!("start.bas: found at {}", full_path); break; }
                _ => {
                    let bas_path = full_path.replace(".ast", ".bas");
                    if let Ok(c) = tokio::fs::read_to_string(&bas_path).await {
                        if !c.is_empty() { found = c; info!("start.bas: found at {}", bas_path); break; }
                    }
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
    let _bot_name_owned = bot_name.to_string();
    let session_for_bas = botlib::models::UserSession {
        id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::json!({"channel": "web"}),
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
        Ok(result) => info!("start.bas: execution result (len={}): {}", result.to_string().len(), result),
        Err(e) => warn!("start.bas: execution error: {}", e),
    }

    let mut drained = 0usize;
    loop {
        match rx.try_recv() {
            Ok(response) => {
                drained += 1;
                info!("start.bas: drained BotResponse[{}]: mtype={}, session={}, content='{}'",
                    drained, i32::from(response.message_type), response.session_id,
                    response.content.chars().take(80).collect::<String>());
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = ws_sender.send(Message::Text(json)).await;
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(_) => break,
        }
    }
    info!("start.bas: drained {} responses immediately", drained);

    send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id).await;
    true
}
