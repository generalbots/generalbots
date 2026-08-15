use super::*;

#[test]
fn thousands_and_decimals() {
    let f = parse_format("#,##0.00");
    assert!(f.use_thousands);
    assert_eq!(f.min_decimal_digits, 2);
    assert_eq!(render_number(1234567.8, &f), "1,234,567.80");
}

#[test]
fn currency_brl() {
    let f = parse_format("R$ #,##0.00");
    assert_eq!(f.currency.as_deref(), Some("R$"));
    assert_eq!(render_number(1234.5, &f), "R$ 1,234.50");
}

#[test]
fn percent_scales() {
    let f = parse_format("0.0%");
    assert_eq!(render_number(0.125, &f), "12.5%");
}

#[test]
fn integer_only() {
    let f = parse_format("0");
    assert_eq!(render_number(12.9, &f), "13");
}

#[test]
fn negative_rounds() {
    let f = parse_format("#,##0.00");
    assert_eq!(render_number(-1234.567, &f), "-1,234.57");
}

#[test]
fn scientific_notation() {
    let f = parse_format("0.00E+00");
    assert!(f.scientific);
    assert_eq!(render_number(12345.0, &f), "1.23E+04");
    assert_eq!(render_number(0.0012, &f), "1.20E-03");
}

#[test]
fn fractions() {
    assert_eq!(render_number(0.5, &parse_format("# ?/?")), "1/2");
    assert_eq!(render_number(1.5, &parse_format("# ?/?")), "1 1/2");
    assert_eq!(render_number(1.0 / 3.0, &parse_format("# ??/??")), "1/3");
}

#[test]
fn accounting_parenthesises_negatives() {
    let f = parse_format("_($* #,##0.00_);_($* (#,##0.00);_($* \"-\"??_);_(@_)");
    assert!(f.accounting);
    assert!(f.neg_in_parens);
    assert!(f.use_thousands);
    assert_eq!(f.min_decimal_digits, 2);
    assert_eq!(render_number(1234.5, &f), "$ 1,234.50");
    assert_eq!(render_number(-1234.5, &f), "$ (1,234.50)");
}

#[test]
fn locale_pt_br_swaps_separators() {
    let f = parse_format("R$ #,##0.00");
    assert_eq!(render_number_locale(1234.5, &f, NumberLocale::PT), "R$ 1.234,50");
    assert_eq!(render_number_locale(1234.5, &f, NumberLocale::EN), "R$ 1,234.50");
}

#[test]
fn currency_negative_without_accounting() {
    let f = parse_format("$#,##0.00");
    assert!(!f.neg_in_parens);
    assert_eq!(render_number(-1234.5, &f), "$ -1,234.50");
}

#[test]
fn quoted_currency_symbol_renders() {
    // Library-generated files often quote the symbol: `"R$" #,##0.00`.
    let f = parse_format("\"R$\" #,##0.00");
    assert_eq!(f.currency.as_deref(), Some("R$"));
    assert_eq!(render_number(1234.5, &f), "R$ 1,234.50");
}

#[test]
fn locale_token_is_stripped() {
    // `[$€-407]#,##0.00` loses the token but keeps the numeric pattern.
    let f = parse_format("[$€-407]#,##0.00");
    assert_eq!(f.currency, None);
    assert!(f.use_thousands);
    assert_eq!(f.min_decimal_digits, 2);
    assert_eq!(render_number(1234.5, &f), "1,234.50");
}

#[test]
fn date_format() {
    let f = parse_format("yyyy-mm-dd");
    assert!(f.is_date);
    // Serial 45658 == 2025-01-01 (1900 date system, off-by-one for the
    // fake 1900-02-29 is already folded into the 1899-12-30 base).
    assert_eq!(render_number(45658.0, &f), "2025-01-01");
}

#[test]
fn time_of_day_renders_fractional_serial() {
    // 0.5 == 1899-12-30 12:00:00 (noon); a time-only format must not render
    // the epoch date's midnight.
    let f = parse_format("hh:mm:ss");
    assert!(f.is_date);
    assert_eq!(render_number(0.5, &f), "12:00:00");
}

#[test]
fn date_and_time_renders_both() {
    let f = parse_format("yyyy-mm-dd hh:mm");
    assert_eq!(render_number(45658.5, &f), "2025-01-01 12:00");
}

#[test]
fn minutes_disambiguated_from_month() {
    // `mm` after an hour code is minutes, not month: noon → "12:00".
    assert_eq!(render_number(0.5, &parse_format("h:mm")), "12:00");
    // `mm` before seconds is clock minutes (`mm:ss`): noon → "00:00".
    assert_eq!(render_number(0.5, &parse_format("mm:ss")), "00:00");
    // `mm` in a date position is still a month: serial 45658 → "01/01/2025".
    assert_eq!(render_number(45658.0, &parse_format("mm/dd/yyyy")), "01/01/2025");
}

#[test]
fn am_pm_renders() {
    let f = parse_format("h:mm AM/PM");
    assert_eq!(render_number(0.5, &f), "12:00 PM");
}

#[test]
fn optional_decimal_places_are_kept() {
    // `0.##` shows up to two decimals, trimming trailing zeros — the `#`
    // placeholders must not be dropped so the value rounds to an integer.
    let f = parse_format("0.##");
    assert_eq!(f.min_decimal_digits, 0);
    assert_eq!(f.max_decimal_digits, 2);
    assert_eq!(render_number(1.5, &f), "1.5");
    assert_eq!(render_number(1.25, &f), "1.25");
    assert_eq!(render_number(1.256, &f), "1.26");
    assert_eq!(render_number(1.0, &f), "1");
}

#[test]
fn mixed_forced_and_optional_decimals() {
    // `0.0#` forces one decimal, allows a second: 1.0 → "1.0", 1.2 → "1.2".
    let f = parse_format("0.0#");
    assert_eq!(f.min_decimal_digits, 1);
    assert_eq!(f.max_decimal_digits, 2);
    assert_eq!(render_number(1.0, &f), "1.0");
    assert_eq!(render_number(1.2, &f), "1.2");
    assert_eq!(render_number(1.25, &f), "1.25");
}

#[test]
fn zero_section_renders() {
    // `0.00;-0.00;"-"` → zero shows a dash, positives/negatives are numeric.
    let f = parse_format("0.00;-0.00;\"-\"");
    assert_eq!(f.zero_code.as_deref(), Some("\"-\""));
    assert_eq!(render_number(0.0, &f), "-");
    assert_eq!(render_number(5.0, &f), "5.00");
    assert_eq!(render_number(-5.0, &f), "-5.00");
}

#[test]
fn accounting_zero_renders_dash() {
    // The accounting zero section `_-* "-"??_-` strips its `_`/`*`/`?`
    // alignment markers to a bare dash.
    let f = parse_format("#,##0.00;[Red](#,##0.00);_-*\"-\"??_-");
    assert_eq!(f.zero_code.as_deref(), Some("_-*\"-\"??_-"));
    assert_eq!(render_number(0.0, &f), "-");
    assert_eq!(render_number(5.0, &f), "5.00");
}

#[test]
fn text_section_renders() {
    assert_eq!(
        apply_format(&CellValue::Text("42".into()), "0;0;0;\"Total: \"@"),
        "Total: 42"
    );
}

#[test]
fn text_passthrough() {
    assert_eq!(apply_format(&CellValue::Text("abc".to_string()), "#,##0"), "abc");
}

#[test]
fn display_falls_back() {
    assert_eq!(display_cell(&CellValue::Number(12.0), None), "12");
    assert_eq!(display_cell(&CellValue::Number(12.0), Some("General")), "12");
}

#[test]
fn apply_formats_to_sheet_renders_but_keeps_typed() {
    let mut sheet = crate::types::Spreadsheet {
        id: "t".into(),
        name: "Test".into(),
        owner_id: "me".into(),
        worksheets: vec![crate::types::Worksheet {
            name: "Sheet1".into(),
            data: std::collections::HashMap::new(),
            ..crate::types::Worksheet::default()
        }],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: std::collections::HashMap::new(),
    };
    sheet.worksheets[0].data.insert(
        "0,0".into(),
        crate::types::CellData {
            value: Some("1234.5".into()),
            typed: Some(CellValue::Number(1234.5)),
            formula: None,
            style: None,
            format: Some("#,##0.00".into()),
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        },
    );
    super::apply_formats_to_sheet(&mut sheet);
    let cell = &sheet.worksheets[0].data["0,0"];
    assert_eq!(cell.value.as_deref(), Some("1,234.50"));
    assert_eq!(cell.typed, Some(CellValue::Number(1234.5)));
}
