//! UNIQUE - Remove duplicate values from an array
//!
//! BASIC Syntax:
//!   result = UNIQUE(array)
//!
//! Examples:
//!   numbers = [1, 2, 2, 3, 3, 3, 4]
//!   unique_numbers = UNIQUE(numbers)  ' Returns [1, 2, 3, 4]
//!
//!   names = ["Alice", "Bob", "Alice", "Charlie"]
//!   unique_names = UNIQUE(names)  ' Returns ["Alice", "Bob", "Charlie"]

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Engine};
use std::collections::HashSet;
use std::sync::Arc;

/// Register the UNIQUE function for removing duplicate values from arrays
pub fn unique_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // UNIQUE - uppercase version
    engine.register_fn("UNIQUE", |arr: Array| -> Array { unique_array(arr) });

    // unique - lowercase version
    engine.register_fn("unique", |arr: Array| -> Array { unique_array(arr) });

    // DISTINCT - SQL-style alias
    engine.register_fn("DISTINCT", |arr: Array| -> Array { unique_array(arr) });

    engine.register_fn("distinct", |arr: Array| -> Array { unique_array(arr) });

    debug!("Registered UNIQUE keyword");
}

/// Helper function to remove duplicates from an array
/// Preserves the order of first occurrence
fn unique_array(arr: Array) -> Array {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Array::new();

    for item in arr {
        // Use string representation for comparison
        // This handles most common types (strings, numbers, bools)
        let key = item.to_string();

        if !seen.contains(&key) {
            seen.insert(key);
            result.push(item);
        }
    }

    result
}
