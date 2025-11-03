use std::sync::Arc;
use log::{error, info, trace};
use crate::shared::state::AppState;
use crate::shared::models::UserSession;
use rhai::Engine;
use rhai::Dynamic;

/// Registers the `SET_CURRENT_CONTEXT` keyword which stores a context value in Redis
/// and marks the context as active.
///
/// # Arguments
///
/// * `state` – Shared application state (Arc<AppState>).
/// * `user` – The current user session (provides user_id and session id).
/// * `engine` – The script engine where the custom syntax will be registered.
pub fn set_current_context_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    // Clone the Redis client (if any) for use inside the async task.
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(
            &["SET_CURRENT_CONTEXT", "$expr$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                // First expression is the context name, second is the value.
                let context_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let context_value = context.eval_expression_tree(&inputs[1])?.to_string();

                info!(
                    "SET_CURRENT_CONTEXT command executed - name: {}, value: {}",
                    context_name,
                    context_value
                );

                // Build a Redis key that is unique per user and session.
                let redis_key = format!(
                    "context:{}:{}",
                    user.user_id,
                    user.id
                );

                trace!(
                    target: "app::set_current_context",
                    "Constructed Redis key: {} for user {}, session {}, context {}",
                    redis_key,
                    user.user_id,
                    user.id,
                    context_name
                );

                        // Use session manager to update context
                        let state = state.clone();
                        let user = user.clone();
                        let context_value = context_value.clone();
                        tokio::spawn(async move {
                            if let Err(e) = state.session_manager.lock().await.update_session_context(
                                &user.id,
                                &user.user_id,
                                context_value
                            ).await {
                                error!("Failed to update session context: {}", e);
                            }
                        });

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}
