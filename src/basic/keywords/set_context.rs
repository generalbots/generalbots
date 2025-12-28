use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn set_context_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(
            ["SET", "CONTEXT", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let context_value = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!(
                    "SET CONTEXT command executed - name: {}, value: {}",
                    context_name,
                    context_value
                );

                let redis_key = format!("context:{}:{}:{}", user.user_id, user.id, context_name);

                trace!(
                    "Constructed Redis key: {} for user {}, session {}, context {}",
                    redis_key,
                    user.user_id,
                    user.id,
                    context_name
                );

                if let Some(cache_client) = &cache {
                    let cache_client = cache_client.clone();

                    trace!(
                        "Cloned cache_client, redis_key ({}) and context_value (len={}) for async task",
                        redis_key,
                        context_value.len()
                    );

                    tokio::spawn(async move {
                        let mut conn = match cache_client.get_multiplexed_async_connection().await {
                            Ok(conn) => {
                                trace!("Cache connection established successfully");
                                conn
                            }
                            Err(e) => {
                                error!("Failed to connect to cache: {}", e);
                                return;
                            }
                        };

                        trace!(
                            "Executing Redis SET command with key: {} and value length: {}",
                            redis_key,
                            context_value.len()
                        );

                        let result: Result<(), redis::RedisError> = redis::cmd("SET")
                            .arg(&redis_key)
                            .arg(&context_value)
                            .query_async(&mut conn)
                            .await;

                        match result {
                            Ok(_) => {
                                trace!("Context value successfully stored in cache");
                            }
                            Err(e) => {
                                error!("Failed to set cache value: {}", e);
                            }
                        }
                    });
                } else {
                    trace!("No cache configured, context not persisted");
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}
