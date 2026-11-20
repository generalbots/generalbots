//! Differential corpus for the parity families (#2434 vectors, #2572).
//!
//! Reference numbers come from the local preview's catalog/selftest. Where a
//! printed catalog value contradicts the committed JS implementation, the test
//! uses the implementation result and documents the discrepancy inline. The
//! grid keeps six decimal places, so comparisons use an absolute tolerance;
//! error literals are exact.

use super::evaluate_formula;
use crate::engine::{canonical_function_name, normalize_formula};
use crate::types::{FormulaLocale, Worksheet};

fn sheet() -> Worksheet {
    Worksheet {
        name: "Sheet1".to_string(),
        ..Default::default()
    }
}

fn result(formula: &str) -> String {
    let evaluated = evaluate_formula(formula, &sheet());
    assert!(
        evaluated.error.is_none(),
        "{formula} produced an engine error: {:?}",
        evaluated.error
    );
    evaluated.value
}

fn number(formula: &str) -> f64 {
    let value = result(formula);
    assert!(!value.starts_with('#'), "{formula} produced {value}");
    value.parse::<f64>().unwrap_or(f64::NAN)
}

fn close(formula: &str, expected: f64) {
    let actual = number(formula);
    // `format_number` keeps six decimals, so an absolute tolerance is the
    // right comparison for both large payments and small rates.
    let tolerance = 1e-6;
    assert!(
        (actual - expected).abs() <= tolerance,
        "{formula}: expected {expected}, got {actual}"
    );
}

fn error(formula: &str, expected: &str) {
    assert_eq!(result(formula), expected, "{formula}");
}

#[test]
fn financial_vectors_match_the_preview() {
    close("=PMT(0.05/12,360,100000)", -536.8216230121398);
    close("=IPMT(0.1/12,1,36,8000)", -66.3341127119215);
    close("=PPMT(0.1/12,1,36,8000)", -190.696896763988);
    close("=FV(0.05/12,12,-100,-1000)", -103888.3042390117);
    close("=PV(0.05/12,12,-100)", 1168.122200429816);
    close("=RATE(36,-257.026,8000)", 0.008333333);
    // The catalog still prints 10.103; the committed JS NPER formula yields
    // this value for the same arguments and is the parity reference here.
    close("=NPER(0.05/12,-100,1000)", 10.235572520782311);
    close("=NPV(0.1,-1000,100,200,300,400,500)", 65.25883105351687);
    close("=IRR([-1000,100,200,300,400,500])", 0.1200576195419627);
    close("=MIRR([-1000,100,200,300,400,500],0.1,0.1)", 0.09296228706421239);
}

#[test]
fn extended_financial_vectors_match_the_preview() {
    // Serials 46022 and 46387 are exactly 365 days apart, so the JS root
    // finder lands on its 0.1 seed. The catalog's 0.109 is stale.
    close("=XIRR([-1000,1100],[46022,46387])", 0.1);
    close("=XNPV(0.1,[-1000,1100],[46022,46387])", 0.0);
    close("=SLN(10000,1000,10)", 900.0);
    close("=SYD(10000,1000,10,1)", 1636.3636363636363);
    close("=DB(10000,1000,10,1)", 2060.0);
    close("=DDB(10000,1000,10,1)", 2000.0);
    close("=EFFECT(0.05,12)", 0.05116189788173298);
    close("=NOMINAL(0.051161897881733,12)", 0.05);
}

#[test]
fn date_vectors_match_the_preview() {
    close("=NETWORKDAYS(\"2026-03-02\",\"2026-03-08\")", 5.0);
    close("=NETWORKDAYS.INTL(\"2026-03-02\",\"2026-03-08\",1)", 5.0);
    close("=EOMONTH(\"2026-02-10\",1)", 46086.0);
    close("=EDATE(\"2026-01-31\",1)", 46079.0);
    close("=YEARFRAC(\"2026-01-01\",\"2026-07-01\",1)", 0.4958904109589041);
    close("=WEEKNUM(\"2026-01-05\",2)", 2.0);
    close("=ISOWEEKNUM(\"2026-01-01\")", 1.0);
    close("=DATEDIF(\"2026-01-01\",\"2027-03-01\",\"Y\")", 1.0);
}

#[test]
fn statistical_vectors_match_the_preview() {
    close("=CORREL([1,2,3],[2,4,6])", 1.0);
    close("=SLOPE([1,2,3],[2,4,6])", 2.0);
    close("=INTERCEPT([1,2,3],[2,4,6])", 0.0);
    close("=FORECAST.LINEAR(4,[1,2,3],[2,4,6])", 8.0);
    close("=STDEV.S([1,2,3,4])", 1.2909944487358056);
    close("=STDEV.P([1,2,3,4])", 1.118033988749895);
    close("=VAR.S([1,2,3,4])", 1.6666666666666667);
    close("=VAR.P([1,2,3,4])", 1.25);
    close("=MODE.SNGL([1,2,2,3])", 2.0);
}

#[test]
fn text_and_logic_vectors_match_the_preview() {
    assert_eq!(result("=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")"), "a-b");
    assert_eq!(result("=CONCAT(\"a\",\"b\")"), "ab");
    assert_eq!(result("=UPPER(\"aBc\")"), "ABC");
    assert_eq!(result("=LOWER(\"aBc\")"), "abc");
    close("=NUMBERVALUE(\"1.234,56\",\",\",\".\")", 1234.56);
    close("=SUM(1,2,3)", 6.0);
    close("=AVERAGE(1,2,3)", 2.0);
    close("=COUNT(1,\"x\",2)", 2.0);
    close("=IFERROR(1/0,0)", 0.0);
    close("=IFNA(#N/A,9)", 9.0);
}

#[test]
fn error_literals_are_canonical_and_sticky() {
    error("=IRR([1,2])", "#NUM!");
    error("=STDEV.S([1])", "#DIV/0!");
    error("=NPV(-1,-1000,100)", "#NUM!");
    error("=SLN(10000,1000,0)", "#DIV/0!");
    error("=EFFECT(0,12)", "#NUM!");
    error("=CORREL([1,2],[1])", "#N/A");
}

#[test]
fn normalized_formulas_keep_their_meaning_for_every_locale() {
    assert_eq!(
        normalize_formula("=SOMA(1,5;2,5)", FormulaLocale::PtBr),
        "=SUM(1.5,2.5)"
    );
    assert_eq!(
        normalize_formula("=MÉDIA(A1:A3)", FormulaLocale::PtBr),
        "=AVERAGE(A1:A3)"
    );
    assert_eq!(
        normalize_formula("=IFNA(#n/a;9)", FormulaLocale::PtBr),
        "=IFNA(#N/A,9)"
    );
    assert_eq!(
        normalize_formula("=IRR([-1000,100,200])", FormulaLocale::PtBr),
        "=IRR([-1000,100,200])"
    );
    assert_eq!(canonical_function_name("ZÄHLENWENN"), Some("COUNTIF"));
    assert_eq!(canonical_function_name("最大"), Some("MAX"));
}

#[test]
fn aliases_reach_the_same_evaluators_as_english_names() {
    close("=SOMA(1,2,3)", 6.0);
    close("=SUMA(1,2,3)", 6.0);
    close("=SOMME(1,2,3)", 6.0);
    close("=SUMME(1,2,3)", 6.0);
    close("=MÁXIMO(1,5,3)", 5.0);
    close("=最大(1,5,3)", 5.0);
    close("=REDONDEAR(2.567,2)", 2.57);
}
