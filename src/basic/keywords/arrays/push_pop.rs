//! PUSH and POP array manipulation functions
//!
//! PUSH - Add element(s) to the end of an array
//! POP - Remove and return the last element from an array
//! SHIFT - Remove and return the first element from an array
//! UNSHIFT - Add element(s) to the beginning of an array

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

/// PUSH - Add an element to the end of an array
/// Returns the new array with the element added
pub fn push_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // PUSH single element
    engine.register_fn("PUSH", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("push", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    // ARRAY_PUSH alias
    engine.register_fn("ARRAY_PUSH", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    // APPEND alias
    engine.register_fn("APPEND", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    engine.register_fn("append", |mut arr: Array, value: Dynamic| -> Array {
        arr.push(value);
        arr
    });

    debug!("Registered PUSH keyword");
}

/// POP - Remove and return the last element from an array
/// Returns the removed element (or unit if array is empty)
pub fn pop_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // POP - returns the popped element
    engine.register_fn("POP", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    engine.register_fn("pop", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    // ARRAY_POP alias
    engine.register_fn("ARRAY_POP", |mut arr: Array| -> Dynamic {
        arr.pop().unwrap_or(Dynamic::UNIT)
    });

    debug!("Registered POP keyword");
}

/// SHIFT - Remove and return the first element from an array
pub fn shift_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SHIFT", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    engine.register_fn("shift", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    // ARRAY_SHIFT alias
    engine.register_fn("ARRAY_SHIFT", |mut arr: Array| -> Dynamic {
        if arr.is_empty() {
            Dynamic::UNIT
        } else {
            arr.remove(0)
        }
    });

    debug!("Registered SHIFT keyword");
}

/// UNSHIFT - Add element(s) to the beginning of an array
pub fn unshift_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("UNSHIFT", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    engine.register_fn("unshift", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    // PREPEND alias
    engine.register_fn("PREPEND", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    engine.register_fn("prepend", |mut arr: Array, value: Dynamic| -> Array {
        arr.insert(0, value);
        arr
    });

    debug!("Registered UNSHIFT keyword");
}

#[cfg(test)]
mod tests {
    use rhai::{Array, Dynamic};

    #[test]
    fn test_push() {
        let mut arr: Array = vec![Dynamic::from(1), Dynamic::from(2)];
        arr.push(Dynamic::from(3));
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[2].as_int().unwrap(), 3);
    }

    #[test]
    fn test_pop() {
        let mut arr: Array = vec![Dynamic::from(1), Dynamic::from(2), Dynamic::from(3)];
        let popped = arr.pop().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(popped.as_int().unwrap(), 3);
    }

    #[test]
    fn test_pop_empty() {
        let mut arr: Array = vec![];
        let popped = arr.pop();
        assert!(popped.is_none());
    }

    #[test]
    fn test_shift() {
        let mut arr: Array = vec![Dynamic::from(1), Dynamic::from(2), Dynamic::from(3)];
        let shifted = arr.remove(0);
        assert_eq!(arr.len(), 2);
        assert_eq!(shifted.as_int().unwrap(), 1);
        assert_eq!(arr[0].as_int().unwrap(), 2);
    }

    #[test]
    fn test_unshift() {
        let mut arr: Array = vec![Dynamic::from(2), Dynamic::from(3)];
        arr.insert(0, Dynamic::from(1));
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_int().unwrap(), 1);
    }
}
