use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;

/// Registers the ISEMPTY function for checking if a value is empty
///
/// BASIC Syntax:
///   result = ISEMPTY(value)
///
/// Returns TRUE if value is:
///   - An empty string ""
///   - An empty array []
///   - An empty map {}
///   - Unit/null type
///
/// Examples:
///   IF ISEMPTY(name) THEN
///     TALK "Please provide your name"
///   END IF
///
///   empty_check = ISEMPTY("")      ' Returns TRUE
///   empty_check = ISEMPTY("hello") ' Returns FALSE
///   empty_check = ISEMPTY([])      ' Returns TRUE
pub fn isempty_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // ISEMPTY - uppercase version
    engine.register_fn("ISEMPTY", |value: Dynamic| -> bool { check_empty(&value) });

    // isempty - lowercase version
    engine.register_fn("isempty", |value: Dynamic| -> bool { check_empty(&value) });

    // IsEmpty - mixed case version
    engine.register_fn("IsEmpty", |value: Dynamic| -> bool { check_empty(&value) });

    debug!("Registered ISEMPTY keyword");
}

/// Helper function to check if a Dynamic value is empty
fn check_empty(value: &Dynamic) -> bool {
    // Check for unit/null type
    if value.is_unit() {
        return true;
    }

    // Check for empty string
    if value.is_string() {
        if let Some(s) = value.clone().into_string().ok() {
            return s.is_empty();
        }
    }

    // Check for empty array
    if value.is_array() {
        if let Ok(arr) = value.clone().into_array() {
            return arr.is_empty();
        }
    }

    // Check for empty map
    if value.is_map() {
        if let Some(map) = value.clone().try_cast::<Map>() {
            return map.is_empty();
        }
    }

    // Check for special "empty" boolean state
    if value.is_bool() {
        // Boolean false is not considered "empty" - it's a valid value
        return false;
    }

    // Numbers are never empty
    if value.is_int() || value.is_float() {
        return false;
    }

    false
}
