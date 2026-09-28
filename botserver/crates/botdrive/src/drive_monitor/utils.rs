// Utility functions for drive_monitor module
// Re-exports from types.rs to maintain module structure
// Re-exported from types.rs for module structure
// pub use super::types::normalize_config_value;

/// Bot source prefixes a git-owned bot materializes from its repository
/// (reform #1501). For such a bot the `drive_files` row is created to trigger a
/// compile from the work copy while no Drive object exists at all, so the
/// "object disappeared" cleanup must not treat the row as garbage.
///
/// Only sources are exempt: `.gbkb` and `.gbdrive` objects are genuinely
/// Drive-owned and are still deleted when they vanish.
pub(crate) fn is_source_prefix(s3_key: &str) -> bool {
    s3_key.contains(".gbdialog/") || s3_key.contains(".gbot/")
}

#[cfg(test)]
mod tests {
    use super::is_source_prefix;

    #[test]
    fn recognizes_the_source_prefixes_only() {
        assert!(is_source_prefix("acme.gbdialog/classify_media.bas"));
        assert!(is_source_prefix("acme.gbot/PROMPT-TELEGRAM.md"));
        assert!(!is_source_prefix("acme.gbkb/docs/manual.pdf"));
        assert!(!is_source_prefix("acme.gbdrive/media/2026/09/nature/x.jpg"));
    }
}
