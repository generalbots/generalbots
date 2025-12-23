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
