use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn get_redis_connection(cache_client: &Arc<redis::Client>) -> Option<redis::Connection> {
    let timeout = Duration::from_millis(50);
    cache_client.get_connection_with_timeout(timeout).ok()
}

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
                let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);
                let mut conn = match get_redis_connection(cache_client) {
                    Some(conn) => conn,
                    None => {
                        trace!("Cache not ready, skipping clear suggestions");
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
    // Each closure needs its own Arc<redis::Client> and UserSession clone
    let cache = state.cache.clone();
    let cache2 = state.cache.clone();
    let cache3 = state.cache.clone();
    let cache4 = state.cache.clone();
    let user_session = user_session.clone();
    let user_session2 = user_session.clone();
    let user_session3 = user_session.clone();
    let user_session4 = user_session.clone();

    // ADD_SUGGESTION_TOOL "tool_name" as "button text"
    engine
        .register_custom_syntax(
            ["ADD_SUGGESTION_TOOL", "$expr$", "as", "$expr$"],
            true,
            move |context, inputs| {
                let tool_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_tool_suggestion(
                    cache.as_ref(),
                    &user_session,
                    &tool_name,
                    None,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // ADD_SUGGESTION_TEXT "text_value" as "button text"
    engine
        .register_custom_syntax(
            ["ADD_SUGGESTION_TEXT", "$expr$", "as", "$expr$"],
            true,
            move |context, inputs| {
                let text_value = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_text_suggestion(cache2.as_ref(), &user_session2, &text_value, &button_text)?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // ADD_SUGGESTION "context_name" as "button text" (register BEFORE simple form so simple form has higher priority)
    engine
        .register_custom_syntax(
            ["ADD_SUGGESTION", "$expr$", "as", "$expr$"],
            true,
            move |context, inputs| {
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_context_suggestion(
                    cache3.as_ref(),
                    &user_session3,
                    &context_name,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // ADD_SUGGESTION "button text" (simple form - sends message on click)
    // Registered LAST so it has HIGHEST priority — Rhai tries this first, falls back to 2-arg form
    engine
        .register_custom_syntax(
            ["ADD_SUGGESTION", "$expr$"],
            true,
            move |context, inputs| {
                let button_text = context.eval_expression_tree(&inputs[0])?.to_string();

                add_text_suggestion(
                    cache4.as_ref(),
                    &user_session4,
                    &button_text,
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
        let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);

        let suggestion = json!({
            "type": "context",
            "context": context_name,
            "text": button_text,
            "action": {
                "type": "select_context",
                "context": context_name
            }
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add context suggestion");
                return Ok(());
            }
        };

        let _: Result<i64, redis::RedisError> = redis::cmd("SADD")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        trace!(
            "Added context suggestion '{}' to session {}",
            context_name,
            user_session.id
        );

        let active_key = format!("active_context:{}:{}", user_session.bot_id, user_session.id);

        let _: Result<i64, redis::RedisError> = redis::cmd("HSET")
            .arg(&active_key)
            .arg(context_name)
            .arg("inactive")
            .query(&mut conn);
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
        let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);

        let suggestion = json!({
            "type": "text_value",
            "text": button_text,
            "value": text_value,
            "action": {
                "type": "send_message",
                "message": text_value
            }
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add text suggestion");
                return Ok(());
            }
        };

        let _: Result<i64, redis::RedisError> = redis::cmd("SADD")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        trace!(
            "Added text suggestion '{}' to session {}",
            text_value,
            user_session.id
        );
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
    info!(
        "ADD_SUGGESTION_TOOL called: tool={}, button={}",
        tool_name, button_text
    );
    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", user_session.bot_id, user_session.id);
        info!("Adding suggestion to Redis key: {}", redis_key);

        let prompt_for_params = params.is_some() && !params.as_ref().unwrap().is_empty();
        let action_obj = json!({
            "type": "invoke_tool",
            "tool": tool_name,
            "params": params,
            "prompt_for_params": prompt_for_params
        });

        let suggestion = json!({
            "type": "invoke_tool",
            "text": button_text,
            "tool": tool_name,
            "action": action_obj
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add tool suggestion");
                return Ok(());
            }
        };

        let _: Result<i64, redis::RedisError> = redis::cmd("SADD")
            .arg(&redis_key)
            .arg(suggestion.to_string())
            .query(&mut conn);

        info!(
            "Added tool suggestion '{}' to session {}",
            tool_name, user_session.id
        );
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
    bot_id: &str,
    session_id: &str,
) -> Vec<crate::core::shared::models::Suggestion> {
    let mut suggestions = Vec::new();

    if let Some(cache_client) = cache {
        let redis_key = format!("suggestions:{}:{}", bot_id, session_id);

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, returning empty suggestions");
                return suggestions;
            }
        };

        // Get all suggestions from the Redis set (deduplicated)
        let result: Result<Vec<String>, redis::RedisError> =
            redis::cmd("SMEMBERS").arg(&redis_key).query(&mut conn);

        match result {
            Ok(items) => {
                for item in items {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&item) {
                        let suggestion = crate::core::shared::models::Suggestion {
                            text: json["text"].as_str().unwrap_or("").to_string(),
                            context: json["context"].as_str().map(|s| s.to_string()),
                            action: json
                                .get("action")
                                .and_then(|v| serde_json::to_string(v).ok()),
                            icon: json
                                .get("icon")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        };
                        suggestions.push(suggestion);
                    }
                }
                info!(
                    "Retrieved {} suggestions for session {}",
                    suggestions.len(),
                    session_id
                );
            }
            Err(e) => error!("Failed to get suggestions from Redis: {}", e),
        }
    } else {
        info!("No cache configured, cannot retrieve suggestions");
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
