use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use botcore::shared::analytics::MetricsCollector;

use crate::sessions::SessionStore;
use crate::types::{Spreadsheet, SpreadsheetMetadata};

/// #2459 — runtime-wide metrics handle. The runtime sets it once at boot with
/// the very collector the exporter scrapes, so states built here share it.
static METRICS: OnceLock<MetricsCollector> = OnceLock::new();

/// Install the shared office-metrics collector (idempotent, called at boot).
pub fn set_metrics_collector(collector: MetricsCollector) {
    if METRICS.set(collector).is_err() {
        log::warn!("botsheet-core: metrics collector already initialized");
    }
}

/// #2459 — record one workbook recalculation duration into the shared
/// collector. `recalculate_workbook` is synchronous and is called from both
/// async handlers and plain code, so the async hook is driven without ever
/// blocking a live runtime: a running runtime gets a detached task, and plain
/// code gets a tiny current-thread runtime. No file name or user id is passed;
/// the hook only sees the opaque `app="sheet"` scope.
pub(crate) fn record_recalc_duration(duration_ms: f64) {
    if !duration_ms.is_finite() || duration_ms < 0.0 {
        return;
    }
    let collector = METRICS.get_or_init(MetricsCollector::new).clone();
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            botcore::shared::analytics::track_office_recalc(&collector, "sheet", None, duration_ms)
                .await;
        });
        return;
    }
    match tokio::runtime::Builder::new_current_thread().build() {
        Ok(runtime) => runtime.block_on(async {
            botcore::shared::analytics::track_office_recalc(&collector, "sheet", None, duration_ms)
                .await;
        }),
        Err(e) => log::warn!("botsheet-core: cannot record recalc duration: {e}"),
    }
}

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
    /// #2459 — shared office metrics (workbook memory, export duration).
    pub metrics_collector: Arc<MetricsCollector>,
}

impl SheetState {
    pub fn new(drive: Option<Arc<dyn DriveOps>>) -> Self {
        Self {
            drive,
            on_save: None,
            sessions: SessionStore::default(),
            metrics_collector: Arc::new(METRICS.get_or_init(MetricsCollector::new).clone()),
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
