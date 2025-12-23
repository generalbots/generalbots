use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn int_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("INT", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("int", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("FIX", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("fix", |n: f64| -> i64 { n.trunc() as i64 });

    debug!("Registered INT keyword");
}

pub fn floor_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("FLOOR", |n: f64| -> i64 { n.floor() as i64 });
    engine.register_fn("floor", |n: f64| -> i64 { n.floor() as i64 });

    debug!("Registered FLOOR keyword");
}

pub fn ceil_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CEIL", |n: f64| -> i64 { n.ceil() as i64 });
    engine.register_fn("ceil", |n: f64| -> i64 { n.ceil() as i64 });
    engine.register_fn("CEILING", |n: f64| -> i64 { n.ceil() as i64 });

    debug!("Registered CEIL keyword");
}

pub fn max_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MAX", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("MAX", |a: f64, b: f64| -> f64 { a.max(b) });
    engine.register_fn("max", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("max", |a: f64, b: f64| -> f64 { a.max(b) });

    debug!("Registered MAX keyword");
}

pub fn min_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MIN", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("MIN", |a: f64, b: f64| -> f64 { a.min(b) });
    engine.register_fn("min", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("min", |a: f64, b: f64| -> f64 { a.min(b) });

    debug!("Registered MIN keyword");
}

pub fn mod_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MOD", |a: i64, b: i64| -> i64 {
        if b == 0 {
            0
        } else {
            a % b
        }
    });
    engine.register_fn("MOD", |a: f64, b: f64| -> f64 {
        if b == 0.0 {
            0.0
        } else {
            a % b
        }
    });
    engine.register_fn("mod", |a: i64, b: i64| -> i64 {
        if b == 0 {
            0
        } else {
            a % b
        }
    });

    debug!("Registered MOD keyword");
}

pub fn sgn_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SGN", |n: i64| -> i64 { n.signum() });
    engine.register_fn("SGN", |n: f64| -> i64 {
        if n > 0.0 {
            1
        } else if n < 0.0 {
            -1
        } else {
            0
        }
    });
    engine.register_fn("sgn", |n: i64| -> i64 { n.signum() });

    debug!("Registered SGN keyword");
}

pub fn sqrt_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SQR", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("sqr", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("SQRT", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("sqrt", |n: f64| -> f64 { n.sqrt() });

    debug!("Registered SQRT keyword");
}

pub fn pow_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("POW", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("pow", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("POWER", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("power", |base: f64, exp: f64| -> f64 { base.powf(exp) });

    debug!("Registered POW keyword");
}
