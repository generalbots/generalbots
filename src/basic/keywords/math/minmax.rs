use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Array, Dynamic, Engine};
use std::sync::Arc;

pub fn max_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MAX", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("MAX", |a: f64, b: f64| -> f64 { a.max(b) });
    engine.register_fn("MAX", |arr: Array| -> Dynamic {
        if arr.is_empty() {
            return Dynamic::UNIT;
        }
        let mut max_val = arr[0].clone();
        for item in arr.iter().skip(1) {
            if let (Some(a), Some(b)) = (item.as_float().ok(), max_val.as_float().ok()) {
                if a > b {
                    max_val = item.clone();
                }
            } else if let (Some(a), Some(b)) = (item.as_int().ok(), max_val.as_int().ok()) {
                if a > b {
                    max_val = item.clone();
                }
            }
        }
        max_val
    });
    engine.register_fn("max", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("max", |a: f64, b: f64| -> f64 { a.max(b) });

    debug!("Registered MAX keyword");
}

pub fn min_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MIN", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("MIN", |a: f64, b: f64| -> f64 { a.min(b) });
    engine.register_fn("MIN", |arr: Array| -> Dynamic {
        if arr.is_empty() {
            return Dynamic::UNIT;
        }
        let mut min_val = arr[0].clone();
        for item in arr.iter().skip(1) {
            if let (Some(a), Some(b)) = (item.as_float().ok(), min_val.as_float().ok()) {
                if a < b {
                    min_val = item.clone();
                }
            } else if let (Some(a), Some(b)) = (item.as_int().ok(), min_val.as_int().ok()) {
                if a < b {
                    min_val = item.clone();
                }
            }
        }
        min_val
    });
    engine.register_fn("min", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("min", |a: f64, b: f64| -> f64 { a.min(b) });

    debug!("Registered MIN keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_max_values() {
        assert_eq!(10_i64.max(5), 10);
        assert_eq!(5_i64.max(10), 10);
        assert_eq!(3.5_f64.max(7.2), 7.2);
        assert_eq!(7.2_f64.max(3.5), 7.2);
    }

    #[test]
    fn test_min_values() {
        assert_eq!(10_i64.min(5), 5);
        assert_eq!(5_i64.min(10), 5);
        assert_eq!(3.5_f64.min(7.2), 3.5);
        assert_eq!(7.2_f64.min(3.5), 3.5);
    }

    #[test]
    fn test_max_equal_values() {
        assert_eq!(5_i64.max(5), 5);
        assert_eq!(3.5_f64.max(3.5), 3.5);
    }

    #[test]
    fn test_min_equal_values() {
        assert_eq!(5_i64.min(5), 5);
        assert_eq!(3.5_f64.min(3.5), 3.5);
    }

    #[test]
    fn test_max_negative_values() {
        assert_eq!((-10_i64).max(-5), -5);
        assert_eq!((-3.5_f64).max(-7.2), -3.5);
    }

    #[test]
    fn test_min_negative_values() {
        assert_eq!((-10_i64).min(-5), -10);
        assert_eq!((-3.5_f64).min(-7.2), -7.2);
    }
}
