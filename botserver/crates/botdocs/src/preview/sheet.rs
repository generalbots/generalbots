//! Spreadsheet previews: one bounded HTML table per worksheet (calamine).

use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Ods, Reader, Xls, Xlsb, Xlsx};

const MAX_ROWS: usize = 200;
const MAX_COLUMNS: usize = 40;

/// One bounded HTML row set per worksheet. A macro keeps the three concrete
/// calamine readers without a generic bound the compiler cannot infer.
macro_rules! read_sheets {
    ($workbook:expr) => {{
        let mut sheets: Vec<(String, Vec<Vec<String>>, usize, usize)> = Vec::new();
        for name in $workbook.sheet_names().to_owned() {
            let Ok(range) = $workbook.worksheet_range(&name) else {
                continue;
            };
            let total_rows = range.height();
            let total_cols = range.width();
            let mut rows: Vec<Vec<String>> = Vec::new();
            for row in range.rows().take(MAX_ROWS) {
                rows.push(row.iter().take(MAX_COLUMNS).map(cell_text).collect());
            }
            sheets.push((name, rows, total_rows, total_cols));
        }
        sheets
    }};
}

pub(super) fn spreadsheet_html(extension: &str, bytes: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(bytes);
    let sheets = match extension {
        "xls" => {
            let mut workbook: Xls<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLS: {error}"))?;
            read_sheets!(workbook)
        }
        "xlsb" => {
            let mut workbook: Xlsb<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLSB: {error}"))?;
            read_sheets!(workbook)
        }
        "ods" => {
            let mut workbook: Ods<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse ODS: {error}"))?;
            read_sheets!(workbook)
        }
        _ => {
            let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLSX: {error}"))?;
            read_sheets!(workbook)
        }
    };

    if sheets.is_empty() {
        return Ok("<p>(empty workbook)</p>".to_string());
    }

    let mut html = String::new();
    for (name, rows, total_rows, total_cols) in sheets {
        html.push_str(&format!(
            r#"<div class="gb-sheet-name">{}</div><table>"#,
            crate::ui_fragments::html_escape(&name)
        ));
        for row in &rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str("<td>");
                html.push_str(&crate::ui_fragments::html_escape(cell));
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
        html.push_str("</table>");
        if total_rows > MAX_ROWS || total_cols > MAX_COLUMNS {
            html.push_str(&format!(
                r#"<p class="gb-preview-note">Showing first {} of {} rows and {} of {} columns.</p>"#,
                rows.len().min(MAX_ROWS),
                total_rows,
                rows.first().map(Vec::len).unwrap_or(0).min(MAX_COLUMNS),
                total_cols
            ));
        }
    }
    Ok(html)
}

/// Cells are rendered as display strings; formulas/dates keep the value
/// calamine exposes (ISO date strings where the format provides one).
fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Int(value) => value.to_string(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Bool(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Error(error) => format!("#ERROR {error:?}"),
    }
}
