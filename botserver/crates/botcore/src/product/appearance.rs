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

#[cfg(test)]
mod tests {
    use super::{accent_in, appearance_path, normalize_accent};

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
}
