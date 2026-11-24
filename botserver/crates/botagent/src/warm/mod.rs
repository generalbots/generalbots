//! Warm agent processes kept between the chat turns of one conversation.
//!
//! An agent CLI that can take several turns on one process (the Codex app
//! server, Claude Code's stream-json input, ACP) is put back here after a
//! turn that ended cleanly, keyed by (workspace, conversation), and taken out
//! by the next turn of the same conversation instead of starting the CLI
//! again inside the workspace.
//!
//! The pool never reads a process: while a process is here nothing answers
//! it, and a permission request it sends meanwhile belongs to no turn. The
//! engine refuses every such request when it takes the process out, before
//! the new turn starts, so a request only ever reaches the gate of the turn
//! that is running ([`IDLE_REFUSAL`]).
//!
//! A process leaves the pool for good when it was idle for [`IDLE_LIMIT`],
//! when it is older than its lifetime, when its owner has more than
//! [`PER_USER`] warm processes (the longest idle goes), when it is replaced,
//! when its conversation is deleted ([`forget_conversation`]) and when the
//! server stops ([`shut_down_all`]). A process that does not match what the
//! turn needs (another model or system prompt) is ended and the turn starts
//! a fresh one.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub(crate) mod observe;

#[cfg(test)]
mod tests;

/// A warm process nobody used for this long is ended.
pub const IDLE_LIMIT: Duration = Duration::from_secs(10 * 60);
/// Warm processes one workspace (one person) keeps per engine.
pub const PER_USER: usize = 3;
/// What a request that arrives while no turn runs is told.
pub const IDLE_REFUSAL: &str = "No chat turn is active; the request is refused.";
const REAP_EVERY: Duration = Duration::from_secs(30);

/// One warm process of an engine.
pub trait WarmProcess: Send + 'static {
    /// Whether the process is still running.
    fn alive(&self) -> bool;
    /// Ends the process without blocking the caller.
    fn shut_down(self);
}

/// Which conversation a warm process serves.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WarmKey {
    pub vm: String,
    pub conversation: String,
}

impl WarmKey {
    pub fn new(vm: &str, conversation: &str) -> Self {
        Self { vm: vm.to_string(), conversation: conversation.to_string() }
    }
}

/// A process with the moment it started.
pub struct Warm<P> {
    pub process: P,
    pub born: Instant,
}

/// How long processes may live and how many one person keeps.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub idle: Duration,
    /// Oldest a process may be when a turn takes it.
    pub max_age: Duration,
    pub per_user: usize,
}

struct Entry<P> {
    process: P,
    fingerprint: String,
    born: Instant,
    idle_since: Instant,
}

/// The warm processes of one engine.
pub struct WarmPool<P> {
    name: &'static str,
    limits: Limits,
    entries: Mutex<HashMap<WarmKey, Entry<P>>>,
    reaper: OnceLock<()>,
}

/// What a turn needs from a process, as a short digest (never the text).
pub fn fingerprint(parts: &[&str]) -> String {
    let mut hasher = DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

fn end_all<P: WarmProcess>(victims: Vec<P>) -> usize {
    let count = victims.len();
    for process in victims {
        process.shut_down();
    }
    count
}

impl<P: WarmProcess> WarmPool<P> {
    pub fn new(name: &'static str, limits: Limits) -> Self {
        Self { name, limits, entries: Mutex::new(HashMap::new()), reaper: OnceLock::new() }
    }

    fn usable(&self, entry: &Entry<P>, now: Instant) -> bool {
        entry.process.alive()
            && now.saturating_duration_since(entry.idle_since) < self.limits.idle
            && now.saturating_duration_since(entry.born) < self.limits.max_age
    }

    /// Takes the conversation's process when it is usable and matches
    /// `fingerprint`; a process that does not is ended.
    pub fn take(&self, key: &WarmKey, fingerprint: &str, now: Instant) -> Option<Warm<P>> {
        let entry = self.entries.lock().ok()?.remove(key)?;
        if entry.fingerprint == fingerprint && self.usable(&entry, now) {
            return Some(Warm { process: entry.process, born: entry.born });
        }
        tracing::info!("{} warm: process not reused (changed, idle or ended); starting a fresh one", self.name);
        entry.process.shut_down();
        None
    }

    /// Keeps `warm` for the conversation's next turn.
    pub fn put(&self, key: WarmKey, fingerprint: String, warm: Warm<P>, now: Instant) {
        if !warm.process.alive() || now.saturating_duration_since(warm.born) >= self.limits.max_age {
            warm.process.shut_down();
            return;
        }
        let mut victims = Vec::new();
        {
            let Ok(mut entries) = self.entries.lock() else {
                warm.process.shut_down();
                return;
            };
            let vm = key.vm.clone();
            let entry = Entry { process: warm.process, fingerprint, born: warm.born, idle_since: now };
            if let Some(old) = entries.insert(key.clone(), entry) {
                victims.push(old.process);
            }
            loop {
                let mine: Vec<(&WarmKey, Instant)> =
                    entries.iter().filter(|(k, _)| k.vm == vm).map(|(k, e)| (k, e.idle_since)).collect();
                if mine.len() <= self.limits.per_user {
                    break;
                }
                let Some(oldest) = mine.iter().filter(|(k, _)| **k != key).min_by_key(|(_, at)| *at).map(|(k, _)| (*k).clone())
                else {
                    break;
                };
                if let Some(old) = entries.remove(&oldest) {
                    victims.push(old.process);
                }
            }
        }
        end_all(victims);
    }

    /// Ends every process that is no longer usable; returns how many.
    pub fn reap(&self, now: Instant) -> usize {
        let victims = match self.entries.lock() {
            Ok(mut entries) => {
                let stale: Vec<WarmKey> =
                    entries.iter().filter(|(_, e)| !self.usable(e, now)).map(|(k, _)| k.clone()).collect();
                stale.iter().filter_map(|k| entries.remove(k)).map(|e| e.process).collect()
            }
            Err(_) => Vec::new(),
        };
        end_all(victims)
    }

    fn remove_where(&self, matches: impl Fn(&WarmKey) -> bool) -> usize {
        let victims = match self.entries.lock() {
            Ok(mut entries) => {
                let keys: Vec<WarmKey> = entries.keys().filter(|k| matches(k)).cloned().collect();
                keys.iter().filter_map(|k| entries.remove(k)).map(|e| e.process).collect()
            }
            Err(_) => Vec::new(),
        };
        end_all(victims)
    }

    /// Ends the processes of one conversation, in any workspace.
    pub fn forget_conversation(&self, conversation: &str) -> usize {
        self.remove_where(|k| k.conversation == conversation)
    }

    /// Ends every process of this engine.
    pub fn clear(&self) -> usize {
        self.remove_where(|_| true)
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Starts the task that ends idle processes of `pool` (once per pool).
pub fn keep_reaped<P: WarmProcess>(pool: &'static WarmPool<P>) {
    if pool.reaper.set(()).is_err() {
        return;
    }
    let Ok(handle) = tokio::runtime::Handle::try_current() else { return };
    handle.spawn(async move {
        let mut tick = tokio::time::interval(REAP_EVERY);
        loop {
            tick.tick().await;
            let ended = pool.reap(Instant::now());
            if ended > 0 {
                tracing::info!("{} warm: {ended} idle process(es) ended", pool.name);
            }
        }
    });
}

/// For the JSON-RPC engines: answers every request a process sent while no
/// turn ran with an error and drops the rest of what it said. `false` when
/// the process ended.
pub(crate) fn refuse_idle_rpc(inbox: &mut crate::duplex::RpcInbox, engine: &str) -> bool {
    use crate::duplex::{Incoming, INTERNAL_ERROR};
    while let Some(message) = inbox.try_next() {
        match message {
            Incoming::Request(request) => {
                tracing::warn!("{engine} warm: a permission request arrived between turns; refused");
                if let Err(error) = request.respond_error(INTERNAL_ERROR, IDLE_REFUSAL) {
                    tracing::info!("{engine} warm: refusal not delivered: {error}");
                }
            }
            Incoming::Ended(_) => return false,
            Incoming::Notification { .. } | Incoming::Unparsed(_) => {}
        }
    }
    true
}

/// Ends the warm processes of a deleted conversation.
pub fn forget_conversation(conversation: &str) {
    let ended = crate::codex::warm_pool().forget_conversation(conversation)
        + crate::claude::bridge::warm_pool().forget_conversation(conversation)
        + crate::antigravity::acp::warm_pool().forget_conversation(conversation);
    if ended > 0 {
        tracing::info!("warm: {ended} agent process(es) of a deleted conversation ended");
    }
}

/// Ends every warm process (server shutdown).
pub fn shut_down_all() {
    let ended = crate::codex::warm_pool().clear()
        + crate::claude::bridge::warm_pool().clear()
        + crate::antigravity::acp::warm_pool().clear();
    tracing::info!("warm: {ended} agent process(es) ended at shutdown");
}

/// The limits of an engine whose turns may last `turn_deadline` on a
/// process that lives at most `lifetime`.
pub fn limits_for(lifetime: Duration, turn_deadline: Duration) -> Limits {
    Limits { idle: IDLE_LIMIT, max_age: lifetime.saturating_sub(turn_deadline), per_user: PER_USER }
}
