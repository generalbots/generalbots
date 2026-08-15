//! Typed cell values (#781).
//!
//! The legacy engine stores every cell value as a `String`. This module is the
//! typed layer the new parser produces and consumes: numbers, text, booleans,
//! logical errors and empty cells all carry their real type so arithmetic,
//! comparisons and date math no longer depend on string round-tripping.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The typed value a formula evaluates to.
///
/// Serialization is deliberately JSON-friendly: numbers stay numbers, booleans
/// stay booleans, and errors become `{"#ERR": "..."}` objects. Cells written
/// through the legacy path continue to be plain strings, and empty is a shell
/// the renderer treats as "no value".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "v", rename_all = "lowercase")]
pub enum CellValue {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
    /// A spreadsheet error value (`#DIV/0!`, `#NAME?`, …).
    Error(String),
}

impl CellValue {
    /// Builds a value from a raw cell string exactly as typing does.
    pub fn parse(input: &str) -> CellValue {
        let t = input.trim();
        if t.is_empty() {
            return CellValue::Empty;
        }
        if let Ok(b) = parse_bool(t) {
            return CellValue::Bool(b);
        }
        if let Ok(n) = t.parse::<f64>() {
            return CellValue::Number(n);
        }
        if is_error_code(t) {
            return CellValue::Error(t.to_string());
        }
        CellValue::Text(t.to_string())
    }

    /// The string form used as a cell's `value` field and displayed in the grid.
    pub fn display(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Number(n) => format_number(*n),
            CellValue::Text(s) => s.clone(),
            CellValue::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
            CellValue::Error(e) => e.clone(),
        }
    }

    /// Numeric view for arithmetic; text that is not a number yields `None`.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Text(s) => s.trim().parse::<f64>().ok(),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Empty | CellValue::Error(_) => None,
        }
    }

    /// Truthiness in the Excel convention: zero and text `FALSE` are false.
    pub fn truthy(&self) -> bool {
        match self {
            CellValue::Bool(b) => *b,
            CellValue::Number(n) => *n != 0.0,
            CellValue::Text(s) => s.eq_ignore_ascii_case("TRUE"),
            CellValue::Error(_) => false,
            CellValue::Empty => false,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, CellValue::Error(_))
    }

    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    if s.eq_ignore_ascii_case("TRUE") {
        Ok(true)
    } else if s.eq_ignore_ascii_case("FALSE") {
        Ok(false)
    } else {
        Err(())
    }
}

/// Recognises the canonical spreadsheet error codes (ECMA-376 `ST_CellType`
/// error literals plus the internal `#ERROR!` catch-all).
fn is_error_code(s: &str) -> bool {
    matches!(
        s,
        "#DIV/0!"
            | "#N/A"
            | "#NAME?"
            | "#NULL!"
            | "#NUM!"
            | "#REF!"
            | "#VALUE!"
            | "#SPILL!"
            | "#CALC!"
            | "#GETTING_DATA"
            | "#ERROR!"
    )
}

/// Formats a number the way a spreadsheet displays it: integers without a
/// decimal point, other values trimmed to six significant decimals.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        // `{:.0}` (not `as i64`) — an `as` cast saturates for magnitudes beyond
        // i64 (e.g. =10^20), silently showing a wrong value.
        format!("{:.0}", n)
    } else {
        let s = format!("{:.6}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_preserves_variants() {
        let cases = vec![
            CellValue::Empty,
            CellValue::Number(1.5),
            CellValue::Number(2.0),
            CellValue::Text("hello".into()),
            CellValue::Bool(true),
            CellValue::Error("#DIV/0!".into()),
        ];
        for c in cases {
            let json = serde_json::to_string(&c).expect("serialize");
            let back: CellValue = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, c, "round-trip mismatch for {json}");
        }
    }

    #[test]
    fn serde_shape_is_tagged_json() {
        let json = serde_json::to_string(&CellValue::Number(42.0)).expect("serialize");
        assert_eq!(json, r#"{"t":"number","v":42.0}"#);
    }

    #[test]
    fn parse_detects_numeric_and_boolean_inputs() {
        assert_eq!(CellValue::parse("123"), CellValue::Number(123.0));
        assert_eq!(CellValue::parse("12.5"), CellValue::Number(12.5));
        assert_eq!(CellValue::parse("TRUE"), CellValue::Bool(true));
        assert_eq!(CellValue::parse("hello"), CellValue::Text("hello".into()));
    }
}
