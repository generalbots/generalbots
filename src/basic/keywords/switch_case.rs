//! SWITCH/CASE keyword implementation for BASIC interpreter
//!
//! This module provides multi-way branching functionality similar to classic BASIC:
//! - SWITCH expression
//! - CASE value
//! - CASE value1, value2
//! - DEFAULT
//! - END SWITCH
//!
//! Note: The actual SWITCH/CASE parsing is handled in the preprocessor (mod.rs)
//! because it requires structural transformation. This module provides helper
//! functions for the runtime evaluation.

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

/// Register SWITCH/CASE helper functions with the Rhai engine
///
/// The SWITCH statement is transformed during preprocessing into nested
/// if-else statements. These helper functions support that transformation.
pub fn switch_keyword(_state: &Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // Helper function to compare switch expression with case value
    // Handles both string and numeric comparisons
    engine.register_fn(
        "__switch_match",
        |expr: Dynamic, case_val: Dynamic| -> bool { switch_match_impl(&expr, &case_val) },
    );

    // String comparison variant
    engine.register_fn("__switch_match_str", |expr: &str, case_val: &str| -> bool {
        expr == case_val
    });

    // Integer comparison variant
    engine.register_fn("__switch_match_int", |expr: i64, case_val: i64| -> bool {
        expr == case_val
    });

    // Float comparison variant
    engine.register_fn("__switch_match_float", |expr: f64, case_val: f64| -> bool {
        (expr - case_val).abs() < f64::EPSILON
    });

    // Multiple values check - returns true if expr matches any value in the array
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

    // Case-insensitive string matching (optional)
    engine.register_fn(
        "__switch_match_icase",
        |expr: &str, case_val: &str| -> bool { expr.to_lowercase() == case_val.to_lowercase() },
    );

    debug!("Registered SWITCH/CASE helper functions");
}

/// Implementation of switch matching logic
///
/// Compares two Dynamic values for equality, handling type coercion
/// where appropriate.
fn switch_match_impl(expr: &Dynamic, case_val: &Dynamic) -> bool {
    // Try string comparison first
    if let (Some(e), Some(c)) = (
        expr.clone().into_string().ok(),
        case_val.clone().into_string().ok(),
    ) {
        return e == c;
    }

    // Try integer comparison
    if let (Some(e), Some(c)) = (expr.as_int().ok(), case_val.as_int().ok()) {
        return e == c;
    }

    // Try float comparison
    if let (Some(e), Some(c)) = (expr.as_float().ok(), case_val.as_float().ok()) {
        return (e - c).abs() < f64::EPSILON;
    }

    // Try boolean comparison
    if let (Some(e), Some(c)) = (expr.as_bool().ok(), case_val.as_bool().ok()) {
        return e == c;
    }

    // Mixed numeric types - int vs float
    if let (Some(e), Some(c)) = (expr.as_int().ok(), case_val.as_float().ok()) {
        return (e as f64 - c).abs() < f64::EPSILON;
    }
    if let (Some(e), Some(c)) = (expr.as_float().ok(), case_val.as_int().ok()) {
        return (e - c as f64).abs() < f64::EPSILON;
    }

    false
}

/// Preprocess SWITCH/CASE blocks in BASIC code
///
/// Transforms:
/// ```basic
/// SWITCH expr
///   CASE "value1"
///     statement1
///   CASE "value2", "value3"
///     statement2
///   DEFAULT
///     statement3
/// END SWITCH
/// ```
///
/// Into equivalent if-else chain:
/// ```basic
/// let __switch_expr = expr;
/// if __switch_expr == "value1" {
///     statement1
/// } else if __switch_expr == "value2" || __switch_expr == "value3" {
///     statement2
/// } else {
///     statement3
/// }
/// ```
pub fn preprocess_switch(input: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut switch_counter = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.to_uppercase().starts_with("SWITCH ") {
            // Extract the switch expression
            let expr = line[7..].trim();
            let var_name = format!("__switch_expr_{}", switch_counter);
            switch_counter += 1;

            // Store the expression in a variable
            result.push_str(&format!("let {} = {};\n", var_name, expr));

            // Process cases until END SWITCH
            i += 1;
            let mut first_case = true;
            let mut _in_default = false;

            while i < lines.len() {
                let case_line = lines[i].trim();
                let case_upper = case_line.to_uppercase();

                if case_upper == "END SWITCH" || case_upper == "END_SWITCH" {
                    // Close the if-else chain
                    result.push_str("}\n");
                    break;
                } else if case_upper.starts_with("CASE ") {
                    // Close previous case if not first
                    if !first_case {
                        result.push_str("} else ");
                    }

                    // Extract case values (may be comma-separated)
                    let values_str = &case_line[5..];
                    let values: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();

                    // Build condition
                    if values.len() == 1 {
                        result.push_str(&format!("if {} == {} {{\n", var_name, values[0]));
                    } else {
                        let conditions: Vec<String> = values
                            .iter()
                            .map(|v| format!("{} == {}", var_name, v))
                            .collect();
                        result.push_str(&format!("if {} {{\n", conditions.join(" || ")));
                    }

                    first_case = false;
                    _in_default = false;
                } else if case_upper == "DEFAULT" {
                    // Close previous case
                    if !first_case {
                        result.push_str("} else {\n");
                    }
                    _in_default = true;
                } else if !case_line.is_empty()
                    && !case_line.starts_with("//")
                    && !case_line.starts_with("'")
                {
                    // Regular statement inside case
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
            // Non-switch line, pass through
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
