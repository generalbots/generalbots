//! CSV export.

use botsheet_core::types::Spreadsheet;

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
                let escaped =
                    if value.contains(',') || value.contains('"') || value.contains('\n') {
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
