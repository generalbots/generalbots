use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn typeof_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TYPEOF", |value: Dynamic| -> String {
        get_type_name(&value)
    });

    engine.register_fn("typeof", |value: Dynamic| -> String {
        get_type_name(&value)
    });

    engine.register_fn("TYPENAME", |value: Dynamic| -> String {
        get_type_name(&value)
    });

    debug!("Registered TYPEOF keyword");
}

pub fn isarray_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISARRAY", |value: Dynamic| -> bool { value.is_array() });

    engine.register_fn("isarray", |value: Dynamic| -> bool { value.is_array() });

    engine.register_fn("IS_ARRAY", |value: Dynamic| -> bool { value.is_array() });

    debug!("Registered ISARRAY keyword");
}

pub fn isnumber_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISNUMBER", |value: Dynamic| -> bool { is_numeric(&value) });

    engine.register_fn("isnumber", |value: Dynamic| -> bool { is_numeric(&value) });

    engine.register_fn("IS_NUMBER", |value: Dynamic| -> bool { is_numeric(&value) });

    engine.register_fn("ISNUMERIC", |value: Dynamic| -> bool { is_numeric(&value) });

    debug!("Registered ISNUMBER keyword");
}

pub fn isstring_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISSTRING", |value: Dynamic| -> bool { value.is_string() });

    engine.register_fn("isstring", |value: Dynamic| -> bool { value.is_string() });

    engine.register_fn("IS_STRING", |value: Dynamic| -> bool { value.is_string() });

    debug!("Registered ISSTRING keyword");
}

pub fn isbool_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ISBOOL", |value: Dynamic| -> bool { value.is_bool() });

    engine.register_fn("isbool", |value: Dynamic| -> bool { value.is_bool() });

    engine.register_fn("IS_BOOL", |value: Dynamic| -> bool { value.is_bool() });

    engine.register_fn("ISBOOLEAN", |value: Dynamic| -> bool { value.is_bool() });

    debug!("Registered ISBOOL keyword");
}

fn get_type_name(value: &Dynamic) -> String {
    if value.is_unit() {
        "null".to_string()
    } else if value.is_bool() {
        "boolean".to_string()
    } else if value.is_int() {
        "integer".to_string()
    } else if value.is_float() {
        "float".to_string()
    } else if value.is_string() {
        "string".to_string()
    } else if value.is_array() {
        "array".to_string()
    } else if value.is_map() {
        "object".to_string()
    } else if value.is_char() {
        "char".to_string()
    } else {
        value.type_name().to_string()
    }
}

fn is_numeric(value: &Dynamic) -> bool {
    if value.is_int() || value.is_float() {
        return true;
    }

    if value.is_string() {
        if let Ok(s) = value.clone().into_string() {
            return s.trim().parse::<f64>().is_ok();
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_type_name() {
        assert_eq!(get_type_name(&Dynamic::UNIT), "null");
        assert_eq!(get_type_name(&Dynamic::from(true)), "boolean");
        assert_eq!(get_type_name(&Dynamic::from(42_i64)), "integer");
        assert_eq!(get_type_name(&Dynamic::from(3.14_f64)), "float");
        assert_eq!(get_type_name(&Dynamic::from("hello")), "string");
    }

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric(&Dynamic::from(42_i64)));
        assert!(is_numeric(&Dynamic::from(3.14_f64)));
        assert!(is_numeric(&Dynamic::from("123")));
        assert!(is_numeric(&Dynamic::from("3.14")));
        assert!(!is_numeric(&Dynamic::from("hello")));
        assert!(!is_numeric(&Dynamic::from(true)));
    }

    #[test]
    fn test_get_type_name_array() {
        let arr = rhai::Array::new();
        assert_eq!(get_type_name(&Dynamic::from(arr)), "array");
    }

    #[test]
    fn test_get_type_name_map() {
        let map = rhai::Map::new();
        assert_eq!(get_type_name(&Dynamic::from(map)), "object");
    }

    #[test]
    fn test_is_numeric_negative() {
        assert!(is_numeric(&Dynamic::from(-42_i64)));
        assert!(is_numeric(&Dynamic::from(-3.14_f64)));
        assert!(is_numeric(&Dynamic::from("-123")));
    }

    #[test]
    fn test_is_numeric_whitespace() {
        assert!(is_numeric(&Dynamic::from("  123  ")));
        assert!(is_numeric(&Dynamic::from(" 3.14 ")));
    }
}
