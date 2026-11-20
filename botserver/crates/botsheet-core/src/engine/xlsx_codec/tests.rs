//! Offline differential/round-trip corpus for the fallback XLSX codec (#2572).
//!
//! Pure tests only: no filesystem, network, server or browser. Every vector is
//! exported to canonical XLSX, re-imported and (where meaningful) evaluated,
//! mirroring `formulas/differential_tests.rs` so an open/save cycle cannot lose
//! formula semantics. Kept in this file to keep `xlsx_codec.rs` under 450 lines.

use super::*;
use crate::formulas::evaluate_formula;
use crate::types::{DateSystem, IterativeSettings, Separators, Worksheet};

fn settings(locale: FormulaLocale, date_system: DateSystem) -> WorkbookSettings {
    WorkbookSettings {
        locale,
        date_system,
        ..WorkbookSettings::default()
    }
}

fn sheet() -> Worksheet {
    Worksheet {
        name: "Sheet1".to_string(),
        ..Default::default()
    }
}

/// Exports and re-imports a formula, asserting every hop is stable. A second
/// open/save cycle must be byte-identical to the first.
fn assert_text_round_trip(
    authored: &str,
    expected_xlsx: &str,
    workbook: &WorkbookSettings,
) -> String {
    let exported = to_xlsx_formula(authored, workbook);
    assert_eq!(exported, expected_xlsx, "export of {authored}");
    assert!(
        is_canonical_xlsx_formula(&exported),
        "export is not import-stable: {exported}"
    );

    let imported = from_xlsx_formula(&exported);
    assert_eq!(imported, exported, "import drifted for {authored}");
    assert_eq!(to_xlsx_formula(&imported, workbook), exported);
    assert_eq!(from_xlsx_formula(&imported), imported);
    imported
}

fn evaluated(formula: &str) -> String {
    let result = evaluate_formula(formula, &sheet());
    assert!(result.error.is_none(), "{formula}: {:?}", result.error);
    result.value
}

/// Numeric results are compared with a tolerance because `format_number`
/// deliberately limits the number of printed decimals.
fn close_value(formula: &str, expected: f64) {
    let value = evaluated(formula);
    let actual: f64 = value.parse().unwrap_or(f64::NAN);
    assert!(
        (actual - expected).abs() <= 1e-6,
        "{formula}: expected {expected}, got {value}"
    );
}

fn assert_value(formula: &str, expected: &str) {
    assert_eq!(evaluated(formula), expected, "{formula}");
}

#[test]
fn test_pt_br_math_and_logic_round_trip() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    let imported = assert_text_round_trip("=SOMA(1,5;2,5)", "=SUM(1.5,2.5)", &br);
    close_value(&imported, 3.0);
    let imported = assert_text_round_trip("=MÉDIA(1;2;3)", "=AVERAGE(1,2,3)", &br);
    close_value(&imported, 2.0);
    let imported = assert_text_round_trip("=MÁXIMO(1,5;3)", "=MAX(1.5,3)", &br);
    close_value(&imported, 3.0);
    let imported = assert_text_round_trip("=MÍNIMO(1,5;3)", "=MIN(1.5,3)", &br);
    close_value(&imported, 1.5);
    let imported = assert_text_round_trip("=ARRED(2,567;2)", "=ROUND(2.567,2)", &br);
    close_value(&imported, 2.57);
    let imported = assert_text_round_trip("=SE(TRUE;1;2)", "=IF(TRUE,1,2)", &br);
    close_value(&imported, 1.0);
}

#[test]
fn test_nested_and_reference_heavy_formulas_round_trip() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    let imported =
        assert_text_round_trip("=ARRED(MÉDIA(1;2;3);2)", "=ROUND(AVERAGE(1,2,3),2)", &br);
    close_value(&imported, 2.0);

    // Anchors, quoted criteria and cross-sheet references must survive
    // byte-for-byte; evaluation of an empty external sheet is not asserted.
    assert_text_round_trip(
        "=SOMASE($A$1:$A$3;\">1\";Sheet2!B1)",
        "=SUMIF($A$1:$A$3,\">1\",Sheet2!B1)",
        &br,
    );
}

#[test]
fn test_implemented_families_pass_through_xlsx_canonical_form() {
    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    for (authored, expected) in [
        ("=PMT(0.05/12,360,100000)", -536.8216230121398),
        ("=IRR([-1000,100,200,300,400,500])", 0.1200576195419627),
        ("=STDEV.S([1,2,3,4])", 1.2909944487358056),
        ("=NUMBERVALUE(\"1.234,56\",\",\",\".\")", 1234.56),
        ("=SUM(1,2,3)", 6.0),
    ] {
        let imported = assert_text_round_trip(authored, authored, &en);
        close_value(&imported, expected);
    }

    let imported =
        assert_text_round_trip("=EOMONTH(\"2026-02-10\",1)", "=EOMONTH(\"2026-02-10\",1)", &en);
    assert_value(&imported, "46086");

    let imported = assert_text_round_trip(
        "=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")",
        "=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")",
        &en,
    );
    assert_value(&imported, "a-b");
}

#[test]
fn test_error_literals_are_canonicalized_and_sticky() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    let imported = assert_text_round_trip("=IFNA(#n/a;9)", "=IFNA(#N/A,9)", &br);
    assert_value(&imported, "9");
    assert_text_round_trip("=IF(#div/0!;0;1)", "=IF(#DIV/0!,0,1)", &br);
    assert_text_round_trip("=SE(#ref!;1;2)", "=IF(#REF!,1,2)", &br);

    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    let imported = assert_text_round_trip("=IRR([1,2])", "=IRR([1,2])", &en);
    assert_value(&imported, "#NUM!");
    let imported = assert_text_round_trip("=CORREL([1,2],[1])", "=CORREL([1,2],[1])", &en);
    assert_value(&imported, "#N/A");
}

#[test]
fn test_every_locale_exports_a_canonical_idempotent_formula() {
    for locale in FormulaLocale::ALL {
        let workbook = settings(locale, DateSystem::Excel1900);
        let authored = if locale.decimal_separator() == ',' {
            "=SOMA(1,5;2,5)"
        } else {
            "=SOMA(1.5,2.5)"
        };
        let exported = to_xlsx_formula(authored, &workbook);
        assert_eq!(
            exported,
            "=SUM(1.5,2.5)",
            "locale {}: {exported}",
            locale.as_str()
        );
        assert_eq!(from_xlsx_formula(&exported), exported);
    }
}

#[test]
fn test_date_system_is_applied_to_serial_conversion() {
    let a = settings(FormulaLocale::En, DateSystem::Excel1900);
    let b = settings(FormulaLocale::En, DateSystem::Excel1904);
    let serial_a = date_to_workbook_serial(2026, 11, 20, &a);
    let serial_b = date_to_workbook_serial(2026, 11, 20, &b);
    assert!(serial_a.is_some() && serial_b.is_some());
    if let (Some(sa), Some(sb)) = (serial_a, serial_b) {
        assert_eq!(sa - sb, crate::engine::EPOCH_DIFF_DAYS);
        assert_eq!(workbook_serial_to_date(sa as f64, &a), Some((2026, 11, 20)));
        assert_eq!(workbook_serial_to_date(sb as f64, &b), Some((2026, 11, 20)));
        // Reading a 1900 serial with a 1904 workbook shifts the date —
        // exactly the drift the export path must avoid.
        assert_ne!(workbook_serial_to_date(sa as f64, &b), Some((2026, 11, 20)));
    }
}

#[test]
fn test_bare_xlsx_formula_text_is_supported() {
    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    let exported = to_xlsx_formula("SUM(A1:B2,1.5)", &en);
    assert_eq!(exported, "SUM(A1:B2,1.5)");
    assert_eq!(from_xlsx_formula(&exported), exported);
    close_value("=SUM(1.5,2.5)", 3.0);
}

/// Twenty numeric reference vectors across financial, date, statistical and
/// text/base families, each exported, re-imported and evaluated. The values
/// are the committed preview vectors from `formulas/differential_tests.rs`.
#[test]
fn test_twenty_numeric_family_vectors_round_trip() {
    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    for (authored, expected) in [
        // financial
        ("=PMT(0.05/12,360,100000)", -536.8216230121398),
        ("=IPMT(0.1/12,1,36,8000)", -66.3341127119215),
        ("=PPMT(0.1/12,1,36,8000)", -190.696896763988),
        ("=FV(0.05/12,12,-100,-1000)", -103888.3042390117),
        ("=PV(0.05/12,12,-100)", 1168.122200429816),
        ("=RATE(36,-257.026,8000)", 0.008333333),
        ("=NPV(0.1,-1000,100,200,300,400,500)", 65.25883105351687),
        ("=IRR([-1000,100,200,300,400,500])", 0.1200576195419627),
        ("=MIRR([-1000,100,200,300,400,500],0.1,0.1)", 0.09296228706421239),
        ("=SLN(10000,1000,10)", 900.0),
        // date
        ("=EDATE(\"2026-01-31\",1)", 46079.0),
        ("=NETWORKDAYS(\"2026-03-02\",\"2026-03-08\")", 5.0),
        ("=YEARFRAC(\"2026-01-01\",\"2026-07-01\",1)", 0.4958904109589041),
        // statistical
        ("=CORREL([1,2,3],[2,4,6])", 1.0),
        ("=SLOPE([1,2,3],[2,4,6])", 2.0),
        ("=STDEV.P([1,2,3,4])", 1.118033988749895),
        ("=VAR.S([1,2,3,4])", 1.6666666666666667),
        // text/base and core math
        ("=SUM(1,2,3)", 6.0),
        ("=AVERAGE(1,2,3)", 2.0),
        ("=NUMBERVALUE(\"1.234,56\",\",\",\".\")", 1234.56),
    ] {
        let imported = assert_text_round_trip(authored, authored, &en);
        close_value(&imported, expected);
    }

    for (authored, expected) in [
        ("=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")", "a-b"),
        ("=CONCAT(\"a\",\"b\")", "ab"),
        ("=UPPER(\"aBc\")", "ABC"),
        ("=LOWER(\"aBc\")", "abc"),
    ] {
        let imported = assert_text_round_trip(authored, authored, &en);
        assert_value(&imported, expected);
    }
}

/// Comma-decimal locales spell the same value differently; all four must
/// converge on the identical canonical English formula.
#[test]
fn test_comma_decimal_locales_converge_on_one_canonical_form() {
    for (alias, localized) in [
        ("pt-BR", "=SOMA(1,5;2,5)"),
        ("es", "=SUMA(1,5;2,5)"),
        ("fr", "=SOMME(1,5;2,5)"),
        ("de", "=SUMME(1,5;2,5)"),
    ] {
        let workbook = settings(FormulaLocale::parse(alias), DateSystem::Excel1900);
        let imported = assert_text_round_trip(localized, "=SUM(1.5,2.5)", &workbook);
        close_value(&imported, 3.0);
    }
}

#[test]
fn test_localized_cases_normalize_deterministically() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    // (authored, canonical XLSX, optional evaluated numeric result)
    for (authored, expected, value) in [
        ("=ARRED(MÉDIA(1;2;3);2)", "=ROUND(AVERAGE(1,2,3),2)", Some(2.0)),
        ("=SE(TRUE;1;2)", "=IF(TRUE,1,2)", Some(1.0)),
        ("=MÁXIMO(1,5;3)", "=MAX(1.5,3)", Some(3.0)),
        ("=MÍNIMO(1,5;3)", "=MIN(1.5,3)", Some(1.5)),
        (
            "=SOMASE($A$1:$A$3;\">1\";Sheet2!B1)",
            "=SUMIF($A$1:$A$3,\">1\",Sheet2!B1)",
            None,
        ),
        (
            "=IRR([-1000;100;200;300;400;500])",
            "=IRR([-1000,100,200,300,400,500])",
            Some(0.1200576195419627),
        ),
        (
            "=CORREL([1;2;3];[2;4;6])",
            "=CORREL([1,2,3],[2,4,6])",
            Some(1.0),
        ),
        ("=IFNA(#n/a;9)", "=IFNA(#N/A,9)", Some(9.0)),
        ("=SE(#div/0!;1;2)", "=IF(#DIV/0!,1,2)", None),
    ] {
        let imported = assert_text_round_trip(authored, expected, &br);
        if let Some(expected) = value {
            close_value(&imported, expected);
        }
    }
}

#[test]
fn test_locale_aliases_are_deterministic_and_idempotent() {
    for locale in FormulaLocale::ALL {
        assert_eq!(FormulaLocale::parse(locale.as_str()), locale);
        assert_eq!(FormulaLocale::parse(locale.as_str()).as_str(), locale.as_str());
    }
    for (alias, parsed) in [
        ("EN_US", FormulaLocale::En),
        ("PT_BR", FormulaLocale::PtBr),
        ("  pt-br  ", FormulaLocale::PtBr),
        ("ZH_CN", FormulaLocale::ZhCn),
        ("unsupported", FormulaLocale::En),
    ] {
        assert_eq!(FormulaLocale::parse(alias), parsed, "alias {alias}");
    }
    // Whichever spelling is used, the exported bytes are identical.
    for alias in ["PT_BR", "pt-BR", "  PtBr  ", "pt_br"] {
        let workbook = settings(FormulaLocale::parse(alias), DateSystem::Excel1900);
        let exported = to_xlsx_formula("=SOMA(1,5;2,5)", &workbook);
        assert_eq!(exported, "=SUM(1.5,2.5)", "alias {alias}");
        assert_eq!(to_xlsx_formula(&exported, &workbook), exported);
    }
}

#[test]
fn test_all_error_literals_survive_both_directions() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    for (authored, expected) in [
        ("=IF(#div/0!;0;1)", "=IF(#DIV/0!,0,1)"),
        ("=IFNA(#n/a;9)", "=IFNA(#N/A,9)"),
        ("=IFERROR(#value!;0)", "=IFERROR(#VALUE!,0)"),
        ("=SE(#ref!;1;2)", "=IF(#REF!,1,2)"),
        ("=IF(#name?;0;1)", "=IF(#NAME?,0,1)"),
    ] {
        assert_text_round_trip(authored, expected, &br);
    }
    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    for code in ["#DIV/0!", "#N/A", "#VALUE!", "#REF!", "#NUM!"] {
        let formula = format!("=IFERROR({code},0)");
        assert_text_round_trip(&formula, &formula, &en);
    }
}

#[test]
fn test_separators_brackets_and_leading_equals_are_stable() {
    let br = settings(FormulaLocale::PtBr, DateSystem::Excel1900);
    assert_text_round_trip("=SOMA(1.234,50;2,5)", "=SUM(1234.50,2.5)", &br);
    assert_text_round_trip("=SOMA(1,5;2,5)", "=SUM(1.5,2.5)", &br);
    assert_text_round_trip("=IRR([-1000;100;200])", "=IRR([-1000,100,200])", &br);

    let en = settings(FormulaLocale::En, DateSystem::Excel1900);
    assert_text_round_trip("SUM(A1:B2,1.5)", "SUM(A1:B2,1.5)", &en);
    assert_text_round_trip("=SUM(A1:B2,1.5)", "=SUM(A1:B2,1.5)", &en);
    assert_eq!(to_xlsx_formula("A1+B2", &br), "A1+B2");
    assert_eq!(to_xlsx_formula("", &en), "");
    assert_eq!(from_xlsx_formula(""), "");
}

#[test]
fn test_both_epochs_round_trip_with_1900_leap_quirk_and_fixed_offset() {
    let a = settings(FormulaLocale::En, DateSystem::Excel1900);
    let b = settings(FormulaLocale::En, DateSystem::Excel1904);

    // The 1904 epoch cannot encode dates before 1904-01-01 as a non-negative
    // serial, so each epoch is exercised over its representable domain.
    for (workbook, label, dates) in [
        (&a, "1900", [
            (1900, 1, 1), (1900, 2, 28), (1900, 3, 1), (1904, 1, 1),
            (1970, 1, 1), (2026, 11, 20), (2100, 3, 1),
        ]),
        (&b, "1904", [
            (1904, 1, 1), (1904, 1, 2), (1904, 2, 29), (1904, 3, 1),
            (1970, 1, 1), (2026, 11, 20), (2100, 3, 1),
        ]),
    ] {
        for (year, month, day) in dates {
            let serial = date_to_workbook_serial(year, month, day, workbook);
            assert!(serial.is_some(), "{label} serial for {year}-{month}-{day}");
            if let Some(serial) = serial {
                let parts = Some((year, month, day));
                assert_eq!(workbook_serial_to_date(serial as f64, workbook), parts);
                assert_eq!(workbook_serial_to_date(serial as f64 + 0.5, workbook), parts);
            }
        }
    }

    // 1900 leap-year quirk: serial 60 is the phantom 1900-02-29.
    assert_eq!(date_to_workbook_serial(1900, 2, 28, &a), Some(59));
    assert_eq!(workbook_serial_to_date(59.0, &a), Some((1900, 2, 28)));
    assert_eq!(workbook_serial_to_date(60.0, &a), Some((1900, 2, 29)));
    assert_eq!(date_to_workbook_serial(1900, 2, 29, &a), None);
    assert_eq!(date_to_workbook_serial(1900, 3, 1, &a), Some(61));
    assert_eq!(workbook_serial_to_date(61.0, &a), Some((1900, 3, 1)));
    // The 1904 system has no phantom day: serial 60 is 1904-03-01.
    assert_eq!(date_to_workbook_serial(1904, 1, 1, &b), Some(0));
    assert_eq!(workbook_serial_to_date(60.0, &b), Some((1904, 3, 1)));

    for (year, month, day) in [(1904, 1, 1), (1970, 1, 1), (2026, 11, 20), (2100, 3, 1)] {
        let sa = date_to_workbook_serial(year, month, day, &a);
        let sb = date_to_workbook_serial(year, month, day, &b);
        assert!(sa.is_some() && sb.is_some(), "offset at {year}-{month}-{day}");
        if let (Some(sa), Some(sb)) = (sa, sb) {
            assert_eq!(sa - sb, crate::engine::EPOCH_DIFF_DAYS, "offset at {year}");
            assert_eq!(sa - sb, 1462, "offset at {year}");
        }
    }
    assert_eq!(crate::engine::EPOCH_DIFF_DAYS, 1462);
}

#[test]
fn test_iterative_settings_clamp_and_recurrence_converges() {
    let raw = [
        IterativeSettings { enabled: true, max_iterations: 0, tolerance: 0.0 },
        IterativeSettings { enabled: true, max_iterations: 5000, tolerance: -1.0 },
        IterativeSettings { enabled: true, max_iterations: 42, tolerance: f64::NAN },
        IterativeSettings { enabled: true, max_iterations: 7, tolerance: f64::INFINITY },
        IterativeSettings { enabled: true, max_iterations: 1, tolerance: 1e-12 },
        IterativeSettings { enabled: true, max_iterations: 1000, tolerance: 1e-9 },
    ];
    for iterative in raw {
        let normalized = WorkbookSettings {
            iterative,
            separators: Separators { decimal: ',', group: ',' },
            locale: FormulaLocale::PtBr,
            ..WorkbookSettings::default()
        }
        .normalized();
        assert!((1_u32..=1000).contains(&normalized.iterative.max_iterations));
        assert!(normalized.iterative.tolerance.is_finite());
        assert!(normalized.iterative.tolerance > 0.0);
        assert_eq!(normalized.separators, FormulaLocale::PtBr.separators());
    }
    let capped = WorkbookSettings { iterative: raw[1], ..WorkbookSettings::default() }.normalized();
    assert_eq!(capped.iterative.max_iterations, 1000);
    assert_eq!(capped.iterative.tolerance, 1e-7);
    let floored = WorkbookSettings { iterative: raw[0], ..WorkbookSettings::default() }.normalized();
    assert_eq!(floored.iterative.max_iterations, 1);
    assert_eq!(floored.iterative.tolerance, 1e-7);

    // Deterministic Babylonian recurrence for sqrt(2), bounded by the
    // normalized settings; it must converge strictly inside the tolerance.
    let workbook = WorkbookSettings::iterative_enabled();
    let target = 2.0_f64;
    let mut x = 1.0_f64;
    let mut steps = 0_u32;
    loop {
        assert!(
            steps < workbook.iterative.max_iterations,
            "recurrence did not converge within {} steps",
            workbook.iterative.max_iterations
        );
        let next = (x + target / x) / 2.0;
        let delta = (next - x).abs();
        x = next;
        steps += 1;
        if delta <= workbook.iterative.tolerance {
            break;
        }
    }
    assert!(steps <= workbook.iterative.max_iterations);
    assert!((x * x - target).abs() <= workbook.iterative.tolerance);
}
