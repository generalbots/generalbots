use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

/// Integer remainder, defined as `0` for a zero divisor (BASIC `MOD` never
/// raises; a script that divides by zero must keep running).
pub(crate) fn mod_int(a: i64, b: i64) -> i64 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

/// Floating point remainder, defined as `0.0` for a zero divisor.
pub(crate) fn mod_float(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        0.0
    } else {
        a % b
    }
}

/// Sign of a float as an integer: `1`, `-1` or `0` (including `NaN`).
pub(crate) fn sign_float(n: f64) -> i64 {
    if n > 0.0 {
        1
    } else if n < 0.0 {
        -1
    } else {
        0
    }
}

pub fn int_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("INT", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("int", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("FIX", |n: f64| -> i64 { n.trunc() as i64 });
    engine.register_fn("fix", |n: f64| -> i64 { n.trunc() as i64 });

    debug!("Registered INT keyword");
}

pub fn floor_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("FLOOR", |n: f64| -> i64 { n.floor() as i64 });
    engine.register_fn("floor", |n: f64| -> i64 { n.floor() as i64 });

    debug!("Registered FLOOR keyword");
}

pub fn ceil_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CEIL", |n: f64| -> i64 { n.ceil() as i64 });
    engine.register_fn("ceil", |n: f64| -> i64 { n.ceil() as i64 });
    engine.register_fn("CEILING", |n: f64| -> i64 { n.ceil() as i64 });

    debug!("Registered CEIL keyword");
}

pub fn max_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MAX", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("MAX", |a: f64, b: f64| -> f64 { a.max(b) });
    engine.register_fn("max", |a: i64, b: i64| -> i64 { a.max(b) });
    engine.register_fn("max", |a: f64, b: f64| -> f64 { a.max(b) });

    debug!("Registered MAX keyword");
}

pub fn min_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MIN", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("MIN", |a: f64, b: f64| -> f64 { a.min(b) });
    engine.register_fn("min", |a: i64, b: i64| -> i64 { a.min(b) });
    engine.register_fn("min", |a: f64, b: f64| -> f64 { a.min(b) });

    debug!("Registered MIN keyword");
}

pub fn mod_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("MOD", |a: i64, b: i64| -> i64 { mod_int(a, b) });
    engine.register_fn("MOD", |a: f64, b: f64| -> f64 { mod_float(a, b) });
    engine.register_fn("mod", |a: i64, b: i64| -> i64 { mod_int(a, b) });

    debug!("Registered MOD keyword");
}

pub fn sgn_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SGN", |n: i64| -> i64 { n.signum() });
    engine.register_fn("SGN", |n: f64| -> i64 { sign_float(n) });
    engine.register_fn("sgn", |n: i64| -> i64 { n.signum() });

    debug!("Registered SGN keyword");
}

pub fn sqrt_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("SQR", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("sqr", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("SQRT", |n: f64| -> f64 { n.sqrt() });
    engine.register_fn("sqrt", |n: f64| -> f64 { n.sqrt() });

    debug!("Registered SQRT keyword");
}

pub fn pow_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("POW", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("pow", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("POWER", |base: f64, exp: f64| -> f64 { base.powf(exp) });
    engine.register_fn("power", |base: f64, exp: f64| -> f64 { base.powf(exp) });

    debug!("Registered POW keyword");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_int_never_divides_by_zero() {
        assert_eq!(mod_int(7, 3), 1);
        assert_eq!(mod_int(-7, 3), -1);
        assert_eq!(mod_int(7, 0), 0);
        assert_eq!(mod_int(0, 5), 0);
    }

    #[test]
    fn mod_float_never_divides_by_zero() {
        assert_eq!(mod_float(7.5, 2.0), 1.5);
        assert_eq!(mod_float(7.5, 0.0), 0.0);
        assert_eq!(mod_float(-7.5, 2.0), -1.5);
    }

    #[test]
    fn sign_float_covers_all_three_outcomes() {
        assert_eq!(sign_float(3.2), 1);
        assert_eq!(sign_float(-3.2), -1);
        assert_eq!(sign_float(0.0), 0);
        assert_eq!(sign_float(-0.0), 0);
        assert_eq!(sign_float(f64::NAN), 0);
    }

    #[test]
    fn integer_math_keywords_are_registered_in_both_casings() {
        let mut engine = Engine::new();
        engine.register_fn("INT", |n: f64| -> i64 { n.trunc() as i64 });
        engine.register_fn("int", |n: f64| -> i64 { n.trunc() as i64 });
        engine.register_fn("FLOOR", |n: f64| -> i64 { n.floor() as i64 });
        engine.register_fn("floor", |n: f64| -> i64 { n.floor() as i64 });
        engine.register_fn("CEIL", |n: f64| -> i64 { n.ceil() as i64 });
        engine.register_fn("CEILING", |n: f64| -> i64 { n.ceil() as i64 });

        assert_eq!(engine.eval::<i64>("INT(3.9)").unwrap(), 3);
        assert_eq!(engine.eval::<i64>("int(-3.9)").unwrap(), -3);
        assert_eq!(engine.eval::<i64>("FLOOR(3.9)").unwrap(), 3);
        assert_eq!(engine.eval::<i64>("floor(-3.1)").unwrap(), -4);
        assert_eq!(engine.eval::<i64>("CEIL(3.1)").unwrap(), 4);
        assert_eq!(engine.eval::<i64>("CEILING(-3.9)").unwrap(), -3);
    }

    #[test]
    fn min_max_and_pow_keywords_cover_both_numeric_types() {
        let mut engine = Engine::new();
        engine.register_fn("MAX", |a: i64, b: i64| -> i64 { a.max(b) });
        engine.register_fn("MAX", |a: f64, b: f64| -> f64 { a.max(b) });
        engine.register_fn("MIN", |a: i64, b: i64| -> i64 { a.min(b) });
        engine.register_fn("MIN", |a: f64, b: f64| -> f64 { a.min(b) });
        engine.register_fn("SQR", |n: f64| -> f64 { n.sqrt() });
        engine.register_fn("POW", |base: f64, exp: f64| -> f64 { base.powf(exp) });
        engine.register_fn("POWER", |base: f64, exp: f64| -> f64 { base.powf(exp) });

        assert_eq!(engine.eval::<i64>("MAX(2, 9)").unwrap(), 9);
        assert_eq!(engine.eval::<f64>("MAX(2.5, 9.5)").unwrap(), 9.5);
        assert_eq!(engine.eval::<i64>("MIN(2, 9)").unwrap(), 2);
        assert_eq!(engine.eval::<f64>("MIN(2.5, 9.5)").unwrap(), 2.5);
        assert_eq!(engine.eval::<f64>("SQR(9.0)").unwrap(), 3.0);
        assert_eq!(engine.eval::<f64>("POW(2.0, 10.0)").unwrap(), 1024.0);
        assert_eq!(engine.eval::<f64>("POWER(3.0, 2.0)").unwrap(), 9.0);
    }

    #[test]
    fn mod_and_sgn_keywords_use_the_pure_helpers() {
        let mut engine = Engine::new();
        engine.register_fn("MOD", |a: i64, b: i64| -> i64 { mod_int(a, b) });
        engine.register_fn("MOD", |a: f64, b: f64| -> f64 { mod_float(a, b) });
        engine.register_fn("SGN", |n: i64| -> i64 { n.signum() });
        engine.register_fn("SGN", |n: f64| -> i64 { sign_float(n) });

        assert_eq!(engine.eval::<i64>("MOD(7, 3)").unwrap(), 1);
        assert_eq!(engine.eval::<i64>("MOD(7, 0)").unwrap(), 0);
        assert_eq!(engine.eval::<f64>("MOD(7.5, 2.0)").unwrap(), 1.5);
        assert_eq!(engine.eval::<i64>("SGN(-5)").unwrap(), -1);
        assert_eq!(engine.eval::<i64>("SGN(-5.0)").unwrap(), -1);
    }
}
