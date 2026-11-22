//! #2522 — machine-readable security notices for import responses.
//!
//! Always compiled (unlike `import_security`, which needs the `xlsx` feature)
//! because the import handler returns a notice even for a package whose
//! parser is unavailable. `message_key` is a catalogue key; the driver adds
//! `file-macros-removed` to every locale and binds this field in the UI.

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SecurityNotice {
    pub code: &'static str,
    pub message_key: &'static str,
    pub count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
}

pub(crate) fn macro_notice(parts: Vec<String>, count: usize) -> SecurityNotice {
    SecurityNotice {
        code: "file-macros-removed",
        message_key: "file-macros-removed",
        count,
        parts,
    }
}

/// Attaches notices to the existing JSON shape; all `Spreadsheet` fields are
/// preserved and the notice list is omitted when nothing was sanitized.
pub(crate) fn with_notices<T: serde::Serialize>(
    sheet: T,
    notices: Vec<SecurityNotice>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut value = serde_json::to_value(sheet).map_err(|error| {
        log::error!("sheet import: response serialization failed: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Value::String("Response serialization failed".to_string())),
        )
    })?;
    if !notices.is_empty() {
        let encoded = serde_json::to_value(notices).map_err(|error| {
            log::error!("sheet import: notice serialization failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String("Response serialization failed".to_string())),
            )
        })?;
        if let Some(map) = value.as_object_mut() {
            map.insert("security_notices".to_string(), encoded);
        }
    }
    Ok(Json(value))
}
