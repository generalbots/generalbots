use std::sync::Arc;
use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::{debug, info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::bot::ws::handler::verify_path_within_workdir;
use botcore::shared::utils::current_org_id;

pub fn load_system_prompt(bot_name: &str) -> String {
    load_system_prompt_for_channel(bot_name, "web")
}

pub fn load_system_prompt_for_channel(bot_name: &str, channel: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let org_id = current_org_id();
    let nil_uuid = uuid::Uuid::nil();

    // Helper to read a prompt by channel suffix, then fall back to base PROMPT.md
    let read_prompt = |gbot_dir: &str| -> Option<String> {
        let channel_file = format!("PROMPT-{}.md", channel.to_uppercase());
        let channel_file_lower = format!("PROMPT-{}.md", channel.to_lowercase());
        let base = |f: &str| -> Option<String> {
            std::fs::read_to_string(f).ok()
        };
        let channel_prompt = base(&format!("{}{}", gbot_dir, channel_file))
            .or_else(|| base(&format!("{}{}", gbot_dir, channel_file_lower)));
        let base_prompt = base(&format!("{}PROMPT.md", gbot_dir))
            .or_else(|| base(&format!("{}prompt.md", gbot_dir)))
            .or_else(|| base(&format!("{}PROMPT.txt", gbot_dir)))
            .or_else(|| base(&format!("{}prompt.txt", gbot_dir)));
        match (channel_prompt, base_prompt) {
            (Some(c), Some(b)) => Some(format!("{}\n\n{}", c, b)),
            (Some(c), None) => Some(c),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    };

    // Primary path: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
    if verify_path_within_workdir(&primary_rel) {
        let gbot_dir = format!("{work_dir}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    // Fallback: {org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let fallback_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
    if org_id != nil_uuid && verify_path_within_workdir(&fallback_rel) {
        let gbot_dir = format!("{work_dir}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/", org_id = org_id);
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    // The materialized org directory is `<org-id>.gborg`, which on a desktop
    // install is the all-zero id: neither path above matches, so the bot used
    // to answer with the generic fallback and the agent never received its
    // instructions. Resolve the prompt through the shared work-tree resolver.
    let channel_file = format!("PROMPT-{}.md", channel.to_uppercase());
    let candidates = [
        channel_file.clone(),
        format!("PROMPT-{}.md", channel.to_lowercase()),
        "PROMPT.md".to_string(),
        "prompt.md".to_string(),
        "PROMPT.txt".to_string(),
        "prompt.txt".to_string(),
    ];
    let channel_prompt = crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(
        &work_dir,
        bot_name,
        "gbot",
        &channel_file,
    )
    .and_then(|path| std::fs::read_to_string(path).ok());
    if let Some(p) = channel_prompt {
        let base = candidates
            .iter()
            .skip(2)
            .find_map(|name| {
                crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(
                    &work_dir,
                    bot_name,
                    "gbot",
                    name,
                )
                .and_then(|path| std::fs::read_to_string(path).ok())
            });
        return match base {
            Some(b) => format!("{p}\n\n{b}"),
            None => p,
        };
    }
    let base_prompt = candidates.iter().skip(2).find_map(|name| {
        crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(&work_dir, bot_name, "gbot", name)
            .and_then(|path| std::fs::read_to_string(path).ok())
    });
    if let Some(p) = base_prompt {
        return p;
    }

    let now = chrono::Utc::now().format("%B %d, %Y").to_string();
    format!("Today is {now}.\n\nYou are a helpful assistant. Be concise. Respond in plain text.")
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
                        // A bot without a custom stylesheet is the normal case, so an
                        // absent file is not a warning; only a real IO failure
                        // (permissions, broken symlink) is worth reporting.
                        if e2.kind() == std::io::ErrorKind::NotFound {
                            debug!(
                                "No styles.css/style.css for this bot at {gbot_dir}; using platform styles"
                            );
                        } else {
                            warn!(
                                "Failed to read styles.css at {} or {}: {}, {}",
                                css_path, alt_path, e1, e2
                            );
                        }
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

/// Replays the static `ADD_SUGGESTION "…"` lines of a bot's start.bas.
/// Redis is the live store, but a restart flushes it; a reopened
/// conversation then had no chips at all because start.bas is skipped for
/// sessions that already have history.
fn suggestions_from_start_bas(source: &str) -> Vec<botlib::models::Suggestion> {
    let mut suggestions = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('\'') {
            continue;
        }
        // Shipped templates use both spellings ("ADD SUGGESTION" in 167 lines,
        // "ADD_SUGGESTION" in 6); matching only the underscore form left a
        // reopened conversation with no chips after Redis was flushed.
        let upper = trimmed.to_ascii_uppercase();
        if !(upper.starts_with("ADD_SUGGESTION") || upper.starts_with("ADD SUGGESTION")) {
            continue;
        }
        let Some(open) = trimmed.find('"') else { continue };
        let rest = &trimmed[open + 1..];
        let Some(close) = rest.find('"') else { continue };
        // `ADD SUGGESTION "scan" AS "Scan a QR Code"` labels the chip with the
        // AS string; the first quoted value is only the command.
        let after = &rest[close + 1..];
        let after_upper = after.to_ascii_uppercase();
        let label = after_upper.find("AS \"").and_then(|position| {
            let label_rest = &after[position + 4..];
            let end = label_rest.find('"')?;
            Some(label_rest[..end].trim().to_string())
        });
        let text = label
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| rest[..close].trim().to_string());
        if !text.is_empty() && suggestions.len() < 12 {
            suggestions.push(botlib::models::Suggestion::new(text));
        }
    }
    suggestions
}

async fn fallback_suggestions(bot_name: &str) -> Vec<botlib::models::Suggestion> {
    let work_path = botcore::shared::utils::get_work_path();
    let rel = format!("{bot_name}.gbdialog/start.bas");
    match botcore::shared::utils::read_bot_file_across_layouts(&work_path, bot_name, &rel).await {
        Some(source) => suggestions_from_start_bas(&source),
        None => {
            log::debug!("start.bas suggestions unavailable for {bot_name}");
            Vec::new()
        }
    }
}

pub async fn send_start_suggestions(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
) -> usize {
    // #1369 — both ids are needed as strings several times below; format them
    // once per call instead of once per use. This runs on every session
    // connect, reconnect and after start.bas.
    let bot_id = bot_uuid.to_string();
    let session = session_id.to_string();

    let mut suggestions = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::add_suggestion::get_suggestions(
                state.cache.as_ref(),
                &bot_id,
                &session,
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    if suggestions.is_empty() {
        suggestions = fallback_suggestions(bot_name).await;
    }
    let switchers = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::switcher::get_switchers(
                state.cache.as_ref(),
                &bot_id,
                &session,
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    let sent = suggestions.len();
    if !suggestions.is_empty() || !switchers.is_empty() {
        info!("ws_handler: sending {} suggestions and {} switchers on reconnect", suggestions.len(), switchers.len());
        let _ = ws_sender.send(Message::Text(serde_json::json!({
            "bot_id": bot_id,
            "user_id": user_id.to_string(),
            "session_id": session,
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
    sent
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
            info!("start.bas skipped: session {session_id} already has messages");
            send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id, bot_name).await;
            return false;
        }
    }

    let session_init_key = botlib::key_utils::build_key("", &["start_bas_executed", &bot_uuid.to_string(), &session_id.to_string()]);
    if let Some(ref cache) = state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("DEL").arg(&session_init_key).query_async(&mut conn).await;
        }
    }

    let work_path = botcore::shared::utils::get_work_path();

    // Cross-layout lookup: bot-named org bucket, standalone bucket, or org workspace.
    let ast_content = {
        let mut found = String::new();
        let rel = format!("{bot_name}.gbdialog/start.ast");
        match botcore::shared::utils::read_bot_file_across_layouts(&work_path, bot_name, &rel).await {
            Some(c) => {
                info!("start.bas: found start.ast for {bot_name} ({} bytes)", c.len());
                found = c;
            }
            None => {
                let bas_rel = format!("{bot_name}.gbdialog/start.bas");
                if let Some(c) = botcore::shared::utils::read_bot_file_across_layouts(&work_path, bot_name, &bas_rel).await {
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
    let session_for_bas = botlib::models::UserSession {
        id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::json!({"channel": "web"}),
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        locked: false,
        lock_pin_hash: None,
    };
    // #1369 — both values are consumed by the call and unused afterwards, so
    // they are moved instead of cloned (`UserSession` owns several Strings).
    let exec_result = crate::basic::ScriptService::execute_script(
        state_for_bas,
        session_for_bas,
        &ast_content,
    ).await;
    match exec_result {
        Ok(result) => info!("start.bas: execution result (len={}): {}", result.len(), result),
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

    send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id, bot_name).await;
    true
}
