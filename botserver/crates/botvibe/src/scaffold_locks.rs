//! #1445 G1 — per-workspace scaffold locks.
//!
//! Workspace seeding raced across three independent paths (project create,
//! the run fire-and-forget scaffold, `run_project_app` re-seed): the
//! `read_dir().next()` emptiness check ran without a lock and two parallel
//! scaffolds for the same new workspace interleaved writes. One exclusive
//! slot per workspace key serializes the check + write so a half-written
//! scaffold is never served.
//!
//! The slots are `tokio::sync::Mutex` (Send guards) because the scaffold
//! body awaits the LLM between the emptiness check and the write; callers
//! hold the returned guard across those await points.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Mutex as AsyncMutex;

static LOCKS: OnceLock<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, Arc<AsyncMutex<()>>>> {
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The exclusive scaffold slot for `key`. Callers lock it (`.await`) and hold
/// the guard across the emptiness check + write.
pub fn lock_for(key: &str) -> Arc<AsyncMutex<()>> {
    let mut reg = match registry().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    reg.entry(key.to_string()).or_default().clone()
}
