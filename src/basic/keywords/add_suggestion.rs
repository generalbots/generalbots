//! ADD SUGGESTION Keyword
//!
//! Provides suggestions/quick replies in conversations.
//! Suggestions can:
//! - Point to KB contexts (existing behavior)
//! - Start tools with optional parameters
//! - When clicked, tools without params will prompt for params first
//!
//! Syntax:
//! - ADD SUGGESTION "context" AS "button text" - Points to KB context
//! - ADD SUGGESTION TOOL "tool_name" AS "button text" - Starts a tool
//! - ADD SUGGESTION TOOL "tool_name" WITH param1, param2 AS "button text" - Tool with params

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

/// Suggestion types
#[derive(Debug, Clone)]
pub enum SuggestionType {
    /// Points to a KB context - when clicked, selects that context
    Context(String),
    /// Starts a tool - when clicked, invokes the tool
    /// If tool has required params and none provided, will prompt user first
    Tool {
        name: String,
        params: Option<Vec<String>>,
    },
}

pub fn clear_suggestions_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();

    // Register with spaces: CLEAR SUGGESTIONS
    engine
        .register_custom_syntax(&["CLEAR", "SUGGESTIONS"], true, move |_context, _inputs| {
            if let Some(cache_client) = &cache {
                let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);
                let mut conn = match cache_client.get_connection() {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to connect to cache: {}", e);
                        return Ok(Dynamic::UNIT);
                    }
                };

                let result: Result<i64, redis::RedisError> =
                    redis::cmd("DEL").arg(&redis_key).query(&mut conn);

                match result {
                    Ok(deleted) => {
                        trace!(
                            "Cleared {} suggestions from session {}",
                            deleted,
                            user_session.id
                        );
                    }
                    Err(e) => error!("Failed to clear suggestions from Redis: {}", e),
                }
            } else {
                trace!("No cache configured, suggestions not cleared");
            }

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}

pub fn add_suggestion_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();
    let cache2 = state.cache.clone();
    let cache3 = state.cache.clone();
    let user_session2 = user_session.clone();
    let user_session3 = user_session.clone();

    // Register: ADD SUGGESTION "context" AS "text"
    // Points to KB context - when clicked, selects that context for queries
    engine
        .register_custom_syntax(
            &["ADD", "SUGGESTION", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_context_suggestion(cache.as_ref(), &user_session, &context_name, &button_text)?;

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();

    // Register: ADD SUGGESTION TOOL "tool_name" AS "text"
    // Starts a tool - if tool requires params, will prompt user first
    engine
        .register_custom_syntax(
            &["ADD", "SUGGESTION", "TOOL", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let tool_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_tool_suggestion(
                    cache2.as_ref(),
                    &user_session2,
                    &tool_name,
                    None,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();

    // Register: ADD SUGGESTION TOOL "tool_name" WITH params AS "text"
    // Starts a tool with pre-filled parameters
    engine
        .register_custom_syntax(
            &[
                "ADD",
                "SUGGESTION",
                "TOOL",
                "$expr$",
                "WITH",
                "$expr$",
                "AS",
                "$expr$",
            ],
            true,
            move |context, inputs| {
                let tool_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let params_value = context.eval_expression_tree(&inputs[1])?;
                let button_text = context.eval_expression_tree(&inputs[2])?.to_string();

                // Parse params - can be array or comma-separated string
                let params = if params_value.is_array() {
                    params_value
                        .cast::<rhai::Array>()
                        .iter()
                        .map(|v| v.to_string())
                        .collect()
                } else {
                    params_value
                        .to_string()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                };

                add_tool_suggestion(
                    cache3.as_ref(),
                    &user_session3,
                    &tool_name,
                    Some(params),
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}

/// Add a context-based suggestion (points to KB)
fn add_context_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    context_name: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

        // Suggestion JSON includes type for client to handle appropriately
        let suggestion = json!({
            "type": "context",
            "context": context_name,
            "text": button_text,
            "action": {
                "type": "select_context",
                "context": context_name
            }
        });

        let mut conn = match cache_client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to cache: {}", e);
                return Ok(());
            }
        };

        let result: Result<i64, redis::RedisError> = redis::cmd("RPUSH")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        match result {
            Ok(length) => {
                trace!(
                    "Added context suggestion '{}' to session {}, total: {}",
                    context_name,
                    user_session.id,
                    length
                );

                // Set context state
                let active_key = format!(
                    "active_context:{}:{}",
                    user_session.user_id, user_session.id
                );

                let _: Result<i64, redis::RedisError> = redis::cmd("HSET")
                    .arg(&active_key)
                    .arg(context_name)
                    .arg("inactive")
                    .query(&mut conn);
            }
            Err(e) => error!("Failed to add suggestion to Redis: {}", e),
        }
    } else {
        trace!("No cache configured, suggestion not added");
    }

    Ok(())
}

/// Add a tool-based suggestion
/// When clicked:
/// - If params provided, executes tool immediately with those params
/// - If no params and tool has required params, prompts user for them first
fn add_tool_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    tool_name: &str,
    params: Option<Vec<String>>,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

        // Suggestion JSON for tool invocation
        let suggestion = json!({
            "type": "tool",
            "tool": tool_name,
            "text": button_text,
            "action": {
                "type": "invoke_tool",
                "tool": tool_name,
                "params": params,
                // If params is None, client should check tool schema
                // and prompt for required params before invoking
                "prompt_for_params": params.is_none()
            }
        });

        let mut conn = match cache_client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to cache: {}", e);
                return Ok(());
            }
        };

        let result: Result<i64, redis::RedisError> = redis::cmd("RPUSH")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        match result {
            Ok(length) => {
                trace!(
                    "Added tool suggestion '{}' to session {}, total: {}, has_params: {}",
                    tool_name,
                    user_session.id,
                    length,
                    params.is_some()
                );
            }
            Err(e) => error!("Failed to add tool suggestion to Redis: {}", e),
        }
    } else {
        trace!("No cache configured, tool suggestion not added");
    }

    Ok(())
}
