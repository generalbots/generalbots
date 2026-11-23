use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
    pub connections: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
}

/// #2834 — Outcome of resolving a caller-supplied `path` against the
/// server-owned designer identity. The bot id and bucket never come from the
/// query string: `bot_id` is resolved by `get_default_bot` and `bucket_name`
/// comes from server configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum DesignerDriveKey {
    /// Validated Drive key, confined to the resolved bot's `.gbapp` prefix.
    Tenant(String),
    /// Plain dialog id (no path separators) — load from the designer DB.
    DialogId,
    /// Rejected: foreign prefix, traversal, or an unresolved (nil) bot.
    Denied,
}

/// True when `path` addresses a Drive object rather than a plain dialog id.
fn is_drive_object_path(path: &str) -> bool {
    path.contains('/') || path.contains('\\') || path.contains(".gbapp")
}

/// Pure, I/O-free resolver applied before any Drive read.
///
/// `default_bot_id` comes from the server's `get_default_bot` closure and
/// `bucket_name` from server config — both are trusted. `requested_path` is the
/// only caller-controlled value and is confined to `<bot>.gbapp/...` for the
/// resolved tenant. Foreign prefixes, traversal and a nil/unresolvable bot are
/// rejected (fail closed).
pub fn resolve_designer_drive_key(
    default_bot_id: uuid::Uuid,
    bucket_name: &str,
    requested_path: &str,
) -> DesignerDriveKey {
    if !is_drive_object_path(requested_path) {
        return DesignerDriveKey::DialogId;
    }
    if default_bot_id.is_nil() {
        return DesignerDriveKey::Denied;
    }
    let bot_slug = bucket_name.trim_end_matches(".gbai").trim();
    if bot_slug.is_empty() || bot_slug.contains('/') || bot_slug.contains('\\') {
        return DesignerDriveKey::Denied;
    }
    let allowed_prefix = format!("{bot_slug}.gbapp/");
    if !requested_path.starts_with(&allowed_prefix) {
        return DesignerDriveKey::Denied;
    }
    let remainder = &requested_path[allowed_prefix.len()..];
    if remainder.is_empty()
        || remainder.starts_with('/')
        || remainder.contains("..")
        || remainder.contains('\\')
        || requested_path.contains('\0')
    {
        return DesignerDriveKey::Denied;
    }
    DesignerDriveKey::Tenant(requested_path.to_string())
}

/// #2831 — strict Designer slug, mirroring `valid_app_id` in
/// `main_module/ui_plan/sanitize_value.rs`: non-empty, bounded and composed
/// only of alphanumerics, `-` or `_`. Applied to every caller/LLM supplied
/// *directory* component (app name) and to the server-derived org/bot slugs.
pub fn is_safe_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
}

/// #2831 — a single safe file component. Like [`is_safe_slug`] but also allows
/// a `.` separator for extensions (`index.html`, `styles.css`). Path
/// separators, `..`, NUL/control characters and absolute paths are rejected by
/// construction because they cannot pass the character allow-list.
pub fn is_safe_file_name(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 128
        || value == "."
        || value == ".."
        || value.contains("..")
        || value.starts_with('/')
        || value.starts_with('\\')
    {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// #2831 — validates a Drive key produced for a Designer write. Requires the
/// server-derived `<bot>.gbapp/` prefix and confines every remaining segment
/// to the same safe-component rule, so an LLM-influenced file name can never
/// retarget the object outside the resolved tenant's `.gbapp` prefix.
pub fn is_confined_drive_key(key: &str, bot_slug: &str) -> bool {
    if !is_safe_slug(bot_slug) || key.contains('\0') || key.contains('\\') {
        return false;
    }
    let prefix = format!("{bot_slug}.gbapp/");
    match key.strip_prefix(&prefix) {
        Some(rest) => !rest.is_empty() && rest.split('/').all(is_safe_file_name),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn tenant_a() -> Uuid {
        Uuid::from_u128(1)
    }

    #[test]
    fn valid_key_within_resolved_bot_prefix() {
        assert_eq!(
            resolve_designer_drive_key(tenant_a(), "default.gbai", "default.gbapp/welcome.bas"),
            DesignerDriveKey::Tenant("default.gbapp/welcome.bas".to_string())
        );
    }

    #[test]
    fn two_tenant_key_is_confined_to_resolved_tenant() {
        let foreign_key = "other.gbapp/secret.bas";
        // Tenant B's key is not readable while tenant A is the resolved bot.
        assert_eq!(
            resolve_designer_drive_key(tenant_a(), "default.gbai", foreign_key),
            DesignerDriveKey::Denied
        );
        // The very same key is allowed only when the server resolves tenant B.
        assert_eq!(
            resolve_designer_drive_key(tenant_a(), "other.gbai", foreign_key),
            DesignerDriveKey::Tenant(foreign_key.to_string())
        );
    }

    #[test]
    fn nil_bot_id_is_denied() {
        assert_eq!(
            resolve_designer_drive_key(Uuid::nil(), "default.gbai", "default.gbapp/welcome.bas"),
            DesignerDriveKey::Denied
        );
    }

    #[test]
    fn traversal_and_absolute_keys_are_denied() {
        for key in [
            "default.gbapp/../other.bas",
            "default.gbapp//x.bas",
            "default.gbapp\\x.bas",
            "../default.gbapp/x.bas",
        ] {
            assert_eq!(
                resolve_designer_drive_key(tenant_a(), "default.gbai", key),
                DesignerDriveKey::Denied,
                "key must be denied: {key}"
            );
        }
    }

    #[test]
    fn plain_dialog_id_falls_back_to_db() {
        assert_eq!(
            resolve_designer_drive_key(tenant_a(), "default.gbai", "a-dialog-uuid"),
            DesignerDriveKey::DialogId
        );
    }

    #[test]
    fn designer_slug_accepts_valid_values() {
        for value in [
            "default",
            "my-app",
            "app_2",
            "00000000-0000-0000-0000-000000000000",
        ] {
            assert!(is_safe_slug(value), "slug must be accepted: {value}");
        }
    }

    #[test]
    fn designer_slug_rejects_traversal_absolute_and_nul() {
        for value in [
            "", "   ", "../etc", "a/b", "a\\b", "/etc", "C:\\x", "app\0x", "a.b", "..",
        ] {
            assert!(!is_safe_slug(value), "slug must be rejected: {value:?}");
        }
        assert!(!is_safe_slug(&"x".repeat(65)));
    }

    #[test]
    fn designer_file_name_accepts_simple_basenames() {
        for value in ["index.html", "styles.css", "app.min.js", "tool", "data-1_v2.bas"] {
            assert!(is_safe_file_name(value), "file must be accepted: {value}");
        }
    }

    #[test]
    fn designer_file_name_rejects_traversal_absolute_and_nul() {
        for value in [
            "", "..", ".", "../x", "a/../b", "/etc/passwd", "etc\\passwd", "a\0b", "a\u{1}b",
        ] {
            assert!(!is_safe_file_name(value), "file must be rejected: {value:?}");
        }
    }

    #[test]
    fn confined_drive_key_accepts_valid_and_rejects_traversal() {
        assert!(is_confined_drive_key("default.gbapp/my-app/index.html", "default"));
        for key in [
            "default.gbapp/../outside.bas",
            "default.gbapp//outside.bas",
            "default.gbapp\\outside.bas",
            "/etc/passwd",
            "other.gbapp/index.html",
            "default.gbapp/a/../b",
            "default.gbapp/x\0y",
        ] {
            assert!(
                !is_confined_drive_key(key, "default"),
                "drive key must be rejected: {key:?}"
            );
        }
    }
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DialogRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub line: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicRequest {
    pub nodes: Vec<MagicNode>,
    pub connections: i32,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicResponse {
    pub improved_code: Option<String>,
    pub explanation: Option<String>,
    pub suggestions: Option<Vec<MagicSuggestion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSuggestion {
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerModifyRequest {
    pub app_name: String,
    pub current_page: Option<String>,
    pub message: String,
    pub context: Option<DesignerContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerContext {
    pub page_html: Option<String>,
    pub tables: Option<Vec<String>>,
    pub recent_changes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerModifyResponse {
    pub success: bool,
    pub message: String,
    pub changes: Vec<DesignerChange>,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerChange {
    pub change_type: String,
    pub file_path: String,
    pub description: String,
    pub preview: Option<String>,
}
