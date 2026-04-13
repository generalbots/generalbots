/// Generic tool executor for LLM tool calls
/// Works across all LLM providers (GLM, OpenAI, Claude, etc.)
use log::{error, info};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

use crate::basic::ScriptService;
use crate::core::shared::state::AppState;
use crate::core::shared::models::schema::bots;
use diesel::prelude::*;

/// Represents a parsed tool call from an LLM
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: Value,
}

/// Result of tool execution
#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub tool_call_id: String,
    pub success: bool,
    pub result: String,
    pub error: Option<String>,
}

/// Generic tool executor - works with any LLM provider
pub struct ToolExecutor;

impl ToolExecutor {
    /// Log tool execution errors to a dedicated log file
    fn log_tool_error(bot_name: &str, tool_name: &str, error_msg: &str) {
        let log_path = std::path::PathBuf::from(crate::core::shared::utils::get_work_path())
            .join(format!("{}_tool_errors.log", bot_name));

        // Create work directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!(
            "[{}] TOOL: {} | ERROR: {}\n",
            timestamp, tool_name, error_msg
        );

        // Append to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        // Also log to system logger
        error!("Tool error in {} (bot: {}): {}", tool_name, bot_name, error_msg);
    }

    /// Convert internal errors to user-friendly messages for browser
    fn format_user_friendly_error(tool_name: &str, error: &str) -> String {
        // Don't expose internal errors to browser - log them and return generic message
        if error.contains("Compilation error") {
            "Desculpe, houve um erro ao processar sua solicitação. Por favor, tente novamente ou entre em contato com a administração."
                .to_string()
        } else if error.contains("Execution error") {
            format!("O processamento da ferramenta '{}' encontrou um problema. Nossa equipe foi notificada.", tool_name)
        } else if error.contains("not found") {
            "Ferramenta não disponível no momento. Por favor, tente novamente mais tarde.".to_string()
        } else {
            "Ocorreu um erro ao processar sua solicitação. Por favor, tente novamente.".to_string()
        }
    }
    /// Parse a tool call JSON from any LLM provider
    /// Handles OpenAI, GLM, Claude formats
    /// Handles both single objects and arrays of tool calls
    pub fn parse_tool_call(chunk: &str) -> Option<ParsedToolCall> {
        // Try to parse as JSON
        let json: Value = serde_json::from_str(chunk).ok()?;

        // Handle array of tool calls (common OpenAI format)
        if let Some(arr) = json.as_array() {
            if let Some(first_tool) = arr.first() {
                return Self::extract_tool_call(first_tool);
            }
        }

        // Check if this is a tool_call type (from GLM wrapper)
        if let Some(tool_type) = json.get("type").and_then(|t| t.as_str()) {
            if tool_type == "tool_call" {
                if let Some(content) = json.get("content") {
                    return Self::extract_tool_call(content);
                }
            }
        }

        // Try direct OpenAI format
        if json.get("function").is_some() {
            return Self::extract_tool_call(&json);
        }

        None
    }

    /// Extract tool call information from various formats
    fn extract_tool_call(tool_data: &Value) -> Option<ParsedToolCall> {
        let function = tool_data.get("function")?;
        let tool_name = function.get("name")?.as_str()?.to_string();
        let arguments_str = function.get("arguments")?.as_str()?;

        // Parse arguments string to JSON
        let arguments: Value = serde_json::from_str(arguments_str).ok()?;

        let id = tool_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Some(ParsedToolCall {
            id,
            tool_name,
            arguments,
        })
    }

    /// Execute a tool call by running the corresponding .bas file
    pub async fn execute_tool_call(
        state: &Arc<AppState>,
        bot_name: &str,
        tool_call: &ParsedToolCall,
        session_id: &Uuid,
        _user_id: &Uuid,
    ) -> ToolExecutionResult {
        info!(
            "[TOOL_EXEC] Executing tool '{}' for bot '{}', session '{}'",
            tool_call.tool_name, bot_name, session_id
        );

        // Get bot_id
        let bot_id = match Self::get_bot_id(state, bot_name) {
            Some(id) => id,
            None => {
                let error_msg = format!("Bot '{}' not found", bot_name);
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                };
            }
        };

            // Load the pre-compiled .ast file (compilation happens only in Drive Monitor)
            let ast_path = Self::get_tool_ast_path(bot_name, &tool_call.tool_name);

            let ast_content = match tokio::fs::read_to_string(&ast_path).await {
                Ok(content) => content,
                Err(e) => {
                    let error_msg = format!("Failed to read tool .ast file {}: {}", ast_path.display(), e);
                    Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                    return ToolExecutionResult {
                        tool_call_id: tool_call.id.clone(),
                        success: false,
                        result: String::new(),
                        error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                    };
                }
            };

            if ast_content.is_empty() {
                let error_msg = "Tool .ast file is empty".to_string();
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                };
            }

        // Get session for ScriptService
        let session = match state.session_manager.lock().await.get_session_by_id(*session_id) {
            Ok(Some(sess)) => sess,
            Ok(None) => {
                let error_msg = "Session not found".to_string();
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                };
            }
            Err(e) => {
                let error_msg = format!("Failed to get session: {}", e);
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                };
            }
        };

            // Execute in blocking thread for ScriptService (which is not async)
            let bot_name_clone = bot_name.to_string();
            let tool_name_clone = tool_call.tool_name.clone();
            let tool_call_id_clone = tool_call.id.clone();
            let arguments_clone = tool_call.arguments.clone();
            let state_clone = state.clone();
            let bot_id_clone = bot_id;

            let execution_result = tokio::task::spawn_blocking(move || {
                Self::execute_tool_script(
                    &state_clone,
                    &bot_name_clone,
                    bot_id_clone,
                    &session,
                    &ast_content,
                    &tool_name_clone,
                    &arguments_clone,
                )
            })
            .await;

        match execution_result {
            Ok(result) => result,
            Err(e) => {
                let error_msg = format!("Task execution error: {}", e);
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                ToolExecutionResult {
                    tool_call_id: tool_call_id_clone,
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                }
            }
        }
    }

    /// Execute the tool script with parameters
    fn execute_tool_script(
        state: &Arc<AppState>,
        bot_name: &str,
        bot_id: Uuid,
        session: &crate::core::shared::models::UserSession,
        ast_content: &str,
        tool_name: &str,
        arguments: &Value,
    ) -> ToolExecutionResult {
        let tool_call_id = format!("tool_{}", uuid::Uuid::new_v4());
        log::info!("[BASIC_EXEC] Tool '{}' starting execution (bot={}, session={})", tool_name, bot_name, session.id);

        // Create ScriptService
        let mut script_service = ScriptService::new(state.clone(), session.clone());
        script_service.load_bot_config_params(state, bot_id);

        // Set tool parameters as variables in the engine scope
        // Note: DATE parameters are now sent by LLM in ISO 8601 format (YYYY-MM-DD)
        // The tool schema with format="date" tells the LLM to use this agnostic format
        if let Some(obj) = arguments.as_object() {
            for (key, value) in obj {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };

                // Set variable in script scope
                if let Err(e) = script_service.set_variable(key, &value_str) {
                    log::warn!("[BASIC_EXEC] Failed to set variable '{}': {}", key, e);
                }
            }
        }

        log::trace!("[BASIC_EXEC] Tool '{}' running .ast ({} chars)", tool_name, ast_content.len());

        // Run the pre-compiled .ast content (compilation happens only in Drive Monitor)
        match script_service.run(ast_content) {
            Ok(result) => {
                log::info!("[BASIC_EXEC] Tool '{}' completed successfully", tool_name);

                // Convert result to string
                let result_str = result.to_string();

                ToolExecutionResult {
                    tool_call_id,
                    success: true,
                    result: result_str,
                    error: None,
                }
            }
            Err(e) => {
                log::error!("[BASIC_EXEC] Tool '{}' execution error: {}", tool_name, e);
                let error_msg = format!("Execution error: {}", e);
                Self::log_tool_error(bot_name, tool_name, &error_msg);
                let user_message = Self::format_user_friendly_error(tool_name, &error_msg);
                ToolExecutionResult {
                    tool_call_id,
                    success: false,
                    result: String::new(),
                    error: Some(user_message),
                }
            }
        }
    }

    /// Get the bot_id from bot_name
    fn get_bot_id(state: &Arc<AppState>, bot_name: &str) -> Option<Uuid> {
        let mut conn = state.conn.get().ok()?;
        bots::table
            .filter(bots::name.eq(bot_name))
            .select(bots::id)
            .first(&mut *conn)
            .ok()
    }

    /// Get the path to a tool's pre-compiled .ast file
    fn get_tool_ast_path(bot_name: &str, tool_name: &str) -> std::path::PathBuf {
        let work_path = std::path::PathBuf::from(crate::core::shared::utils::get_work_path())
            .join(format!("{}.gbai", bot_name))
            .join(format!("{}.gbdialog", bot_name))
            .join(format!("{}.ast", tool_name));

        work_path
    }

    /// Execute a tool directly by name (without going through LLM)
    pub async fn execute_tool_by_name(
        state: &Arc<AppState>,
        bot_name: &str,
        tool_name: &str,
        session_id: &Uuid,
        user_id: &Uuid,
    ) -> ToolExecutionResult {
        let tool_call_id = format!("direct_{}", Uuid::new_v4());

        info!(
            "[TOOL_EXEC] Direct tool invocation: '{}' for bot '{}', session '{}'",
            tool_name, bot_name, session_id
        );

        // Ensure websocket_session_id and channel are set on the session so TALK can route correctly
        {
            let mut sm = state.session_manager.lock().await;
            if let Ok(Some(sess)) = sm.get_session_by_id(*session_id) {
                let needs_update = if let serde_json::Value::Object(ref map) = sess.context_data {
                    !map.contains_key("websocket_session_id") || !map.contains_key("channel")
                } else {
                    true
                };
                if needs_update {
                    let mut updated = sess.clone();
                    if let serde_json::Value::Object(ref mut map) = updated.context_data {
                        if !map.contains_key("websocket_session_id") {
                            map.insert(
                                "websocket_session_id".to_string(),
                                serde_json::Value::String(session_id.to_string()),
                            );
                        }
                        if !map.contains_key("channel") {
                            map.insert(
                                "channel".to_string(),
                                serde_json::Value::String("web".to_string()),
                            );
                        }
                    } else {
                        let mut map = serde_json::Map::new();
                        map.insert("websocket_session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                        map.insert("channel".to_string(), serde_json::Value::String("web".to_string()));
                        updated.context_data = serde_json::Value::Object(map);
                    }
                    let context_json = serde_json::to_string(&updated.context_data).unwrap_or_default();
                    let _ = sm.update_session_context(session_id, user_id, context_json);
                }
            }
        }

        let tool_call = ParsedToolCall {
            id: tool_call_id.clone(),
            tool_name: tool_name.to_string(),
            arguments: Value::Object(serde_json::Map::new()),
        };

        Self::execute_tool_call(state, bot_name, &tool_call, session_id, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_call_glm_format() {
        let chunk = r#"{"type":"tool_call","content":{"id":"call_123","type":"function","function":{"name":"test_tool","arguments":"{\"param1\":\"value1\"}"}}}"#;

        let result = ToolExecutor::parse_tool_call(chunk);
        assert!(result.is_some());

        let tool_call = result.unwrap();
        assert_eq!(tool_call.tool_name, "test_tool");
        assert_eq!(tool_call.arguments["param1"], "value1");
    }

    #[test]
    fn test_parse_tool_call_openai_format() {
        let chunk = r#"{"id":"call_123","type":"function","function":{"name":"test_tool","arguments":"{\"param1\":\"value1\"}"}}"#;

        let result = ToolExecutor::parse_tool_call(chunk);
        assert!(result.is_some());

        let tool_call = result.unwrap();
        assert_eq!(tool_call.tool_name, "test_tool");
        assert_eq!(tool_call.arguments["param1"], "value1");
    }
}
