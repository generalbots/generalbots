//! XLSX export (rust_xlsxwriter).

use botsheet_core::types::{CellStyle, Spreadsheet};

use super::util::cell_number;

pub fn export_to_xlsx(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    let mut workbook = rust_xlsxwriter::Workbook::new();

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

            let mut format = rust_xlsxwriter::Format::new();

            if let Some(ref style) = cell.style {
                format = apply_style_to_format(format, style);
            }

            if let Some(ref formula) = cell.formula {
                worksheet
                    .write_formula_with_format(row, col, formula.as_str(), &format)
                    .map_err(|e| e.to_string())?;
            } else if let Some(num) = cell_number(cell) {
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

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

fn apply_style_to_format(
    mut format: rust_xlsxwriter::Format,
    style: &CellStyle,
) -> rust_xlsxwriter::Format {
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
            "center" => format.set_align(rust_xlsxwriter::FormatAlign::Center),
            "right" => format.set_align(rust_xlsxwriter::FormatAlign::Right),
            _ => format.set_align(rust_xlsxwriter::FormatAlign::Left),
        };
    }
    if let Some(size) = style.font_size {
        format = format.set_font_size(size as f64);
    }
    format
}

fn parse_color(color_str: &str) -> Option<rust_xlsxwriter::Color> {
    let hex = color_str.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(rust_xlsxwriter::Color::RGB(
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        ))
    } else {
        None
    }
}
