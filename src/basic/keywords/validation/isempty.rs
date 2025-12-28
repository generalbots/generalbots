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
        if let Ok(s) = value.clone().into_string() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::{Array, Map};

    #[test]
    fn test_empty_string() {
        let value = Dynamic::from("");
        assert!(check_empty(&value));
    }

    #[test]
    fn test_non_empty_string() {
        let value = Dynamic::from("hello");
        assert!(!check_empty(&value));
    }

    #[test]
    fn test_empty_array() {
        let value = Dynamic::from(Array::new());
        assert!(check_empty(&value));
    }

    #[test]
    fn test_non_empty_array() {
        let arr: Array = vec![Dynamic::from(1)];
        let value = Dynamic::from(arr);
        assert!(!check_empty(&value));
    }

    #[test]
    fn test_empty_map() {
        let value = Dynamic::from(Map::new());
        assert!(check_empty(&value));
    }

    #[test]
    fn test_unit() {
        let value = Dynamic::UNIT;
        assert!(check_empty(&value));
    }

    #[test]
    fn test_number_not_empty() {
        let value = Dynamic::from(0);
        assert!(!check_empty(&value));
    }

    #[test]
    fn test_bool_not_empty() {
        let value = Dynamic::from(false);
        assert!(!check_empty(&value));
    }
}
