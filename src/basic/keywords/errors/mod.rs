pub mod on_error;
pub mod throw;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine, EvalAltResult, Map, Position};
use std::sync::Arc;

pub use on_error::{
    clear_last_error, get_error_number, get_last_error, handle_error, handle_string_error,
    is_error_resume_next_active, set_error_resume_next, set_last_error,
};

pub fn register_error_functions(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    throw_keyword(&state, user.clone(), engine);
    error_keyword(&state, user.clone(), engine);
    is_error_keyword(&state, user.clone(), engine);
    assert_keyword(&state, user.clone(), engine);
    log_error_keyword(&state, user.clone(), engine);

    on_error::register_on_error_keywords(state, user, engine);

    debug!("Registered all error handling functions");
}

pub fn throw_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn(
        "THROW",
        |message: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            Err(Box::new(EvalAltResult::ErrorRuntime(
                message.into(),
                Position::NONE,
            )))
        },
    );

    engine.register_fn(
        "throw",
        |message: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            Err(Box::new(EvalAltResult::ErrorRuntime(
                message.into(),
                Position::NONE,
            )))
        },
    );

    engine.register_fn(
        "RAISE",
        |message: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            Err(Box::new(EvalAltResult::ErrorRuntime(
                message.into(),
                Position::NONE,
            )))
        },
    );

    debug!("Registered THROW keyword");
}

pub fn error_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ERROR", |message: &str| -> Map {
        let mut map = Map::new();
        map.insert("error".into(), Dynamic::from(true));
        map.insert("message".into(), Dynamic::from(message.to_string()));
        map
    });

    engine.register_fn("error", |message: &str| -> Map {
        let mut map = Map::new();
        map.insert("error".into(), Dynamic::from(true));
        map.insert("message".into(), Dynamic::from(message.to_string()));
        map
    });

    debug!("Registered ERROR keyword");
}

pub fn is_error_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("IS_ERROR", |v: Dynamic| -> bool {
        if v.is_map() {
            if let Some(map) = v.try_cast::<Map>() {
                return map.contains_key("error")
                    && map
                        .get("error")
                        .map(|e| e.as_bool().unwrap_or(false))
                        .unwrap_or(false);
            }
        }
        false
    });

    engine.register_fn("is_error", |v: Dynamic| -> bool {
        if v.is_map() {
            if let Some(map) = v.try_cast::<Map>() {
                return map.contains_key("error")
                    && map
                        .get("error")
                        .map(|e| e.as_bool().unwrap_or(false))
                        .unwrap_or(false);
            }
        }
        false
    });

    engine.register_fn("ISERROR", |v: Dynamic| -> bool {
        if v.is_map() {
            if let Some(map) = v.try_cast::<Map>() {
                return map.contains_key("error")
                    && map
                        .get("error")
                        .map(|e| e.as_bool().unwrap_or(false))
                        .unwrap_or(false);
            }
        }
        false
    });

    engine.register_fn("GET_ERROR_MESSAGE", |v: Dynamic| -> String {
        if v.is_map() {
            if let Some(map) = v.try_cast::<Map>() {
                if let Some(msg) = map.get("message") {
                    return msg.to_string();
                }
            }
        }
        String::new()
    });

    debug!("Registered IS_ERROR keyword");
}

pub fn assert_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn(
        "ASSERT",
        |condition: bool, message: &str| -> Result<bool, Box<EvalAltResult>> {
            if condition {
                Ok(true)
            } else {
                Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("Assertion failed: {}", message).into(),
                    Position::NONE,
                )))
            }
        },
    );

    engine.register_fn(
        "assert",
        |condition: bool, message: &str| -> Result<bool, Box<EvalAltResult>> {
            if condition {
                Ok(true)
            } else {
                Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("Assertion failed: {}", message).into(),
                    Position::NONE,
                )))
            }
        },
    );

    debug!("Registered ASSERT keyword");
}

pub fn log_error_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LOG_ERROR", |message: &str| {
        log::error!("BASIC Script Error: {}", message);
    });

    engine.register_fn("log_error", |message: &str| {
        log::error!("BASIC Script Error: {}", message);
    });

    engine.register_fn("LOG_WARN", |message: &str| {
        log::warn!("BASIC Script Warning: {}", message);
    });

    engine.register_fn("LOG_INFO", |message: &str| {
        log::info!("BASIC Script: {}", message);
    });

    engine.register_fn("LOG_DEBUG", |message: &str| {
        log::trace!("BASIC Script Debug: {}", message);
    });

    debug!("Registered LOG_ERROR keyword");
}

#[cfg(test)]
mod tests {
    use rhai::{Dynamic, Map};

    #[test]
    fn test_error_map() {
        let mut map = Map::new();
        map.insert("error".into(), Dynamic::from(true));
        map.insert("message".into(), Dynamic::from("test error"));

        assert!(map.contains_key("error"));
        assert!(map.get("error").unwrap().as_bool().unwrap_or(false));
    }
}
