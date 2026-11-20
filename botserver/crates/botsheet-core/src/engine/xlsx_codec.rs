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
#[path = "xlsx_codec/tests.rs"]
mod tests;
