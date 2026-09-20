//! `proxy_sites::release` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Marker file recording that a site directory is vibe-managed. Presence in
/// an existing directory makes redeploys safe (they only replace the project
/// payload, never foreign files).
pub(crate) const MARKER_FILE: &str = ".gb-vibe-site";

/// How many previous-release payload dirs and config backups to retain.
pub(crate) const RELEASE_RETENTION: usize = 10;

/// Process-wide serialization for every site mutation. A std Mutex is fine:
/// contention is publish-frequency (human scale) and holders do incus IO.
pub(crate) static PUBLISH_LOCK: LazyLock<std::sync::Mutex<()>> =
    LazyLock::new(|| std::sync::Mutex::new(()));

/// Serialize all site mutations; the guard is released on scope exit (also
/// on error paths). Poisoning is tolerated: a panicked publisher must not
/// wedge every future publish — recover and continue.
pub(crate) fn lock_publish() -> std::sync::MutexGuard<'static, ()> {
    match PUBLISH_LOCK.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}
