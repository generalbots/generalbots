//! Standalone HTML table export.

use botsheet_core::types::Spreadsheet;

use super::util::column_to_letter;

pub fn export_to_html(sheet: &Spreadsheet) -> String {
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>"#);
    html.push_str(&html_escape(&sheet.name));
    html.push_str(r#"</title>
<style>
body { font-family: Arial, sans-serif; margin: 20px; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
th { background-color: #4285f4; color: white; }
tr:nth-child(even) { background-color: #f9f9f9; }
tr:hover { background-color: #f1f1f1; }
</style>
</head>
<body>
"#);

    for ws in &sheet.worksheets {
        html.push_str(&format!("<h2>{}</h2>\n", html_escape(&ws.name)));
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
