use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::trace;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde_json::json;
use std::sync::Arc;

use super::types::InputType;

pub fn hear_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_hear_basic(Arc::clone(&state), user.clone(), engine);
    register_hear_as_type(Arc::clone(&state), user.clone(), engine);
    register_hear_as_menu(state, user, engine);
}

/// Block the Rhai thread until the user sends input.
/// Registers a sync_channel receiver in AppState::hear_channels keyed by session_id,
/// then blocks. The async message handler calls `deliver_hear_input` to unblock it.
fn hear_block(state: &Arc<AppState>, session_id: uuid::Uuid, variable_name: &str, wait_data: serde_json::Value) -> Result<Dynamic, Box<EvalAltResult>> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);

    // Register the sender so the async handler can find it
    if let Ok(mut map) = state.hear_channels.lock() {
        map.insert(session_id, tx);
    }

    // Mark session as waiting and store metadata in Redis (for UI hints like menus)
    let state_clone = Arc::clone(state);
    let var = variable_name.to_string();
    let (init_tx, init_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            rt.block_on(async move {
                {
                    let mut sm = state_clone.session_manager.lock().await;
                    sm.mark_waiting(session_id);
                }
                if let Some(redis) = &state_clone.cache {
                    if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
                        let key = format!("hear:{session_id}:{var}");
                        let _: Result<(), _> = redis::cmd("SET")
                            .arg(&key)
                            .arg(wait_data.to_string())
                            .arg("EX")
                            .arg(3600)
                            .query_async(&mut conn)
                            .await;
                    }
                }
            });
        }
        let _ = init_tx.send(());
    });
    let _ = init_rx.recv();

    trace!("HEAR {variable_name}: blocking thread, waiting for user input");

    // Block the Rhai thread (runs in spawn_blocking, so this is safe).
    // Timeout is configurable via bot config "hear-timeout-secs", default 1 hour.
    let timeout_secs: u64 = state.conn.get().ok()
        .and_then(|mut conn| {
            #[derive(diesel::QueryableByName)]
            struct Row { #[diesel(sql_type = diesel::sql_types::Text)] config_value: String }
            diesel::sql_query(
                "SELECT config_value FROM bot_configuration WHERE config_key = 'hear-timeout-secs' LIMIT 1"
            ).load::<Row>(&mut conn).ok()
            .and_then(|rows| rows.into_iter().next())
            .and_then(|r| r.config_value.parse().ok())
        })
        .unwrap_or(3600);

    match rx.recv_timeout(std::time::Duration::from_secs(timeout_secs)) {
        Ok(value) => {
            trace!("HEAR {variable_name}: received '{value}', resuming script");
            Ok(value.into())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            if let Ok(mut map) = state.hear_channels.lock() {
                map.remove(&session_id);
            }
            Err(Box::new(EvalAltResult::ErrorRuntime(
                format!("HEAR timed out after {timeout_secs}s").into(),
                rhai::Position::NONE,
            )))
        }
        Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
            "HEAR channel closed".into(),
            rhai::Position::NONE,
        ))),
    }
}

/// Called by the async message handler when the user sends a reply.
/// Unblocks the waiting Rhai thread.
pub fn deliver_hear_input(state: &AppState, session_id: uuid::Uuid, value: String) -> bool {
    if let Ok(mut map) = state.hear_channels.lock() {
        if let Some(tx) = map.remove(&session_id) {
            return tx.send(value).is_ok();
        }
    }
    false
}

fn register_hear_basic(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(["HEAR", "$ident$"], true, move |context, inputs| {
            let variable_name = inputs[0]
                .get_string_value()
                .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                    "Expected identifier".into(),
                    rhai::Position::NONE,
                )))?
                .to_lowercase();

            let value = hear_block(&state_clone, session_id, &variable_name, json!({
                "variable": variable_name,
                "type": "any",
                "waiting": true
            }))?;

            context.scope_mut().set_or_push(&variable_name, value.clone());
            Ok(value)
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
            move |context, inputs| {
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

                let value = hear_block(&state_clone, session_id, &variable_name, json!({
                    "variable": variable_name,
                    "type": type_name.to_lowercase(),
                    "waiting": true,
                    "max_retries": 3
                }))?;

                context.scope_mut().set_or_push(&variable_name, value.clone());
                Ok(value)
            },
        )
        .expect("valid syntax registration");
}

fn register_hear_as_menu(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let bot_id = user.bot_id;
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

                // Store suggestions in Redis for UI
                let state_for_suggestions = Arc::clone(&state_clone);
                let opts_clone = options.clone();
                let bot_id_clone = bot_id;
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();
                    if let Ok(rt) = rt {
                        rt.block_on(async move {
                            if let Some(redis) = &state_for_suggestions.cache {
                                if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
                                    let key = format!("suggestions:{}:{}", bot_id_clone, session_id);
                                    for opt in &opts_clone {
                                        let _: Result<(), _> = redis::cmd("RPUSH")
                                            .arg(&key)
                                            .arg(json!({"text": opt, "value": opt}).to_string())
                                            .query_async(&mut conn)
                                            .await;
                                    }
                                }
                            }
                        });
                    }
                    let _ = tx.send(());
                });
                let _ = rx.recv();

                let value = hear_block(&state_clone, session_id, &variable_name, json!({
                    "variable": variable_name,
                    "type": "menu",
                    "options": options,
                    "waiting": true
                }))?;

                context.scope_mut().set_or_push(&variable_name, value.clone());
                Ok(value)
            },
        )
        .expect("valid syntax registration");
}
