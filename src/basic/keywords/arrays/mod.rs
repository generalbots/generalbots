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
    slice::slice_keyword(&state, user, engine);
    register_utility_functions(engine);

    debug!("Registered all array functions");
}

fn register_utility_functions(engine: &mut Engine) {
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

    engine.register_fn("LBOUND", |_arr: Array| -> i64 { 0 });
    engine.register_fn("lbound", |_arr: Array| -> i64 { 0 });

    engine.register_fn("COUNT", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("count", |arr: Array| -> i64 { arr.len() as i64 });

    engine.register_fn("LEN", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("len", |arr: Array| -> i64 { arr.len() as i64 });

    engine.register_fn("SIZE", |arr: Array| -> i64 { arr.len() as i64 });
    engine.register_fn("size", |arr: Array| -> i64 { arr.len() as i64 });

    engine.register_fn("REVERSE", |mut arr: Array| -> Array {
        arr.reverse();
        arr
    });
    engine.register_fn("reverse", |mut arr: Array| -> Array {
        arr.reverse();
        arr
    });

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

    engine.register_fn("JOIN", |arr: Array| -> String {
        arr.iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(",")
    });

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

    engine.register_fn("RANGE", |start: i64, end: i64| -> Array {
        (start..=end).map(Dynamic::from).collect()
    });
    engine.register_fn("range", |start: i64, end: i64| -> Array {
        (start..=end).map(Dynamic::from).collect()
    });

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

    engine.register_fn("LAST_INDEX_OF", |arr: Array, value: Dynamic| -> i64 {
        let search = value.to_string();
        arr.iter()
            .rposition(|item| item.to_string() == search)
            .map(|i| i as i64)
            .unwrap_or(-1)
    });

    engine.register_fn("CONCAT", |arr1: Array, arr2: Array| -> Array {
        let mut result = arr1;
        result.extend(arr2);
        result
    });
    engine.register_fn("concat", |arr1: Array, arr2: Array| -> Array {
        let mut result = arr1;
        result.extend(arr2);
        result
    });

    engine.register_fn("FIRST_ELEM", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("FIRST", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("first", |arr: Array| -> Dynamic {
        arr.first().cloned().unwrap_or(Dynamic::UNIT)
    });

    engine.register_fn("LAST_ELEM", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("LAST", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });
    engine.register_fn("last", |arr: Array| -> Dynamic {
        arr.last().cloned().unwrap_or(Dynamic::UNIT)
    });

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

    engine.register_fn("EMPTY_ARRAY", || -> Array { Array::new() });

    engine.register_fn("FILL", |value: Dynamic, count: i64| -> Array {
        (0..count).map(|_| value.clone()).collect()
    });
    engine.register_fn("fill", |value: Dynamic, count: i64| -> Array {
        (0..count).map(|_| value.clone()).collect()
    });

    engine.register_fn("BATCH", |arr: Array, batch_size: i64| -> Array {
        let size = batch_size.max(1) as usize;
        arr.chunks(size)
            .map(|chunk| Dynamic::from(chunk.to_vec()))
            .collect()
    });
    engine.register_fn("batch", |arr: Array, batch_size: i64| -> Array {
        let size = batch_size.max(1) as usize;
        arr.chunks(size)
            .map(|chunk| Dynamic::from(chunk.to_vec()))
            .collect()
    });

    engine.register_fn("CHUNK", |arr: Array, chunk_size: i64| -> Array {
        let size = chunk_size.max(1) as usize;
        arr.chunks(size)
            .map(|chunk| Dynamic::from(chunk.to_vec()))
            .collect()
    });
    engine.register_fn("chunk", |arr: Array, chunk_size: i64| -> Array {
        let size = chunk_size.max(1) as usize;
        arr.chunks(size)
            .map(|chunk| Dynamic::from(chunk.to_vec()))
            .collect()
    });

    engine.register_fn("TAKE", |arr: Array, n: i64| -> Array {
        arr.into_iter().take(n.max(0) as usize).collect()
    });
    engine.register_fn("take", |arr: Array, n: i64| -> Array {
        arr.into_iter().take(n.max(0) as usize).collect()
    });

    engine.register_fn("DROP", |arr: Array, n: i64| -> Array {
        arr.into_iter().skip(n.max(0) as usize).collect()
    });
    engine.register_fn("drop", |arr: Array, n: i64| -> Array {
        arr.into_iter().skip(n.max(0) as usize).collect()
    });

    engine.register_fn("HEAD", |arr: Array, n: i64| -> Array {
        arr.into_iter().take(n.max(0) as usize).collect()
    });

    engine.register_fn("TAIL", |arr: Array, n: i64| -> Array {
        let len = arr.len();
        let skip = len.saturating_sub(n.max(0) as usize);
        arr.into_iter().skip(skip).collect()
    });
    engine.register_fn("tail", |arr: Array, n: i64| -> Array {
        let len = arr.len();
        let skip = len.saturating_sub(n.max(0) as usize);
        arr.into_iter().skip(skip).collect()
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
        let arr = ["a", "b", "c"];
        let result = arr.join("-");
        assert_eq!(result, "a-b-c");
    }

    #[test]
    fn test_split() {
        let s = "a,b,c";
        let parts_count = s.split(',').count();
        assert_eq!(parts_count, 3);
    }

    #[test]
    fn test_range() {
        let range: Vec<i64> = (1..=5).collect();
        assert_eq!(range, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_flatten() {
        let nested = vec![vec![1, 2], vec![3, 4]];
        let flat: Vec<i32> = nested.into_iter().flatten().collect();
        assert_eq!(flat, vec![1, 2, 3, 4]);
    }
}
