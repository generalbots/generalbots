//! Markdown table export.

use botsheet_core::types::Spreadsheet;

use super::util::column_to_letter;

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
                let value = ws
                    .data
                    .get(&key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_default();
                let escaped = value.replace('|', "\\|");
                md.push_str(&format!(" {escaped} |"));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    md
}
