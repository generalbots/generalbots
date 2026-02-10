use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Context(String),

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

    engine
        .register_custom_syntax(["CLEAR", "SUGGESTIONS"], true, move |_context, _inputs| {
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
        .expect("valid syntax registration");
}

pub fn add_suggestion_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();
    let cache2 = state.cache.clone();
    let cache3 = state.cache.clone();
    let cache4 = state.cache.clone();
    let cache5 = state.cache.clone();
    let user_session2 = user_session.clone();
    let user_session3 = user_session.clone();
    let user_session4 = user_session.clone();
    let user_session5 = user_session.clone();

    // ADD SUGGESTION "context_name" AS "button text"
    engine
        .register_custom_syntax(
            ["ADD", "SUGGESTION", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_context_suggestion(cache.as_ref(), &user_session, &context_name, &button_text)?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // ADD SUGGESTION TEXT "$expr$" AS "button text"
    // Creates a suggestion that sends the text as a user message when clicked
    engine
        .register_custom_syntax(
            ["ADD", "SUGGESTION", "TEXT", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let text_value = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_text_suggestion(cache4.as_ref(), &user_session4, &text_value, &button_text)?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // ADD_SUGGESTION_TOOL "tool_name" AS "button text" - underscore version to avoid syntax conflicts
    engine
        .register_custom_syntax(
            ["ADD_SUGGESTION_TOOL", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let tool_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_tool_suggestion(
                    cache5.as_ref(),
                    &user_session5,
                    &tool_name,
                    None,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    engine
        .register_custom_syntax(
            ["ADD", "SUGGESTION", "TOOL", "$expr$", "AS", "$expr$"],
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
        .expect("valid syntax registration");

    engine
        .register_custom_syntax(
            [
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
        .expect("valid syntax registration");
}

fn add_context_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    context_name: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

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

fn add_text_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    text_value: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

        let suggestion = json!({
            "type": "text_value",
            "text": button_text,
            "value": text_value,
            "action": {
                "type": "send_message",
                "message": text_value
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
                    "Added text suggestion '{}' to session {}, total: {}",
                    text_value,
                    user_session.id,
                    length
                );
            }
            Err(e) => error!("Failed to add text suggestion to Redis: {}", e),
        }
    } else {
        trace!("No cache configured, text suggestion not added");
    }

    Ok(())
}

fn add_tool_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    tool_name: &str,
    params: Option<Vec<String>>,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

        // Create action object and serialize it to JSON string
        let action_obj = json!({
            "type": "invoke_tool",
            "tool": tool_name,
            "params": params,
            "prompt_for_params": params.is_none()
        });
        let action_str = action_obj.to_string();

        let suggestion = json!({
            "text": button_text,
            "action": action_str
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
                info!(
                    "Added tool suggestion '{}' to session {}, total: {}",
                    tool_name,
                    user_session.id,
                    length
                );
            }
            Err(e) => error!("Failed to add tool suggestion to Redis: {}", e),
        }
    } else {
        trace!("No cache configured, tool suggestion not added");
    }

    Ok(())
}

/// Retrieve suggestions from Valkey/Redis for a given user session
/// Returns a vector of Suggestion structs that can be included in BotResponse
/// Note: This function clears suggestions from Redis after fetching them to prevent duplicates
pub fn get_suggestions(
    cache: Option<&Arc<redis::Client>>,
    user_id: &str,
    session_id: &str,
) -> Vec<crate::shared::models::Suggestion> {
    let mut suggestions = Vec::new();

    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_id, session_id);

        let mut conn = match cache_client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to cache: {}", e);
                return suggestions;
            }
        };

        // Get all suggestions from the Redis list
        let result: Result<Vec<String>, redis::RedisError> = redis::cmd("LRANGE")
            .arg(&redis_key)
            .arg(0)
            .arg(-1)
            .query(&mut conn);

        match result {
            Ok(items) => {
                for item in items {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&item) {
                        let suggestion = crate::shared::models::Suggestion {
                            text: json["text"].as_str().unwrap_or("").to_string(),
                            context: json["context"].as_str().map(|s| s.to_string()),
                            action: json.get("action").and_then(|v| serde_json::to_string(v).ok()),
                            icon: json.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        };
                        suggestions.push(suggestion);
                    }
                }
                info!(
                    "[SUGGESTIONS] Retrieved {} suggestions for session {}",
                    suggestions.len(),
                    session_id
                );

                // DO NOT clear suggestions from Redis - keep them persistent for the session
                // TODO: This may cause suggestions to appear multiple times, need better solution
                // if !suggestions.is_empty() {
                //     let _: Result<i64, redis::RedisError> = redis::cmd("DEL")
                //         .arg(&redis_key)
                //         .query(&mut conn);
                //     info!(
                //         "[SUGGESTIONS] Cleared {} suggestions from Redis for session {}",
                //         suggestions.len(),
                //         session_id
                //     );
                // }
            }
            Err(e) => error!("Failed to get suggestions from Redis: {}", e),
        }
    } else {
        info!("[SUGGESTIONS] No cache configured, cannot retrieve suggestions");
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_suggestion_json_context() {
        let suggestion = json!({
            "type": "context",
            "context": "products",
            "text": "View Products",
            "action": {
                "type": "select_context",
                "context": "products"
            }
        });

        assert_eq!(suggestion["type"], "context");
        assert_eq!(suggestion["action"]["type"], "select_context");
    }

    #[test]
    fn test_suggestion_json_tool_no_params() {
        let suggestion = json!({
            "type": "tool",
            "tool": "search_kb",
            "text": "Search Knowledge Base",
            "action": {
                "type": "invoke_tool",
                "tool": "search_kb",
                "params": Option::<Vec<String>>::None,
                "prompt_for_params": true
            }
        });

        assert_eq!(suggestion["type"], "tool");
        assert_eq!(suggestion["action"]["prompt_for_params"], true);
    }

    #[test]
    fn test_suggestion_json_tool_with_params() {
        let params = vec!["query".to_string(), "products".to_string()];
        let suggestion = json!({
            "type": "tool",
            "tool": "search_kb",
            "text": "Search Products",
            "action": {
                "type": "invoke_tool",
                "tool": "search_kb",
                "params": params,
                "prompt_for_params": false
            }
        });

        assert_eq!(suggestion["type"], "tool");
        assert_eq!(suggestion["action"]["prompt_for_params"], false);
        assert!(suggestion["action"]["params"].is_array());
    }
}
