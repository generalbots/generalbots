use base64::Engine;
use crate::sheet::types::{CellStyle, Spreadsheet};
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

pub fn export_to_xlsx(sheet: &Spreadsheet) -> Result<String, String> {
    let mut workbook = Workbook::new();

    for ws in &sheet.worksheets {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&ws.name).map_err(|e| e.to_string())?;

        for (key, cell) in &ws.data {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 2 {
                continue;
            }
            let (row, col) = match (parts[0].parse::<u32>(), parts[1].parse::<u16>()) {
                (Ok(r), Ok(c)) => (r, c),
                _ => continue,
            };

            let value = cell.value.as_deref().unwrap_or("");

            let mut format = Format::new();

            if let Some(ref style) = cell.style {
                format = apply_style_to_format(format, style);
            }

            if let Some(ref formula) = cell.formula {
                worksheet
                    .write_formula_with_format(row, col, formula.as_str(), &format)
                    .map_err(|e| e.to_string())?;
            } else if let Ok(num) = value.parse::<f64>() {
                worksheet
                    .write_number_with_format(row, col, num, &format)
                    .map_err(|e| e.to_string())?;
            } else {
                worksheet
                    .write_string_with_format(row, col, value, &format)
                    .map_err(|e| e.to_string())?;
            }
        }

        if let Some(ref widths) = ws.column_widths {
            for (col, width) in widths {
                worksheet
                    .set_column_width(*col as u16, *width)
                    .map_err(|e| e.to_string())?;
            }
        }

        if let Some(ref heights) = ws.row_heights {
            for (row, height) in heights {
                worksheet
                    .set_row_height(*row, *height)
                    .map_err(|e| e.to_string())?;
            }
        }

        if let Some(frozen_rows) = ws.frozen_rows {
            if let Some(frozen_cols) = ws.frozen_cols {
                worksheet
                    .set_freeze_panes(frozen_rows, frozen_cols as u16)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let buffer = workbook.save_to_buffer().map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buffer))
}

fn apply_style_to_format(mut format: Format, style: &CellStyle) -> Format {
    if let Some(ref bg) = style.background {
        if let Some(color) = parse_color(bg) {
            format = format.set_background_color(color);
        }
    }
    if let Some(ref fg) = style.color {
        if let Some(color) = parse_color(fg) {
            format = format.set_font_color(color);
        }
    }
    if let Some(ref weight) = style.font_weight {
        if weight == "bold" {
            format = format.set_bold();
        }
    }
    if let Some(ref style_val) = style.font_style {
        if style_val == "italic" {
            format = format.set_italic();
        }
    }
    if let Some(ref align) = style.text_align {
        format = match align.as_str() {
            "center" => format.set_align(FormatAlign::Center),
            "right" => format.set_align(FormatAlign::Right),
            _ => format.set_align(FormatAlign::Left),
        };
    }
    if let Some(ref size) = style.font_size {
        format = format.set_font_size(*size as f64);
    }
    format
}

fn parse_color(color_str: &str) -> Option<Color> {
    let hex = color_str.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::RGB(
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        ))
    } else {
        None
    }
}

pub fn export_to_csv(sheet: &Spreadsheet) -> String {
    let mut csv = String::new();
    if let Some(worksheet) = sheet.worksheets.first() {
        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in worksheet.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }
        for row in 0..=max_row {
            let mut row_values = Vec::new();
            for col in 0..=max_col {
                let key = format!("{},{}", row, col);
                let value = worksheet
                    .data
                    .get(&key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_default();
                let escaped = if value.contains(',') || value.contains('"') || value.contains('\n')
                {
                    format!("\"{}\"", value.replace('"', "\"\""))
                } else {
                    value
                };
                row_values.push(escaped);
            }
            csv.push_str(&row_values.join(","));
            csv.push('\n');
        }
    }
    csv
}

pub fn export_to_json(sheet: &Spreadsheet) -> String {
    serde_json::to_string_pretty(sheet).unwrap_or_default()
}
