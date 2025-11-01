use std::sync::Arc;
use log::{error, info, trace};
use crate::shared::state::AppState;
use crate::shared::models::UserSession;
use rhai::Engine;
use rhai::Dynamic;

pub fn set_context_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    // Clone the Redis client (if any) for use inside the async task.
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(
            &["SET_CONTEXT", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                // First expression is the context name, second is the value.
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let context_value = context.eval_expression_tree(&inputs[1])?.to_string();

                info!(
                    "SET CONTEXT command executed - name: {}, value: {}",
                    context_name,
                    context_value
                );

                // Build a Redis key that is unique per user and session.
                let redis_key = format!(
                    "context:{}:{}:{}",
                    user.user_id,
                    user.id,
                    context_name
                );

                trace!(
                    target: "app::set_context",
                    "Constructed Redis key: {} for user {}, session {}, context {}",
                    redis_key,
                    user.user_id,
                    user.id,
                    context_name
                );

                // If a Redis client is configured, perform the SET operation asynchronously.
                if let Some(cache_client) = &cache {
                    trace!("Redis client is available, preparing to set context value");

                    // Clone values needed inside the async block.
                    let cache_client = cache_client.clone();
                    let redis_key = redis_key.clone();
                    let context_value = context_value.clone();

                    trace!(
                        "Cloned cache_client, redis_key ({}) and context_value (len={}) for async task",
                        redis_key,
                        context_value.len()
                    );

                    // Spawn a background task so we don't need an async closure here.
                    tokio::spawn(async move {
                        trace!("Async task started for SET_CONTEXT operation");

                        // Acquire an async Redis connection.
                        let mut conn = match cache_client.get_multiplexed_async_connection().await {
                            Ok(conn) => {
                                trace!("Successfully acquired async Redis connection");
                                conn
                            }
                            Err(e) => {
                                error!("Failed to connect to cache: {}", e);
                                trace!("Aborting SET_CONTEXT task due to connection error");
                                return;
                            }
                        };

                        // Perform the SET command.
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
                                trace!("Successfully set context in Redis for key {}", redis_key);
                            }
                            Err(e) => {
                                error!("Failed to set cache value: {}", e);
                                trace!("SET_CONTEXT Redis SET command failed");
                            }
                        }
                    });
                } else {
                    trace!("No Redis client configured; SET_CONTEXT will not persist to cache");
                }

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
