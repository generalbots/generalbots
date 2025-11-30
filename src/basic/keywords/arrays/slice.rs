//! SLICE function for extracting portions of arrays
//!
//! BASIC Syntax:
//!   result = SLICE(array, start)           ' From start to end
//!   result = SLICE(array, start, end)      ' From start to end (exclusive)
//!
//! Examples:
//!   arr = [1, 2, 3, 4, 5]
//!   part = SLICE(arr, 2)        ' Returns [3, 4, 5] (0-based index)
//!   part = SLICE(arr, 1, 3)     ' Returns [2, 3] (indices 1 and 2)

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

/// Register the SLICE keyword for array slicing
pub fn slice_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // SLICE with start only (from start to end)
    engine.register_fn("SLICE", |arr: Array, start: i64| -> Array {
        slice_array(&arr, start, None)
    });

    engine.register_fn("slice", |arr: Array, start: i64| -> Array {
        slice_array(&arr, start, None)
    });

    // SLICE with start and end
    engine.register_fn("SLICE", |arr: Array, start: i64, end: i64| -> Array {
        slice_array(&arr, start, Some(end))
    });

    engine.register_fn("slice", |arr: Array, start: i64, end: i64| -> Array {
        slice_array(&arr, start, Some(end))
    });

    // SUBARRAY alias
    engine.register_fn("SUBARRAY", |arr: Array, start: i64, end: i64| -> Array {
        slice_array(&arr, start, Some(end))
    });

    // MID for arrays (similar to MID$ for strings)
    engine.register_fn(
        "MID_ARRAY",
        |arr: Array, start: i64, length: i64| -> Array {
            let end = start + length;
            slice_array(&arr, start, Some(end))
        },
    );

    // TAKE - take first n elements
    engine.register_fn("TAKE", |arr: Array, count: i64| -> Array {
        slice_array(&arr, 0, Some(count))
    });

    engine.register_fn("take", |arr: Array, count: i64| -> Array {
        slice_array(&arr, 0, Some(count))
    });

    // DROP - drop first n elements
    engine.register_fn("DROP", |arr: Array, count: i64| -> Array {
        slice_array(&arr, count, None)
    });

    engine.register_fn("drop", |arr: Array, count: i64| -> Array {
        slice_array(&arr, count, None)
    });

    // HEAD - first element
    engine.register_fn("HEAD", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });

    engine.register_fn("head", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });

    // TAIL - all elements except first
    engine.register_fn("TAIL", |arr: Array| -> Array {
        if arr.len() <= 1 {
            Array::new()
        } else {
            arr[1..].to_vec()
        }
    });

    engine.register_fn("tail", |arr: Array| -> Array {
        if arr.len() <= 1 {
            Array::new()
        } else {
            arr[1..].to_vec()
        }
    });

    // INIT - all elements except last
    engine.register_fn("INIT", |arr: Array| -> Array {
        if arr.is_empty() {
            Array::new()
        } else {
            arr[..arr.len() - 1].to_vec()
        }
    });

    // LAST_ELEM - last element
    engine.register_fn("LAST", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });

    debug!("Registered SLICE keyword");
}

/// Helper function to slice an array
fn slice_array(arr: &Array, start: i64, end: Option<i64>) -> Array {
    let len = arr.len() as i64;

    // Handle negative indices (count from end)
    let start_idx = if start < 0 {
        (len + start).max(0) as usize
    } else {
        (start as usize).min(arr.len())
    };

    let end_idx = match end {
        Some(e) => {
            if e < 0 {
                (len + e).max(0) as usize
            } else {
                (e as usize).min(arr.len())
            }
        }
        None => arr.len(),
    };

    if start_idx >= end_idx || start_idx >= arr.len() {
        return Array::new();
    }

    arr[start_idx..end_idx].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_array() -> Array {
        vec![
            Dynamic::from(1),
            Dynamic::from(2),
            Dynamic::from(3),
            Dynamic::from(4),
            Dynamic::from(5),
        ]
    }

    #[test]
    fn test_slice_from_start() {
        let arr = make_test_array();
        let result = slice_array(&arr, 2, None);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].as_int().unwrap(), 3);
    }

    #[test]
    fn test_slice_with_end() {
        let arr = make_test_array();
        let result = slice_array(&arr, 1, Some(3));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].as_int().unwrap(), 2);
        assert_eq!(result[1].as_int().unwrap(), 3);
    }

    #[test]
    fn test_slice_negative_start() {
        let arr = make_test_array();
        let result = slice_array(&arr, -2, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].as_int().unwrap(), 4);
        assert_eq!(result[1].as_int().unwrap(), 5);
    }

    #[test]
    fn test_slice_negative_end() {
        let arr = make_test_array();
        let result = slice_array(&arr, 0, Some(-2));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].as_int().unwrap(), 1);
        assert_eq!(result[2].as_int().unwrap(), 3);
    }

    #[test]
    fn test_slice_out_of_bounds() {
        let arr = make_test_array();
        let result = slice_array(&arr, 10, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_slice_empty_range() {
        let arr = make_test_array();
        let result = slice_array(&arr, 3, Some(2));
        assert!(result.is_empty());
    }
}
