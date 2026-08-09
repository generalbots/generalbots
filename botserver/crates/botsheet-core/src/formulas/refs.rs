//! Cell reference and range parsing.
//!
//! Every coordinate entering the formula engine passes through here, so all
//! conversions are bounds-checked and return `None` for anything that is not a
//! valid reference.

use crate::types::Worksheet;

use super::helpers::resolve_cell_value;

/// Highest column index addressable by a cell reference (XFD, zero-based).
pub const MAX_COL_INDEX: u32 = 16_383;

/// Highest row index addressable by a cell reference (row 1048576, zero-based).
pub const MAX_ROW_INDEX: u32 = 1_048_575;

/// Maximum number of cells a single range is allowed to expand to.
/// Ranges wider than this are clamped to the populated area of the worksheet
/// so that a reference such as `A1:XFD1048576` cannot block the request thread.
pub const MAX_RANGE_CELLS: u64 = 1_000_000;

/// Converts a column name such as `A`, `AB` or `XFD` into a zero-based index.
///
/// Returns `None` when the name is empty, contains non-alphabetic characters,
/// or addresses a column beyond [`MAX_COL_INDEX`]. Column names are limited to
/// three letters, which is the widest valid spreadsheet column; longer tokens
/// are named ranges or identifiers, not cell references.
pub fn col_name_to_index(name: &str) -> Option<u32> {
    if name.is_empty() || name.len() > 3 {
        return None;
    }
    let mut col: u32 = 0;
    for ch in name.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        col = col
            .checked_mul(26)?
            .checked_add(upper as u32 - 'A' as u32 + 1)?;
    }
    let index = col.checked_sub(1)?;
    if index > MAX_COL_INDEX {
        return None;
    }
    Some(index)
}

pub fn parse_range(range: &str) -> Option<((u32, u32), (u32, u32))> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_cell_ref(parts[0].trim())?;
    let end = parse_cell_ref(parts[1].trim())?;
    Some((start, end))
}

pub fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let cell_ref = cell_ref.trim();
    if cell_ref.is_empty() {
        return None;
    }
    let bytes = cell_ref.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    // A leading `$` marks an absolute reference and carries no positional meaning.
    if i < bytes.len() && bytes[i] == b'$' {
        i += 1;
    }
    let col_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    let col_end = i;

    if i < bytes.len() && bytes[i] == b'$' {
        i += 1;
    }
    let row_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let row_end = i;
    
    if col_start == col_end || row_start == row_end {
        return None;
    }
    
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < bytes.len() {
        return None;
    }
    
    let col_str = std::str::from_utf8(&bytes[col_start..col_end]).ok()?;
    let col = col_name_to_index(col_str)?;

    let row_str = std::str::from_utf8(&bytes[row_start..row_end]).ok()?;
    // Rows are one-based in every reference syntax, so row 0 is not a valid reference.
    let row = row_str.parse::<u32>().ok()?.checked_sub(1)?;
    if row > MAX_ROW_INDEX {
        return None;
    }
    Some((row, col))
}

/// Returns the highest populated row and column of the worksheet, if any.
fn used_bounds(worksheet: &Worksheet) -> Option<(u32, u32)> {
    let mut bounds: Option<(u32, u32)> = None;
    for key in worksheet.data.keys() {
        let Some((row, col)) = parse_key(key) else {
            continue;
        };
        bounds = Some(match bounds {
            Some((max_row, max_col)) => (max_row.max(row), max_col.max(col)),
            None => (row, col),
        });
    }
    bounds
}

fn parse_key(key: &str) -> Option<(u32, u32)> {
    let (row, col) = key.split_once(',')?;
    Some((row.parse().ok()?, col.parse().ok()?))
}

/// Normalizes a range so the start precedes the end and the span stays bounded.
///
/// Whole-sheet references expand to billions of coordinates, which would stall
/// the evaluating thread. Oversized ranges are therefore reduced to the
/// populated area of the worksheet and then hard-capped at [`MAX_RANGE_CELLS`].
pub fn clamp_range(
    start: (u32, u32),
    end: (u32, u32),
    worksheet: &Worksheet,
) -> ((u32, u32), (u32, u32)) {
    let lo_row = start.0.min(end.0);
    let hi_row = start.0.max(end.0);
    let lo_col = start.1.min(end.1);
    let hi_col = start.1.max(end.1);

    let span = |hi_r: u32, hi_c: u32| -> u64 {
        (u64::from(hi_r - lo_row) + 1) * (u64::from(hi_c - lo_col) + 1)
    };

    if span(hi_row, hi_col) <= MAX_RANGE_CELLS {
        return ((lo_row, lo_col), (hi_row, hi_col));
    }

    let (mut hi_row, mut hi_col) = match used_bounds(worksheet) {
        Some((max_row, max_col)) => (hi_row.min(max_row.max(lo_row)), hi_col.min(max_col.max(lo_col))),
        None => (lo_row, lo_col),
    };

    if span(hi_row, hi_col) > MAX_RANGE_CELLS {
        let cols = u64::from(hi_col - lo_col) + 1;
        let max_rows = (MAX_RANGE_CELLS / cols).max(1);
        let allowed_hi_row = lo_row.saturating_add(u32::try_from(max_rows - 1).unwrap_or(u32::MAX));
        hi_row = hi_row.min(allowed_hi_row);
        if span(hi_row, hi_col) > MAX_RANGE_CELLS {
            hi_col = lo_col;
        }
    }

    ((lo_row, lo_col), (hi_row, hi_col))
}

pub fn get_range_values(range: &str, worksheet: &Worksheet) -> Vec<f64> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        if let Ok(val) = resolve_cell_value(range.trim(), worksheet).parse::<f64>() {
            return vec![val];
        }
        return Vec::new();
    }
    let (start, end) = match parse_range(range) {
        Some(r) => clamp_range(r.0, r.1, worksheet),
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    let mut key = String::with_capacity(32);
    use std::fmt::Write;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            key.clear();
            let _ = write!(&mut key, "{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                if let Some(ref value) = cell.value {
                    if let Ok(num) = value.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        }
    }
    values
}

pub fn get_range_string_values(range: &str, worksheet: &Worksheet) -> Vec<String> {
    let (start, end) = match parse_range(range) {
        Some(r) => clamp_range(r.0, r.1, worksheet),
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    let mut key = String::with_capacity(32);
    use std::fmt::Write;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            key.clear();
            let _ = write!(&mut key, "{},{}", row, col);
            let value = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.clone())
                .unwrap_or_default();
            values.push(value);
        }
    }
    values
}

pub fn resolve_cell_references(expr: &str, worksheet: &Worksheet) -> String {
    static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
    let Some(regex) = RE
        .get_or_init(|| regex::Regex::new(r"\$?\b([A-Z]{1,3})\$?([0-9]+)\b").ok())
        .as_ref()
    else {
        log::error!("cell reference regex failed to compile; returning expression unchanged");
        return expr.to_string();
    };

    regex
        .replace_all(expr, |cap: &regex::Captures| {
            let Some(col) = col_name_to_index(&cap[1]) else {
                return cap[0].to_string();
            };
            let Some(row) = cap[2].parse::<u32>().ok().and_then(|r| r.checked_sub(1)) else {
                return cap[0].to_string();
            };
            if row > MAX_ROW_INDEX {
                return cap[0].to_string();
            }
            let key = format!("{row},{col}");

            worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.clone())
                .unwrap_or_else(|| "0".to_string())
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CellData;
    use std::collections::HashMap;

    fn numeric_sheet(rows: u32) -> Worksheet {
        let mut data = HashMap::new();
        for row in 0..rows {
            data.insert(
                format!("{row},0"),
                CellData {
                    value: Some("2".to_string()),
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                    locked: None,
                    has_comment: None,
                    array_formula_id: None,
                },
            );
        }
        Worksheet {
            data,
            ..Worksheet::default()
        }
    }

    #[test]
    fn rejects_invalid_column_names() {
        assert_eq!(col_name_to_index(""), None);
        assert_eq!(col_name_to_index("XFE"), None);
        assert_eq!(col_name_to_index("ZZZZZZZZZZ"), None);
        assert_eq!(col_name_to_index("A1"), None);
    }

    #[test]
    fn accepts_column_bounds() {
        assert_eq!(col_name_to_index("A"), Some(0));
        assert_eq!(col_name_to_index("XFD"), Some(MAX_COL_INDEX));
    }

    #[test]
    fn rejects_out_of_range_cell_refs() {
        assert_eq!(parse_cell_ref("A0"), None);
        assert_eq!(parse_cell_ref("XFE1"), None);
        assert_eq!(parse_cell_ref("A1048577"), None);
        assert_eq!(parse_cell_ref("NAMEDRANGE2024"), None);
    }

    #[test]
    fn accepts_absolute_references() {
        assert_eq!(parse_cell_ref("$A$1"), Some((0, 0)));
        assert_eq!(parse_cell_ref("$A1"), Some((0, 0)));
        assert_eq!(parse_cell_ref("A$1"), Some((0, 0)));
        assert_eq!(parse_range("$A$1:$A$3"), Some(((0, 0), (2, 0))));
    }

    #[test]
    fn resolves_absolute_references_in_arithmetic() {
        let sheet = numeric_sheet(1);
        assert_eq!(resolve_cell_references("$A$1+1", &sheet), "2+1");
    }

    #[test]
    fn accepts_cell_ref_bounds() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
        assert_eq!(
            parse_cell_ref("XFD1048576"),
            Some((MAX_ROW_INDEX, MAX_COL_INDEX))
        );
    }

    #[test]
    fn clamps_whole_sheet_range_to_used_area() {
        let sheet = numeric_sheet(500);
        let (start, end) = clamp_range((0, 0), (MAX_ROW_INDEX, MAX_COL_INDEX), &sheet);
        assert_eq!(start, (0, 0));
        assert_eq!(end, (499, 0));
    }

    #[test]
    fn clamps_whole_sheet_range_on_empty_sheet() {
        let sheet = Worksheet::default();
        let (start, end) = clamp_range((0, 0), (MAX_ROW_INDEX, MAX_COL_INDEX), &sheet);
        assert_eq!(start, (0, 0));
        assert_eq!(end, (0, 0));
    }

    #[test]
    fn keeps_small_ranges_intact() {
        let sheet = numeric_sheet(10);
        let (start, end) = clamp_range((2, 0), (5, 3), &sheet);
        assert_eq!(start, (2, 0));
        assert_eq!(end, (5, 3));
    }

    #[test]
    fn normalizes_reversed_ranges() {
        let sheet = numeric_sheet(10);
        let (start, end) = clamp_range((5, 3), (2, 0), &sheet);
        assert_eq!(start, (2, 0));
        assert_eq!(end, (5, 3));
    }

    #[test]
    fn sums_clamped_range_without_walking_whole_sheet() {
        let sheet = numeric_sheet(500);
        let values = get_range_values("A1:XFD1048576", &sheet);
        assert_eq!(values.len(), 500);
    }

    #[test]
    fn leaves_unresolvable_tokens_untouched() {
        let sheet = numeric_sheet(1);
        // A named range must not be rewritten as a cell value.
        assert_eq!(
            resolve_cell_references("NAMEDRANGE2024+1", &sheet),
            "NAMEDRANGE2024+1"
        );
        // A valid reference resolves to the stored value.
        assert_eq!(resolve_cell_references("A1+1", &sheet), "2+1");
    }
}
