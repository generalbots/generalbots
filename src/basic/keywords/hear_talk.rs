use crate::shared::models::{BotResponse, UserSession};
use crate::shared::state::AppState;
use log::{debug, error, info};
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;

pub fn hear_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(&["HEAR", "$ident$"], true, move |_context, inputs| {
            let variable_name = inputs[0]
                .get_string_value()
                .expect("Expected identifier as string")
                .to_string();

            info!(
                "HEAR command waiting for user input to store in variable: {}",
                variable_name
            );

            let state_for_spawn = Arc::clone(&state_clone);
            let session_id_clone = session_id;
            let var_name_clone = variable_name.clone();

            tokio::spawn(async move {
                debug!(
                    "HEAR: Setting session {} to wait for input for variable '{}'",
                    session_id_clone, var_name_clone
                );

                let mut session_manager = state_for_spawn.session_manager.lock().await;
                session_manager.mark_waiting(session_id_clone);

                if let Some(redis_client) = &state_for_spawn.cache {
                    let mut conn = match redis_client.get_multiplexed_async_connection().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Failed to connect to cache: {}", e);
                            return;
                        }
                    };

                    let key = format!("hear:{}:{}", session_id_clone, var_name_clone);
                    let _: Result<(), _> = redis::cmd("SET")
                        .arg(&key)
                        .arg("waiting")
                        .query_async(&mut conn)
                        .await;
                }
            });

            Err(Box::new(EvalAltResult::ErrorRuntime(
                "Waiting for user input".into(),
                rhai::Position::NONE,
            )))
        })
        .unwrap();
}

pub fn talk_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["TALK", "$expr$"], true, move |context, inputs| {
            // Evaluate the expression that produces the message text.
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            info!("TALK command executed: {}", message);
            debug!("TALK: Sending message: {}", message);

            // Build the bot response that will be sent back to the client.
            let bot_id = "default_bot".to_string();
            let response = BotResponse {
                bot_id,
                user_id: user_clone.user_id.to_string(),
                session_id: user_clone.id.to_string(),
                channel: "web".to_string(),
                content: message,
                message_type: 1,
                stream_token: None,
                is_complete: true,
                suggestions: Vec::new(),
            };

            let user_id = user_clone.id.to_string();

            // Try to acquire the lock on the response_channels map. The map is protected
            // by an async `tokio::sync::Mutex`, so we use `try_lock` to avoid awaiting
            // inside this non‑async closure.
            match state_clone.response_channels.try_lock() {
                Ok(response_channels) => {
                    if let Some(tx) = response_channels.get(&user_id) {
                        // Use `try_send` to avoid blocking the runtime.
                        if let Err(e) = tx.try_send(response.clone()) {
                            error!("Failed to send TALK message via WebSocket: {}", e);
                        } else {
                            debug!("TALK message sent successfully via WebSocket");
                        }
                    } else {
                        debug!(
                            "No WebSocket connection found for session {}, sending via web adapter",
                            user_id
                        );
                        // The web adapter method is async (`send_message_to_session`), so we
                        // spawn a detached task to perform the send without blocking.
                        let web_adapter = Arc::clone(&state_clone.web_adapter);
                        let resp_clone = response.clone();
                        let sess_id = user_id.clone();
                        tokio::spawn(async move {
                            if let Err(e) = web_adapter
                                .send_message_to_session(&sess_id, resp_clone)
                                .await
                            {
                                error!("Failed to send TALK message via web adapter: {}", e);
                            } else {
                                debug!("TALK message sent successfully via web adapter");
                            }
                        });
                    }
                }
                Err(_) => {
                    error!("Failed to acquire lock on response_channels for TALK command");
                }
            }

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}

