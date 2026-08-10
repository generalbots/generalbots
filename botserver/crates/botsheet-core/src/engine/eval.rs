//! AST evaluator (#782, #783, #784).
//!
//! Walks the parsed expression tree, resolving references against the
//! worksheet set, applying operators with correct precedence and typed values,
//! and routing function calls to the legacy 170-function dispatcher. This
//! gives nested calls, `&`, `^`, `$`-anchored and cross-sheet (`Sheet2!A1`)
//! references real behaviour without rewriting the large legacy library.

use crate::formulas::evaluate_function_call;
use crate::types::Worksheet;

use super::ast::Expr;
use super::cross_sheet::resolve_sheet_args;
use super::references::Reference;
use super::value::{format_number, CellValue};

/// Evaluates a parsed expression inside the worksheet at `current` index.
///
/// Sheet-qualified references such as `Sheet2!A1` resolve against the given
/// worksheet set by name; unknown sheets yield a typed `#REF!` error (#783).
pub fn eval_expr_in(expr: &Expr, worksheets: &[Worksheet], current: usize) -> CellValue {
    match expr {
        Expr::Literal(v) => v.clone(),
        Expr::Name(name) => CellValue::Text(name.clone()),
        Expr::Reference(r) => resolve_reference(r, worksheets, current),
        Expr::Range(_, _) => CellValue::Error("VALUE!".to_string()),
        Expr::Unary { op, expr } => {
            let v = eval_expr_in(expr, worksheets, current);
            apply_unary(op, v)
        }
        Expr::Binary { op, left, right } => {
            let l = eval_expr_in(left, worksheets, current);
            let r = eval_expr_in(right, worksheets, current);
            apply_binary(op, l, r)
        }
        Expr::Call { name, raw, .. } => {
            // The legacy dispatcher expects `NAME(...)` and parses args itself.
            // Use the direct dispatcher (not evaluate_formula) to avoid
            // re-entering the typed engine for the nested call. The legacy
            // library is single-worksheet: cross-sheet arguments are resolved
            // eagerly so `SUM(Sheet2!A1:A3)` still works (#783).
            let resolved_args = resolve_sheet_args(raw, name, worksheets);
            let formula = format!("={name}({resolved_args})");
            let result = evaluate_function_call(&formula, &worksheets[current]);
            if let Some(err) = result.error {
                CellValue::Error(err)
            } else {
                CellValue::parse(&result.value)
            }
        }
    }
}

/// Evaluates a parsed expression inside a single worksheet (compatibility
/// entry point; delegates to the worksheet-set flavor).
pub fn eval_expr(expr: &Expr, worksheet: &Worksheet) -> CellValue {
    eval_expr_in(expr, std::slice::from_ref(worksheet), 0)
}

fn resolve_reference(r: &Reference, worksheets: &[Worksheet], current: usize) -> CellValue {
    let sheet_name = r.sheet.as_deref();
    let key = format!("{},{}", r.row, r.col);
    if let Some(name) = sheet_name {
        // Find the worksheet by name (case-insensitive) and resolve there.
        if let Some(ws) = worksheets
            .iter()
            .find(|w| w.name.eq_ignore_ascii_case(name))
        {
            return cell_value_at(ws, &key);
        }
        return CellValue::Error("REF!".to_string());
    }
    let Some(ws) = worksheets.get(current) else {
        return CellValue::Error("REF!".to_string());
    };
    cell_value_at(ws, &key)
}

fn cell_value_at(worksheet: &Worksheet, key: &str) -> CellValue {
    match worksheet.data.get(key) {
        Some(cell) => {
            if cell.formula.is_some() {
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
        "+" => match v.as_number() {
            Some(n) => CellValue::Number(n),
            None => CellValue::Error("VALUE!".to_string()),
        },
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
    evaluate_typed_in(formula, std::slice::from_ref(worksheet), 0)
}

/// Top-level typed evaluation of a formula against the worksheet set, so
/// `Sheet2!A1` cross-sheet references resolve by name (#783).
pub fn evaluate_typed_in(formula: &str, worksheets: &[Worksheet], current: usize) -> CellValue {
    let body = match formula.strip_prefix('=') {
        Some(b) => b,
        None => return CellValue::parse(formula),
    };
    match super::ast::parse(body) {
        Ok(expr) => eval_expr_in(&expr, worksheets, current),
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
                        typed: None,
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

    #[test]
    fn unary_plus_on_text_is_typed_error() {
        let ws = ws_with(&[("0,0", "abc")]);
        assert_eq!(ev("=+A1", &ws), CellValue::Error("VALUE!".to_string()));
        assert_eq!(ev("=+5", &Worksheet::default()), CellValue::Number(5.0));
    }
}