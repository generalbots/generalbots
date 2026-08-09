//! AST evaluator (#782, #784).
//!
//! Walks the parsed expression tree, resolving references against the
//! worksheet, applying operators with correct precedence and typed values, and
//! routing function calls to the legacy 170-function dispatcher. This gives
//! nested calls, `&`, `^` and `$`-anchored references real behaviour without
//! rewriting the large legacy function library.

use crate::formulas::evaluate_function_call;
use crate::types::Worksheet;

use super::ast::{Expr, Reference};
use super::value::{format_number, CellValue};

/// Evaluates a parsed expression to a typed value.
pub fn eval_expr(expr: &Expr, worksheet: &Worksheet) -> CellValue {
    match expr {
        Expr::Literal(v) => v.clone(),
        Expr::Name(name) => CellValue::Text(name.clone()),
        Expr::Reference(r) => resolve_reference(r, worksheet),
        Expr::Range(_, _) => CellValue::Error("VALUE!".to_string()),
        Expr::Unary { op, expr } => {
            let v = eval_expr(expr, worksheet);
            apply_unary(op, v)
        }
        Expr::Binary { op, left, right } => {
            let l = eval_expr(left, worksheet);
            let r = eval_expr(right, worksheet);
            apply_binary(op, l, r)
        }
        Expr::Call { name, raw, .. } => {
            // The legacy dispatcher expects `NAME(...)` and parses args itself.
            // Use the direct dispatcher (not evaluate_formula) to avoid
            // re-entering the typed engine for the nested call.
            let formula = format!("={name}({raw})");
            let result = evaluate_function_call(&formula, worksheet);
            if let Some(err) = result.error {
                CellValue::Error(err)
            } else {
                CellValue::parse(&result.value)
            }
        }
    }
}

fn resolve_reference(r: &Reference, worksheet: &Worksheet) -> CellValue {
    // Sheet-qualified references are not supported by the single-worksheet
    // legacy model; report a typed error rather than a silent wrong answer.
    if r.sheet.is_some() {
        return CellValue::Error("REF!".to_string());
    }
    let key = format!("{},{}", r.row, r.col);
    match worksheet.data.get(&key) {
        Some(cell) => {
            if let Some(ref formula) = cell.formula {
                // A formula cell resolves to its cached value; the dependency
                // engine recalculates it before consumers read it.
                if let Some(ref v) = cell.value {
                    CellValue::parse(v)
                } else {
                    CellValue::Empty
                }
            } else if let Some(ref v) = cell.value {
                CellValue::parse(v)
            } else {
                CellValue::Empty
            }
        }
        None => CellValue::Empty,
    }
}

fn apply_unary(op: &str, v: CellValue) -> CellValue {
    match op {
        "+" => CellValue::Number(v.as_number().unwrap_or(f64::NAN)),
        "-" => match v.as_number() {
            Some(n) => CellValue::Number(-n),
            None => CellValue::Error("VALUE!".to_string()),
        },
        "%" => match v.as_number() {
            Some(n) => CellValue::Number(n / 100.0),
            None => CellValue::Error("VALUE!".to_string()),
        },
        _ => CellValue::Error("NAME?.".to_string()),
    }
}

fn apply_binary(op: &str, l: CellValue, r: CellValue) -> CellValue {
    if op == "&" {
        return CellValue::Text(format!("{}{}", text_of(&l), text_of(&r)));
    }
    if let Some(cmp) = apply_comparison(op, &l, &r) {
        return cmp;
    }
    let ln = l.as_number();
    let rn = r.as_number();
    match (ln, rn) {
        (Some(a), Some(b)) => match op {
            "+" => CellValue::Number(a + b),
            "-" => CellValue::Number(a - b),
            "*" => CellValue::Number(a * b),
            "/" => {
                if b == 0.0 {
                    CellValue::Error("DIV/0!".to_string())
                } else {
                    CellValue::Number(a / b)
                }
            }
            "^" => CellValue::Number(a.powf(b)),
            _ => CellValue::Error("VALUE!".to_string()),
        },
        _ => CellValue::Error("VALUE!".to_string()),
    }
}

fn apply_comparison(op: &str, l: &CellValue, r: &CellValue) -> Option<CellValue> {
    if !matches!(op, "=" | "<>" | "<" | ">" | "<=" | ">=") {
        return None;
    }
    let result = match (l.as_number(), r.as_number()) {
        (Some(a), Some(b)) => match op {
            "=" => a == b,
            "<>" => a != b,
            "<" => a < b,
            ">" => a > b,
            "<=" => a <= b,
            ">=" => a >= b,
            _ => false,
        },
        _ => match op {
            "=" => text_of(l).eq_ignore_ascii_case(&text_of(r)),
            "<>" => !text_of(l).eq_ignore_ascii_case(&text_of(r)),
            "<" => text_of(l) < text_of(r),
            ">" => text_of(l) > text_of(r),
            "<=" => text_of(l) <= text_of(r),
            ">=" => text_of(l) >= text_of(r),
            _ => false,
        },
    };
    Some(CellValue::Bool(result))
}

fn text_of(v: &CellValue) -> String {
    match v {
        CellValue::Number(n) => format_number(*n),
        CellValue::Text(s) => s.clone(),
        CellValue::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        CellValue::Empty => String::new(),
        CellValue::Error(e) => format!("#{e}!"),
    }
}

/// Top-level typed evaluation of a formula (including the leading `=`).
pub fn evaluate_typed(formula: &str, worksheet: &Worksheet) -> CellValue {
    let body = match formula.strip_prefix('=') {
        Some(b) => b,
        None => return CellValue::parse(formula),
    };
    match super::ast::parse(body) {
        Ok(expr) => eval_expr(&expr, worksheet),
        Err(_) => CellValue::Error("ERROR!".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CellData;
    use std::collections::HashMap;

    fn ws_with(values: &[(&str, &str)]) -> Worksheet {
        let mut data = HashMap::new();
        for (key, val) in values {
            data.insert(
                key.to_string(),
                CellData {
                    value: Some(val.to_string()),
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                    locked: None,
                    has_comment: None,
                    array_formula_id: None,
                },
            );
        }
        Worksheet {
            data,
            ..Worksheet::default()
        }
    }

    fn ev(formula: &str, ws: &Worksheet) -> CellValue {
        evaluate_typed(formula, ws)
    }

    #[test]
    fn precedence() {
        assert_eq!(ev("=1+2*3", &Worksheet::default()), CellValue::Number(7.0));
        assert_eq!(ev("=2*3+1", &Worksheet::default()), CellValue::Number(7.0));
    }

    #[test]
    fn exponent_is_right_associative() {
        assert_eq!(ev("=2^3^2", &Worksheet::default()), CellValue::Number(512.0));
    }

    #[test]
    fn unary_minus_binds_looser_than_exponent() {
        assert_eq!(ev("=-2^2", &Worksheet::default()), CellValue::Number(-4.0));
    }

    #[test]
    fn concat_operator() {
        assert_eq!(
            ev("=\"Total: \"&A1", &ws_with(&[("0,0", "7")])),
            CellValue::Text("Total: 7".to_string())
        );
    }

    #[test]
    fn reference_resolution() {
        assert_eq!(ev("=A1+B1", &ws_with(&[("0,0", "10"), ("0,1", "5")])), CellValue::Number(15.0));
    }

    #[test]
    fn nested_call_routes_to_legacy() {
        // SUM + arithmetic is handled by the legacy evaluator through the AST.
        let ws = ws_with(&[("0,0", "1"), ("1,0", "2"), ("2,0", "3")]);
        assert_eq!(ev("=SUM(A1:A3)+1", &ws), CellValue::Number(7.0));
    }

    #[test]
    fn division_by_zero_is_typed_error() {
        assert_eq!(ev("=1/0", &Worksheet::default()), CellValue::Error("DIV/0!".to_string()));
    }

    #[test]
    fn comparisons() {
        assert_eq!(ev("=1<2", &Worksheet::default()), CellValue::Bool(true));
        assert_eq!(ev("=3>=3", &Worksheet::default()), CellValue::Bool(true));
        assert_eq!(ev("=\"a\"=\"a\"", &Worksheet::default()), CellValue::Bool(true));
    }

    #[test]
    fn absolute_reference_unchanged_by_value() {
        let ws = ws_with(&[("0,0", "42")]);
        assert_eq!(ev("=$A$1", &ws), CellValue::Number(42.0));
    }
}