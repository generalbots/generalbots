use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;

pub async fn run_tool_exec(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    tool_name: &str,
    channel: &str,
) {
    let work_path = botcore::shared::utils::get_work_path();
    let read_tool = |gbdialog_dir: &str| -> Option<String> {
        let ast_p = format!("{gbdialog_dir}/{tool_name}.ast");
        let bas_p = ast_p.replace(".ast", ".bas");
        std::fs::read_to_string(&ast_p).ok()
            .or_else(|| std::fs::read_to_string(&bas_p).ok())
    };

    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/");
    let tool_content = if crate::core::bot::ws::handler::verify_path_within_workdir(&primary_rel) {
        let gbdialog_dir = format!("{work_path}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/");
        read_tool(&gbdialog_dir)
    } else {
        None
    };

    let ast_content = tool_content.unwrap_or_default();

    if !ast_content.is_empty() {
        let state_for_tool = state.clone();
        let tool_name_clone = tool_name.to_string();
        let channel_name = channel.to_string();
        let session_for_tool = botlib::models::UserSession {
            id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_uuid,
            title: String::new(),
            context_data: serde_json::json!({"channel": channel_name}),
            current_tool: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        tokio::task::spawn_blocking(move || {
            let mut svc = crate::basic::ScriptService::new(
                state_for_tool.clone(), session_for_tool,
            );
            svc.load_bot_config_params(&state_for_tool, bot_uuid);
            if let Err(e) = svc.run(&ast_content) {
                log::warn!("Tool '{tool_name_clone}' execution error: {e}");
            }
        });
    }
}

pub fn is_generic_greeting(text: &str) -> bool {
    let trimmed = text.trim().to_lowercase();
    let greetings = [
        "ola", "oi", "hey", "hello", "hi", "bom dia", "boa tarde", "boa noite",
        "olá", "oie", "bem-vindo", "e ai", "e aí", "tudo bem", "td bem",
    ];
    if greetings.contains(&trimmed.as_str()) {
        return true;
    }
    let word_count = trimmed.split_whitespace().count();
    word_count <= 2
}

pub async fn run_llm_tool_call(
    sink: &dyn ChannelSink,
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    full_response: &str,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    user_text: &str,
) {
    use crate::core::bot::ws::handler::validate_bot_name;
    use crate::core::bot::ws::handler::verify_path_within_workdir;
    use botcore::shared::utils::get_work_path;

    let tool_call_trigger = "\"__tool_call__\":";
    let tc_start = match full_response.find(tool_call_trigger) {
        Some(pos) => full_response[..pos].rfind('{').unwrap_or(pos),
        None => return,
    };
    let tc_json = &full_response[tc_start..];
    if let Ok(tool_call) = serde_json::from_str::<serde_json::Value>(tc_json) {
        let raw_tool_name = tool_call.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tool_args = tool_call.get("arguments").and_then(|v| v.as_str()).unwrap_or("");
        log::info!("LLM tool_call: executing tool '{raw_tool_name}' with args: {tool_args}");
        if raw_tool_name.is_empty() { return; }

        if is_generic_greeting(user_text) {
            log::info!("Blocking tool '{raw_tool_name}' on short generic user message: '{user_text}'");
            return;
        }

        let tool_name = match validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("LLM tool_call: invalid tool name '{raw_tool_name}': {e}");
                return;
            }
        };

        let work_path = get_work_path();
        let rel_tool_path = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
        if !verify_path_within_workdir(&rel_tool_path) {
            log::error!("Path traversal detected in LLM tool_call for tool: {tool_name}");
            return;
        }

        let ast_path = format!("{work_path}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
        let ast_content = match tokio::fs::read_to_string(&ast_path).await {
            Ok(c) if !c.is_empty() => c,
            _ => {
                let bas_path = ast_path.replace(".ast", ".bas");
                tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
            }
        };

        if !ast_content.is_empty() {
            let state_for_tool = state.clone();
            let tool_name_cl = tool_name.clone();
            let work_path_for_mcp = work_path.clone();
            let bot_name_for_mcp = bot_name.to_string();
            let tool_name_for_mcp = tool_name.clone();

            let parsed_args: serde_json::Value = serde_json::from_str(tool_args).unwrap_or(serde_json::Value::Null);
            let injected_args = parsed_args.clone();
            let channel_name = sink.channel_type();
            let mut context_data = if parsed_args.is_object() {
                parsed_args
            } else {
                serde_json::json!({})
            };
            if let Some(obj) = context_data.as_object_mut() {
                obj.insert("channel".to_string(), serde_json::Value::String(channel_name.to_string()));
            }

            let session_for_tool = botlib::models::UserSession {
                id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_uuid,
                title: String::new(),
                context_data,
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let _ = tokio::task::spawn_blocking(move || {
                let mut svc = crate::basic::ScriptService::new(
                    state_for_tool.clone(), session_for_tool,
                );
                svc.load_bot_config_params(&state_for_tool, bot_uuid);
                let mut injected_param_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                if let Some(obj) = injected_args.as_object() {
                    for (key, value) in obj {
                        injected_param_names.insert(key.clone());
                        match value {
                            serde_json::Value::String(s) => {
                                let _ = svc.set_variable(&key, s);
                            }
                            serde_json::Value::Number(n) => {
                                let _ = svc.set_variable(&key, &n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                let _ = svc.set_variable(&key, if *b { "true" } else { "false" });
                            }
                            _ => {}
                        }
                    }
                }
                let mcp_path = format!("{bot_name_for_mcp}.gborg/{bot_name_for_mcp}.gbai/{bot_name_for_mcp}.gbdialog/{tool_name_for_mcp}.mcp.json");
                let mcp_full = std::path::Path::new(&work_path_for_mcp).join(&mcp_path);
                if mcp_full.exists() {
                    if let Ok(mcp_content) = std::fs::read_to_string(&mcp_full) {
                        if let Ok(mcp_val) = serde_json::from_str::<serde_json::Value>(&mcp_content) {
                            if let Some(props) = mcp_val.get("input_schema").and_then(|s| s.get("properties")).and_then(|p| p.as_object()) {
                                for (param_name, _) in props {
                                    let clean_name = param_name.trim_end_matches(":string").to_string();
                                    if !injected_param_names.contains(&clean_name) {
                                        let _ = svc.set_variable(&clean_name, "");
                                    }
                                }
                            }
                        }
                    }
                }
                if let Err(e) = svc.run(&ast_content) {
                    log::warn!("Tool '{tool_name_cl}' execution error: {e}");
                }
            }).await;

            // Drain rx to forward tool responses to the sink
            for _ in 0..50 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                match rx.try_recv() {
                    Ok(response) => {
                        let _ = sink.send_bot_response(&response).await;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}