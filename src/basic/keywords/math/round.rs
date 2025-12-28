use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn round_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("ROUND", |n: f64| -> i64 { n.round() as i64 });
    engine.register_fn("ROUND", |n: f64, decimals: i64| -> f64 {
        let factor = 10_f64.powi(decimals as i32);
        (n * factor).round() / factor
    });
    engine.register_fn("round", |n: f64| -> i64 { n.round() as i64 });
    engine.register_fn("round", |n: f64, decimals: i64| -> f64 {
        let factor = 10_f64.powi(decimals as i32);
        (n * factor).round() / factor
    });

    debug!("Registered ROUND keyword");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_round_basic() {
        assert_eq!(3.7_f64.round() as i64, 4);
        assert_eq!(3.2_f64.round() as i64, 3);
        assert_eq!((-3.7_f64).round() as i64, -4);
    }

    #[test]
    fn test_round_decimals() {
        let n = 2.56789_f64;
        let decimals = 2;
        let factor = 10_f64.powi(decimals);
        let result = (n * factor).round() / factor;
        assert!((result - 2.57).abs() < 0.001);
    }

    #[test]
    fn test_round_negative_decimals() {
        let n = 1234.5_f64;
        let decimals = -2;
        let factor = 10_f64.powi(decimals);
        let result = (n * factor).round() / factor;
        assert!((result - 1200.0).abs() < 0.001);
    }
}
