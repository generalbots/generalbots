//! Helpers shared by more than one exporter.

use botsheet_core::engine::value::CellValue;
use botsheet_core::types::CellData;

/// The numeric value a cell should export as: a typed `Number`, or — for
/// untyped cells (CSV/ODS) — a display value that parses as a float. A typed
/// `Text` (`0123`) stays text so it is never exported as the number 123.
pub(super) fn cell_number(cell: &CellData) -> Option<f64> {
    match cell.typed.as_ref() {
        Some(CellValue::Number(n)) => Some(*n),
        Some(_) => None,
        None => cell.value.as_deref()?.trim().parse::<f64>().ok(),
    }
}

/// Zero-based column index to spreadsheet letters (`0 -> A`).
pub(super) fn column_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col + 1;
    while n > 0 {
        n -= 1;
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    result
}
