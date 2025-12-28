use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn abs_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ABS", |n: i64| -> i64 { n.abs() });
    engine.register_fn("ABS", |n: f64| -> f64 { n.abs() });
    engine.register_fn("abs", |n: i64| -> i64 { n.abs() });
    engine.register_fn("abs", |n: f64| -> f64 { n.abs() });

    debug!("Registered ABS keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_abs_positive() {
        assert_eq!(42_i64.abs(), 42);
        assert!((3.5_f64.abs() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_abs_negative() {
        assert_eq!((-42_i64).abs(), 42);
        assert!(((-3.5_f64).abs() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_abs_zero() {
        assert_eq!(0_i64.abs(), 0);
        assert!(0.0_f64.abs().abs() < f64::EPSILON);
    }
}
