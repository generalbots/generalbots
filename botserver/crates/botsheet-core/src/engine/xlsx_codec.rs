//! Deterministic XLSX <-> engine formula codec (#2572).
//!
//! The binary `.xlsx` I/O lives in the `botsheet` crate (`rust_xlsxwriter` on
//! export, `umya_spreadsheet` on import). This module is the engine-side
//! contract that path should call so an open/save cycle is idempotent:
//!
//! * **export** — [`to_xlsx_formula`] canonicalizes a preview/engine formula
//!   with the workbook's locale, [`date_to_workbook_serial`] encodes dates with
//!   the workbook's 1900/1904 epoch, and the caller sets the workbook's
//!   `date1904` flag from [`crate::types::WorkbookSettings::date_system`];
//! * **import** — [`from_xlsx_formula`] normalizes the XLSX (English) spelling
//!   into the engine's canonical form and [`workbook_serial_to_date`] decodes
//!   serials with the same epoch.
//!
//! A leading `=` is preserved when present and never invented, so the codec
//! works both with engine formulas (`=SUM(A1)`) and with the bare text carried
//! in an XLSX `<f>` element (`SUM(A1)`).
//!
//! Writer note: `rust_xlsxwriter` 0.102.0 has no public API for the workbook
//! `date1904` flag, so a writer using it cannot fully honor
//! [`crate::types::DateSystem::Excel1904`] without patching `xl/workbook.xml`.
//! The umya writer (`botsheet/src/storage/xlsx_write.rs`) exposes
//! `set_date1904` and is the practical path for that flag.

use crate::engine::{date_to_serial, normalize_formula, serial_to_date_parts};
use crate::types::{FormulaLocale, WorkbookSettings};

/// Converts an engine/preview formula into the canonical XLSX spelling using
/// the workbook's locale (English names, dot decimals, comma separators,
/// uppercase error literals). Bracketed preview arrays keep their brackets.
pub fn to_xlsx_formula(formula: &str, settings: &WorkbookSettings) -> String {
    normalize_formula(formula, settings.locale)
}

/// Converts an XLSX formula into the engine's canonical spelling. XLSX already
/// stores English names, dot decimals and comma separators, so normalization
/// with the English locale is a deterministic no-op on already-saved formulas.
pub fn from_xlsx_formula(formula: &str) -> String {
    normalize_formula(formula, FormulaLocale::En)
}

/// True when a formula is already in the canonical form that import produces.
/// Exporting and re-importing a formula must always satisfy this invariant.
pub fn is_canonical_xlsx_formula(formula: &str) -> bool {
    from_xlsx_formula(formula) == formula
}

/// Encodes a calendar date as the serial used by the workbook's date system.
pub fn date_to_workbook_serial(
    year: i32,
    month: u32,
    day: u32,
    settings: &WorkbookSettings,
) -> Option<i64> {
    date_to_serial(year, month, day, settings.date_system)
}

/// Decodes a workbook serial (fractional part = time of day) into calendar
/// parts using the workbook's date system.
pub fn workbook_serial_to_date(
    serial: f64,
    settings: &WorkbookSettings,
) -> Option<(i32, u32, u32)> {
    serial_to_date_parts(serial, settings.date_system)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulas::evaluate_formula;
    use crate::types::{DateSystem, Worksheet};

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

    /// Exports and re-imports a formula, asserting every hop is stable. A
    /// second open/save cycle must be byte-identical to the first.
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
    fn pt_br_math_and_logic_round_trip() {
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
    fn nested_and_reference_heavy_formulas_round_trip() {
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
    fn implemented_families_pass_through_xlsx_canonical_form() {
        let en = settings(FormulaLocale::En, DateSystem::Excel1900);
        // Financial (including bracketed preview arrays), date, statistical,
        // text/base. These names are already canonical; export/import must not
        // alter them and evaluation must keep producing the reference value.
        let cases: [(&str, f64); 5] = [
            ("=PMT(0.05/12,360,100000)", -536.8216230121398),
            ("=IRR([-1000,100,200,300,400,500])", 0.1200576195419627),
            ("=STDEV.S([1,2,3,4])", 1.2909944487358056),
            ("=NUMBERVALUE(\"1.234,56\",\",\",\".\")", 1234.56),
            ("=SUM(1,2,3)", 6.0),
        ];
        for (authored, expected) in cases {
            let imported = assert_text_round_trip(authored, authored, &en);
            close_value(&imported, expected);
        }

        let imported = assert_text_round_trip(
            "=EOMONTH(\"2026-02-10\",1)",
            "=EOMONTH(\"2026-02-10\",1)",
            &en,
        );
        assert_value(&imported, "46086");

        let imported = assert_text_round_trip(
            "=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")",
            "=TEXTJOIN(\"-\",TRUE,\"a\",\"\",\"b\")",
            &en,
        );
        assert_value(&imported, "a-b");
    }

    #[test]
    fn error_literals_are_canonicalized_and_sticky() {
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
    fn every_locale_exports_a_canonical_idempotent_formula() {
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
    fn date_system_is_applied_to_serial_conversion() {
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
    fn bare_xlsx_formula_text_is_supported() {
        let en = settings(FormulaLocale::En, DateSystem::Excel1900);
        let exported = to_xlsx_formula("SUM(A1:B2,1.5)", &en);
        assert_eq!(exported, "SUM(A1:B2,1.5)");
        assert_eq!(from_xlsx_formula(&exported), exported);
        close_value("=SUM(1.5,2.5)", 3.0);
    }
}
