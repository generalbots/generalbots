#[cfg(feature = "xlsx")]
pub mod chart_read;
pub mod drive_ops;
#[cfg(feature = "xlsx")]
pub mod format_codes;
pub mod import;
pub mod import_binary;
pub mod import_ods;
pub mod xlsx_comments;
pub mod xlsx_external_links;
pub mod xlsx_layout;
pub mod xlsx_layout_patch;
pub mod xlsx_passthrough;
#[cfg(test)]
mod xlsx_passthrough_tests;
pub mod xlsx_read;
pub mod xlsx_rename;
pub mod xlsx_rich_text;
pub mod xlsx_rules;
pub mod xlsx_shared_strings;
pub mod xlsx_source;
pub mod xlsx_workbook;
pub mod xlsx_workbook_patch;
pub mod xlsx_write;
pub mod xlsx_write_charts;

pub use import::*;
pub use xlsx_read::load_xlsx_from_bytes;
pub use xlsx_source::{
    plan_for, produce_xlsx, retains_snapshot, save_back_message, save_back_target, RegenerateReason,
    SaveBackTarget, XlsxPlan,
};
pub use xlsx_write::{
    apply_umya_style, convert_to_xlsx, extract_cell_style, get_col_letter, merge_into_original,
};

use botsheet_core::state::{DriveOps, SheetSaveHook, SheetState};
use botsheet_core::types::Spreadsheet;
use std::sync::Arc;

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
) -> Result<(Spreadsheet, umya_spreadsheet::Workbook), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let bytes = drive.get_object("gbo", file_path).await?;

    load_xlsx_from_bytes(&bytes, user_id, file_path)
}

pub async fn update_xlsx_cell(
    workbook: &mut umya_spreadsheet::Workbook,
    sheet_name: &str,
    row: u32,
    col: u32,
    value: Option<&str>,
    formula: Option<&str>,
    style: Option<&crate::types::CellStyle>,
) -> Result<(), String> {
    let sheet = workbook
        .sheet_by_name_mut(sheet_name)
        .map_err(|_| format!("Sheet '{sheet_name}' not found"))?;

    let cell = sheet.cell_mut((col + 1, row + 1));

    if let Some(f) = formula {
        let formula_str = if let Some(stripped) = f.strip_prefix('=') { stripped } else { f };
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
    workbook: &umya_spreadsheet::Workbook,
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

/// Sibling destination for a workbook that cannot be patched in place:
/// `report.xlsx` -> `report.gbsheet.xlsx` (#2043).
pub fn sibling_xlsx_path(source_path: &str) -> String {
    let base = source_path
        .strip_suffix(".xlsx")
        .or_else(|| source_path.strip_suffix(".xlsm"))
        .or_else(|| source_path.strip_suffix(".xls"))
        .or_else(|| source_path.strip_suffix(".ods"))
        .unwrap_or(source_path);
    format!("{base}.gbsheet.xlsx")
}

/// Creates a post-save hook that writes the edited workbook back to Drive.
///
/// #2043 — the destination follows the source snapshot:
/// * an OOXML package that can be patched is written back to the **opened
///   file** (`source_path`); the zip-level passthrough copies every part it
///   does not model, so charts, comments, conditional formats and images
///   survive;
/// * anything else (`.ods`/`.xls` snapshot, or no snapshot at all) goes to the
///   `<name>.gbsheet.xlsx` sibling so a rebuilt workbook never replaces the
///   original — [`save_back_message`] names that file for the user.
///
/// A failed patch returns the error instead of silently regenerating: the
/// original stays untouched and the caller reports it (#2043).
/// Runs asynchronously via tokio::spawn (fire-and-forget) to avoid
/// blocking the tokio runtime from within a sync closure.
pub fn create_save_back_hook(drive: Arc<dyn DriveOps>) -> SheetSaveHook {
    Arc::new(move |sheet: &Spreadsheet| {
        let Some(target) = save_back_target(sheet) else {
            return Ok(());
        };

        // Bytes are produced synchronously so a patch failure surfaces here
        // rather than turning into a silent, lossy rebuild.
        let produced = produce_xlsx(sheet)?;
        if let Some(reason) = produced.plan.regenerate_reason() {
            log::warn!(
                "sheet save-back regenerated {} from the model: {}",
                target.path,
                reason.description()
            );
        }

        let d = drive.clone();
        let bucket = target.bucket.clone();
        let dest_path = target.path.clone();
        let xlsx_bytes = produced.bytes;
        tokio::spawn(async move {
            if let Err(e) = d
                .put_object(
                    &bucket,
                    &dest_path,
                    xlsx_bytes,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                )
                .await
            {
                log::error!("Failed to save xlsx back to Drive ({dest_path}): {e}");
            }
        });

        Ok(())
    })
}
