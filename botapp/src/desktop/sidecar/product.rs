//! Keeps the stack's `.product.desktop-local` in step with the bundled one.

use log::{info, warn};

/// Copy `src` to `dst`, replacing `dst` when their content differs. The
/// previous `dst` is kept as `{dst}.previous` in the same directory so the
/// user can inspect what changed after an upgrade. A missing `src` is a
/// no-op; any I/O failure is logged but does not abort the sidecar start.
pub(super) fn sync_product_file(src: &std::path::Path, dst: &std::path::Path) {
    if !src.exists() {
        return;
    }
    let src_content = match std::fs::read(src) {
        Ok(c) => c,
        Err(e) => {
            warn!("Could not read product file {}: {e}", src.display());
            return;
        }
    };
    if dst.exists() {
        if let Ok(dst_content) = std::fs::read(dst) {
            if src_content == dst_content {
                return;
            }
        }
        let prev = dst.with_extension("desktop-local.previous");
        if let Err(e) = std::fs::rename(dst, &prev) {
            warn!(
                "Could not keep previous product file at {}: {e}",
                prev.display()
            );
        } else {
            info!(
                "Replaced stale product file; previous version kept at {}",
                prev.display()
            );
        }
    }
    if let Err(e) = std::fs::copy(src, dst) {
        warn!(
            "Could not copy product file {} to {}: {e}",
            src.display(),
            dst.display()
        );
    }
}
