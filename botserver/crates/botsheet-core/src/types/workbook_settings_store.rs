//! Hot cache of active workbook settings (#2572).
//!
//! The persisted workbook JSON is the source of truth. This registry only
//! keeps settings for workbooks currently open on the server so the save hook
//! can embed them without changing the external `Spreadsheet` literal shape.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use super::workbook_settings::WorkbookSettings;

static WORKBOOK_SETTINGS: OnceLock<RwLock<HashMap<String, WorkbookSettings>>> = OnceLock::new();

fn settings_store() -> &'static RwLock<HashMap<String, WorkbookSettings>> {
    WORKBOOK_SETTINGS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Remembers the active settings for a workbook.
pub fn set_workbook_settings(sheet_id: &str, settings: WorkbookSettings) {
    match settings_store().write() {
        Ok(mut store) => {
            store.insert(sheet_id.to_string(), settings);
        }
        Err(poisoned) => {
            poisoned.into_inner().insert(sheet_id.to_string(), settings);
        }
    }
}

/// Returns cached settings, or the defaults for a workbook never opened here.
pub fn workbook_settings(sheet_id: &str) -> WorkbookSettings {
    match settings_store().read() {
        Ok(store) => store.get(sheet_id).copied().unwrap_or_default(),
        Err(poisoned) => poisoned
            .into_inner()
            .get(sheet_id)
            .copied()
            .unwrap_or_default(),
    }
}

pub fn forget_workbook_settings(sheet_id: &str) {
    match settings_store().write() {
        Ok(mut store) => {
            store.remove(sheet_id);
        }
        Err(poisoned) => {
            poisoned.into_inner().remove(sheet_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FormulaLocale;

    #[test]
    fn settings_store_is_per_workbook() {
        let id = "settings-store-test";
        forget_workbook_settings(id);
        let custom = WorkbookSettings {
            locale: FormulaLocale::De,
            ..WorkbookSettings::default()
        };
        set_workbook_settings(id, custom);
        assert_eq!(workbook_settings(id), custom);
        assert_eq!(workbook_settings("another-sheet"), WorkbookSettings::default());
        forget_workbook_settings(id);
        assert_eq!(workbook_settings(id), WorkbookSettings::default());
    }
}
