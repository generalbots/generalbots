use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

pub fn contains_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CONTAINS", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("contains", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("IN_ARRAY", |value: Dynamic, arr: Array| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("in_array", |value: Dynamic, arr: Array| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("INCLUDES", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("includes", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("HAS", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("has", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    debug!("Registered CONTAINS keyword");
}

fn array_contains(arr: &Array, value: &Dynamic) -> bool {
    let search_str = value.to_string();

    for item in arr {
        if items_equal(item, value) {
            return true;
        }

        if item.to_string() == search_str {
            return true;
        }
    }

    false
}

fn items_equal(a: &Dynamic, b: &Dynamic) -> bool {
    if a.is_int() && b.is_int() {
        return a.as_int().unwrap_or(0) == b.as_int().unwrap_or(1);
    }

    if a.is_float() && b.is_float() {
        let af = a.as_float().unwrap_or(0.0);
        let bf = b.as_float().unwrap_or(1.0);
        return (af - bf).abs() < f64::EPSILON;
    }

    if a.is_int() && b.is_float() {
        #[allow(clippy::cast_precision_loss)]
        let af = a.as_int().unwrap_or(0) as f64;
        let bf = b.as_float().unwrap_or(1.0);
        return (af - bf).abs() < f64::EPSILON;
    }

    if a.is_float() && b.is_int() {
        let af = a.as_float().unwrap_or(0.0);
        #[allow(clippy::cast_precision_loss)]
        let bf = b.as_int().unwrap_or(1) as f64;
        return (af - bf).abs() < f64::EPSILON;
    }

    if a.is_bool() && b.is_bool() {
        return a.as_bool().unwrap_or(false) == b.as_bool().unwrap_or(true);
    }

    if a.is_string() && b.is_string() {
        return a.clone().into_string().unwrap_or_default()
            == b.clone().into_string().unwrap_or_default();
    }

    false
}
