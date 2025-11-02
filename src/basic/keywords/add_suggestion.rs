use crate::shared::state::AppState;
use crate::shared::models::UserSession;
use log::{trace, debug, error, info};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

pub fn add_suggestion_keyword(state: Arc<AppState>, user_session: UserSession, engine: &mut Engine) {
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(&["ADD_SUGGESTION", "$expr$", "AS", "$expr$"], true, move |context, inputs| {
            let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
            let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

            info!("ADD_SUGGESTION command executed: context='{}', text='{}'", context_name, button_text);

            if let Some(cache_client) = &cache {
                let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);
                let suggestion = json!({ "context": context_name, "text": button_text });

                let mut conn = match cache_client.get_connection() {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to connect to cache: {}", e);
                        return Ok(Dynamic::UNIT);
                    }
                };

                // Append suggestion to Redis list - RPUSH returns the new length as i64
                let result: Result<i64, redis::RedisError> = redis::cmd("RPUSH")
                    .arg(&redis_key)
                    .arg(suggestion.to_string())
                    .query(&mut conn);

                match result {
                    Ok(length) => {
                        trace!("Suggestion added successfully to Redis key {}, new length: {}", redis_key, length);

                        // Also register context as inactive initially
                        let active_key = format!("active_context:{}:{}", user_session.user_id, user_session.id);
                        let hset_result: Result<i64, redis::RedisError> = redis::cmd("HSET")
                            .arg(&active_key)
                            .arg(&context_name)
                            .arg("inactive")
                            .query(&mut conn);

                        match hset_result {
                            Ok(fields_added) => {
                                trace!("Context state set to inactive for {}, fields added: {}", context_name, fields_added)
                            },
                            Err(e) => error!("Failed to set context state: {}", e),
                        }
                    }
                    Err(e) => error!("Failed to add suggestion to Redis: {}", e),
                }
            } else {
                debug!("No Cache client configured; suggestion will not persist");
            }

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}
