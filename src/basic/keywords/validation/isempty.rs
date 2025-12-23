use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;




















pub fn isempty_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {

    engine.register_fn("ISEMPTY", |value: Dynamic| -> bool { check_empty(&value) });


    engine.register_fn("isempty", |value: Dynamic| -> bool { check_empty(&value) });


    engine.register_fn("IsEmpty", |value: Dynamic| -> bool { check_empty(&value) });

    debug!("Registered ISEMPTY keyword");
}


fn check_empty(value: &Dynamic) -> bool {

    if value.is_unit() {
        return true;
    }


    if value.is_string() {
        if let Some(s) = value.clone().into_string().ok() {
            return s.is_empty();
        }
    }


    if value.is_array() {
        if let Ok(arr) = value.clone().into_array() {
            return arr.is_empty();
        }
    }


    if value.is_map() {
        if let Some(map) = value.clone().try_cast::<Map>() {
            return map.is_empty();
        }
    }


    if value.is_bool() {

        return false;
    }


    if value.is_int() || value.is_float() {
        return false;
    }

    false
}
