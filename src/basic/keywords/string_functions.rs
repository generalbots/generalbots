//! String function keywords for BASIC interpreter
//!
//! This module provides classic BASIC string manipulation functions:
//! - INSTR: Find position of substring within string
//! - IS_NUMERIC: Check if a string can be parsed as a number

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// Register the INSTR keyword with the Rhai engine
///
/// INSTR returns the 1-based position of a substring within a string.
/// Returns 0 if not found.
///
/// Syntax:
///   position = INSTR(string, substring)
///   position = INSTR(start, string, substring)
pub fn instr_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // Two-argument version: INSTR(string, substring)
    engine.register_fn("INSTR", |haystack: &str, needle: &str| -> i64 {
        instr_impl(1, haystack, needle)
    });

    // Alias with lowercase
    engine.register_fn("instr", |haystack: &str, needle: &str| -> i64 {
        instr_impl(1, haystack, needle)
    });

    // Three-argument version: INSTR(start, string, substring)
    engine.register_fn("INSTR", |start: i64, haystack: &str, needle: &str| -> i64 {
        instr_impl(start, haystack, needle)
    });

    engine.register_fn("instr", |start: i64, haystack: &str, needle: &str| -> i64 {
        instr_impl(start, haystack, needle)
    });

    debug!("Registered INSTR keyword");
}

/// Implementation of INSTR function
///
/// # Arguments
/// * `start` - 1-based starting position for the search
/// * `haystack` - The string to search in
/// * `needle` - The substring to find
///
/// # Returns
/// * 1-based position of the first occurrence, or 0 if not found
pub fn instr_impl(start: i64, haystack: &str, needle: &str) -> i64 {
    // Handle edge cases
    if haystack.is_empty() || needle.is_empty() {
        return 0;
    }

    // Convert 1-based start to 0-based index
    let start_idx = if start < 1 { 0 } else { (start - 1) as usize };

    // Ensure start is within bounds
    if start_idx >= haystack.len() {
        return 0;
    }

    // Search from the starting position
    match haystack[start_idx..].find(needle) {
        Some(pos) => (start_idx + pos + 1) as i64, // Convert back to 1-based
        None => 0,
    }
}

/// Register the IS_NUMERIC / IS NUMERIC keyword with the Rhai engine
///
/// IS_NUMERIC tests whether a string can be converted to a number.
///
/// Syntax:
///   result = IS NUMERIC(value)
///   result = IS_NUMERIC(value)
pub fn is_numeric_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // Register as IS_NUMERIC (with underscore for Rhai compatibility)
    engine.register_fn("IS_NUMERIC", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    // Lowercase variant
    engine.register_fn("is_numeric", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    // Also register as ISNUMERIC (single word)
    engine.register_fn("ISNUMERIC", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    engine.register_fn("isnumeric", |value: &str| -> bool {
        is_numeric_impl(value)
    });

    // Handle Dynamic type for flexibility
    engine.register_fn("IS_NUMERIC", |value: Dynamic| -> bool {
        match value.clone().into_string() {
            Ok(s) => is_numeric_impl(&s),
            Err(_) => {
                // If it's already a number, return true
                value.is::<i64>() || value.is::<f64>()
            }
        }
    });

    debug!("Registered IS_NUMERIC keyword");
}

/// Implementation of IS_NUMERIC function
///
/// # Arguments
/// * `value` - The string value to test
///
/// # Returns
/// * `true` if the value can be parsed as a number
/// * `false` otherwise
pub fn is_numeric_impl(value: &str) -> bool {
    let trimmed = value.trim();

    // Empty string is not numeric
    if trimmed.is_empty() {
        return false;
    }

    // Try parsing as integer first
    if trimmed.parse::<i64>().is_ok() {
        return true;
    }

    // Try parsing as float
    if trimmed.parse::<f64>().is_ok() {
        return true;
    }

    false
}

/// Register the NOT operator for boolean negation
/// This enables `NOT IS_NUMERIC(x)` syntax
pub fn not_operator(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("NOT", |value: bool| -> bool { !value });

    engine.register_fn("not", |value: bool| -> bool { !value });

    debug!("Registered NOT operator");
}

/// Register OR operator for boolean operations
/// This enables `a = "" OR NOT IS_NUMERIC(a)` syntax
pub fn logical_operators(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // OR operator
    engine.register_fn("OR", |a: bool, b: bool| -> bool { a || b });
    engine.register_fn("or", |a: bool, b: bool| -> bool { a || b });

    // AND operator
    engine.register_fn("AND", |a: bool, b: bool| -> bool { a && b });
    engine.register_fn("and", |a: bool, b: bool| -> bool { a && b });

    debug!("Registered logical operators (OR, AND)");
}

/// Register the LOWER function for string case conversion
pub fn lower_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LOWER", |s: &str| -> String { s.to_lowercase() });

    engine.register_fn("lower", |s: &str| -> String { s.to_lowercase() });

    engine.register_fn("LCASE", |s: &str| -> String { s.to_lowercase() });

    debug!("Registered LOWER/LCASE keyword");
}

/// Register the UPPER function for string case conversion
pub fn upper_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("UPPER", |s: &str| -> String { s.to_uppercase() });

    engine.register_fn("upper", |s: &str| -> String { s.to_uppercase() });

    engine.register_fn("UCASE", |s: &str| -> String { s.to_uppercase() });

    debug!("Registered UPPER/UCASE keyword");
}

/// Register the LEN function for string length
pub fn len_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LEN", |s: &str| -> i64 { s.len() as i64 });

    engine.register_fn("len", |s: &str| -> i64 { s.len() as i64 });

    debug!("Registered LEN keyword");
}

/// Register the TRIM function for whitespace removal
pub fn trim_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TRIM", |s: &str| -> String { s.trim().to_string() });

    engine.register_fn("trim", |s: &str| -> String { s.trim().to_string() });

    engine.register_fn("LTRIM", |s: &str| -> String { s.trim_start().to_string() });

    engine.register_fn("ltrim", |s: &str| -> String { s.trim_start().to_string() });

    engine.register_fn("RTRIM", |s: &str| -> String { s.trim_end().to_string() });

    engine.register_fn("rtrim", |s: &str| -> String { s.trim_end().to_string() });

    debug!("Registered TRIM/LTRIM/RTRIM keywords");
}

/// Register the LEFT function for extracting left portion of string
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

/// Register the RIGHT function for extracting right portion of string
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

/// Register the MID function for extracting substring
pub fn mid_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // MID(string, start) - from start to end
    engine.register_fn("MID", |s: &str, start: i64| -> String {
        let start_idx = if start < 1 { 0 } else { (start - 1) as usize };
        s.chars().skip(start_idx).collect()
    });

    // MID(string, start, length) - from start for length chars
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

/// Register the REPLACE function for string replacement
pub fn replace_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("REPLACE", |s: &str, find: &str, replace: &str| -> String {
        s.replace(find, replace)
    });

    engine.register_fn("replace", |s: &str, find: &str, replace: &str| -> String {
        s.replace(find, replace)
    });

    debug!("Registered REPLACE keyword");
}

/// Register all string functions
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
