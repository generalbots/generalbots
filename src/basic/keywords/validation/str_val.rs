//! Type conversion functions: VAL, STR, CINT, CDBL
//!
//! These functions convert between string and numeric types,
//! following classic BASIC conventions.

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// VAL - Convert string to number (float)
/// Returns 0.0 if conversion fails
pub fn val_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("VAL", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });

    engine.register_fn("val", |s: &str| -> f64 {
        s.trim().parse::<f64>().unwrap_or(0.0)
    });

    // Also handle Dynamic input
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

/// STR - Convert number to string
pub fn str_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn("STR", |n: i64| -> String { n.to_string() });

    engine.register_fn("str", |n: i64| -> String { n.to_string() });

    engine.register_fn("STR", |n: f64| -> String {
        // Remove trailing zeros for cleaner output
        let s = format!("{}", n);
        s
    });

    engine.register_fn("str", |n: f64| -> String { format!("{}", n) });

    // Handle Dynamic input
    engine.register_fn("STR", |v: Dynamic| -> String { v.to_string() });

    engine.register_fn("str", |v: Dynamic| -> String { v.to_string() });

    debug!("Registered STR keyword");
}

/// CINT - Convert to integer (rounds to nearest integer)
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

    // Handle Dynamic
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

/// CDBL - Convert to double (floating point)
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

    // Handle Dynamic
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_val_parsing() {
        assert_eq!("123.45".trim().parse::<f64>().unwrap_or(0.0), 123.45);
        assert_eq!("  456  ".trim().parse::<f64>().unwrap_or(0.0), 456.0);
        assert_eq!("abc".trim().parse::<f64>().unwrap_or(0.0), 0.0);
    }

    #[test]
    fn test_cint_rounding() {
        assert_eq!(2.4_f64.round() as i64, 2);
        assert_eq!(2.5_f64.round() as i64, 3);
        assert_eq!(2.6_f64.round() as i64, 3);
        assert_eq!((-2.5_f64).round() as i64, -3);
    }
}
