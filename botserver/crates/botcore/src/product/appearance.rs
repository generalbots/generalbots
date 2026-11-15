//! Appearance chosen on this installation.
//!
//! The first-run wizard stores the accent colour the person picked in
//! `<stack>/conf/system/appearance.json`. It is an installation choice, not a
//! product decision, so it lives beside the stack and not in the `.product`
//! file, which the desktop shell replaces on every upgrade.

use std::path::{Path, PathBuf};

fn appearance_path(stack: &Path) -> PathBuf {
    stack.join("conf").join("system").join("appearance.json")
}

/// Accepts a `#rrggbb` colour only, so the value can be handed to a stylesheet.
fn normalize_accent(raw: &str) -> Option<String> {
    let digits = raw.trim().strip_prefix('#')?;
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(format!("#{}", digits.to_ascii_lowercase()))
    } else {
        None
    }
}

/// Reads the accent stored under `stack`, if there is a valid one.
pub fn accent_in(stack: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(appearance_path(stack)).ok()?;
    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            log::warn!("appearance.json is not valid JSON: {error}");
            return None;
        }
    };
    value
        .get("accent")
        .and_then(|accent| accent.as_str())
        .and_then(normalize_accent)
}

/// Accent of the running installation (`BOTSERVER_STACK_PATH`), if chosen.
pub fn installation_accent() -> Option<String> {
    let stack = std::env::var("BOTSERVER_STACK_PATH")
        .or_else(|_| std::env::var("GBO_STACK_PATH"))
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    accent_in(Path::new(&stack))
}

/// Resolves the stack path from the environment.
pub fn stack_path() -> Option<PathBuf> {
    std::env::var("BOTSERVER_STACK_PATH")
        .or_else(|_| std::env::var("GBO_STACK_PATH"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

/// Validates `raw` as a `#rrggbb` colour, creates the directory tree if needed
/// and writes `appearance.json` atomically (temp file + rename).
///
/// Returns the normalised colour on success.
pub fn write_accent(stack: &Path, raw: &str) -> Result<String, AppearanceError> {
    let colour = normalize_accent(raw).ok_or(AppearanceError::InvalidColour)?;
    let target = appearance_path(stack);
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(AppearanceError::Io)?;
    }
    let json = serde_json::json!({ "accent": colour });
    let bytes = serde_json::to_string_pretty(&json).map_err(AppearanceError::Json)?;
    // Atomic write: temp file in the same directory, then rename.
    let tmp = target.with_extension("json.tmp");
    std::fs::write(&tmp, bytes.as_bytes()).map_err(AppearanceError::Io)?;
    std::fs::rename(&tmp, &target).map_err(AppearanceError::Io)?;
    Ok(colour)
}

/// Removes the stored accent, returning to the product default.
pub fn clear_accent(stack: &Path) -> Result<(), AppearanceError> {
    let target = appearance_path(stack);
    match std::fs::remove_file(&target) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppearanceError::Io(e)),
    }
}

/// Errors from the appearance write path.
#[derive(Debug)]
pub enum AppearanceError {
    InvalidColour,
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for AppearanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidColour => f.write_str("accent must be a #rrggbb hex colour"),
            Self::Io(e) => write!(f, "appearance I/O: {e}"),
            Self::Json(e) => write!(f, "appearance JSON: {e}"),
        }
    }
}

impl std::error::Error for AppearanceError {}

#[cfg(test)]
mod tests {
    use super::{accent_in, appearance_path, clear_accent, normalize_accent, write_accent};

    #[test]
    fn only_a_hex_colour_is_accepted() {
        assert_eq!(normalize_accent("#F28C28").as_deref(), Some("#f28c28"));
        for raw in ["", "f28c28", "#fff", "#f28c2g", "red", "#f28c28;}"] {
            assert_eq!(normalize_accent(raw), None, "{raw:?} must be refused");
        }
    }

    #[test]
    fn the_stored_accent_is_read_back() {
        let stack = std::env::temp_dir().join(format!("gb-appearance-{}", std::process::id()));
        let file = appearance_path(&stack);
        let dir = file.parent().map(std::path::Path::to_path_buf);
        assert_eq!(accent_in(&stack), None, "no file means no accent");
        if let Some(dir) = dir {
            std::fs::create_dir_all(dir).ok();
        }
        std::fs::write(&file, r##"{ "accent": "#2563EB" }"##).ok();
        assert_eq!(accent_in(&stack).as_deref(), Some("#2563eb"));
        std::fs::write(&file, r#"{ "accent": "javascript:1" }"#).ok();
        assert_eq!(accent_in(&stack), None);
        std::fs::remove_dir_all(&stack).ok();
    }

    #[test]
    fn write_accent_creates_and_reads_back() {
        let stack = std::env::temp_dir().join(format!("gb-write-accent-{}", std::process::id()));
        let result = write_accent(&stack, "#A1B2C3");
        assert_eq!(result.ok(), Some("#a1b2c3".to_string()));
        assert_eq!(accent_in(&stack).as_deref(), Some("#a1b2c3"));
        std::fs::remove_dir_all(&stack).ok();
    }

    #[test]
    fn write_accent_rejects_invalid() {
        let stack = std::env::temp_dir().join(format!("gb-write-invalid-{}", std::process::id()));
        assert!(write_accent(&stack, "red").is_err());
        assert!(write_accent(&stack, "#fff").is_err());
        assert!(write_accent(&stack, "").is_err());
        std::fs::remove_dir_all(&stack).ok();
    }

    #[test]
    fn clear_accent_removes_the_file() {
        let stack = std::env::temp_dir().join(format!("gb-clear-accent-{}", std::process::id()));
        write_accent(&stack, "#112233").ok();
        assert!(accent_in(&stack).is_some());
        assert!(clear_accent(&stack).is_ok());
        assert!(accent_in(&stack).is_none());
        // Clearing when already absent is fine.
        assert!(clear_accent(&stack).is_ok());
        std::fs::remove_dir_all(&stack).ok();
    }
}
