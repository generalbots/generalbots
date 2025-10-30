use crate::shared::state::AppState;
use crate::shared::models::UserSession;
use log::{debug, error, info};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;

pub fn add_suggestion_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let cache = state.redis_client.clone();

    engine
        .register_custom_syntax(&["ADD_SUGGESTION", "$expr$", "$expr$"], true, move |context, inputs| {
            let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
            let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

            info!("ADD_SUGGESTION command executed: context='{}', text='{}'", context_name, button_text);

            if let Some(cache_client) = &cache {
                let cache_client = cache_client.clone();
                let redis_key = format!("suggestions:{}:{}", user.user_id, user.id);
                let suggestion = json!({ "context": context_name, "text": button_text });

                tokio::spawn(async move {
                    let mut conn = match cache_client.get_multiplexed_async_connection().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to connect to cache: {}", e);
                            return;
                        }
                    };

                    // Append suggestion to Redis list
                    let result: Result<(), redis::RedisError> = redis::cmd("RPUSH")
                        .arg(&redis_key)
                        .arg(suggestion.to_string())
                        .query_async(&mut conn)
                        .await;

                    match result {
                        Ok(_) => {
                            debug!("Suggestion added successfully to Redis key {}", redis_key);

                            // Also register context as inactive initially
                            let active_key = format!("active_context:{}:{}", user.user_id, user.id);
                            let _: Result<(), redis::RedisError> = redis::cmd("HSET")
                                .arg(&active_key)
                                .arg(&context_name)
                                .arg("inactive")
                                .query_async(&mut conn)
                                .await
                                .unwrap_or_else(|e| {
                                    error!("Failed to set context state: {}", e);
                                    ()
                                });
                        }
                        Err(e) => error!("Failed to add suggestion to Redis: {}", e),
                    }
                });
            } else {
                debug!("No Redis client configured; suggestion will not persist");
            }

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}
