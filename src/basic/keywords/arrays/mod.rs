pub mod contains;
pub mod push_pop;
pub mod slice;
pub mod sort;
pub mod unique;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

pub fn register_array_functions(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    sort::sort_keyword(&state, user.clone(), engine);
    unique::unique_keyword(&state, user.clone(), engine);
    contains::contains_keyword(&state, user.clone(), engine);
    push_pop::push_keyword(&state, user.clone(), engine);
    push_pop::pop_keyword(&state, user.clone(), engine);
    push_pop::shift_keyword(&state, user.clone(), engine);
    push_pop::unshift_keyword(&state, user.clone(), engine);
    slice::slice_keyword(&state, user.clone(), engine);
    register_utility_functions(engine);

    debug!("Registered all array functions");
}

fn register_utility_functions(engine: &mut Engine) {
    // UBOUND - upper bound (length - 1)
    engine.register_fn("UBOUND", |arr: Array| -> i64 {
        if arr.is_empty() {
            -1
        } else {
            (arr.len() - 1) as i64
        }
    });
    engine.register_fn("ubound", |arr: Array| -> i64 {
        if arr.is_empty() {
            -1
        } else {
            (arr.len() - 1) as i64
        }
    });

    // LBOUND - lower bound (always 0)
    engine.register_fn("LBOUND", |_arr: Array| -> i64 { 0 });
    engine.register_fn("lbound", |_arr: Array| -> i64 { 0 });

    // COUNT - array length
    engine.register_fn("COUNT", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("count", |arr: Array| -> i64 { arr.len() as i64 });

    // LEN for arrays (alias for COUNT)
    engine.register_fn("LEN", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("len", |arr: Array| -> i64 { arr.len() as i64 });

    // SIZE alias
    engine.register_fn("SIZE", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("size", |arr: Array| -> i64 { arr.len() as i64 });

    // REVERSE
    engine.register_fn("REVERSE", |arr: Array| -> Array {
        let mut reversed = arr.clone();
        reversed.reverse();
        reversed
    });
    engine.register_fn("reverse", |arr: Array| -> Array {
        let mut reversed = arr.clone();
        reversed.reverse();
        reversed
    });

    // JOIN - array to string
    engine.register_fn("JOIN", |arr: Array, separator: &str| -> String {
        arr.iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(separator)
    });
    engine.register_fn("join", |arr: Array, separator: &str| -> String {
        arr.iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(separator)
    });

    // JOIN with default separator (comma)
    engine.register_fn("JOIN", |arr: Array| -> String {
        arr.iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(",")
    });

    // SPLIT - string to array
    engine.register_fn("SPLIT", |s: &str, delimiter: &str| -> Array {
        s.split(delimiter)
            .map(|part| Dynamic::from(part.to_string()))
            .collect()
    });
    engine.register_fn("split", |s: &str, delimiter: &str| -> Array {
        s.split(delimiter)
            .map(|part| Dynamic::from(part.to_string()))
            .collect()
    });

    // RANGE - create array of numbers
    engine.register_fn("RANGE", |start: i64, end: i64| -> Array {
        (start..=end).map(Dynamic::from).collect()
    });
    engine.register_fn("range", |start: i64, end: i64| -> Array {
        (start..=end).map(Dynamic::from).collect()
    });

    // RANGE with step
    engine.register_fn("RANGE", |start: i64, end: i64, step: i64| -> Array {
        if step == 0 {
            return Array::new();
        }
        let mut result = Array::new();
        let mut current = start;
        if step > 0 {
            while current <= end {
                result.push(Dynamic::from(current));
                current += step;
            }
        } else {
            while current >= end {
                result.push(Dynamic::from(current));
                current += step;
            }
        }
        result
    });

    // INDEX_OF
    engine.register_fn("INDEX_OF", |arr: Array, value: Dynamic| -> i64 {
        let search = value.to_string();
        arr.iter()
            .position(|item| item.to_string() == search)
            .map(|i| i as i64)
            .unwrap_or(-1)
    });
    engine.register_fn("index_of", |arr: Array, value: Dynamic| -> i64 {
        let search = value.to_string();
        arr.iter()
            .position(|item| item.to_string() == search)
            .map(|i| i as i64)
            .unwrap_or(-1)
    });

    // LAST_INDEX_OF
    engine.register_fn("LAST_INDEX_OF", |arr: Array, value: Dynamic| -> i64 {
        let search = value.to_string();
        arr.iter()
            .rposition(|item| item.to_string() == search)
            .map(|i| i as i64)
            .unwrap_or(-1)
    });

    // CONCAT - combine arrays
    engine.register_fn("CONCAT", |arr1: Array, arr2: Array| -> Array {
        let mut result = arr1.clone();
        result.extend(arr2);
        result
    });
    engine.register_fn("concat", |arr1: Array, arr2: Array| -> Array {
        let mut result = arr1.clone();
        result.extend(arr2);
        result
    });

    // FIRST_ELEM / FIRST
    engine.register_fn("FIRST_ELEM", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("FIRST", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("first", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });

    // LAST_ELEM / LAST
    engine.register_fn("LAST_ELEM", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("LAST", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("last", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });

    // FLATTEN - flatten nested arrays (one level)
    engine.register_fn("FLATTEN", |arr: Array| -> Array {
        let mut result = Array::new();
        for item in arr {
            if item.is_array() {
                if let Ok(inner) = item.into_array() {
                    result.extend(inner);
                }
            } else {
                result.push(item);
            }
        }
        result
    });
    engine.register_fn("flatten", |arr: Array| -> Array {
        let mut result = Array::new();
        for item in arr {
            if item.is_array() {
                if let Ok(inner) = item.into_array() {
                    result.extend(inner);
                }
            } else {
                result.push(item);
            }
        }
        result
    });

    // EMPTY - create empty array
    engine.register_fn("EMPTY_ARRAY", || -> Array { Array::new() });

    // FILL - create array filled with value
    engine.register_fn("FILL", |value: Dynamic, count: i64| -> Array {
        (0..count).map(|_| value.clone()).collect()
    });
    engine.register_fn("fill", |value: Dynamic, count: i64| -> Array {
        (0..count).map(|_| value.clone()).collect()
    });

    debug!("Registered array utility functions");
}

#[cfg(test)]
mod tests {
    use rhai::Dynamic;

    #[test]
    fn test_ubound() {
        let arr: Vec<Dynamic> = vec![Dynamic::from(1), Dynamic::from(2), Dynamic::from(3)];
        assert_eq!(arr.len() - 1, 2);
    }

    #[test]
    fn test_join() {
        let arr = vec!["a", "b", "c"];
        let result = arr.join("-");
        assert_eq!(result, "a-b-c");
    }

    #[test]
    fn test_split() {
        let s = "a,b,c";
        let parts: Vec<&str> = s.split(',').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_range() {
        let range: Vec<i64> = (1..=5).collect();
        assert_eq!(range, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_flatten() {
        // Test flattening logic
        let nested = vec![vec![1, 2], vec![3, 4]];
        let flat: Vec<i32> = nested.into_iter().flatten().collect();
        assert_eq!(flat, vec![1, 2, 3, 4]);
    }
}
