//! Sidecar log bookkeeping (#1895): boot banner and inode freshness.
//!
//! Split out of `sidecar.rs` so the process-owning type stays small.

use log::warn;
use std::io::Write;
use std::path::Path;

/// Formats the banner stamped at the top of every sidecar boot.
///
/// Pure so the exact shape is testable: the banner is the proof that the file
/// being read is the file this boot is writing.
pub(super) fn boot_banner(stamp: u64, version: &str, path: &Path) -> String {
    format!(
        "\n=== generalbots sidecar boot (unix {stamp}, app {version}) log={} ===",
        path.display()
    )
}

/// Writes the timestamped boot banner into the sidecar log (#1895).
///
/// The banner is the proof that the file being read is the file this boot is
/// writing: a log without a banner for the running process is stale.
pub(super) fn write_boot_banner(file: &std::fs::File, path: &Path) {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let banner = boot_banner(stamp, &crate::build_info::version(), path);
    let mut handle = file;
    if let Err(e) = writeln!(handle, "{banner}") {
        warn!("could not write the sidecar boot banner: {e}");
    }
    if let Err(e) = handle.flush() {
        warn!("could not flush the sidecar boot banner: {e}");
    }
}

/// Inode of `path`, used to detect a log file re-created under the sidecar.
#[cfg(unix)]
pub(super) fn file_inode(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).ok().map(|meta| meta.ino())
}

/// Windows has no inode to compare; the banner still proves freshness.
#[cfg(not(unix))]
pub(super) fn file_inode(_path: &Path) -> Option<u64> {
    None
}

/// Whether the log file was re-created between two inode reads.
///
/// A missing inode on either side means "cannot tell", which must not be
/// reported as a rotation.
pub(super) fn log_was_recreated(before: Option<u64>, after: Option<u64>) -> bool {
    match (before, after) {
        (Some(before), Some(after)) => before != after,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{boot_banner, log_was_recreated};
    use std::path::Path;

    #[test]
    fn the_banner_names_the_process_clock_version_and_log() {
        let banner = boot_banner(1_700_000_000, "6.8.0", Path::new("/tmp/stack/logs/x.log"));
        assert_eq!(
            banner,
            "\n=== generalbots sidecar boot (unix 1700000000, app 6.8.0) log=/tmp/stack/logs/x.log ==="
        );
        // The leading newline separates this boot from the previous run.
        assert!(banner.starts_with('\n'));
        assert!(banner.ends_with(" ==="));
    }

    #[test]
    fn a_zero_stamp_still_produces_a_banner() {
        let banner = boot_banner(0, "0.0.0", Path::new("x.log"));
        assert!(banner.contains("unix 0"));
        assert!(banner.contains("log=x.log"));
    }

    #[test]
    fn only_a_changed_inode_counts_as_a_rotation() {
        assert!(!log_was_recreated(Some(7), Some(7)));
        assert!(log_was_recreated(Some(7), Some(8)));
        // Windows reports no inode at all.
        assert!(!log_was_recreated(None, None));
        assert!(!log_was_recreated(Some(7), None));
        assert!(!log_was_recreated(None, Some(7)));
    }
}
