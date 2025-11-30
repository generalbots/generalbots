use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

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

    // Register with spaces: ADD SUGGESTION "key" AS "text"
    engine
        .register_custom_syntax(
            &["ADD", "SUGGESTION", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                if let Some(cache_client) = &cache {
                    let redis_key =
                        format!("suggestions:{}:{}", user_session.user_id, user_session.id);
                    let suggestion = json!({ "context": context_name, "text": button_text });

                    let mut conn = match cache_client.get_connection() {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to connect to cache: {}", e);
                            return Ok(Dynamic::UNIT);
                        }
                    };

                    let result: Result<i64, redis::RedisError> = redis::cmd("RPUSH")
                        .arg(&redis_key)
                        .arg(suggestion.to_string())
                        .query(&mut conn);

                    match result {
                        Ok(length) => {
                            trace!(
                                "Added suggestion to session {}, total suggestions: {}",
                                user_session.id,
                                length
                            );

                            let active_key = format!(
                                "active_context:{}:{}",
                                user_session.user_id, user_session.id
                            );

                            let hset_result: Result<i64, redis::RedisError> = redis::cmd("HSET")
                                .arg(&active_key)
                                .arg(&context_name)
                                .arg("inactive")
                                .query(&mut conn);

                            match hset_result {
                                Ok(_fields_added) => {
                                    trace!("Set context state for session {}", user_session.id);
                                }
                                Err(e) => error!("Failed to set context state: {}", e),
                            }
                        }
                        Err(e) => error!("Failed to add suggestion to Redis: {}", e),
                    }
                } else {
                    trace!("No cache configured, suggestion not added");
                }

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
