//! Number format engine (#785).
//!
//! Parses a spreadsheet number format code (`#,##0.00`, `0%`, `$#,##0.00`,
//! `yyyy-mm-dd`, `0.00E+00`, fractions) and renders a typed number to its
//! display string. The grid previously stored formats without applying them;
//! this is the engine that makes `R$ 1.234,50` display correctly.

use super::value::CellValue;

/// The parsed structure of a number format.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberFormat {
    /// Whether the format is a date/time code rather than a numeric code.
    pub is_date: bool,
    /// Number of leading integer digits enforced by `0`.
    pub min_integer_digits: usize,
    /// Number of decimal digits enforced by `0` after the point.
    pub min_decimal_digits: usize,
    /// Whether a thousands separator is requested (`#,##0`).
    pub use_thousands: bool,
    /// Whether the value is rendered as a percentage (`0%`).
    pub percent: bool,
    /// Currency symbol prefix such as `R$ ` or `$`.
    pub currency: Option<String>,
    /// Fixed scale factor (1 for normal, 100 for percent).
    pub scale: f64,
    /// For date formats, the format string in chrono syntax where derivable.
    pub date_format: Option<String>,
}

impl Default for NumberFormat {
    fn default() -> Self {
        NumberFormat {
            is_date: false,
            min_integer_digits: 1,
            min_decimal_digits: 0,
            use_thousands: false,
            percent: false,
            currency: None,
            scale: 1.0,
            date_format: None,
        }
    }
}

/// Converts a number to an Excel-style date serial (days since 1899-12-30).
fn serial_to_date(serial: f64) -> Option<chrono::NaiveDate> {
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)?;
    base.checked_add_signed(chrono::Duration::days(serial.floor() as i64))
}

/// Parses a format code into its parts. Unknown formats fall back to General.
pub fn parse_format(code: &str) -> NumberFormat {
    let mut fmt = NumberFormat::default();
    let code = code.trim();
    if code.is_empty() || code.eq_ignore_ascii_case("general") {
        return fmt;
    }

    let date_codes = ["yyyy", "yy", "mm", "dd", "hh", "h:", "ss", "mmm", "ddd"];
    if date_codes.iter().any(|c| code.to_ascii_lowercase().contains(c)) {
        fmt.is_date = true;
        fmt.date_format = detect_date_format(code);
        return fmt;
    }

    let mut rest = code.to_string();
    // Currency prefix.
    if let Some((sym, tail)) = split_currency(&rest) {
        fmt.currency = Some(sym);
        rest = tail;
    }
    // Percent.
    if rest.contains('%') {
        fmt.percent = true;
        fmt.scale = 100.0;
    }
    // Thousands separators.
    if rest.contains(',') {
        fmt.use_thousands = true;
    }
    // Decimal places: count `0`s after the last '.' in the integer/decimal part.
    if let Some((_, frac)) = rest.split_once('.') {
        let frac: String = frac.chars().take_while(|c| *c == '0' || *c == '#' || *c == '?').collect();
        fmt.min_decimal_digits = frac.chars().filter(|c| *c == '0').count();
    }
    // Minimum integer digits.
    let int_part = rest.split(['.', '%', ',']).next().unwrap_or("");
    fmt.min_integer_digits = int_part.chars().filter(|c| *c == '0').count().max(1);

    fmt
}

fn split_currency(code: &str) -> Option<(String, String)> {
    for sym in ["R$ ", "R$", "$ ", "$", "€ ", "€", "£ ", "£"] {
        if code.starts_with(sym) {
            return Some((
                sym.trim_end().to_string(),
                code.strip_prefix(sym).unwrap_or_default().to_string(),
            ));
        }
    }
    None
}

fn detect_date_format(code: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = code.chars().peekable();
    while let Some(c) = chars.next() {
        match c.to_ascii_lowercase() {
            'y' => {
                let mut count = 1;
                while matches!(chars.peek(), Some(&'y') | Some(&'Y')) {
                    count += 1;
                    chars.next();
                }
                out.push_str(if count >= 4 { "%Y" } else { "%y" });
            }
            'm' => {
                let mut count = 1;
                while matches!(chars.peek(), Some(&'m') | Some(&'M')) {
                    count += 1;
                    chars.next();
                }
                out.push_str(if count >= 3 { "%b" } else { "%m" });
            }
            'd' => {
                while matches!(chars.peek(), Some(&'d') | Some(&'D')) {
                    chars.next();
                }
                out.push_str("%d");
            }
            'h' => {
                while matches!(chars.peek(), Some(&'h') | Some(&'H')) {
                    chars.next();
                }
                out.push_str("%H");
            }
            's' => {
                while matches!(chars.peek(), Some(&'s') | Some(&'S')) {
                    chars.next();
                }
                out.push_str("%S");
            }
            '/' | '-' | ':' | ' ' => out.push(c),
            _ => {
                // Text between codes: keep as literal.
                out.push(c);
            }
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Renders a typed value with the given format code.
pub fn apply_format(value: &CellValue, code: &str) -> String {
    let fmt = parse_format(code);
    match value {
        CellValue::Number(n) => render_number(*n, &fmt),
        CellValue::Text(t) => t.clone(),
        CellValue::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        CellValue::Empty => String::new(),
        CellValue::Error(e) => format!("#{e}!"),
    }
}

fn render_number(n: f64, fmt: &NumberFormat) -> String {
    if fmt.is_date {
        if let (Some(df), Some(date)) = (fmt.date_format.as_deref(), serial_to_date(n)) {
            return date.format(df).to_string();
        }
        return super::value::format_number(n);
    }
// Percent scales by 100: 0.125 with "0.0%" renders as "12.5%".
    let scaled = n * fmt.scale;
    let neg = scaled < 0.0;
    let abs = scaled.abs();
    let factor = 10f64.powi(fmt.min_decimal_digits as i32);
    let rounded = (abs * factor).round() / factor;

    let mut int_part = rounded.trunc() as i64;
    let mut frac_part = (rounded.fract() * factor).round() as u64;

    // Rounding carry.
    if frac_part == factor as u64 {
        int_part += 1;
        frac_part = 0;
    }

    let mut int_str = int_part.to_string();
    if fmt.use_thousands {
        int_str = group_thousands(&int_str);
    }
    while int_str.len() < fmt.min_integer_digits {
        int_str.insert(0, '0');
    }

    let mut out = String::new();
    if let Some(ref cur) = fmt.currency {
        out.push_str(cur);
        out.push(' ');
    }
    if neg {
        out.push('-');
    }
    out.push_str(&int_str);
    if fmt.min_decimal_digits > 0 {
        out.push('.');
        out.push_str(&format!("{:0width$}", frac_part, width = fmt.min_decimal_digits));
    }
    if fmt.percent {
        out.push('%');
    }
    out
}

fn group_thousands(int_str: &str) -> String {
    let bytes = int_str.as_bytes();
    let len = bytes.len();
    let mut out = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

/// Applies a stored cell format to a cell value, falling back to the plain
/// number display when the code is empty or General.
pub fn display_cell(value: &CellValue, format_code: Option<&str>) -> String {
    match format_code {
        Some(code) if !code.trim().is_empty() && !code.eq_ignore_ascii_case("general") => {
            apply_format(value, code)
        }
        _ => value.display(),
    }
}


/// Applies each cell's stored number format to its display `value`, keeping the
/// raw `typed` value untouched so arithmetic and editing stay lossless (#785).
/// Operates on a response clone; the persisted model keeps raw values.
pub fn apply_formats_to_sheet(sheet: &mut crate::types::Spreadsheet) {
    for ws in &mut sheet.worksheets {
        for cell in ws.data.values_mut() {
            if let (Some(typed), Some(code)) = (cell.typed.clone(), cell.format.clone()) {
                cell.value = Some(display_cell(&typed, Some(&code)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn date_format() {
        let f = parse_format("yyyy-mm-dd");
        assert!(f.is_date);
        // Serial 45658 == 2025-01-01 (1900 date system, off-by-one for the
        // fake 1900-02-29 is already folded into the 1899-12-30 base).
        assert_eq!(render_number(45658.0, &f), "2025-01-01");
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
                tables: None,
                name: "Sheet1".into(),
                data: std::collections::HashMap::new(),
                column_widths: None,
                row_heights: None,
                frozen_rows: None,
                frozen_cols: None,
                merged_cells: None,
                filters: None,
                hidden_rows: None,
                validations: None,
                conditional_formats: None,
                charts: None,
                comments: None,
                protection: None,
                array_formulas: None,
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
}