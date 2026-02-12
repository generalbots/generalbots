use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn instr_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("INSTR", |haystack: &str, needle: &str| -> i64 {
        instr_impl(1, haystack, needle)
    });

    engine.register_fn("instr", |haystack: &str, needle: &str| -> i64 {
        instr_impl(1, haystack, needle)
    });

    engine.register_fn("INSTR", |start: i64, haystack: &str, needle: &str| -> i64 {
        instr_impl(start, haystack, needle)
    });

    engine.register_fn("instr", |start: i64, haystack: &str, needle: &str| -> i64 {
        instr_impl(start, haystack, needle)
    });

    debug!("Registered INSTR keyword");
}

pub fn instr_impl(start: i64, haystack: &str, needle: &str) -> i64 {
    if haystack.is_empty() || needle.is_empty() {
        return 0;
    }

    let start_idx = if start < 1 { 0 } else { (start - 1) as usize };

    if start_idx >= haystack.len() {
        return 0;
    }

    match haystack[start_idx..].find(needle) {
        Some(pos) => (start_idx + pos + 1) as i64,
        None => 0,
    }
}

pub fn is_numeric_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("IS_NUMERIC", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    engine.register_fn("is_numeric", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    engine.register_fn("ISNUMERIC", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    engine.register_fn("isnumeric", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    engine.register_fn("IS_NUMERIC", |value: Dynamic| -> bool {
        match value.clone().into_string() {
            Ok(s) => is_numeric_impl(&s),
            Err(_) => value.is::<i64>() || value.is::<f64>(),
        }
    });

    debug!("Registered IS_NUMERIC keyword");
}

pub fn is_numeric_impl(value: &str) -> bool {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return false;
    }

    if trimmed.parse::<i64>().is_ok() {
        return true;
    }

    if trimmed.parse::<f64>().is_ok() {
        return true;
    }

    false
}

pub fn not_operator(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("NOT", |value: bool| -> bool { !value });

    engine.register_fn("not", |value: bool| -> bool { !value });

    debug!("Registered NOT operator");
}

pub fn logical_operators(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("OR", |a: bool, b: bool| -> bool { a || b });
    engine.register_fn("or", |a: bool, b: bool| -> bool { a || b });

    engine.register_fn("AND", |a: bool, b: bool| -> bool { a && b });
    engine.register_fn("and", |a: bool, b: bool| -> bool { a && b });

    debug!("Registered logical operators (OR, AND)");
}

pub fn lower_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LOWER", |s: &str| -> String { s.to_lowercase() });

    engine.register_fn("lower", |s: &str| -> String { s.to_lowercase() });

    engine.register_fn("LCASE", |s: &str| -> String { s.to_lowercase() });

    debug!("Registered LOWER/LCASE keyword");
}

pub fn upper_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("UPPER", |s: &str| -> String { s.to_uppercase() });

    engine.register_fn("upper", |s: &str| -> String { s.to_uppercase() });

    engine.register_fn("UCASE", |s: &str| -> String { s.to_uppercase() });

    debug!("Registered UPPER/UCASE keyword");
}

pub fn len_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LEN", |s: &str| -> i64 { s.len() as i64 });

    engine.register_fn("len", |s: &str| -> i64 { s.len() as i64 });

    debug!("Registered LEN keyword");
}

pub fn trim_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TRIM", |s: &str| -> String { s.trim().to_string() });

    engine.register_fn("trim", |s: &str| -> String { s.trim().to_string() });

    engine.register_fn("LTRIM", |s: &str| -> String { s.trim_start().to_string() });

    engine.register_fn("ltrim", |s: &str| -> String { s.trim_start().to_string() });

    engine.register_fn("RTRIM", |s: &str| -> String { s.trim_end().to_string() });

    engine.register_fn("rtrim", |s: &str| -> String { s.trim_end().to_string() });

    debug!("Registered TRIM/LTRIM/RTRIM keywords");
}

pub fn left_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LEFT", |s: &str, count: i64| -> String {
        let count = count.max(0) as usize;
        s.chars().take(count).collect()
    });

    engine.register_fn("left", |s: &str, count: i64| -> String {
        let count = count.max(0) as usize;
        s.chars().take(count).collect()
    });

    debug!("Registered LEFT keyword");
}

pub fn right_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("RIGHT", |s: &str, count: i64| -> String {
        let count = count.max(0) as usize;
        let len = s.chars().count();
        if count >= len {
            s.to_string()
        } else {
            s.chars().skip(len - count).collect()
        }
    });

    engine.register_fn("right", |s: &str, count: i64| -> String {
        let count = count.max(0) as usize;
        let len = s.chars().count();
        if count >= len {
            s.to_string()
        } else {
            s.chars().skip(len - count).collect()
        }
    });

    debug!("Registered RIGHT keyword");
}

pub fn mid_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MID", |s: &str, start: i64| -> String {
        let start_idx = if start < 1 { 0 } else { (start - 1) as usize };
        s.chars().skip(start_idx).collect()
    });

    engine.register_fn("MID", |s: &str, start: i64, length: i64| -> String {
        let start_idx = if start < 1 { 0 } else { (start - 1) as usize };
        let len = length.max(0) as usize;
        s.chars().skip(start_idx).take(len).collect()
    });

    engine.register_fn("mid", |s: &str, start: i64| -> String {
        let start_idx = if start < 1 { 0 } else { (start - 1) as usize };
        s.chars().skip(start_idx).collect()
    });

    engine.register_fn("mid", |s: &str, start: i64, length: i64| -> String {
        let start_idx = if start < 1 { 0 } else { (start - 1) as usize };
        let len = length.max(0) as usize;
        s.chars().skip(start_idx).take(len).collect()
    });

    debug!("Registered MID keyword");
}

pub fn replace_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("REPLACE", |s: &str, find: &str, replace: &str| -> String {
        s.replace(find, replace)
    });

    engine.register_fn("replace", |s: &str, find: &str, replace: &str| -> String {
        s.replace(find, replace)
    });

    debug!("Registered REPLACE keyword");
}

pub fn register_string_functions(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    instr_keyword(&state, user.clone(), engine);
    is_numeric_keyword(&state, user.clone(), engine);
    not_operator(&state, user.clone(), engine);
    logical_operators(&state, user.clone(), engine);
    lower_keyword(&state, user.clone(), engine);
    upper_keyword(&state, user.clone(), engine);
    len_keyword(&state, user.clone(), engine);
    trim_keyword(&state, user.clone(), engine);
    left_keyword(&state, user.clone(), engine);
    right_keyword(&state, user.clone(), engine);
    mid_keyword(&state, user.clone(), engine);
    replace_keyword(&state, user, engine);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instr_basic() {
        assert_eq!(instr_impl(1, "Hello, World!", "World"), 8);
        assert_eq!(instr_impl(1, "Hello, World!", "o"), 5);
        assert_eq!(instr_impl(1, "Hello, World!", "xyz"), 0);
    }

    #[test]
    fn test_instr_with_start() {
        assert_eq!(instr_impl(1, "one two one", "one"), 1);
        assert_eq!(instr_impl(2, "one two one", "one"), 9);
        assert_eq!(instr_impl(10, "one two one", "one"), 0);
    }

    #[test]
    fn test_instr_edge_cases() {
        assert_eq!(instr_impl(1, "", "test"), 0);
        assert_eq!(instr_impl(1, "test", ""), 0);
        assert_eq!(instr_impl(1, "", ""), 0);
    }

    #[test]
    fn test_is_numeric_integers() {
        assert!(is_numeric_impl("42"));
        assert!(is_numeric_impl("-17"));
        assert!(is_numeric_impl("0"));
        assert!(is_numeric_impl("  42  "));
    }

    #[test]
    fn test_is_numeric_decimals() {
        assert!(is_numeric_impl("3.14"));
        assert!(is_numeric_impl("-0.5"));
        assert!(is_numeric_impl("0.0"));
    }

    #[test]
    fn test_is_numeric_scientific() {
        assert!(is_numeric_impl("1e10"));
        assert!(is_numeric_impl("2.5E-3"));
        assert!(is_numeric_impl("-1.5e+2"));
    }

    #[test]
    fn test_is_numeric_invalid() {
        assert!(!is_numeric_impl(""));
        assert!(!is_numeric_impl("abc"));
        assert!(!is_numeric_impl("12abc"));
        assert!(!is_numeric_impl("$100"));
    }

    #[test]
    fn test_upper_lower() {
        assert_eq!("hello".to_uppercase(), "HELLO");
        assert_eq!("HELLO".to_lowercase(), "hello");
    }

    #[test]
    fn test_trim() {
        assert_eq!("  hello  ".trim(), "hello");
        assert_eq!("  hello  ".trim_start(), "hello  ");
        assert_eq!("  hello  ".trim_end(), "  hello");
    }

    #[test]
    fn test_left_right() {
        let s = "Hello, World!";
        let left: String = s.chars().take(5).collect();
        assert_eq!(left, "Hello");

        let len = s.chars().count();
        let right: String = s.chars().skip(len - 6).collect();
        assert_eq!(right, "World!");
    }

    #[test]
    fn test_mid() {
        let s = "Hello, World!";
        let mid: String = s.chars().skip(7).take(5).collect();
        assert_eq!(mid, "World");
    }

    #[test]
    fn test_replace() {
        assert_eq!("Hello, World!".replace("World", "Rust"), "Hello, Rust!");
    }

    #[test]
    fn test_len() {
        assert_eq!("Hello".len(), 5);
        assert_eq!("".len(), 0);
    }
}
