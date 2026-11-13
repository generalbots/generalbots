//! Cross-sheet criteria functions (`SUMIFS`, `COUNTIFS`, `AVERAGEIFS`, …).
//!
//! The legacy criteria library reads ranges through a single worksheet, so a
//! generated model such as
//! `SUMIFS(Data!$G$2:$G$1201, Data!$B$2:$B$1201, $A5, Data!$F$2:$F$1201,"Revenue")`
//! summed nothing and every KPI rendered as `0` (#2152). This module resolves
//! each range against the worksheet it names and applies the criteria by
//! relative position, the way Excel aligns the ranges.

use crate::formulas::{matches_criteria, split_args};
use crate::types::Worksheet;

use super::value::{format_number, CellValue};

/// A parsed `Sheet!$A$1:$B$2` argument.
struct RangeRef {
    sheet: Option<String>,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
}

fn strip_dollars(text: &str) -> String {
    text.chars().filter(|c| *c != '$').collect()
}

fn parse_cell(text: &str) -> Option<(u32, u32)> {
    let cleaned = strip_dollars(text.trim());
    let split = cleaned
        .find(|c: char| c.is_ascii_digit())
        .filter(|index| *index > 0)?;
    let (letters, digits) = cleaned.split_at(split);
    if !letters.chars().all(|c| c.is_ascii_alphabetic()) || digits.is_empty() {
        return None;
    }
    let mut column = 0u32;
    for ch in letters.to_ascii_uppercase().chars() {
        column = column * 26 + (ch as u32 - 64);
    }
    let row: u32 = digits.parse().ok()?;
    if column == 0 || row == 0 {
        return None;
    }
    Some((row - 1, column - 1))
}

/// Splits `Sheet!A1:B2` into its sheet name and cell bounds.
fn parse_range_arg(arg: &str) -> Option<RangeRef> {
    let text = arg.trim();
    let (sheet, cells) = match text.rsplit_once('!') {
        Some((sheet, cells)) => (
            Some(
                sheet
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string(),
            ),
            cells,
        ),
        None => (None, text),
    };
    let (first, last) = match cells.split_once(':') {
        Some((first, last)) => (first, last),
        None => (cells, cells),
    };
    let (start_row, start_col) = parse_cell(first)?;
    let (end_row, end_col) = parse_cell(last)?;
    Some(RangeRef {
        sheet,
        start_row: start_row.min(end_row),
        start_col: start_col.min(end_col),
        end_row: start_row.max(end_row),
        end_col: start_col.max(end_col),
    })
}

fn worksheet_for<'a>(
    worksheets: &'a [Worksheet],
    sheet: Option<&str>,
    current: usize,
) -> Option<&'a Worksheet> {
    match sheet {
        Some(name) => worksheets.iter().find(|ws| ws.name.eq_ignore_ascii_case(name)),
        None => worksheets.get(current),
    }
}

/// Values of a range in row-major order: the display text plus the numeric
/// view used by `SUMIFS`/`AVERAGEIFS`.
fn range_cells(
    worksheets: &[Worksheet],
    reference: &RangeRef,
    current: usize,
) -> Vec<(String, Option<f64>)> {
    let Some(worksheet) = worksheet_for(worksheets, reference.sheet.as_deref(), current) else {
        return Vec::new();
    };
    let mut cells = Vec::new();
    for row in reference.start_row..=reference.end_row {
        for col in reference.start_col..=reference.end_col {
            let key = format!("{row},{col}");
            let Some(cell) = worksheet.data.get(&key) else {
                cells.push((String::new(), None));
                continue;
            };
            let text = cell.value.clone().unwrap_or_default();
            let number = cell
                .typed
                .as_ref()
                .and_then(|typed| typed.as_number())
                .or_else(|| text.trim().parse::<f64>().ok());
            cells.push((text, number));
        }
    }
    cells
}

/// Text a criteria argument compares against: literals stay literal, cell
/// references resolve to their value (so `$A5` matches the month label).
fn criterion_text(worksheets: &[Worksheet], arg: &str, current: usize) -> String {
    let text = arg.trim();
    if text.len() >= 2 {
        let quoted = (text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\''));
        if quoted {
            return text[1..text.len() - 1].to_string();
        }
    }
    let (sheet, cells) = match text.rsplit_once('!') {
        Some((sheet, cells)) => (
            Some(sheet.trim().trim_matches('\'').to_string()),
            cells.to_string(),
        ),
        None => (None, text.to_string()),
    };
    if let Some((row, col)) = parse_cell(&cells) {
        if let Some(worksheet) = worksheet_for(worksheets, sheet.as_deref(), current) {
            let key = format!("{row},{col}");
            if let Some(cell) = worksheet.data.get(&key) {
                return cell.value.clone().unwrap_or_default();
            }
        }
    }
    text.to_string()
}

fn numbers_at(cells: &[(String, Option<f64>)], index: usize) -> f64 {
    cells.get(index).and_then(|(_, number)| *number).unwrap_or(0.0)
}

/// SUMIFS(sum_range, criteria_range1, criteria1, …)
fn sumifs(args: &[&str], worksheets: &[Worksheet], current: usize) -> Option<f64> {
    if args.len() < 3 || args.len() % 2 == 0 {
        return None;
    }
    let sum_ref = parse_range_arg(args[0])?;
    let sums = range_cells(worksheets, &sum_ref, current);
    let criteria: Vec<(Vec<(String, Option<f64>)>, String)> = args[1..]
        .chunks(2)
        .filter_map(|pair| {
            if pair.len() != 2 {
                return None;
            }
            let reference = parse_range_arg(pair[0])?;
            let expected = criterion_text(worksheets, pair[1], current);
            Some((range_cells(worksheets, &reference, current), expected))
        })
        .collect();
    if criteria.len() * 2 != args.len() - 1 {
        return None;
    }
    let mut total = 0.0;
    for index in 0..sums.len() {
        let matched = criteria.iter().all(|(cells, expected)| {
            cells
                .get(index)
                .map(|(text, _)| matches_criteria(text, expected))
                .unwrap_or(false)
        });
        if matched {
            total += numbers_at(&sums, index);
        }
    }
    Some(total)
}

/// SUMIF(criteria_range, criteria, [sum_range])
fn sumif(args: &[&str], worksheets: &[Worksheet], current: usize) -> Option<f64> {
    if args.len() < 2 {
        return None;
    }
    let criteria_ref = parse_range_arg(args[0])?;
    let criteria_cells = range_cells(worksheets, &criteria_ref, current);
    let expected = criterion_text(worksheets, args[1], current);
    let sums = if args.len() > 2 {
        range_cells(worksheets, &parse_range_arg(args[2])?, current)
    } else {
        range_cells(worksheets, &criteria_ref, current)
    };
    let mut total = 0.0;
    for (index, (text, _)) in criteria_cells.iter().enumerate() {
        if matches_criteria(text, &expected) {
            total += numbers_at(&sums, index);
        }
    }
    Some(total)
}

/// AVERAGEIFS(average_range, criteria_range1, criteria1, …)
fn averageifs(args: &[&str], worksheets: &[Worksheet], current: usize) -> Option<f64> {
    if args.len() < 3 {
        return None;
    }
    let values = sumifs(args, worksheets, current)?;
    // The criteria pairs start at index 1 (index 0 is the average range), so
    // COUNTIFS receives an even argument list.
    let count = countifs(&args[1..], worksheets, current)?;
    if count == 0.0 {
        return None;
    }
    Some(values / count)
}

/// COUNTIFS(criteria_range1, criteria1, …)
fn countifs(args: &[&str], worksheets: &[Worksheet], current: usize) -> Option<f64> {
    if args.len() < 2 || args.len() % 2 != 0 {
        return None;
    }
    let criteria: Vec<(Vec<(String, Option<f64>)>, String)> = args
        .chunks(2)
        .filter_map(|pair| {
            let reference = parse_range_arg(pair[0])?;
            let expected = criterion_text(worksheets, pair[1], current);
            Some((range_cells(worksheets, &reference, current), expected))
        })
        .collect();
    if criteria.len() * 2 != args.len() {
        return None;
    }
    let rows = criteria.first().map(|(cells, _)| cells.len()).unwrap_or(0);
    let mut count = 0.0;
    for index in 0..rows {
        let matched = criteria.iter().all(|(cells, expected)| {
            cells
                .get(index)
                .map(|(text, _)| matches_criteria(text, expected))
                .unwrap_or(false)
        });
        if matched {
            count += 1.0;
        }
    }
    Some(count)
}

/// AVERAGEIF(criteria_range, criteria, [average_range])
fn averageif(args: &[&str], worksheets: &[Worksheet], current: usize) -> Option<f64> {
    if args.len() < 2 {
        return None;
    }
    let criteria_ref = parse_range_arg(args[0])?;
    let criteria_cells = range_cells(worksheets, &criteria_ref, current);
    let expected = criterion_text(worksheets, args[1], current);
    let values = if args.len() > 2 {
        range_cells(worksheets, &parse_range_arg(args[2])?, current)
    } else {
        range_cells(worksheets, &criteria_ref, current)
    };
    let mut total = 0.0;
    let mut count = 0.0;
    for (index, (text, _)) in criteria_cells.iter().enumerate() {
        if matches_criteria(text, &expected) {
            total += numbers_at(&values, index);
            count += 1.0;
        }
    }
    if count == 0.0 {
        return None;
    }
    Some(total / count)
}

/// Evaluates a criteria-function call when it names a range, so cross-sheet
/// arguments are read from their own worksheet. Returns `None` for anything
/// this module does not handle, leaving the legacy dispatcher in charge.
pub fn evaluate(
    name: &str,
    raw: &str,
    worksheets: &[Worksheet],
    current: usize,
) -> Option<CellValue> {
    let upper = name.trim().to_ascii_uppercase();
    let args = split_args(raw);
    let args: Vec<&str> = args.iter().map(|arg| arg.trim()).collect();
    match upper.as_str() {
        "SUMIFS" | "SOMASES" => sumifs(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        "SUMIF" | "SOMASE" => sumif(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        "COUNTIFS" | "CONT.SES" => countifs(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        "COUNTIF" | "CONT.SE" => countifs(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        "AVERAGEIFS" | "MÉDIASES" | "MEDIASES" => averageifs(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        "AVERAGEIF" | "MÉDIASE" | "MEDIASE" => averageif(&args, worksheets, current)
            .map(|value| CellValue::Number(value)),
        _ => None,
    }
}

/// Human-readable form used in error paths and tests.
pub fn describe(value: f64) -> String {
    format_number(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::value::CellValue;
    use crate::types::CellData;
    use std::collections::HashMap;

    fn text(value: &str) -> CellData {
        CellData {
            value: Some(value.to_string()),
            typed: Some(CellValue::Text(value.to_string())),
            ..CellData::default()
        }
    }

    fn number(value: f64) -> CellData {
        CellData {
            value: Some(value.to_string()),
            typed: Some(CellValue::Number(value)),
            ..CellData::default()
        }
    }

    fn data_sheet() -> Worksheet {
        let mut data = HashMap::new();
        // Row 1 headers, rows 2-4 amounts, months and types.
        data.insert("1,1".to_string(), text("2025-01"));
        data.insert("1,5".to_string(), text("Revenue"));
        data.insert("1,6".to_string(), number(100.0));
        data.insert("2,1".to_string(), text("2025-01"));
        data.insert("2,5".to_string(), text("COGS"));
        data.insert("2,6".to_string(), number(40.0));
        data.insert("3,1".to_string(), text("2025-02"));
        data.insert("3,5".to_string(), text("Revenue"));
        data.insert("3,6".to_string(), number(250.0));
        Worksheet {
            name: "Data".to_string(),
            data,
            ..Worksheet::default()
        }
    }

    fn monthly_sheet() -> Worksheet {
        let mut data = HashMap::new();
        data.insert("0,0".to_string(), text("2025-01"));
        data.insert("1,0".to_string(), text("2025-02"));
        Worksheet {
            name: "Monthly".to_string(),
            data,
            ..Worksheet::default()
        }
    }

    #[test]
    fn sumifs_reads_cross_sheet_ranges() {
        let sheets = vec![monthly_sheet(), data_sheet()];
        let value = evaluate(
            "SUMIFS",
            "Data!$G$2:$G$4,Data!$B$2:$B$4,$A1,Data!$F$2:$F$4,\"Revenue\"",
            &sheets,
            0,
        );
        assert_eq!(value, Some(CellValue::Number(100.0)), "January revenue");
    }

    #[test]
    fn sumifs_second_month() {
        let sheets = vec![monthly_sheet(), data_sheet()];
        let value = evaluate(
            "SUMIFS",
            "Data!G2:G4,Data!B2:B4,$A2,Data!F2:F4,\"Revenue\"",
            &sheets,
            0,
        );
        assert_eq!(value, Some(CellValue::Number(250.0)), "February revenue");
    }

    #[test]
    fn countifs_counts_matches() {
        let sheets = vec![monthly_sheet(), data_sheet()];
        let value = evaluate("COUNTIFS", "Data!B2:B4,\"2025-01\"", &sheets, 0);
        assert_eq!(value, Some(CellValue::Number(2.0)));
    }

    #[test]
    fn sumif_without_sum_range_uses_criteria_range() {
        let sheets = vec![monthly_sheet(), data_sheet()];
        let value = evaluate("SUMIF", "Data!F2:F4,\"Revenue\",Data!G2:G4", &sheets, 0);
        assert_eq!(value, Some(CellValue::Number(350.0)));
    }
}
