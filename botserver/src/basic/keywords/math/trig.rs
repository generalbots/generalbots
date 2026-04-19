use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_sin() {
        assert!((0.0_f64.sin() - 0.0).abs() < 0.0001);
        assert!((std::f64::consts::FRAC_PI_2.sin() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cos() {
        assert!((0.0_f64.cos() - 1.0).abs() < 0.0001);
        assert!((std::f64::consts::PI.cos() - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_tan() {
        assert!((0.0_f64.tan() - 0.0).abs() < 0.0001);
        assert!((std::f64::consts::FRAC_PI_4.tan() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_log() {
        assert!((100.0_f64.log10() - 2.0).abs() < 0.0001);
        assert!((std::f64::consts::E.ln() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_exp() {
        assert!(0.0_f64.exp_m1().abs() < 0.0001);
        assert!((1.0_f64.exp() - std::f64::consts::E).abs() < 0.0001);
    }

    #[test]
    fn test_pi() {
        let pi = std::f64::consts::PI;
        assert!(pi > 3.0);
        assert!(pi < 4.0);
    }

    #[test]
    fn test_asin() {
        assert!((0.0_f64.asin() - 0.0).abs() < 0.0001);
        assert!((1.0_f64.asin() - std::f64::consts::FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_acos() {
        assert!((1.0_f64.acos() - 0.0).abs() < 0.0001);
        assert!((0.0_f64.acos() - std::f64::consts::FRAC_PI_2).abs() < 0.0001);
    }

    #[test]
    fn test_atan() {
        assert!((0.0_f64.atan() - 0.0).abs() < 0.0001);
        assert!((1.0_f64.atan() - std::f64::consts::FRAC_PI_4).abs() < 0.0001);
    }
}
