use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::trace;
use rhai::{Engine, EvalAltResult};
use serde_json::json;
use std::sync::Arc;

use super::types::InputType;

pub fn hear_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_hear_basic(Arc::clone(&state), user.clone(), engine);

    register_hear_as_type(Arc::clone(&state), user.clone(), engine);

    register_hear_as_menu(state, user, engine);
}

fn register_hear_basic(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(["HEAR", "$ident$"], true, move |_context, inputs| {
            let variable_name = inputs[0]
                .get_string_value()
                .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                    "Expected identifier as string".into(),
                    rhai::Position::NONE,
                )))?
                .to_lowercase();

            trace!(
                "HEAR command waiting for user input to store in variable: {}",
                variable_name
            );

            let state_for_spawn = Arc::clone(&state_clone);
            let session_id_clone = session_id;

            tokio::spawn(async move {
                trace!(
                    "HEAR: Setting session {} to wait for input for variable '{}'",
                    session_id_clone,
                    variable_name
                );

                {
                    let mut session_manager = state_for_spawn.session_manager.lock().await;
                    session_manager.mark_waiting(session_id_clone);
                }

                if let Some(redis_client) = &state_for_spawn.cache {
                    if let Ok(conn) = redis_client.get_multiplexed_async_connection().await {
                        let mut conn = conn;
                        let key = format!("hear:{session_id_clone}:{variable_name}");
                        let wait_data = json!({
                            "variable": variable_name,
                            "type": "any",
                            "waiting": true,
                            "retry_count": 0
                        });
                        let _: Result<(), _> = redis::cmd("SET")
                            .arg(key)
                            .arg(wait_data.to_string())
                            .arg("EX")
                            .arg(3600)
                            .query_async(&mut conn)
                            .await;
                    }
                }
            });

            Err(Box::new(EvalAltResult::ErrorRuntime(
                "Waiting for user input".into(),
                rhai::Position::NONE,
            )))
        })
        .expect("valid syntax registration");
}

fn register_hear_as_type(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["HEAR", "$ident$", "AS", "$ident$"],
            true,
            move |_context, inputs| {
                let variable_name = inputs[0]
                    .get_string_value()
                    .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                        "Expected identifier for variable".into(),
                        rhai::Position::NONE,
                    )))?
                    .to_lowercase();
                let type_name = inputs[1]
                    .get_string_value()
                    .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                        "Expected identifier for type".into(),
                        rhai::Position::NONE,
                    )))?
                    .to_string();

                let _input_type = InputType::parse_type(&type_name);

                trace!("HEAR {variable_name} AS {type_name} - waiting for validated input");

                let state_for_spawn = Arc::clone(&state_clone);
                let session_id_clone = session_id;
                let var_name_clone = variable_name;
                let type_clone = type_name;

                tokio::spawn(async move {
                    {
                        let mut session_manager = state_for_spawn.session_manager.lock().await;
                        session_manager.mark_waiting(session_id_clone);
                    }

                    if let Some(redis_client) = &state_for_spawn.cache {
                        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
                        {
                            let key = format!("hear:{session_id_clone}:{var_name_clone}");
                            let wait_data = json!({
                                "variable": var_name_clone,
                                "type": type_clone.to_lowercase(),
                                "waiting": true,
                                "retry_count": 0,
                                "max_retries": 3
                            });
                            let _: Result<(), _> = redis::cmd("SET")
                                .arg(key)
                                .arg(wait_data.to_string())
                                .arg("EX")
                                .arg(3600)
                                .query_async(&mut conn)
                                .await;
                        }
                    }
                });

                Err(Box::new(EvalAltResult::ErrorRuntime(
                    "Waiting for user input".into(),
                    rhai::Position::NONE,
                )))
            },
        )
        .expect("valid syntax registration");
}

fn register_hear_as_menu(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["HEAR", "$ident$", "AS", "$expr$"],
            true,
            move |context, inputs| {
                let variable_name = inputs[0]
                    .get_string_value()
                    .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                        "Expected identifier for variable".into(),
                        rhai::Position::NONE,
                    )))?
                    .to_lowercase();

                let options_expr = context.eval_expression_tree(&inputs[1])?;
                let options_str = options_expr.to_string();

                let input_type = InputType::parse_type(&options_str);
                if input_type != InputType::Any {
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        "Use HEAR AS TYPE syntax".into(),
                        rhai::Position::NONE,
                    )));
                }

                let options: Vec<String> = if options_str.starts_with('[') {
                    serde_json::from_str(&options_str).unwrap_or_default()
                } else {
                    options_str
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };

                if options.is_empty() {
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        "Menu requires at least one option".into(),
                        rhai::Position::NONE,
                    )));
                }

                trace!("HEAR {} AS MENU with options: {:?}", variable_name, options);

                let state_for_spawn = Arc::clone(&state_clone);
                let session_id_clone = session_id;
                let var_name_clone = variable_name;
                let options_clone = options;

                tokio::spawn(async move {
                    {
                        let mut session_manager = state_for_spawn.session_manager.lock().await;
                        session_manager.mark_waiting(session_id_clone);
                    }

                    if let Some(redis_client) = &state_for_spawn.cache {
                        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await
                        {
                            let key = format!("hear:{session_id_clone}:{var_name_clone}");
                            let wait_data = json!({
                                "variable": var_name_clone,
                                "type": "menu",
                                "options": options_clone,
                                "waiting": true,
                                "retry_count": 0
                            });
                            let _: Result<(), _> = redis::cmd("SET")
                                .arg(key)
                                .arg(wait_data.to_string())
                                .arg("EX")
                                .arg(3600)
                                .query_async(&mut conn)
                                .await;

                            let suggestions_key =
                                format!("suggestions:{session_id_clone}:{session_id_clone}");
                            for opt in &options_clone {
                                let suggestion = json!({
                                    "text": opt,
                                    "value": opt
                                });
                                let _: Result<(), _> = redis::cmd("RPUSH")
                                    .arg(&suggestions_key)
                                    .arg(suggestion.to_string())
                                    .query_async(&mut conn)
                                    .await;
                            }
                        }
                    }
                });

                Err(Box::new(EvalAltResult::ErrorRuntime(
                    "Waiting for user input".into(),
                    rhai::Position::NONE,
                )))
            },
        )
        .expect("valid syntax registration");
}
