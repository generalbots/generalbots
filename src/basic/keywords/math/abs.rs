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
