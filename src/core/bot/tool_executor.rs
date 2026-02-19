/// Generic tool executor for LLM tool calls
/// Works across all LLM providers (GLM, OpenAI, Claude, etc.)
use log::{error, info, warn};
use serde_json::Value;
// use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
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
        let log_path = Path::new("work").join(format!("{}_tool_errors.log", bot_name));

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
        error!("[TOOL_ERROR] Bot: {}, Tool: {}, Error: {}", bot_name, tool_name, error_msg);
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

        // Load the .bas tool file
        let bas_path = Self::get_tool_bas_path(bot_name, &tool_call.tool_name);

        if !bas_path.exists() {
            let error_msg = format!("Tool file not found: {:?}", bas_path);
            Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
            return ToolExecutionResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                result: String::new(),
                error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
            };
        }

        // Read the .bas file
        let bas_script = match tokio::fs::read_to_string(&bas_path).await {
            Ok(script) => script,
            Err(e) => {
                let error_msg = format!("Failed to read tool file: {}", e);
                Self::log_tool_error(bot_name, &tool_call.tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id: tool_call.id.clone(),
                    success: false,
                    result: String::new(),
                    error: Some(Self::format_user_friendly_error(&tool_call.tool_name, &error_msg)),
                };
            }
        };

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
                &bas_script,
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
        bas_script: &str,
        tool_name: &str,
        arguments: &Value,
    ) -> ToolExecutionResult {
        let tool_call_id = format!("tool_{}", uuid::Uuid::new_v4());

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
                    warn!("[TOOL_EXEC] Failed to set variable '{}': {}", key, e);
                }
            }
        }

        // Compile tool script (filters PARAM/DESCRIPTION lines and converts BASIC to Rhai)
        let ast = match script_service.compile_tool_script(&bas_script) {
            Ok(ast) => ast,
            Err(e) => {
                let error_msg = format!("Compilation error: {}", e);
                Self::log_tool_error(bot_name, tool_name, &error_msg);
                let user_message = Self::format_user_friendly_error(tool_name, &error_msg);
                return ToolExecutionResult {
                    tool_call_id,
                    success: false,
                    result: String::new(),
                    error: Some(user_message),
                };
            }
        };

        // Run the script
        match script_service.run(&ast) {
            Ok(result) => {
                info!("[TOOL_EXEC] Tool '{}' executed successfully", tool_name);

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

    /// Get the path to a tool's .bas file
    fn get_tool_bas_path(bot_name: &str, tool_name: &str) -> std::path::PathBuf {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

        // Try data directory first
        let data_path = Path::new(&home_dir)
            .join("data")
            .join(format!("{}.gbai", bot_name))
            .join(format!("{}.gbdialog", bot_name))
            .join(format!("{}.bas", tool_name));

        if data_path.exists() {
            return data_path;
        }

        // Try work directory (for development/testing)
        let work_path = Path::new(&home_dir)
            .join("gb")
            .join("work")
            .join(format!("{}.gbai", bot_name))
            .join(format!("{}.gbdialog", bot_name))
            .join(format!("{}.bas", tool_name));

        work_path
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
