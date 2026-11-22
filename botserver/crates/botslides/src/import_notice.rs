//! #2522 — machine-readable security notices for presentation imports.
//!
//! Mirrors the Docs/Sheets notice shape so the suite can render one shared
//! status surface. `message_key` is a catalogue key already shipped in every
//! locale (`ui-common.ftl`); the backend never emits a user-facing literal.

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

/// A bounded, sanitized description of one security action taken at import.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SecurityNotice {
    pub code: &'static str,
    pub message_key: &'static str,
    pub count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
}

/// Notice for a macro-enabled package whose executable parts were removed.
pub(crate) fn macro_notice(parts: Vec<String>, count: usize) -> SecurityNotice {
    SecurityNotice {
        code: "file-macros-removed",
        message_key: "file-macros-removed",
        count,
        parts,
    }
}

/// Attaches notices to the existing JSON model. Every presentation field is
/// preserved and the field is omitted when nothing was sanitized.
pub(crate) fn with_notices<T: Serialize>(
    presentation: T,
    notices: Vec<SecurityNotice>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut value = serde_json::to_value(presentation).map_err(|error| {
        log::error!("slides import: response serialization failed: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Value::String("Response serialization failed".to_string())),
        )
    })?;
    if !notices.is_empty() {
        let encoded = serde_json::to_value(notices).map_err(|error| {
            log::error!("slides import: notice serialization failed: {error}");
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_macro_notice_uses_only_catalogue_keys() {
        let notice = macro_notice(vec!["ppt/vbaproject.bin".to_string()], 1);
        assert_eq!(notice.code, "file-macros-removed");
        assert_eq!(notice.message_key, "file-macros-removed");
        assert_eq!(notice.count, 1);
        assert_eq!(notice.parts, vec!["ppt/vbaproject.bin".to_string()]);
    }

    #[test]
    fn notices_are_attached_without_losing_model_fields() {
        let model = json!({ "id": "deck-1", "name": "Import" });
        let result = with_notices(
            model,
            vec![macro_notice(vec!["ppt/vbaproject.bin".to_string()], 1)],
        );
        assert!(result.is_ok(), "serialization must not fail");
        if let Ok(Json(value)) = result {
            assert_eq!(value["id"], "deck-1");
            assert_eq!(value["name"], "Import");
            assert_eq!(
                value["security_notices"][0]["message_key"],
                "file-macros-removed"
            );
            assert_eq!(value["security_notices"][0]["count"], 1);
        }
    }

    #[test]
    fn an_empty_notice_list_is_omitted() {
        let result = with_notices(json!({ "id": "deck-2" }), Vec::new());
        assert!(result.is_ok(), "serialization must not fail");
        if let Ok(Json(value)) = result {
            assert!(value.get("security_notices").is_none());
        }
    }
}
