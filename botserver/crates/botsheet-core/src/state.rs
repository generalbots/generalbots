use std::sync::Arc;

use async_trait::async_trait;

use crate::sessions::SessionStore;
use crate::types::{Spreadsheet, SpreadsheetMetadata};

/// Post-save hook that runs after `save_sheet_to_drive` persists the JSON.
/// Used by the `botsheet` crate to export back to the original xlsx in Drive.
pub type SheetSaveHook = Arc<dyn Fn(&Spreadsheet) -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct SheetState {
    pub drive: Option<Arc<dyn DriveOps>>,
    pub on_save: Option<SheetSaveHook>,
    /// Live in-memory document sessions (#789): shared state, oplog,
    /// versioning, debounced persistence and idle eviction.
    pub sessions: SessionStore,
}

impl SheetState {
    pub fn new(drive: Option<Arc<dyn DriveOps>>) -> Self {
        Self {
            drive,
            on_save: None,
            sessions: SessionStore::new(),
        }
    }
}

#[async_trait]
pub trait DriveOps: Send + Sync {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String>;

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String>;

    async fn list_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, String>;

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), String>;
}

pub fn get_user_sheets_path(user_id: &str) -> String {
    format!("users/{}/sheets", user_id)
}

/// Returns true when the sheet owner is the legacy placeholder, meaning
/// pre-identity sheets remain readable for compatibility (#789).
fn is_legacy_owned(owner: &str) -> bool {
    owner.is_empty() || owner == "default-user"
}

/// Read access rule (#789): the owner, any shared user, and every legacy
/// (pre-identity) sheet are readable; otherwise access is denied.
pub fn can_read_sheet(user_id: &str, sheet: &crate::types::Spreadsheet) -> bool {
    if sheet.owner_id == user_id {
        return true;
    }
    if is_legacy_owned(&sheet.owner_id) {
        return true;
    }
    sheet.acl.contains_key(user_id)
}

/// Write access rule (#789): owner, legacy sheets, or explicit `edit` grant.
pub fn can_write_sheet(user_id: &str, sheet: &crate::types::Spreadsheet) -> bool {
    if sheet.owner_id == user_id {
        return true;
    }
    if is_legacy_owned(&sheet.owner_id) {
        return true;
    }
    matches!(sheet.acl.get(user_id).map(String::as_str), Some("edit"))
}

/// Returns `Ok(())` when the user may mutate the sheet; otherwise an
/// `Access denied` error suitable for a 403-style response.
pub fn ensure_write_allowed(user_id: &str, sheet: &crate::types::Spreadsheet) -> Result<(), String> {
    if can_write_sheet(user_id, sheet) {
        Ok(())
    } else {
        Err("Access denied".to_string())
    }
}

pub async fn save_sheet_to_drive(
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

    drive
        .put_object("gbo", &path, content.into_bytes(), "application/json")
        .await?;

    // Run post-save hook (xlsx export back to original bucket/path)
    if let Some(ref hook) = state.on_save {
        hook(sheet)?;
    }

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

    Ok(sheet)
}

pub async fn load_sheet_by_id(
    state: &SheetState,
    user_id: &str,
    sheet_id: &str,
) -> Result<crate::types::Spreadsheet, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let bytes = drive.get_object("gbo", &path).await?;

    let sheet: crate::types::Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;

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

    for key in &keys {
        if key.ends_with(".json") {
            let id = key
                .split('/')
                .next_back()
                .unwrap_or("")
                .strip_suffix(".json")
                .unwrap_or("")
                .to_string();
            if let Ok(sheet) = load_sheet_by_id(state, user_id, &id).await {
                sheets.push(SpreadsheetMetadata {
                    id: sheet.id,
                    name: sheet.name,
                    owner_id: sheet.owner_id,
                    created_at: sheet.created_at,
                    updated_at: sheet.updated_at,
                    worksheet_count: sheet.worksheets.len(),
                });
            }
        }
    }

    sheets.sort_by_key(|s| std::cmp::Reverse(s.updated_at));

    Ok(sheets)
}

pub async fn delete_sheet_from_drive(
    state: &SheetState,
    user_id: &str,
    sheet_id: &Option<String>,
) -> Result<(), String> {
    let sheet_id = sheet_id
        .as_ref()
        .ok_or_else(|| "Sheet ID is required".to_string())?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let json_path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);
    let xlsx_path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet_id);

    let _ = drive.delete_object("gbo", &json_path).await;
    let _ = drive.delete_object("gbo", &xlsx_path).await;

    Ok(())
}
