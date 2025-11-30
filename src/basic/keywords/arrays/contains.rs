//! CONTAINS function for checking array membership
//!
//! BASIC Syntax:
//!   result = CONTAINS(array, value)
//!
//! Returns TRUE if the value exists in the array, FALSE otherwise.
//!
//! Examples:
//!   names = ["Alice", "Bob", "Charlie"]
//!   IF CONTAINS(names, "Bob") THEN
//!     TALK "Found Bob!"
//!   END IF

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

/// Registers the CONTAINS function for array membership checking
pub fn contains_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // CONTAINS - uppercase version
    engine.register_fn("CONTAINS", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    // contains - lowercase version
    engine.register_fn("contains", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    // IN_ARRAY - alternative name (PHP style)
    engine.register_fn("IN_ARRAY", |value: Dynamic, arr: Array| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("in_array", |value: Dynamic, arr: Array| -> bool {
        array_contains(&arr, &value)
    });

    // INCLUDES - JavaScript style
    engine.register_fn("INCLUDES", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("includes", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    // HAS - short form
    engine.register_fn("HAS", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    engine.register_fn("has", |arr: Array, value: Dynamic| -> bool {
        array_contains(&arr, &value)
    });

    debug!("Registered CONTAINS keyword");
}

/// Helper function to check if an array contains a value
fn array_contains(arr: &Array, value: &Dynamic) -> bool {
    let search_str = value.to_string();

    for item in arr {
        // Try exact type match first
        if items_equal(item, value) {
            return true;
        }

        // Fall back to string comparison
        if item.to_string() == search_str {
            return true;
        }
    }

    false
}

/// Helper function to compare two Dynamic values
fn items_equal(a: &Dynamic, b: &Dynamic) -> bool {
    // Both integers
    if a.is_int() && b.is_int() {
        return a.as_int().unwrap_or(0) == b.as_int().unwrap_or(1);
    }

    // Both floats
    if a.is_float() && b.is_float() {
        let af = a.as_float().unwrap_or(0.0);
        let bf = b.as_float().unwrap_or(1.0);
        return (af - bf).abs() < f64::EPSILON;
    }

    // Int and float comparison
    if a.is_int() && b.is_float() {
        let af = a.as_int().unwrap_or(0) as f64;
        let bf = b.as_float().unwrap_or(1.0);
        return (af - bf).abs() < f64::EPSILON;
    }

    if a.is_float() && b.is_int() {
        let af = a.as_float().unwrap_or(0.0);
        let bf = b.as_int().unwrap_or(1) as f64;
        return (af - bf).abs() < f64::EPSILON;
    }

    // Both booleans
    if a.is_bool() && b.is_bool() {
        return a.as_bool().unwrap_or(false) == b.as_bool().unwrap_or(true);
    }

    // Both strings
    if a.is_string() && b.is_string() {
        return a.clone().into_string().unwrap_or_default()
            == b.clone().into_string().unwrap_or_default();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_string() {
        let mut arr = Array::new();
        arr.push(Dynamic::from("Alice"));
        arr.push(Dynamic::from("Bob"));
        arr.push(Dynamic::from("Charlie"));

        assert!(array_contains(&arr, &Dynamic::from("Bob")));
        assert!(!array_contains(&arr, &Dynamic::from("David")));
    }

    #[test]
    fn test_contains_integer() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from(2_i64));
        arr.push(Dynamic::from(3_i64));

        assert!(array_contains(&arr, &Dynamic::from(2_i64)));
        assert!(!array_contains(&arr, &Dynamic::from(5_i64)));
    }

    #[test]
    fn test_contains_float() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(1.5_f64));
        arr.push(Dynamic::from(2.5_f64));
        arr.push(Dynamic::from(3.5_f64));

        assert!(array_contains(&arr, &Dynamic::from(2.5_f64)));
        assert!(!array_contains(&arr, &Dynamic::from(4.5_f64)));
    }

    #[test]
    fn test_contains_bool() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(true));
        arr.push(Dynamic::from(false));

        assert!(array_contains(&arr, &Dynamic::from(true)));
        assert!(array_contains(&arr, &Dynamic::from(false)));
    }

    #[test]
    fn test_contains_empty_array() {
        let arr = Array::new();
        assert!(!array_contains(&arr, &Dynamic::from("anything")));
    }

    #[test]
    fn test_items_equal_integers() {
        assert!(items_equal(&Dynamic::from(5_i64), &Dynamic::from(5_i64)));
        assert!(!items_equal(&Dynamic::from(5_i64), &Dynamic::from(6_i64)));
    }

    #[test]
    fn test_items_equal_strings() {
        assert!(items_equal(
            &Dynamic::from("hello"),
            &Dynamic::from("hello")
        ));
        assert!(!items_equal(
            &Dynamic::from("hello"),
            &Dynamic::from("world")
        ));
    }
}
