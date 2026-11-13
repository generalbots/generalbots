//! Chart series resolution for workbooks loaded from storage.
//!
//! The chart objects persisted next to a workbook keep the values that were
//! resolved when the package was imported. A package whose formulas had no
//! cached result therefore stored charts with empty `data`/`labels`, and the
//! grid drew 300x180 empty rooms (#2152). After the workbook is recalculated
//! the series are resolved again from the stored range references.

use crate::types::{ChartConfig, ChartDataset, Worksheet};

/// Guards against a pathological `data_range` string.
const MAX_RANGE_TOKENS: usize = 64;
const DEFAULT_SERIES_COLOR: &str = "#3b82f6";

/// Splits a stored range spec into individual range tokens.
///
/// A multi-series chart persists its references concatenated
/// (`'Monthly'!$B$5:$B$28'Monthly'!$E$5:$E$28`), so the sheet boundary is the
/// quote that opens the next reference.
pub fn split_range_tokens(spec: &str) -> Vec<String> {
    let chars: Vec<char> = spec.trim().chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < chars.len() && tokens.len() < MAX_RANGE_TOKENS {
        let start = index;
        if chars[index] == '\'' {
            index += 1;
            while index < chars.len() && chars[index] != '\'' {
                index += 1;
            }
            if index < chars.len() {
                index += 1; // closing quote
            }
            if index < chars.len() && chars[index] == '!' {
                index += 1;
            }
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '$'
                    || chars[index] == ':')
            {
                index += 1;
            }
        } else {
            while index < chars.len() && chars[index] != '\'' {
                index += 1;
            }
        }
        let token: String = chars[start..index].iter().collect();
        let trimmed = token.trim();
        if trimmed.is_empty() {
            index += 1;
        } else {
            tokens.push(trimmed.to_string());
        }
    }
    tokens
}

/// `'Monthly'!$B$5:$B$28` → (`Monthly`, 4, 1, 27, 1); indices are 0-based.
pub fn parse_range_token(token: &str) -> Option<(String, u32, u32, u32, u32)> {
    let (sheet, rest) = match token.rsplit_once('!') {
        Some((sheet, rest)) => (sheet.trim().trim_matches('\'').to_string(), rest),
        None => (String::new(), token),
    };
    let cleaned = rest.replace('$', "");
    let (first, last) = match cleaned.split_once(':') {
        Some((first, last)) => (first.to_string(), last.to_string()),
        None => (cleaned.clone(), cleaned),
    };
    let (c0, r0) = parse_cell(&first)?;
    let (c1, r1) = parse_cell(&last)?;
    Some((sheet, r0, c0, r1, c1))
}

fn parse_cell(text: &str) -> Option<(u32, u32)> {
    let letters: String = text
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if letters.is_empty() || digits.is_empty() {
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
    Some((column - 1, row - 1))
}

fn sheet_index(worksheets: &[Worksheet], sheet: &str, fallback: usize) -> Option<usize> {
    if sheet.is_empty() {
        return Some(fallback);
    }
    worksheets
        .iter()
        .position(|ws| ws.name.eq_ignore_ascii_case(sheet))
        .or(Some(fallback))
}

fn resolve_numbers(
    worksheets: &[Worksheet],
    sheet: &str,
    (r0, c0, r1, c1): (u32, u32, u32, u32),
    fallback: usize,
) -> Vec<f64> {
    let Some(index) = sheet_index(worksheets, sheet, fallback) else {
        return Vec::new();
    };
    let Some(worksheet) = worksheets.get(index) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for row in r0..=r1 {
        for col in c0..=c1 {
            let key = format!("{row},{col}");
            let Some(cell) = worksheet.data.get(&key) else {
                continue;
            };
            let number = cell
                .typed
                .as_ref()
                .and_then(|typed| typed.as_number())
                .or_else(|| {
                    cell.value
                        .as_deref()
                        .and_then(|value| value.trim().parse::<f64>().ok())
                });
            if let Some(number) = number {
                values.push(number);
            }
        }
    }
    values
}

fn resolve_labels(
    worksheets: &[Worksheet],
    sheet: &str,
    (r0, c0, r1, c1): (u32, u32, u32, u32),
    fallback: usize,
) -> Vec<String> {
    let Some(index) = sheet_index(worksheets, sheet, fallback) else {
        return Vec::new();
    };
    let Some(worksheet) = worksheets.get(index) else {
        return Vec::new();
    };
    let mut labels = Vec::new();
    for row in r0..=r1 {
        for col in c0..=c1 {
            let key = format!("{row},{col}");
            if let Some(cell) = worksheet.data.get(&key) {
                labels.push(cell.value.clone().unwrap_or_default());
            }
        }
    }
    labels
}

/// Series resolved in a first pass, while the workbook is still only read.
struct ResolvedChart {
    series: Vec<Vec<f64>>,
    labels: Vec<String>,
}

fn resolve_chart(worksheets: &[Worksheet], chart: &ChartConfig, fallback: usize) -> ResolvedChart {
    let series: Vec<Vec<f64>> = split_range_tokens(&chart.data_range)
        .iter()
        .filter_map(|token| parse_range_token(token))
        .map(|(sheet, r0, c0, r1, c1)| resolve_numbers(worksheets, &sheet, (r0, c0, r1, c1), fallback))
        .collect();
    let labels = split_range_tokens(&chart.label_range)
        .first()
        .and_then(|token| parse_range_token(token))
        .map(|(sheet, r0, c0, r1, c1)| resolve_labels(worksheets, &sheet, (r0, c0, r1, c1), fallback))
        .unwrap_or_default();
    ResolvedChart { series, labels }
}

/// Refills every chart whose series are empty. Returns the number of charts
/// updated. Charts already carrying data are left untouched so a user's own
/// series (added in the UI) is never replaced by the stored range.
pub fn refresh_chart_data(worksheets: &mut [Worksheet]) -> usize {
    let plan: Vec<Vec<Option<ResolvedChart>>> = worksheets
        .iter()
        .enumerate()
        .map(|(index, worksheet)| match worksheet.charts.as_ref() {
            Some(charts) => charts
                .iter()
                .map(|chart| {
                    let has_data = chart.datasets.iter().any(|set| !set.data.is_empty());
                    if has_data {
                        return None;
                    }
                    let resolved = resolve_chart(worksheets, chart, index);
                    if resolved.series.iter().all(|set| set.is_empty()) {
                        None
                    } else {
                        Some(resolved)
                    }
                })
                .collect(),
            None => Vec::new(),
        })
        .collect();

    let mut updated = 0;
    for (index, charts) in worksheets.iter_mut().enumerate() {
        let Some(charts) = charts.charts.as_mut() else {
            continue;
        };
        for (chart_index, chart) in charts.iter_mut().enumerate() {
            let Some(resolved) = plan
                .get(index)
                .and_then(|plans| plans.get(chart_index))
                .and_then(|plan| plan.as_ref())
            else {
                continue;
            };
            if chart.datasets.is_empty() {
                chart.datasets = resolved
                    .series
                    .iter()
                    .map(|data| ChartDataset {
                        label: String::new(),
                        data: data.clone(),
                        color: DEFAULT_SERIES_COLOR.to_string(),
                        background_color: None,
                    })
                    .collect();
            } else {
                for (series_index, data) in resolved.series.iter().enumerate() {
                    if let Some(dataset) = chart.datasets.get_mut(series_index) {
                        dataset.data = data.clone();
                    }
                }
            }
            if chart.labels.is_empty() && !resolved.labels.is_empty() {
                chart.labels = resolved.labels.clone();
            }
            updated += 1;
        }
    }
    updated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::value::CellValue;
    use crate::types::{CellData, ChartConfig, ChartDataset, ChartOptions, ChartPosition};
    use std::collections::HashMap;

    fn number_cell(value: f64) -> CellData {
        CellData {
            value: Some(value.to_string()),
            typed: Some(CellValue::Number(value)),
            ..CellData::default()
        }
    }

    fn text_cell(value: &str) -> CellData {
        CellData {
            value: Some(value.to_string()),
            typed: Some(CellValue::Text(value.to_string())),
            ..CellData::default()
        }
    }

    fn worksheet(name: &str, cells: Vec<(&str, CellData)>) -> Worksheet {
        Worksheet {
            name: name.to_string(),
            data: cells
                .into_iter()
                .map(|(key, cell)| (key.to_string(), cell))
                .collect::<HashMap<_, _>>(),
            ..Worksheet::default()
        }
    }

    fn chart(data_range: &str, label_range: &str, series: usize) -> ChartConfig {
        ChartConfig {
            id: "chart-1".to_string(),
            chart_type: "bar".to_string(),
            title: "Revenue".to_string(),
            data_range: data_range.to_string(),
            label_range: label_range.to_string(),
            position: ChartPosition {
                row: 0,
                col: 0,
                width: 300,
                height: 180,
            },
            options: ChartOptions::default(),
            datasets: (0..series)
                .map(|_| ChartDataset {
                    label: String::new(),
                    data: Vec::new(),
                    color: "#3b82f6".to_string(),
                    background_color: None,
                })
                .collect(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn splits_concatenated_multi_series_spec() {
        let tokens = split_range_tokens("'Monthly'!$B$5:$B$28'Monthly'!$E$5:$E$28");
        assert_eq!(tokens.len(), 2, "tokens: {tokens:?}");
        assert_eq!(tokens[0], "'Monthly'!$B$5:$B$28");
        assert_eq!(tokens[1], "'Monthly'!$E$5:$E$28");
    }

    #[test]
    fn parses_quoted_and_plain_tokens() {
        assert_eq!(
            parse_range_token("'Pivots'!$B$5:$B$9"),
            Some(("Pivots".to_string(), 4, 1, 8, 1))
        );
        assert_eq!(
            parse_range_token("Data!G2:G4"),
            Some(("Data".to_string(), 1, 6, 3, 6))
        );
    }

    #[test]
    fn refills_empty_chart_series() {
        let mut dashboard = worksheet("Dashboard", vec![]);
        dashboard.charts = Some(vec![chart("'Pivots'!$B$5:$B$6", "'Pivots'!$A$5:$A$6", 1)]);
        let pivots = worksheet(
            "Pivots",
            vec![
                ("4,0", text_cell("North")),
                ("5,0", text_cell("South")),
                ("4,1", number_cell(10.0)),
                ("5,1", number_cell(20.0)),
            ],
        );
        let mut sheets = vec![dashboard, pivots];
        assert_eq!(refresh_chart_data(&mut sheets), 1);
        let chart = &sheets[0].charts.as_ref().expect("charts")[0];
        assert_eq!(chart.datasets[0].data, vec![10.0, 20.0]);
        assert_eq!(chart.labels, vec!["North", "South"]);
    }

    #[test]
    fn keeps_charts_that_already_have_data() {
        let mut dashboard = worksheet("Dashboard", vec![]);
        let mut existing = chart("'Pivots'!$B$5:$B$6", "'Pivots'!$A$5:$A$6", 1);
        existing.datasets[0].data = vec![3.0];
        dashboard.charts = Some(vec![existing]);
        let mut sheets = vec![dashboard];
        assert_eq!(refresh_chart_data(&mut sheets), 0);
    }
}
