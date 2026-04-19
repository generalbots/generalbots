




use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;



pub fn val_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("VAL", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });

    engine.register_fn("val", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });


    engine.register_fn("VAL", |v: Dynamic| -> f64 {
        if v.is_int() {
            return v.as_int().unwrap_or(0) as f64;
        }
        if v.is_float() {
            return v.as_float().unwrap_or(0.0);
        }
        v.to_string().trim().parse::<f64>().unwrap_or(0.0)
    });

    debug!("Registered VAL keyword");
}


pub fn str_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("STR", |n: i64| -> String { n.to_string() });

    engine.register_fn("str", |n: i64| -> String { n.to_string() });

    engine.register_fn("STR", |n: f64| -> String {

        let s = format!("{}", n);
        s
    });

    engine.register_fn("str", |n: f64| -> String { format!("{}", n) });


    engine.register_fn("STR", |v: Dynamic| -> String { v.to_string() });

    engine.register_fn("str", |v: Dynamic| -> String { v.to_string() });

    debug!("Registered STR keyword");
}


pub fn cint_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CINT", |n: f64| -> i64 { n.round() as i64 });

    engine.register_fn("cint", |n: f64| -> i64 { n.round() as i64 });

    engine.register_fn("CINT", |n: i64| -> i64 { n });

    engine.register_fn("cint", |n: i64| -> i64 { n });

    engine.register_fn("CINT", |s: &str| -> i64 {
        s.trim()
            .parse::<f64>()
            .map(|f| f.round() as i64)
            .unwrap_or(0)
    });

    engine.register_fn("cint", |s: &str| -> i64 {
        s.trim()
            .parse::<f64>()
            .map(|f| f.round() as i64)
            .unwrap_or(0)
    });


    engine.register_fn("CINT", |v: Dynamic| -> i64 {
        if v.is_int() {
            return v.as_int().unwrap_or(0);
        }
        if v.is_float() {
            return v.as_float().unwrap_or(0.0).round() as i64;
        }
        v.to_string()
            .trim()
            .parse::<f64>()
            .map(|f| f.round() as i64)
            .unwrap_or(0)
    });

    debug!("Registered CINT keyword");
}


pub fn cdbl_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CDBL", |n: i64| -> f64 { n as f64 });

    engine.register_fn("cdbl", |n: i64| -> f64 { n as f64 });

    engine.register_fn("CDBL", |n: f64| -> f64 { n });

    engine.register_fn("cdbl", |n: f64| -> f64 { n });

    engine.register_fn("CDBL", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });

    engine.register_fn("cdbl", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });


    engine.register_fn("CDBL", |v: Dynamic| -> f64 {
        if v.is_float() {
            return v.as_float().unwrap_or(0.0);
        }
        if v.is_int() {
            return v.as_int().unwrap_or(0) as f64;
        }
        v.to_string().trim().parse::<f64>().unwrap_or(0.0)
    });

    debug!("Registered CDBL keyword");
}
