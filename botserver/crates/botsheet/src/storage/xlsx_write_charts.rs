//! Chart emission for the regenerated XLSX package.
//!
//! Kept apart from `xlsx_write.rs` (which owns cells and styles) so both files
//! stay small. A regenerated package used to carry the numbers only, so a sheet
//! that had a chart opened in Excel without it (#2150 SHEET-01 step 6).

use crate::types::{ChartConfig, ChartPosition, Worksheet};

/// Adds one model chart to the regenerated sheet.
///
/// A regenerated package used to carry the numbers only, so a sheet that had a
/// chart opened in Excel without it (#2150 SHEET-01 step 6). umya writes the
/// chart XML and its drawing anchor, which is what makes the chart survive.
pub(super) fn attach_chart(worksheet: &Worksheet, umya_sheet: &mut umya_spreadsheet::Worksheet, config: &ChartConfig) {
    let Some(series) = absolute_range(&config.data_range) else {
        log::warn!(
            "chart {} has an invalid data range '{}'; it is skipped on export",
            config.id,
            config.data_range
        );
        return;
    };
    let range = format!("{}!{series}", quoted_sheet_name(&worksheet.name));
    let (start, end) = chart_anchor(&config.position);

    let mut from_marker = umya_spreadsheet::structs::drawing::spreadsheet::MarkerType::default();
    from_marker.set_coordinate(&start);
    let mut to_marker = umya_spreadsheet::structs::drawing::spreadsheet::MarkerType::default();
    to_marker.set_coordinate(&end);

    let mut chart = umya_spreadsheet::structs::Chart::default();
    chart.new_chart(
        &chart_type(&config.chart_type),
        from_marker,
        to_marker,
        vec![range.as_str()],
    );
    if !config.title.trim().is_empty() {
        chart.set_title(config.title.trim());
    }
    if let Some(series_title) = series_title(worksheet, &config.data_range) {
        chart.set_series_title(vec![series_title.as_str()]);
    }
    umya_sheet.add_chart(chart);
}

/// Maps the model's chart name onto umya's chart families.
fn chart_type(name: &str) -> umya_spreadsheet::structs::ChartType {
    use umya_spreadsheet::structs::ChartType;
    match name.trim().to_ascii_lowercase().as_str() {
        "line" | "line_chart" => ChartType::LineChart,
        "pie" | "pie_chart" | "donut" | "doughnut" | "doughnut_chart" => ChartType::DoughnutChart,
        "area" | "area_chart" => ChartType::AreaChart,
        "scatter" | "scatter_chart" => ChartType::ScatterChart,
        "radar" | "radar_chart" => ChartType::RadarChart,
        _ => ChartType::BarChart,
    }
}

/// Converts `B2:B11` into `$B$2:$B$11`, the form a chart series reference needs.
fn absolute_range(range: &str) -> Option<String> {
    let ((start_row, start_col), (end_row, end_col)) =
        botsheet_core::formulas::refs::parse_range_or_single(range)?;
    let start = format!("${}${}", column_name(start_col), start_row + 1);
    let end = format!("${}${}", column_name(end_col), end_row + 1);
    Some(format!("{start}:{end}"))
}

/// Quotes a worksheet name the way a formula reference requires.
fn quoted_sheet_name(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return name.to_string();
    }
    format!("'{}'", name.replace('\'', "''"))
}

fn column_name(col: u32) -> String {
    botsheet_core::formulas::refs::col_index_to_name(col).unwrap_or_else(|| "A".to_string())
}

/// Two-cell anchor for a stored chart position (0-based row/col in pixels).
fn chart_anchor(position: &ChartPosition) -> (String, String) {
    let apps_column = position.width / 64;
    let apps_row = position.height / 20;
    let start = format!("{}{}", column_name(position.col), position.row + 1);
    let end = format!(
        "{}{}",
        column_name(position.col + apps_column.max(4)),
        position.row + apps_row.max(8) + 1
    );
    (start, end)
}

/// Series name comes from the cell just above the first data cell (`GDP`).
fn series_title(worksheet: &Worksheet, data_range: &str) -> Option<String> {
    let ((start_row, start_col), _) =
        botsheet_core::formulas::refs::parse_range_or_single(data_range)?;
    if start_row == 0 {
        return None;
    }
    let key = format!("{},{}", start_row - 1, start_col);
    let title = worksheet.data.get(&key)?.value.clone()?;
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

