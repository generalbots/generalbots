//! Script runner for a tool the model requested.
//!
//! Returns the tool's captured output: the caller feeds it back to the model
//! for a follow-up turn. Without it the chat ended on the raw tool output
//! (e.g. an `ls -la` listing, a JSON blob, …) instead of an answer.

use super::is_placeholder_value::is_placeholder_value;
use super::super::types::SessionIdentity;

pub(super) async fn run_script_tool(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    tool_name: &str,
    ast_content: &str,
    tool_args: &str,
    work_path: &str,
) -> String {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
    } = *identity;
    let mut tool_output = String::new();

    // Declarative gate: only execute tools the bot script associated
    // with this session via USE TOOL (e.g. inside IF role = "admin").
    if !crate::core::bot::tool_context::is_tool_associated_with_session(
        &state.conn, &session_id, tool_name,
    ) {
        log::info!(
            "LLM tool_call: tool '{tool_name}' not associated with session {session_id}, skipping"
        );
        return String::new();
    }
    let state_for_tool = state.clone();
    let work_path_for_mcp = work_path.to_string();
    let bot_name_for_mcp = bot_name.to_string();
    let tool_name_for_mcp = tool_name.to_string();
    let script_source = ast_content.to_string();

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
        id: session_id, user_id, branch_id: uuid::Uuid::nil(), bot_id: bot_uuid,
        title: String::new(),
        context_data,
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    locked: false, lock_pin_hash: None, };
    let tool_name_err = tool_name_for_mcp.clone();
    let tool_run = tokio::task::spawn_blocking(move || {
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
                        let clean_val = if is_placeholder_value(s) { String::new() } else { s.clone() };
                        let _ = svc.set_variable(key, &clean_val);
                    }
                    serde_json::Value::Number(n) => {
                        let _ = svc.set_variable(key, &n.to_string());
                    }
                    serde_json::Value::Bool(b) => {
                        let _ = svc.set_variable(key, if *b { "true" } else { "false" });
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
        svc.run(&script_source).map_err(|e| e.to_string())
    }).await;

    // A failed tool used to reach only the log, so the user watched the
    // turn stop with no answer at all (observed on the web tool
    // button: "Variable not found: path"). The detail stays in the
    // log; the user is told which action failed.
    if let Ok(Err(detail)) = tool_run {
        log::warn!("Tool '{tool_name_err}' execution error: {detail}");
        let message = format!(
            "Não consegui executar '{tool_name_err}'. Tente novamente em instantes."
        );
        let resp = botlib::models::BotResponse::new(
            bot_uuid.to_string(),
            session_id.to_string(),
            user_id.to_string(),
            &message,
            sink.channel_type(),
        );
        if tool_output.is_empty() {
            tool_output = message.clone();
        }
        let _ = sink.send_bot_response(&resp).await;
    }

    // Drain rx to forward tool responses to the sink
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        match rx.try_recv() {
            Ok(response) => {
                if !response.content.trim().is_empty() {
                    if !tool_output.is_empty() {
                        tool_output.push('\n');
                    }
                    tool_output.push_str(&response.content);
                }
                let _ = sink.send_bot_response(&response).await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => continue,
            Err(_) => break,
        }
    }
    tool_output
}
