//! Persists the accent colour chosen in the first-run wizard.
//!
//! The suite reads it back through `/api/product`, so the desktop opens in the
//! colour the person picked. Only a `#rrggbb` value is stored; anything else is
//! refused instead of reaching a stylesheet.

use std::path::PathBuf;

use log::info;

use super::paths::stack_path;

/// File the server reads the accent from (see `botcore::product::appearance`).
pub(super) fn appearance_path() -> PathBuf {
    stack_path()
        .join("conf")
        .join("system")
        .join("appearance.json")
}

/// Normalises a `#rrggbb` colour to lower case, or `None` when it is not one.
pub(super) fn normalize_accent(raw: &str) -> Option<String> {
    let value = raw.trim();
    let digits = value.strip_prefix('#')?;
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(format!("#{}", digits.to_ascii_lowercase()))
    } else {
        None
    }
}

/// Writes the accent to the stack. An empty choice keeps the default identity
/// and writes nothing.
pub(super) fn write_accent(raw: &str) -> Result<(), String> {
    if raw.trim().is_empty() {
        return Ok(());
    }
    let accent =
        normalize_accent(raw).ok_or_else(|| format!("accent `{raw}` is not a #rrggbb colour"))?;
    let path = appearance_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    let body = serde_json::to_string_pretty(&serde_json::json!({ "accent": accent }))
        .map_err(|e| format!("serialize appearance: {e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    info!("wizard: accent {accent} written to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_accent;

    #[test]
    fn a_hex_colour_is_kept_in_lower_case() {
        assert_eq!(normalize_accent(" #F28C28 ").as_deref(), Some("#f28c28"));
        assert_eq!(normalize_accent("#2563eb").as_deref(), Some("#2563eb"));
    }

    #[test]
    fn anything_else_is_refused() {
        for raw in ["", "f28c28", "#f28", "#f28c2g", "red", "#f28c28;}", "url(x)"] {
            assert_eq!(normalize_accent(raw), None, "{raw:?} must be refused");
        }
    }
}
