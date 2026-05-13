use std::sync::Arc;

use async_trait::async_trait;

use crate::types::SpreadsheetMetadata;

#[derive(Clone)]
pub struct SheetState {
    pub drive: Option<Arc<dyn DriveOps>>,
}

impl SheetState {
    pub fn new(drive: Option<Arc<dyn DriveOps>>) -> Self {
        Self { drive }
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

pub fn get_current_user_id() -> String {
    "default-user".to_string()
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
                .last()
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

    sheets.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

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
