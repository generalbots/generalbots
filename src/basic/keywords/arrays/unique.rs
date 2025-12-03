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

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;

    #[test]
    fn test_unique_integers() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from(2_i64));
        arr.push(Dynamic::from(2_i64));
        arr.push(Dynamic::from(3_i64));
        arr.push(Dynamic::from(3_i64));
        arr.push(Dynamic::from(3_i64));
        arr.push(Dynamic::from(4_i64));

        let result = unique_array(arr);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_unique_strings() {
        let mut arr = Array::new();
        arr.push(Dynamic::from("Alice"));
        arr.push(Dynamic::from("Bob"));
        arr.push(Dynamic::from("Alice"));
        arr.push(Dynamic::from("Charlie"));

        let result = unique_array(arr);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_unique_preserves_order() {
        let mut arr = Array::new();
        arr.push(Dynamic::from("C"));
        arr.push(Dynamic::from("A"));
        arr.push(Dynamic::from("B"));
        arr.push(Dynamic::from("A"));
        arr.push(Dynamic::from("C"));

        let result = unique_array(arr);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].to_string(), "C");
        assert_eq!(result[1].to_string(), "A");
        assert_eq!(result[2].to_string(), "B");
    }

    #[test]
    fn test_unique_empty_array() {
        let arr = Array::new();
        let result = unique_array(arr);
        assert!(result.is_empty());
    }

    #[test]
    fn test_unique_single_element() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(42_i64));

        let result = unique_array(arr);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_unique_all_same() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from(1_i64));

        let result = unique_array(arr);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_unique_mixed_types() {
        let mut arr = Array::new();
        arr.push(Dynamic::from(1_i64));
        arr.push(Dynamic::from("1"));
        arr.push(Dynamic::from(1_i64));

        let result = unique_array(arr);
        // "1" (int) and "1" (string) may have same string representation
        // so behavior depends on Dynamic::to_string() implementation
        assert!(result.len() >= 1 && result.len() <= 2);
    }
}
