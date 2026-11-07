use axum::extract::ws::Message;
use futures_util::SinkExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{LlmTurn, WsSink};

/// Executes a `__tool_call__` emitted by the model for this web session.
/// Returns `true` when the turn must stop before persisting its response (the
/// tool name or its path failed validation), matching the original early
/// returns.
pub(super) async fn maybe_execute(
    ws_sender: &mut WsSink,
    rx: &mut mpsc::Receiver<botlib::models::BotResponse>,
    turn: &LlmTurn<'_>,
    full_response: &str,
) -> bool {
    let LlmTurn {
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *turn;
    let tool_call_trigger = "\"__tool_call__\":";
    if !full_response.contains(tool_call_trigger) {
        return false;
    }
    let Ok(tool_call) = serde_json::from_str::<serde_json::Value>(full_response) else {
        return false;
    };
    let raw_tool_name = tool_call
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tool_args = tool_call
        .get("arguments")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    log::info!(
        "LLM tool_call: executing tool '{}' with args: {}",
        raw_tool_name,
        tool_args
    );
    if raw_tool_name.is_empty() {
        return false;
    }
    let tool_name = match crate::core::bot::ws::handler::validate_bot_name(&raw_tool_name) {
        Ok(n) => n,
        Err(e) => {
            log::warn!(
                "LLM tool_call: invalid tool name '{}': {}",
                raw_tool_name,
                e
            );
            return true;
        }
    };

    let work_path = botcore::shared::utils::get_work_path();
    let rel_tool_path =
        format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
    if !crate::core::bot::ws::handler::verify_path_within_workdir(&rel_tool_path) {
        log::error!(
            "Path traversal detected in LLM tool_call for tool: {}",
            tool_name
        );
        return true;
    }

    let ast_path =
        format!("{work_path}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
    let ast_content = match tokio::fs::read_to_string(&ast_path).await {
        Ok(c) if !c.is_empty() => c,
        _ => {
            let bas_path = ast_path.replace(".ast", ".bas");
            tokio::fs::read_to_string(&bas_path)
                .await
                .unwrap_or_default()
        }
    };

    if !ast_content.is_empty() {
        let state_for_tool = state.clone();
        let tool_name_cl = tool_name.clone();
        let work_path_for_mcp = work_path.clone();
        let bot_name_for_mcp = bot_name.to_string();
        let tool_name_for_mcp = tool_name.clone();

        let parsed_args: serde_json::Value =
            serde_json::from_str(tool_args).unwrap_or(serde_json::Value::Null);
        let injected_args = parsed_args.clone();
        let context_data = if parsed_args.is_object() {
            parsed_args
        } else {
            serde_json::Value::Null
        };
        log::info!("Tool '{}' context_data: {:?}", tool_name, context_data);

        let session_for_tool = botlib::models::UserSession {
            id: session_id,
            user_id,
            branch_id: Uuid::nil(),
            bot_id: bot_uuid,
            title: String::new(),
            context_data,
            current_tool: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            locked: false,
            lock_pin_hash: None,
        };
        let _ = tokio::task::spawn_blocking(move || {
            let mut svc =
                crate::basic::ScriptService::new(state_for_tool.clone(), session_for_tool);
            svc.load_bot_config_params(&state_for_tool, bot_uuid);
            let mut injected_param_names: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            if let Some(obj) = injected_args.as_object() {
                for (key, value) in obj {
                    let rhai_key = key.to_lowercase();
                    injected_param_names.insert(rhai_key.clone());
                    match value {
                        serde_json::Value::String(s) => {
                            let _ = svc.set_variable(&rhai_key, s);
                        }
                        serde_json::Value::Number(n) => {
                            let _ = svc.set_variable(&rhai_key, &n.to_string());
                        }
                        serde_json::Value::Bool(b) => {
                            let _ = svc.set_variable(
                                &rhai_key,
                                if *b { "true" } else { "false" },
                            );
                        }
                        _ => {}
                    }
                }
            }
            let mcp_path = format!(
                "{bot_name_for_mcp}.gborg/{bot_name_for_mcp}.gbai/{bot_name_for_mcp}.gbdialog/{tool_name_for_mcp}.mcp.json"
            );
            let mcp_full = std::path::Path::new(&work_path_for_mcp).join(&mcp_path);
            if mcp_full.exists() {
                if let Ok(mcp_content) = std::fs::read_to_string(&mcp_full) {
                    if let Ok(mcp_val) =
                        serde_json::from_str::<serde_json::Value>(&mcp_content)
                    {
                        if let Some(props) = mcp_val
                            .get("input_schema")
                            .and_then(|s| s.get("properties"))
                            .and_then(|p| p.as_object())
                        {
                            for (param_name, _) in props {
                                let clean_name =
                                    param_name.trim_end_matches(":string").to_lowercase();
                                if !injected_param_names.contains(&clean_name) {
                                    let _ = svc.set_variable(&clean_name, "");
                                }
                            }
                        }
                    }
                }
            }
            if let Err(e) = svc.run(&ast_content) {
                log::warn!("Tool '{}' execution error: {}", tool_name_cl, e);
            }
        })
        .await;

        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            match rx.try_recv() {
                Ok(response) => {
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = ws_sender.send(Message::Text(json)).await;
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => continue,
                Err(_) => break,
            }
        }
    }
    false
}
