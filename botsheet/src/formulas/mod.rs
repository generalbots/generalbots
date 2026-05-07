pub mod criteria;
pub mod date;
pub mod helpers;
pub mod logic;
pub mod lookup;
pub mod math;
pub mod text;

pub use helpers::{
    col_name_to_index, count_matching, evaluate_condition, format_number, get_range_string_values,
    get_range_values, matches_criteria, parse_cell_ref, parse_range, resolve_cell_references,
    resolve_cell_value, split_args,
};

use crate::types::{FormulaResult, Worksheet};

pub fn evaluate_formula(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    if !formula.starts_with('=') {
        return FormulaResult {
            value: formula.to_string(),
            error: None,
        };
    }

    let expr = formula[1..].to_uppercase();

    let evaluators: Vec<fn(&str, &Worksheet) -> Option<String>> = vec![
        math::evaluate_sum,
        math::evaluate_average,
        math::evaluate_count,
        criteria::evaluate_counta,
        criteria::evaluate_countblank,
        criteria::evaluate_countif,
        criteria::evaluate_sumif,
        criteria::evaluate_averageif,
        math::evaluate_max,
        math::evaluate_min,
        logic::evaluate_if,
        logic::evaluate_iferror,
        lookup::evaluate_vlookup,
        lookup::evaluate_hlookup,
        lookup::evaluate_index_match,
        text::evaluate_concatenate,
        text::evaluate_left,
        text::evaluate_right,
        text::evaluate_mid,
        text::evaluate_len,
        text::evaluate_trim,
        text::evaluate_upper,
        text::evaluate_lower,
        text::evaluate_proper,
        text::evaluate_substitute,
        math::evaluate_round,
        math::evaluate_roundup,
        math::evaluate_rounddown,
        math::evaluate_abs,
        math::evaluate_sqrt,
        math::evaluate_power,
        math::evaluate_mod_formula,
        logic::evaluate_and,
        logic::evaluate_or,
        logic::evaluate_not,
        date::evaluate_today,
        date::evaluate_now,
        date::evaluate_date,
        date::evaluate_year,
        date::evaluate_month,
        date::evaluate_day,
        date::evaluate_datedif,
        math::evaluate_arithmetic,
    ];

    for evaluator in evaluators {
        if let Some(result) = evaluator(&expr, worksheet) {
            return FormulaResult {
                value: result,
                error: None,
            };
        }
    }

    FormulaResult {
        value: "#ERROR!".to_string(),
        error: Some("Invalid formula".to_string()),
    }
}
