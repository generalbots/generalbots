use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::trace;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde_json::json;
use std::sync::Arc;

pub fn hear_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    register_hear_basic(state, user.clone(), engine);
    register_hear_as_type(state, user.clone(), engine);
    register_hear_as_menu(state, user, engine);
}

fn hear_block(state: &Arc<dyn BasicRuntime>, session_id: uuid::Uuid, variable_name: &str, wait_data: serde_json::Value) -> Result<Dynamic, Box<EvalAltResult>> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);

    if let Ok(mut map) = state.hear_channels().lock() {
        map.insert(session_id, tx);
    }

    let state_clone = Arc::clone(state);
    let var = variable_name.to_string();
    let (init_tx, init_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            rt.block_on(async move {
                let sm = state_clone.session_manager();
                let sm_lock = sm.lock().await;
                drop(sm_lock);
                if let Some(redis) = state_clone.cache_client().as_ref() {
                    if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
                        // TODO(#477): Pass org when available (UserSession does not carry org yet)
                        let key = botlib::key_utils::build_key("", &["hear", &session_id.to_string(), &var]);
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

    let timeout_secs: u64 = read_hear_timeout_from_vault()
        .unwrap_or(3600);

    match rx.recv_timeout(std::time::Duration::from_secs(timeout_secs)) {
        Ok(value) => {
            trace!("HEAR {variable_name}: received '{value}', resuming script");
            Ok(value.into())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            if let Ok(mut map) = state.hear_channels().lock() {
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

pub fn deliver_hear_input(state: &Arc<dyn BasicRuntime>, session_id: uuid::Uuid, value: String) -> bool {
    if let Ok(mut map) = state.hear_channels().lock() {
        if let Some(tx) = map.remove(&session_id) {
            return tx.send(value).is_ok();
        }
    }
    false
}

fn register_hear_basic(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(state);

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

fn register_hear_as_type(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(state);

    engine
        .register_custom_syntax(
            ["HEAR", "$expr$", "as", "$ident$"],
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

fn register_hear_as_menu(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let session_id = user.id;
    let state_clone = Arc::clone(state);

    engine
        .register_custom_syntax(
            ["HEAR", "$expr$", "as", "$ident$"],
            true,
            move |context, inputs| {
                    let variable_name = inputs[1]
                    .get_string_value()
                    .ok_or_else(|| Box::new(EvalAltResult::ErrorRuntime(
                        "Expected identifier for variable".into(),
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
            },
        )
        .expect("valid syntax registration");
}

fn read_hear_timeout_from_vault() -> Option<u64> {
    use botcoresecrets::manager::SecretsManager;
    let sm = SecretsManager::get_clone().ok()?;
    if !sm.is_enabled() {
        return None;
    }
    let path = format!("gbo/{}/{}/{}", uuid::Uuid::nil(), uuid::Uuid::nil(), uuid::Uuid::nil());
    let sm_clone = sm.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move {
                sm_clone.get_value(&path, "hear-timeout-secs").await.ok()
            })
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv().ok().flatten()?.parse().ok()
}
