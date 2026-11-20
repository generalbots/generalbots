use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::trace;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn set_context_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let cache = state.cache_client().clone();

    if let Err(e) = engine
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

                // #2547 — org-scoped key plus dual-written legacy key (for
                // pre-field sessions and readers without an org).
                let parts = ["context", &user.user_id.to_string(), &user.id.to_string(), &context_name];
                let redis_key = botlib::key_utils::scoped_key(user.org_id(), &parts);
                let legacy_key = botlib::key_utils::legacy_key(&parts);

                trace!(
                    "Constructed Redis key: {} for user {}, session {}, context {}",
                    redis_key,
                    user.user_id,
                    user.id,
                    context_name
                );

                if let Some(cache_client) = &cache {
                    let cache_client_arc: Arc<redis::Client> = cache_client.clone();
                    let redis_key_clone = redis_key.clone();
                    let legacy_key_clone = legacy_key.clone();
                    let context_value_clone = context_value.clone();

                    trace!(
                        "Cloned cache_client, redis_key ({}) and context_value (len={}) for async task",
                        redis_key,
                        context_value.len()
                    );

                    tokio::spawn(async move {
                        let mut conn = match cache_client_arc.get_multiplexed_async_connection().await {
                            Ok(conn) => {
                                trace!("Cache connection established successfully");
                                conn
                            }
                            Err(e) => {
                                log::error!("Failed to connect to cache: {}", e);
                                return;
                            }
                        };

                        trace!(
                            "Executing Redis SET command with key: {} and value length: {}",
                            redis_key_clone,
                            context_value_clone.len()
                        );

                        for key in [&redis_key_clone, &legacy_key_clone] {
                            let result: Result<(), redis::RedisError> = redis::cmd("SET")
                                .arg(key)
                                .arg(&context_value_clone)
                                .query_async(&mut conn)
                                .await;
                            if let Err(e) = result {
                                log::error!("Failed to set cache value: {}", e);
                            }
                        }
                        trace!("Context value successfully stored in cache");
                    });
                } else {
                    trace!("No cache configured, context not persisted");
                }

                Ok(Dynamic::UNIT)
            },
        )
    {
        log::error!("Failed to register the custom syntax: {e}");
    }
}
