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

pub fn export_to_html(sheet: &Spreadsheet) -> String {
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>"#);
    html.push_str(&sheet.name);
    html.push_str(r#"</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4285f4; color: white; }
        tr:nth-child(even) { background-color: #f9f9f9; }
        tr:hover { background-color: #f1f1f1; }
        .sheet-tabs { margin-bottom: 20px; }
        .sheet-tab { padding: 10px 20px; background: #e0e0e0; border: none; cursor: pointer; }
        .sheet-tab.active { background: #4285f4; color: white; }
    </style>
</head>
<body>
"#);

    for ws in &sheet.worksheets {
        html.push_str(&format!("<h2>{}</h2>\n", ws.name));
        html.push_str("<table>\n");

        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in ws.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        html.push_str("<thead><tr><th></th>");
        for col in 0..=max_col {
            let col_letter = column_to_letter(col);
            html.push_str(&format!("<th>{col_letter}</th>"));
        }
        html.push_str("</tr></thead>\n<tbody>\n");

        for row in 0..=max_row {
            html.push_str(&format!("<tr><td><strong>{}</strong></td>", row + 1));
            for col in 0..=max_col {
                let key = format!("{row},{col}");
                let cell = ws.data.get(&key);
                let value = cell.and_then(|c| c.value.clone()).unwrap_or_default();
                let style = cell.and_then(|c| c.style.as_ref());

                let mut style_str = String::new();
                if let Some(s) = style {
                    if let Some(ref bg) = s.background {
                        style_str.push_str(&format!("background-color:{bg};"));
                    }
                    if let Some(ref color) = s.color {
                        style_str.push_str(&format!("color:{color};"));
                    }
                    if let Some(ref weight) = s.font_weight {
                        style_str.push_str(&format!("font-weight:{weight};"));
                    }
                    if let Some(ref align) = s.text_align {
                        style_str.push_str(&format!("text-align:{align};"));
                    }
                }

                let escaped_value = html_escape(&value);
                if style_str.is_empty() {
                    html.push_str(&format!("<td>{escaped_value}</td>"));
                } else {
                    html.push_str(&format!("<td style=\"{style_str}\">{escaped_value}</td>"));
                }
            }
            html.push_str("</tr>\n");
        }
        html.push_str("</tbody></table>\n");
    }

    html.push_str("</body></html>");
    html
}

fn column_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col + 1;
    while n > 0 {
        n -= 1;
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    result
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn export_to_ods(sheet: &Spreadsheet) -> Result<String, String> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
    xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
    office:version="1.2">
<office:body>
<office:spreadsheet>
"#);

    for ws in &sheet.worksheets {
        xml.push_str(&format!("<table:table table:name=\"{}\">\n", ws.name));

        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in ws.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        for _ in 0..=max_col {
            xml.push_str("<table:table-column/>\n");
        }

        for row in 0..=max_row {
            xml.push_str("<table:table-row>\n");
            for col in 0..=max_col {
                let key = format!("{row},{col}");
                let value = ws.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();
                let formula = ws.data.get(&key).and_then(|c| c.formula.clone());

                if let Some(f) = formula {
                    xml.push_str(&format!(
                        "<table:table-cell table:formula=\"{}\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        f, value
                    ));
                } else if let Ok(num) = value.parse::<f64>() {
                    xml.push_str(&format!(
                        "<table:table-cell office:value-type=\"float\" office:value=\"{}\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        num, value
                    ));
                } else {
                    xml.push_str(&format!(
                        "<table:table-cell office:value-type=\"string\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        value
                    ));
                }
            }
            xml.push_str("</table:table-row>\n");
        }
        xml.push_str("</table:table>\n");
    }

    xml.push_str("</office:spreadsheet>\n</office:body>\n</office:document-content>");
    Ok(xml)
}

pub fn export_to_pdf_data(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    let html = export_to_html(sheet);
    Ok(html.into_bytes())
}

pub fn export_to_markdown(sheet: &Spreadsheet) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", sheet.name));

    for ws in &sheet.worksheets {
        md.push_str(&format!("## {}\n\n", ws.name));

        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in ws.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        if max_col == 0 && max_row == 0 && ws.data.is_empty() {
            md.push_str("*Empty worksheet*\n\n");
            continue;
        }

        md.push('|');
        for col in 0..=max_col {
            let col_letter = column_to_letter(col);
            md.push_str(&format!(" {col_letter} |"));
        }
        md.push('\n');

        md.push('|');
        for _ in 0..=max_col {
            md.push_str(" --- |");
        }
        md.push('\n');

        for row in 0..=max_row {
            md.push('|');
            for col in 0..=max_col {
                let key = format!("{row},{col}");
                let value = ws.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();
                let escaped = value.replace('|', "\\|");
                md.push_str(&format!(" {escaped} |"));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    md
}
