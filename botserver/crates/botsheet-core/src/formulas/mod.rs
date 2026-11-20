pub mod ai;
pub mod arrays;
pub mod criteria;
pub mod date;
pub mod dynamic;
pub mod financial;
pub mod financial_x;
pub mod helpers;
pub mod logic;
pub mod numeric_args;
pub mod lookup;
pub mod math;
pub mod pivot;
pub mod refs;
pub mod statistical;
pub mod text;
pub mod text365;
pub mod text_join;
pub mod trig;

#[cfg(test)]
mod differential_tests;
#[cfg(test)]
mod tests_core;

pub use helpers::{
    count_matching, evaluate_condition, format_number, matches_criteria, resolve_cell_value,
    split_args,
};
pub use refs::{
    cell_to_a1, clamp_range, col_index_to_name, col_name_to_index, get_range_string_values,
    get_range_values, parse_cell_ref, parse_range, resolve_cell_references, MAX_COL_INDEX,
    MAX_RANGE_CELLS, MAX_ROW_INDEX,
};

use crate::types::{FormulaResult, Worksheet};

/// Entry point for formula evaluation.
///
/// The typed engine (#781-785) is tried first: a real lexer + Pratt parser
/// handles operator precedence, `&`, `^`, nested calls, `$` anchors and typed
/// values. If the formula does not parse (for example a legacy quirk), we fall
/// back to the string-dispatcher evaluator so nothing that used to work breaks.
pub fn evaluate_formula(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    evaluate_formula_in(formula, std::slice::from_ref(worksheet), 0)
}

/// Formula evaluation against the worksheet set, so `Sheet2!A1` cross-sheet
/// references resolve by name instead of erroring (#783).
pub fn evaluate_formula_in(
    formula: &str,
    worksheets: &[Worksheet],
    current: usize,
) -> FormulaResult {
    if !formula.starts_with('=') {
        return FormulaResult {
            value: formula.to_string(),
            error: None,
        };
    }
    let body = &formula[1..];
    match crate::engine::parse(body) {
        Ok(expr) => {
            let value = crate::engine::eval_expr_in(&expr, worksheets, current);
            FormulaResult {
                value: value.display(),
                error: None,
            }
        }
        Err(_) => evaluate_legacy(formula, &worksheets[current]),
    }
}

/// Legacy string-dispatcher evaluation, used for function calls parsed by the
/// new engine and as a fallback when the typed parser rejects a formula.
pub fn evaluate_function_call(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    evaluate_legacy(formula, worksheet)
}

fn evaluate_legacy(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    // Drop the leading '=' without assuming it is there (an empty formula used
    // to panic with "byte index 1 is out of bounds") and without cutting into a
    // multi-byte first character.
    let expr = crate::engine::canonicalize_dispatch_formula(
        formula.strip_prefix('=').unwrap_or(formula),
    );

    // Otimização de busca direta O(1) com base no prefixo do nome da função.
    // Evita iterar linearmente sobre mais de 170 ponteiros de função de fórmula em casos de fallback.
    let mut resolved = None;
    if expr.ends_with(')') {
        if let Some((func_name, _)) = expr.split_once('(') {
            resolved = match func_name.trim() {
                "BOT_AI_PROMPT" => ai::evaluate_bot_ai_prompt(&expr, worksheet),
                "SUM" | "SOMA" => math::evaluate_sum(&expr, worksheet),
                "AVERAGE" | "MÉDIA" | "MEDIA" => math::evaluate_average(&expr, worksheet),
                "COUNT" => math::evaluate_count(&expr, worksheet),
                "COUNTA" => criteria::evaluate_counta(&expr, worksheet),
                "COUNTBLANK" => criteria::evaluate_countblank(&expr, worksheet),
                "COUNTIF" | "CONT.SE" => criteria::evaluate_countif(&expr, worksheet),
                "SUMIF" | "SOMASE" => criteria::evaluate_sumif(&expr, worksheet),
                "AVERAGEIF" | "MÉDIASE" | "MEDIASE" => {
                    criteria::evaluate_averageif(&expr, worksheet)
                }
                "MAX" => math::evaluate_max(&expr, worksheet),
                "MIN" => math::evaluate_min(&expr, worksheet),
                "IF" => logic::evaluate_if(&expr, worksheet),
                "IFERROR" => logic::evaluate_iferror(&expr, worksheet),
                "IFNA" => logic::evaluate_ifna(&expr, worksheet),
                "ISNA" => logic::evaluate_isna(&expr, worksheet),
                "VLOOKUP" => lookup::evaluate_vlookup(&expr, worksheet),
                "HLOOKUP" => lookup::evaluate_hlookup(&expr, worksheet),
                "INDEX" => lookup::evaluate_index_match(&expr, worksheet),
                "CONCATENATE" => text::evaluate_concatenate(&expr, worksheet),
                "LEFT" => text::evaluate_left(&expr, worksheet),
                "RIGHT" => text::evaluate_right(&expr, worksheet),
                "MID" => text::evaluate_mid(&expr, worksheet),
                "LEN" => text::evaluate_len(&expr, worksheet),
                "TRIM" => text::evaluate_trim(&expr, worksheet),
                "UPPER" => text::evaluate_upper(&expr, worksheet),
                "LOWER" => text::evaluate_lower(&expr, worksheet),
                "PROPER" => text::evaluate_proper(&expr, worksheet),
                "SUBSTITUTE" => text::evaluate_substitute(&expr, worksheet),
                "ROUND" => math::evaluate_round(&expr, worksheet),
                "ROUNDUP" => math::evaluate_roundup(&expr, worksheet),
                "ROUNDDOWN" => math::evaluate_rounddown(&expr, worksheet),
                "ABS" => math::evaluate_abs(&expr, worksheet),
                "SQRT" => math::evaluate_sqrt(&expr, worksheet),
                "POWER" => math::evaluate_power(&expr, worksheet),
                "MOD" => math::evaluate_mod_formula(&expr, worksheet),
                "AND" => logic::evaluate_and(&expr, worksheet),
                "OR" => logic::evaluate_or(&expr, worksheet),
                "NOT" => logic::evaluate_not(&expr, worksheet),
                "TODAY" => date::evaluate_today(&expr, worksheet),
                "NOW" => date::evaluate_now(&expr, worksheet),
                "DATE" => date::evaluate_date(&expr, worksheet),
                "YEAR" => date::evaluate_year(&expr, worksheet),
                "MONTH" => date::evaluate_month(&expr, worksheet),
                "DAY" => date::evaluate_day(&expr, worksheet),
                "DATEDIF" => date::evaluate_datedif(&expr, worksheet),
                "PRODUCT" => math::evaluate_product(&expr, worksheet),
                "STDEV" => math::evaluate_stdev(&expr, worksheet),
                "STDEVP" => math::evaluate_stdevp(&expr, worksheet),
                "MEDIAN" => math::evaluate_median(&expr, worksheet),
                "CEILING" => math::evaluate_ceiling(&expr, worksheet),
                "FLOOR" => math::evaluate_floor(&expr, worksheet),
                "INT" => math::evaluate_int(&expr, worksheet),
                "EXP" => math::evaluate_exp(&expr, worksheet),
                "LN" => math::evaluate_ln(&expr, worksheet),
                "LOG" => math::evaluate_log(&expr, worksheet),
                "LOG10" => math::evaluate_log10(&expr, worksheet),
                "SIGN" => math::evaluate_sign(&expr, worksheet),
                "PI" => math::evaluate_pi(&expr, worksheet),
                "RAND" => math::evaluate_rand(&expr, worksheet),
                "RANDBETWEEN" => math::evaluate_randbetween(&expr, worksheet),
                "SIN" => trig::evaluate_sin(&expr, worksheet),
                "COS" => trig::evaluate_cos(&expr, worksheet),
                "TAN" => trig::evaluate_tan(&expr, worksheet),
                "ASIN" => trig::evaluate_asin(&expr, worksheet),
                "ACOS" => trig::evaluate_acos(&expr, worksheet),
                "ATAN" => trig::evaluate_atan(&expr, worksheet),
                "ATAN2" => trig::evaluate_atan2(&expr, worksheet),
                "ISBLANK" => logic::evaluate_isblank(&expr, worksheet),
                "ISNUMBER" => logic::evaluate_isnumber(&expr, worksheet),
                "ISTEXT" => logic::evaluate_istext(&expr, worksheet),
                "ISERROR" => logic::evaluate_iserror(&expr, worksheet),
                "ISLOGICAL" => logic::evaluate_islogical(&expr, worksheet),
                "ISNONTEXT" => logic::evaluate_isnontext(&expr, worksheet),
                "TRUE" => logic::evaluate_true(&expr, worksheet),
                "FALSE" => logic::evaluate_false(&expr, worksheet),
                "REPLACE" => text::evaluate_replace(&expr, worksheet),
                "FIND" => text::evaluate_find(&expr, worksheet),
                "SEARCH" => text::evaluate_search(&expr, worksheet),
                "EXACT" => text::evaluate_exact(&expr, worksheet),
                "REPT" => text::evaluate_rept(&expr, worksheet),
                "TEXT" => text::evaluate_text(&expr, worksheet),
                "VALUE" => text::evaluate_value(&expr, worksheet),
                "HOUR" => date::evaluate_hour(&expr, worksheet),
                "MINUTE" => date::evaluate_minute(&expr, worksheet),
                "SECOND" => date::evaluate_second(&expr, worksheet),
                "XLOOKUP" => lookup::evaluate_xlookup(&expr, worksheet),
                "MATCH" => lookup::evaluate_match(&expr, worksheet),
                "CHOOSE" => lookup::evaluate_choose(&expr, worksheet),
                "SUMIFS" | "SOMASES" => criteria::evaluate_sumifs(&expr, worksheet),
                "COUNTIFS" | "CONT.SES" => criteria::evaluate_countifs(&expr, worksheet),
                "AVERAGEIFS" | "MÉDIASES" | "MEDIASES" => {
                    criteria::evaluate_averageifs(&expr, worksheet)
                }
                "MAXIFS" => criteria::evaluate_maxifs(&expr, worksheet),
                "MINIFS" => criteria::evaluate_minifs(&expr, worksheet),
                "LET" => dynamic::evaluate_let(&expr, worksheet),
                "LAMBDA" => dynamic::evaluate_lambda(&expr, worksheet),
                "MAP" => dynamic::evaluate_map(&expr, worksheet),
                "REDUCE" => dynamic::evaluate_reduce(&expr, worksheet),
                "BYROW" => dynamic::evaluate_byrow(&expr, worksheet),
                "BYCOL" => dynamic::evaluate_bycol(&expr, worksheet),
                "MAKEARRAY" => dynamic::evaluate_makearray(&expr, worksheet),
                "REDUCE_ARITHMETIC" => dynamic::evaluate_reduce_arithmetic(&expr, worksheet),
                "FILTER" => arrays::evaluate_filter(&expr, worksheet),
                "SORT" => arrays::evaluate_sort(&expr, worksheet),
                "SORTBY" => arrays::evaluate_sortby(&expr, worksheet),
                "UNIQUE" => arrays::evaluate_unique(&expr, worksheet),
                "SEQUENCE" => arrays::evaluate_sequence(&expr, worksheet),
                "RANDARRAY" => arrays::evaluate_randarray(&expr, worksheet),
                "TOCOL" => arrays::evaluate_tocol(&expr, worksheet),
                "TOROW" => arrays::evaluate_torow(&expr, worksheet),
                "WRAPCOLS" => arrays::evaluate_wrapcols(&expr, worksheet),
                "WRAPROWS" => arrays::evaluate_wraprows(&expr, worksheet),
                "HSTACK" => arrays::evaluate_hstack(&expr, worksheet),
                "VSTACK" => arrays::evaluate_vstack(&expr, worksheet),
                "CHOOSEROWS" => arrays::evaluate_chooserows(&expr, worksheet),
                "CHOOSECOLS" => arrays::evaluate_choosecols(&expr, worksheet),
                "TAKE" => arrays::evaluate_take(&expr, worksheet),
                "DROP" => arrays::evaluate_drop(&expr, worksheet),
                "EXPAND" => arrays::evaluate_expand(&expr, worksheet),
                "TRIMRANGE" => arrays::evaluate_trimrange(&expr, worksheet),
                "GROUPBY" => pivot::evaluate_groupby(&expr, worksheet),
                "PIVOTBY" => pivot::evaluate_pivotby(&expr, worksheet),
                "PERCENTOF" => pivot::evaluate_percentof(&expr, worksheet),
                "SUBTOTAL" => pivot::evaluate_subtotal(&expr, worksheet),
                "AGGREGATE" => pivot::evaluate_aggregate(&expr, worksheet),
                "PERCENTILE" => pivot::evaluate_percentile(&expr, worksheet),
                "QUARTILE" => pivot::evaluate_quartile(&expr, worksheet),
                "RANK" => pivot::evaluate_rank(&expr, worksheet),
                "TEXTSPLIT" => text365::evaluate_textsplit(&expr, worksheet),
                "TEXTAFTER" => text365::evaluate_textafter(&expr, worksheet),
                "TEXTBEFORE" => text365::evaluate_textbefore(&expr, worksheet),
                "ARRAYTOTEXT" => text365::evaluate_arraytotext(&expr, worksheet),
                "VALUETOTEXT" => text365::evaluate_valuetotext(&expr, worksheet),
                "NUMBERVALUE" => text365::evaluate_numbervalue(&expr, worksheet),
                "UNICHAR" => text365::evaluate_unichar(&expr, worksheet),
                "UNICODE" => text365::evaluate_unicode(&expr, worksheet),
                "ARABIC" => text365::evaluate_arabic(&expr, worksheet),
                "ROMAN" => text365::evaluate_roman(&expr, worksheet),
                "BASE64ENCODE" => text365::evaluate_base64encode(&expr, worksheet),
                "BASE64DECODE" => text365::evaluate_base64decode(&expr, worksheet),
                "URLENCODE" => text365::evaluate_urlencode(&expr, worksheet),
                "PMT" => financial::evaluate_pmt(&expr, worksheet),
                "IPMT" => financial::evaluate_ipmt(&expr, worksheet),
                "PPMT" => financial::evaluate_ppmt(&expr, worksheet),
                "FV" => financial::evaluate_fv(&expr, worksheet),
                "PV" => financial::evaluate_pv(&expr, worksheet),
                "RATE" => financial::evaluate_rate(&expr, worksheet),
                "NPER" => financial::evaluate_nper(&expr, worksheet),
                "NPV" => financial::evaluate_npv(&expr, worksheet),
                "IRR" => financial::evaluate_irr(&expr, worksheet),
                "MIRR" => financial::evaluate_mirr(&expr, worksheet),
                "XIRR" => financial_x::evaluate_xirr(&expr, worksheet),
                "XNPV" => financial_x::evaluate_xnpv(&expr, worksheet),
                "SLN" => financial_x::evaluate_sln(&expr, worksheet),
                "SYD" => financial_x::evaluate_syd(&expr, worksheet),
                "DB" => financial_x::evaluate_db(&expr, worksheet),
                "DDB" => financial_x::evaluate_ddb(&expr, worksheet),
                "EFFECT" => financial_x::evaluate_effect(&expr, worksheet),
                "NOMINAL" => financial_x::evaluate_nominal(&expr, worksheet),
                "CORREL" => statistical::evaluate_correl(&expr, worksheet),
                "SLOPE" => statistical::evaluate_slope(&expr, worksheet),
                "INTERCEPT" => statistical::evaluate_intercept(&expr, worksheet),
                "FORECAST.LINEAR" => statistical::evaluate_forecast_linear(&expr, worksheet),
                "STDEV.S" => statistical::evaluate_stdev_s(&expr, worksheet),
                "STDEV.P" => statistical::evaluate_stdev_p(&expr, worksheet),
                "VAR.S" => statistical::evaluate_var_s(&expr, worksheet),
                "VAR.P" => statistical::evaluate_var_p(&expr, worksheet),
                "MODE.SNGL" => statistical::evaluate_mode_sngl(&expr, worksheet),
                "TEXTJOIN" => text_join::evaluate_textjoin(&expr, worksheet),
                "CONCAT" => text_join::evaluate_concat(&expr, worksheet),
                "NETWORKDAYS" => date::evaluate_networkdays(&expr, worksheet),
                "NETWORKDAYS.INTL" => date::evaluate_networkdays_intl(&expr, worksheet),
                "EDATE" => date::evaluate_edate(&expr, worksheet),
                "EOMONTH" => date::evaluate_eomonth(&expr, worksheet),
                "YEARFRAC" => date::evaluate_yearfrac(&expr, worksheet),
                "WEEKNUM" => date::evaluate_weeknum(&expr, worksheet),
                "ISOWEEKNUM" => date::evaluate_isoweeknum(&expr, worksheet),
                _ => None,
            };
        }
    }

    if resolved.is_none() {
        resolved = math::evaluate_arithmetic(&expr, worksheet);
    }

    if let Some(result) = resolved {
        return FormulaResult {
            value: result,
            error: None,
        };
    }

    FormulaResult {
        value: "#ERROR!".to_string(),
        error: Some("Invalid formula".to_string()),
    }
}
