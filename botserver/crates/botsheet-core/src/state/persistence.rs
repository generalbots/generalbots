use super::*;
use crate::types::{
    embed_settings_in_workbook_json, settings_from_workbook_bytes, set_workbook_settings,
    workbook_settings,
};

pub async fn save_sheet_to_drive(
    state: &SheetState,
    user_id: &str,
    sheet: &crate::types::Spreadsheet,
) -> Result<(), String> {
    persist_sheet_to_drive(state, user_id, sheet).await?;

    // Post-save hook (xlsx export back to original bucket/path)
    if let Some(ref hook) = state.on_save {
        hook(sheet)?;
    }

    Ok(())
}

/// Persists a sheet's JSON + listing sidecar WITHOUT firing the on-save xlsx
/// export hook. Used by the load-from-drive path: merely opening a file must
/// not rewrite the source .xlsx (which would drop charts via the umya
/// round-trip), even though the load persists a working copy of the JSON.
pub async fn persist_sheet_to_drive(
    state: &SheetState,
    user_id: &str,
    sheet: &crate::types::Spreadsheet,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet.id);
    let content =
        serde_json::to_string_pretty(sheet).map_err(|e| format!("Serialization error: {e}"))?;
    let content = embed_settings_in_workbook_json(&content, &workbook_settings(&sheet.id))?;

    drive
        .put_object("gbo", &path, content.into_bytes(), "application/json")
        .await?;

    // Keep the lightweight listing sidecar in sync (#789, gap 24): listing a
    // library no longer deserialises every workbook, only this small record.
    let meta_path = format!("{}/{}.meta.json", get_user_sheets_path(user_id), sheet.id);
    let meta = SpreadsheetMetadata {
        id: sheet.id.clone(),
        name: sheet.name.clone(),
        owner_id: sheet.owner_id.clone(),
        created_at: sheet.created_at,
        updated_at: sheet.updated_at,
        worksheet_count: sheet.worksheets.len(),
        acl: sheet.acl.clone(),
    };
    let meta_json =
        serde_json::to_string(&meta).map_err(|e| format!("Serialization error: {e}"))?;
    drive
        .put_object(
            "gbo",
            &meta_path,
            meta_json.into_bytes(),
            "application/json",
        )
        .await?;

    Ok(())
}

pub async fn load_sheet_from_drive(
    state: &SheetState,
    user_id: &str,
    sheet_id: &Option<String>,
) -> Result<crate::types::Spreadsheet, String> {
    let sheet_id = sheet_id
        .as_ref()
        .ok_or_else(|| "Sheet ID is required".to_string())?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let bytes = drive.get_object("gbo", &path).await?;

    let sheet: crate::types::Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;
    set_workbook_settings(&sheet.id, settings_from_workbook_bytes(&bytes));

    Ok(sheet)
}

pub async fn load_sheet_by_id(
    state: &SheetState,
    user_id: &str,
    sheet_id: &str,
) -> Result<crate::types::Spreadsheet, String> {
    // A missing id would otherwise become a degenerate S3 GET of `sheets/.json`
    // whose raw storage error leaked to the client — validate it first.
    if sheet_id.trim().is_empty() {
        return Err("Sheet ID is required".to_string());
    }
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let bytes = drive.get_object("gbo", &path).await.map_err(|error| {
        if error.contains("404") {
            "Sheet not found".to_string()
        } else {
            log::error!("sheet load of {path} failed: {error}");
            "Failed to load sheet".to_string()
        }
    })?;

    let mut sheet: crate::types::Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;
    set_workbook_settings(&sheet.id, settings_from_workbook_bytes(&bytes));

    // Workbooks saved before import-time recalculation (and packages generated
    // with formulas but no cached results) still hold empty formula cells.
    // Fill them on load so the grid, exports and the AI never see a blank
    // model (#2152 follow-up).
    let recalculated = crate::recalc::recalculate_if_needed(&mut sheet.worksheets);
    if recalculated > 0 {
        log::info!("sheet {sheet_id} recalculated {recalculated} formula cells on load");
    }
    // Charts persisted next to an uncached workbook hold empty series.
    let refreshed = crate::chart_data::refresh_chart_data(&mut sheet.worksheets);
    if refreshed > 0 {
        log::info!("sheet {sheet_id} refreshed {refreshed} chart series on load");
    }

    if !can_read_sheet(user_id, &sheet) {
        return Err("Access denied: sheet is not shared with this user".to_string());
    }

    Ok(sheet)
}

pub async fn list_sheets_from_drive(
    state: &SheetState,
    user_id: &str,
) -> Result<Vec<SpreadsheetMetadata>, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let prefix = format!("{}/", get_user_sheets_path(user_id));

    let keys = drive.list_objects("gbo", &prefix).await?;

    let mut sheets = Vec::new();

    // Prefer the lightweight `.meta.json` sidecar so listing a large library
    // does not deserialise every workbook (#789, gap 24); fall back to a full
    // load for documents written before the sidecar existed.
    let mut loaded_ids: Vec<String> = Vec::new();
    for key in &keys {
        if let Some(id) = key
            .strip_prefix(&prefix)
            .and_then(|k| k.strip_suffix(".meta.json"))
        {
            if let Ok(bytes) = drive.get_object("gbo", key).await {
                if let Ok(meta) = serde_json::from_slice::<SpreadsheetMetadata>(&bytes) {
                    if can_read_sheet(user_id, &metadata_shell(&meta)) {
                        sheets.push(meta);
                        loaded_ids.push(id.to_string());
                    }
                }
            }
        }
    }

    for key in &keys {
        if !key.ends_with(".json") || key.ends_with(".meta.json") {
            continue;
        }
        let id = key
            .split('/')
            .next_back()
            .unwrap_or("")
            .strip_suffix(".json")
            .unwrap_or("")
            .to_string();
        if loaded_ids.contains(&id) {
            continue;
        }
        if let Ok(sheet) = load_sheet_by_id(state, user_id, &id).await {
            sheets.push(SpreadsheetMetadata {
                id: sheet.id,
                name: sheet.name,
                owner_id: sheet.owner_id,
                created_at: sheet.created_at,
                updated_at: sheet.updated_at,
                worksheet_count: sheet.worksheets.len(),
                acl: sheet.acl,
            });
        }
    }

    sheets.sort_by_key(|s| std::cmp::Reverse(s.updated_at));

    Ok(sheets)
}

/// Builds a bare spreadsheet-shaped record from sidecar metadata purely for
/// the access-control check; it never carries cell data.
fn metadata_shell(meta: &SpreadsheetMetadata) -> Spreadsheet {
    Spreadsheet {
        id: meta.id.clone(),
        name: meta.name.clone(),
        owner_id: meta.owner_id.clone(),
        worksheets: Vec::new(),
        created_at: meta.created_at,
        updated_at: meta.updated_at,
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: meta.acl.clone(),
    }
}

pub async fn delete_sheet_from_drive(
    state: &SheetState,
    user_id: &str,
    sheet_id: &str,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let json_path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);
    let meta_path = format!("{}/{}.meta.json", get_user_sheets_path(user_id), sheet_id);
    let xlsx_path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet_id);

    if let Err(e) = drive.delete_object("gbo", &json_path).await {
        log::warn!("delete sheet json '{json_path}' failed: {e}");
    }
    if let Err(e) = drive.delete_object("gbo", &meta_path).await {
        log::warn!("delete sheet meta '{meta_path}' failed: {e}");
    }
    if let Err(e) = drive.delete_object("gbo", &xlsx_path).await {
        log::warn!("delete sheet xlsx '{xlsx_path}' failed: {e}");
    }

    Ok(())
}
