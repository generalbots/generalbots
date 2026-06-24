pub mod drive_ops;
pub mod import;
pub mod xlsx_read;
pub mod xlsx_write;

pub use drive_ops::*;
pub use import::*;
pub use xlsx_read::load_xlsx_from_bytes;
pub use xlsx_write::{apply_umya_style, convert_to_xlsx, extract_cell_style, get_col_letter};

use crate::state::SheetState;
use crate::types::Spreadsheet;

pub async fn save_sheet_as_xlsx(
    state: &SheetState,
    user_id: &str,
    sheet: &Spreadsheet,
) -> Result<Vec<u8>, String> {
    let xlsx_bytes = convert_to_xlsx(sheet)?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.xlsx",
        crate::state::get_user_sheets_path(user_id),
        sheet.id
    );

    drive
        .put_object(
            "gbo",
            &path,
            xlsx_bytes.clone(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .await?;

    Ok(xlsx_bytes)
}

pub async fn load_xlsx_from_drive(
    state: &SheetState,
    user_id: &str,
    file_path: &str,
) -> Result<(Spreadsheet, umya_spreadsheet::Spreadsheet), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let bytes = drive.get_object("gbo", file_path).await?;

    load_xlsx_from_bytes(&bytes, user_id, file_path)
}

pub async fn update_xlsx_cell(
    workbook: &mut umya_spreadsheet::Spreadsheet,
    sheet_name: &str,
    row: u32,
    col: u32,
    value: Option<&str>,
    formula: Option<&str>,
    style: Option<&crate::types::CellStyle>,
) -> Result<(), String> {
    let sheet = workbook
        .get_sheet_by_name_mut(sheet_name)
        .ok_or_else(|| format!("Sheet '{sheet_name}' not found"))?;

    let cell = sheet.get_cell_mut((col + 1, row + 1));

    if let Some(f) = formula {
        let formula_str = if f.starts_with('=') { &f[1..] } else { f };
        cell.set_formula(formula_str);
    } else if let Some(v) = value {
        if let Ok(num) = v.parse::<f64>() {
            cell.set_value_number(num);
        } else if v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("false") {
            cell.set_value_bool(v.eq_ignore_ascii_case("true"));
        } else {
            cell.set_value_string(v);
        }
    } else {
        cell.set_value_string("");
    }

    if let Some(s) = style {
        apply_umya_style(cell, s);
    }

    Ok(())
}

pub async fn save_workbook_to_drive(
    state: &SheetState,
    user_id: &str,
    sheet_id: &str,
    workbook: &umya_spreadsheet::Spreadsheet,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.xlsx",
        crate::state::get_user_sheets_path(user_id),
        sheet_id
    );

    let mut buf = std::io::Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(workbook, &mut buf)
        .map_err(|e| format!("Failed to write xlsx: {e}"))?;

    drive
        .put_object(
            "gbo",
            &path,
            buf.into_inner(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .await?;

    Ok(())
}
