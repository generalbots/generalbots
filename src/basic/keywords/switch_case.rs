use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::fmt::Write;
use std::sync::Arc;

pub fn switch_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine.register_fn(
        "__switch_match",
        |expr: Dynamic, case_val: Dynamic| -> bool { switch_match_impl(&expr, &case_val) },
    );

    engine.register_fn("__switch_match_str", |expr: &str, case_val: &str| -> bool {
        expr == case_val
    });

    engine.register_fn("__switch_match_int", |expr: i64, case_val: i64| -> bool {
        expr == case_val
    });

    engine.register_fn("__switch_match_float", |expr: f64, case_val: f64| -> bool {
        (expr - case_val).abs() < f64::EPSILON
    });

    engine.register_fn(
        "__switch_match_any",
        |expr: Dynamic, cases: rhai::Array| -> bool {
            for case_val in cases {
                if switch_match_impl(&expr, &case_val) {
                    return true;
                }
            }
            false
        },
    );

    engine.register_fn(
        "__switch_match_icase",
        |expr: &str, case_val: &str| -> bool { expr.to_lowercase() == case_val.to_lowercase() },
    );

    debug!("Registered SWITCH/CASE helper functions");
}

pub fn switch_match_impl(expr: &Dynamic, case_val: &Dynamic) -> bool {
    if let (Some(e), Some(c)) = (
        expr.clone().into_string().ok(),
        case_val.clone().into_string().ok(),
    ) {
        return e == c;
    }

    if let (Some(e), Some(c)) = (expr.as_int().ok(), case_val.as_int().ok()) {
        return e == c;
    }

    if let (Some(e), Some(c)) = (expr.as_float().ok(), case_val.as_float().ok()) {
        return (e - c).abs() < f64::EPSILON;
    }

    if let (Some(e), Some(c)) = (expr.as_bool().ok(), case_val.as_bool().ok()) {
        return e == c;
    }

    if let (Some(e), Some(c)) = (expr.as_int().ok(), case_val.as_float().ok()) {
        return (e as f64 - c).abs() < f64::EPSILON;
    }
    if let (Some(e), Some(c)) = (expr.as_float().ok(), case_val.as_int().ok()) {
        return (e - c as f64).abs() < f64::EPSILON;
    }

    false
}

pub fn preprocess_switch(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut switch_counter = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.to_uppercase().starts_with("SWITCH ") {
            let expr = line[7..].trim();
            let var_name = format!("__switch_expr_{}", switch_counter);
            switch_counter += 1;

            let _ = writeln!(result, "let {} = {};", var_name, expr);

            i += 1;
            let mut first_case = true;
            let mut _in_default = false;

            while i < lines.len() {
                let case_line = lines[i].trim();
                let case_upper = case_line.to_uppercase();

                if case_upper == "END SWITCH" || case_upper == "END_SWITCH" {
                    result.push_str("}\n");
                    break;
                } else if case_upper.starts_with("CASE ") {
                    if !first_case {
                        result.push_str("} else ");
                    }

                    let values_str = &case_line[5..];
                    let values: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();

                    if values.len() == 1 {
                        let _ = writeln!(result, "if {} == {} {{", var_name, values[0]);
                    } else {
                        let conditions: Vec<String> = values
                            .iter()
                            .map(|v| format!("{} == {}", var_name, v))
                            .collect();
                        let _ = writeln!(result, "if {} {{", conditions.join(" || "));
                    }

                    first_case = false;
                    _in_default = false;
                } else if case_upper == "DEFAULT" {
                    if !first_case {
                        result.push_str("} else {\n");
                    }
                    _in_default = true;
                } else if !case_line.is_empty()
                    && !case_line.starts_with("//")
                    && !case_line.starts_with('\'')
                {
                    result.push_str("    ");
                    result.push_str(case_line);
                    if !case_line.ends_with(';')
                        && !case_line.ends_with('{')
                        && !case_line.ends_with('}')
                    {
                        result.push(';');
                    }
                    result.push('\n');
                }

                i += 1;
            }
        } else {
            result.push_str(lines[i]);
            result.push('\n');
        }

        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;

    #[test]
    fn test_switch_match_strings() {
        let a = Dynamic::from("hello");
        let b = Dynamic::from("hello");
        let c = Dynamic::from("world");

        assert!(switch_match_impl(&a, &b));
        assert!(!switch_match_impl(&a, &c));
    }

    #[test]
    fn test_switch_match_integers() {
        let a = Dynamic::from(42_i64);
        let b = Dynamic::from(42_i64);
        let c = Dynamic::from(0_i64);

        assert!(switch_match_impl(&a, &b));
        assert!(!switch_match_impl(&a, &c));
    }

    #[test]
    fn test_switch_match_floats() {
        let a = Dynamic::from(3.14_f64);
        let b = Dynamic::from(3.14_f64);
        let c = Dynamic::from(2.71_f64);

        assert!(switch_match_impl(&a, &b));
        assert!(!switch_match_impl(&a, &c));
    }

    #[test]
    fn test_switch_match_mixed_numeric() {
        let int_val = Dynamic::from(42_i64);
        let float_val = Dynamic::from(42.0_f64);

        assert!(switch_match_impl(&int_val, &float_val));
    }

    #[test]
    fn test_preprocess_simple_switch() {
        let input = r#"
SWITCH role
  CASE "admin"
    x = 1
  CASE "user"
    x = 2
  DEFAULT
    x = 0
END SWITCH
"#;
        let output = preprocess_switch(input);
        assert!(output.contains("__switch_expr_"));
        assert!(output.contains("if"));
        assert!(output.contains("else"));
    }

    #[test]
    fn test_preprocess_multiple_values() {
        let input = r#"
SWITCH day
  CASE "saturday", "sunday"
    weekend = true
  DEFAULT
    weekend = false
END SWITCH
"#;
        let output = preprocess_switch(input);
        assert!(output.contains("||"));
    }
}
