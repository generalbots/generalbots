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
pub fn ensure_write_allowed(
    user_id: &str,
    sheet: &crate::types::Spreadsheet,
) -> Result<(), String> {
    if can_write_sheet(user_id, sheet) {
        Ok(())
    } else {
        Err("Access denied".to_string())
    }
}

mod persistence;
pub use persistence::{
    delete_sheet_from_drive, list_sheets_from_drive, load_sheet_by_id, load_sheet_from_drive,
    persist_sheet_to_drive, save_sheet_to_drive,
};

#[cfg(test)]
mod tests;
