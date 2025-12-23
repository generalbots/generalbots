use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

pub fn push_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("PUSH", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("push", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("ARRAY_PUSH", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("APPEND", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("append", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    debug!("Registered PUSH keyword");
}

pub fn pop_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("POP", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    engine.register_fn("pop", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    engine.register_fn("ARRAY_POP", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    debug!("Registered POP keyword");
}

pub fn shift_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SHIFT", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    engine.register_fn("shift", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    engine.register_fn("ARRAY_SHIFT", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    debug!("Registered SHIFT keyword");
}

pub fn unshift_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("UNSHIFT", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    engine.register_fn("unshift", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    engine.register_fn("PREPEND", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    engine.register_fn("prepend", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    debug!("Registered UNSHIFT keyword");
}
