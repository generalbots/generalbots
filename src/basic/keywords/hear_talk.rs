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

pub async fn execute_talk(state: Arc<AppState>, user_session: UserSession, message: String) -> Result<BotResponse, Box<dyn std::error::Error>> {
    info!("Executing TALK with message: {}", message);
    debug!("TALK: Sending message: {}", message);

    let mut suggestions = Vec::new();
    
            if let Some(redis_client) = &state.cache {
                let mut conn: redis::aio::MultiplexedConnection = redis_client.get_multiplexed_async_connection().await?;

                let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);
                debug!("Loading suggestions from Redis key: {}", redis_key);
                let suggestions_json: Result<Vec<String>, _> = redis::cmd("LRANGE")
                    .arg(redis_key.as_str())
                    .arg(0)
                    .arg(-1)
                    .query_async(&mut conn)
                    .await;

                match suggestions_json {
                    Ok(suggestions_json) => {
                        debug!("Found suggestions in Redis: {:?}", suggestions_json);
                        suggestions = suggestions_json.into_iter()
                            .filter_map(|s| serde_json::from_str(&s).ok())
                            .collect();
                        debug!("Parsed suggestions: {:?}", suggestions);
                    }
                    Err(e) => {
                        error!("Failed to load suggestions from Redis: {}", e);
                    }
                }
            }

    let response = BotResponse {
        bot_id: user_session.bot_id.to_string(),
        user_id: user_session.user_id.to_string(),
        session_id: user_session.id.to_string(),
        channel: "web".to_string(),
        content: message,
        message_type: 1,
        stream_token: None,
        is_complete: true,
        suggestions,
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    let user_id = user_session.id.to_string();
    let response_clone = response.clone();

    match state.response_channels.try_lock() {
        Ok(response_channels) => {
            if let Some(tx) = response_channels.get(&user_id) {
                if let Err(e) = tx.try_send(response_clone) {
                    error!("Failed to send TALK message via WebSocket: {}", e);
                } else {
                    debug!("TALK message sent successfully via WebSocket");
                }
            } else {
                debug!("No WebSocket connection found for session {}, sending via web adapter", user_id);
                let web_adapter = Arc::clone(&state.web_adapter);
                tokio::spawn(async move {
                    if let Err(e) = web_adapter.send_message_to_session(&user_id, response_clone).await {
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

    Ok(response)
}

pub fn talk_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["TALK", "$expr$"], true, move |context, inputs| {
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_talk = Arc::clone(&state_clone);
            let user_for_talk = user_clone.clone();

            tokio::spawn(async move {
                if let Err(e) = execute_talk(state_for_talk, user_for_talk, message).await {
                    error!("Error executing TALK command: {}", e);
                }
            });

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}
