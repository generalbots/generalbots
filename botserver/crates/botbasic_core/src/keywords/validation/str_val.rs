use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// Parse a trimmed string as a float, defaulting to `0.0` (BASIC coercers never
/// fail: an unparseable value becomes zero).
pub(crate) fn parse_float_or_zero(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}

/// Parse a trimmed string as a float and round it, defaulting to `0`.
pub(crate) fn parse_rounded_int_or_zero(s: &str) -> i64 {
    s.trim()
        .parse::<f64>()
        .map(|f| f.round() as i64)
        .unwrap_or(0)
}

/// Coerce any Rhai value to a float: ints and floats pass through, everything
/// else is stringified and parsed.
pub(crate) fn coerce_float(value: &Dynamic) -> f64 {
    if value.is_int() {
        return value.as_int().unwrap_or(0) as f64;
    }
    if value.is_float() {
        return value.as_float().unwrap_or(0.0);
    }
    parse_float_or_zero(&value.to_string())
}

/// Coerce any Rhai value to a rounded integer.
pub(crate) fn coerce_rounded_int(value: &Dynamic) -> i64 {
    if value.is_int() {
        return value.as_int().unwrap_or(0);
    }
    if value.is_float() {
        return value.as_float().unwrap_or(0.0).round() as i64;
    }
    parse_rounded_int_or_zero(&value.to_string())
}

pub fn val_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("VAL", |s: &str| -> f64 { parse_float_or_zero(s) });
    engine.register_fn("val", |s: &str| -> f64 { parse_float_or_zero(s) });
    engine.register_fn("VAL", |v: Dynamic| -> f64 { coerce_float(&v) });

    debug!("Registered VAL keyword");
}

pub fn str_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("STR", |n: i64| -> String { n.to_string() });
    engine.register_fn("str", |n: i64| -> String { n.to_string() });
    engine.register_fn("STR", |n: f64| -> String { format!("{n}") });
    engine.register_fn("str", |n: f64| -> String { format!("{n}") });
    engine.register_fn("STR", |v: Dynamic| -> String { v.to_string() });
    engine.register_fn("str", |v: Dynamic| -> String { v.to_string() });

    debug!("Registered STR keyword");
}

pub fn cint_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CINT", |n: f64| -> i64 { n.round() as i64 });
    engine.register_fn("cint", |n: f64| -> i64 { n.round() as i64 });
    engine.register_fn("CINT", |n: i64| -> i64 { n });
    engine.register_fn("cint", |n: i64| -> i64 { n });
    engine.register_fn("CINT", |s: &str| -> i64 { parse_rounded_int_or_zero(s) });
    engine.register_fn("cint", |s: &str| -> i64 { parse_rounded_int_or_zero(s) });
    engine.register_fn("CINT", |v: Dynamic| -> i64 { coerce_rounded_int(&v) });

    debug!("Registered CINT keyword");
}

pub fn cdbl_keyword(_state: &Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("CDBL", |n: i64| -> f64 { n as f64 });
    engine.register_fn("cdbl", |n: i64| -> f64 { n as f64 });
    engine.register_fn("CDBL", |n: f64| -> f64 { n });
    engine.register_fn("cdbl", |n: f64| -> f64 { n });
    engine.register_fn("CDBL", |s: &str| -> f64 { parse_float_or_zero(s) });
    engine.register_fn("cdbl", |s: &str| -> f64 { parse_float_or_zero(s) });
    engine.register_fn("CDBL", |v: Dynamic| -> f64 { coerce_float(&v) });

    debug!("Registered CDBL keyword");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_float_trims_and_defaults_to_zero() {
        assert_eq!(parse_float_or_zero(" 12.5 "), 12.5);
        assert_eq!(parse_float_or_zero("-3"), -3.0);
        assert_eq!(parse_float_or_zero("abc"), 0.0);
        assert_eq!(parse_float_or_zero(""), 0.0);
    }

    #[test]
    fn parse_rounded_int_uses_banker_free_rounding() {
        assert_eq!(parse_rounded_int_or_zero("2.4"), 2);
        assert_eq!(parse_rounded_int_or_zero("2.5"), 3);
        assert_eq!(parse_rounded_int_or_zero("-2.5"), -3);
        assert_eq!(parse_rounded_int_or_zero(" 7 "), 7);
        assert_eq!(parse_rounded_int_or_zero("nope"), 0);
    }

    #[test]
    fn coerce_float_handles_int_float_and_text() {
        assert_eq!(coerce_float(&Dynamic::from(7_i64)), 7.0);
        assert_eq!(coerce_float(&Dynamic::from(7.25_f64)), 7.25);
        assert_eq!(coerce_float(&Dynamic::from("3.5")), 3.5);
        assert_eq!(coerce_float(&Dynamic::from("text")), 0.0);
    }

    #[test]
    fn coerce_rounded_int_handles_int_float_and_text() {
        assert_eq!(coerce_rounded_int(&Dynamic::from(7_i64)), 7);
        assert_eq!(coerce_rounded_int(&Dynamic::from(7.6_f64)), 8);
        assert_eq!(coerce_rounded_int(&Dynamic::from("2.5")), 3);
        assert_eq!(coerce_rounded_int(&Dynamic::from("text")), 0);
    }

    #[test]
    fn conversion_keywords_round_trip_values() {
        let mut engine = Engine::new();
        engine.register_fn("VAL", |s: &str| -> f64 { parse_float_or_zero(s) });
        engine.register_fn("STR", |n: f64| -> String { format!("{n}") });
        engine.register_fn("CINT", |s: &str| -> i64 { parse_rounded_int_or_zero(s) });
        engine.register_fn("CDBL", |n: i64| -> f64 { n as f64 });

        assert_eq!(engine.eval::<f64>(r#"VAL("42.5")"#).unwrap(), 42.5);
        assert_eq!(engine.eval::<f64>(r#"VAL("x")"#).unwrap(), 0.0);
        assert_eq!(engine.eval::<String>(r#"STR(42.5)"#).unwrap(), "42.5");
        assert_eq!(engine.eval::<i64>(r#"CINT("2.6")"#).unwrap(), 3);
        assert_eq!(engine.eval::<f64>("CDBL(9)").unwrap(), 9.0);
    }
}
