use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn sin_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SIN", |n: f64| -> f64 { n.sin() });
    engine.register_fn("sin", |n: f64| -> f64 { n.sin() });
    engine.register_fn("ASIN", |n: f64| -> f64 { n.asin() });
    engine.register_fn("asin", |n: f64| -> f64 { n.asin() });

    debug!("Registered SIN keyword");
}

pub fn cos_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("COS", |n: f64| -> f64 { n.cos() });
    engine.register_fn("cos", |n: f64| -> f64 { n.cos() });
    engine.register_fn("ACOS", |n: f64| -> f64 { n.acos() });
    engine.register_fn("acos", |n: f64| -> f64 { n.acos() });

    debug!("Registered COS keyword");
}

pub fn tan_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("TAN", |n: f64| -> f64 { n.tan() });
    engine.register_fn("tan", |n: f64| -> f64 { n.tan() });
    engine.register_fn("ATAN", |n: f64| -> f64 { n.atan() });
    engine.register_fn("atan", |n: f64| -> f64 { n.atan() });
    engine.register_fn("ATN", |n: f64| -> f64 { n.atan() });

    debug!("Registered TAN keyword");
}

pub fn log_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("LOG", |n: f64| -> f64 { n.ln() });
    engine.register_fn("log", |n: f64| -> f64 { n.ln() });
    engine.register_fn("LOG10", |n: f64| -> f64 { n.log10() });
    engine.register_fn("log10", |n: f64| -> f64 { n.log10() });

    debug!("Registered LOG keyword");
}

pub fn exp_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("EXP", |n: f64| -> f64 { n.exp() });
    engine.register_fn("exp", |n: f64| -> f64 { n.exp() });

    debug!("Registered EXP keyword");
}

pub fn pi_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("PI", || -> f64 { std::f64::consts::PI });
    engine.register_fn("pi", || -> f64 { std::f64::consts::PI });

    debug!("Registered PI keyword");
}
