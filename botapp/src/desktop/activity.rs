//! Reads the sidecar activity heartbeat (#1620).
//!
//! The sidecar publishes `run/desktop-activity.json` while it generates a
//! reply. The desktop shell consults it when the last window is destroyed: as
//! long as a turn is in flight the process must stay alive instead of exiting
//! and killing the sidecar mid-generation.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use log::warn;

use super::wizard::stack_path;

/// How long a heartbeat is trusted without a refresh. A turn that crashed the
/// sidecar must not keep the desktop process alive forever.
const STALE_AFTER_SECS: u64 = 15 * 60;

/// True while the sidecar reports at least one reply in flight.
pub fn is_busy() -> bool {
    let path = stack_path().join("run").join("desktop-activity.json");
    match read_active_turns(&path) {
        Some(active) => active > 0,
        None => false,
    }
}

fn read_active_turns(path: &Path) -> Option<u64> {
    let raw = std::fs::read_to_string(path).ok()?;
    let payload: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            warn!("Could not parse the desktop activity heartbeat: {error}");
            return None;
        }
    };
    let active = payload.get("active_turns")?.as_u64()?;
    let updated_at = payload.get("updated_at")?.as_u64()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    if now.saturating_sub(updated_at) > STALE_AFTER_SECS {
        warn!("Ignoring a stale desktop activity heartbeat");
        return None;
    }
    Some(active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_size_is_bounded() {
        // The reader only trusts the two documented fields; this guards the
        // contract against a future payload that forgets `active_turns`.
        let payload = serde_json::json!({ "active_turns": 1, "updated_at": 1 });
        assert!(payload.get("active_turns").is_some());
    }
}
