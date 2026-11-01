//! Desktop activity heartbeat (#1620).
//!
//! The desktop shell must not exit while a reply is being generated. The
//! sidecar records how many LLM turns are in flight in a small JSON file inside
//! the stack directory; the Tauri app reads it synchronously when its last
//! window is destroyed and keeps the process alive while the file reports
//! activity. A file is used on purpose: no extra HTTP poll, no new dependency
//! and it works before the API routes are ready.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ACTIVE_TURNS: AtomicUsize = AtomicUsize::new(0);

/// Marks one in-flight reply for as long as the guard is alive.
pub struct ActivityGuard;

impl ActivityGuard {
    /// Registers a new in-flight turn and refreshes the heartbeat file.
    pub fn begin() -> Self {
        let active = ACTIVE_TURNS.fetch_add(1, Ordering::SeqCst) + 1;
        write_snapshot(active);
        Self
    }
}

impl Drop for ActivityGuard {
    fn drop(&mut self) {
        let previous = ACTIVE_TURNS.load(Ordering::SeqCst);
        let active = previous.saturating_sub(1);
        ACTIVE_TURNS.store(active, Ordering::SeqCst);
        write_snapshot(active);
    }
}

/// Number of replies the sidecar is generating right now.
pub fn active_turns() -> usize {
    ACTIVE_TURNS.load(Ordering::SeqCst)
}

fn stack_root() -> Option<std::path::PathBuf> {
    std::env::var("BOTSERVER_STACK_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(std::path::PathBuf::from)
}

/// Publishes the current activity so the desktop shell can read it even when
/// the last webview has already been destroyed.
fn write_snapshot(active: usize) {
    let Some(root) = stack_root() else {
        return;
    };
    let dir = root.join("run");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let updated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let payload = serde_json::json!({
        "active_turns": active,
        "updated_at": updated_at,
    });
    let path = dir.join("desktop-activity.json");
    let temporary = dir.join("desktop-activity.json.tmp");
    if std::fs::write(&temporary, payload.to_string()).is_ok() {
        let _ = std::fs::rename(&temporary, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_tracks_nested_turns() {
        let before = active_turns();
        let outer = ActivityGuard::begin();
        assert_eq!(active_turns(), before + 1);
        {
            let _inner = ActivityGuard::begin();
            assert_eq!(active_turns(), before + 2);
        }
        assert_eq!(active_turns(), before + 1);
        drop(outer);
        assert_eq!(active_turns(), before);
    }
}
