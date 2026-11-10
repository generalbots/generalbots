//! Sidecar log bookkeeping (#1895): boot banner and inode freshness.
//!
//! Split out of `sidecar.rs` so the process-owning type stays small.

use log::warn;
use std::io::Write;
use std::path::Path;

/// Writes the timestamped boot banner into the sidecar log (#1895).
///
/// The banner is the proof that the file being read is the file this boot is
/// writing: a log without a banner for the running process is stale.
pub(super) fn write_boot_banner(file: &std::fs::File, path: &Path) {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let mut handle = file;
    if let Err(e) = writeln!(
        handle,
        "\n=== generalbots sidecar boot (unix {stamp}, app {}) log={} ===",
        env!("CARGO_PKG_VERSION"),
        path.display()
    ) {
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
