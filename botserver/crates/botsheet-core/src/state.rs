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
            sessions: SessionStore::default(),
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
        .put_object("gbo", &meta_path, meta_json.into_bytes(), "application/json")
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
    let meta_path = format!("{}/{}.meta.json", get_user_sheets_path(user_id), sheet_id);
    let xlsx_path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet_id);

    let _ = drive.delete_object("gbo", &json_path).await;
    let _ = drive.delete_object("gbo", &meta_path).await;
    let _ = drive.delete_object("gbo", &xlsx_path).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemDrive(Mutex<HashMap<String, Vec<u8>>>);

    #[async_trait]
    impl DriveOps for MemDrive {
        async fn put_object(
            &self,
            _bucket: &str,
            key: &str,
            body: Vec<u8>,
            _content_type: &str,
        ) -> Result<(), String> {
            self.0.lock().unwrap().insert(key.to_string(), body);
            Ok(())
        }

        async fn get_object(&self, _bucket: &str, key: &str) -> Result<Vec<u8>, String> {
            self.0
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or_else(|| format!("no such object {key}"))
        }

        async fn list_objects(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
            let map = self.0.lock().unwrap();
            let mut keys: Vec<String> = map
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect();
            keys.sort();
            Ok(keys)
        }

        async fn delete_object(&self, _bucket: &str, key: &str) -> Result<(), String> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn sample_sheet(owner: &str) -> Spreadsheet {
        let mut acl = HashMap::new();
        acl.insert("alice".to_string(), "edit".to_string());
        Spreadsheet {
            id: "s1".to_string(),
            name: "Report".to_string(),
            owner_id: owner.to_string(),
            worksheets: vec![crate::types::Worksheet::default()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            named_ranges: None,
            external_links: None,
            source_bucket: None,
            source_path: None,
            source_bytes: None,
            acl,
        }
    }

    #[tokio::test]
    async fn save_writes_sidecar_and_list_uses_it() {
        let drive = Arc::new(MemDrive::default());
        let state = SheetState {
            drive: Some(drive.clone()),
            on_save: None,
            sessions: SessionStore::default(),
        };
        let sheet = sample_sheet("alice");
        save_sheet_to_drive(&state, "alice", &sheet).await.unwrap();

        {
            let map = drive.0.lock().unwrap();
            let meta_key: Vec<&String> =
                map.keys().filter(|k| k.ends_with(".meta.json")).collect();
            assert_eq!(meta_key.len(), 1, "save writes exactly one sidecar");
            let meta: SpreadsheetMetadata =
                serde_json::from_slice(&map[meta_key[0]]).expect("sidecar parses");
            assert_eq!(meta.id, "s1");
            assert_eq!(meta.worksheet_count, 1);
            assert_eq!(meta.acl.get("alice").map(String::as_str), Some("edit"));
        }

        let listed = list_sheets_from_drive(&state, "alice").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Report");
    }

    #[tokio::test]
    async fn list_loads_legacy_documents_without_sidecar() {
        let drive = Arc::new(MemDrive::default());
        let state = SheetState {
            drive: Some(drive.clone()),
            on_save: None,
            sessions: SessionStore::default(),
        };
        let sheet = sample_sheet("alice");
        let path = format!("{}/{}.json", get_user_sheets_path("alice"), sheet.id);
        let content = serde_json::to_string(&sheet).unwrap();
        drive
            .put_object("gbo", &path, content.into_bytes(), "application/json")
            .await
            .unwrap();

        let listed = list_sheets_from_drive(&state, "alice").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Report");
    }

    #[tokio::test]
    async fn delete_removes_sidecar() {
        let drive = Arc::new(MemDrive::default());
        let state = SheetState {
            drive: Some(drive.clone()),
            on_save: None,
            sessions: SessionStore::default(),
        };
        let sheet = sample_sheet("alice");
        save_sheet_to_drive(&state, "alice", &sheet).await.unwrap();
        delete_sheet_from_drive(&state, "alice", &Some("s1".to_string()))
            .await
            .unwrap();
        let map = drive.0.lock().unwrap();
        assert!(map.is_empty(), "delete clears json, sidecar and xlsx");
    }
}
