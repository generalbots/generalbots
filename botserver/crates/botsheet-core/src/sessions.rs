use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::state::{save_sheet_to_drive, SheetState};
use crate::dependency_graph::DepGraph;
use crate::types::Spreadsheet;

/// One oplog entry: a single user operation applied to the sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetOp {
    pub seq: u64,
    pub user_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub at: i64,
}

/// A live, in-memory document session (#789). Multiple clients mutate the same
/// `Arc<SheetSession>`; the version counter and oplog give clients a cheap
/// change feed, and a debounced background task persists to Drive so every
/// keystroke does not hit object storage.
pub struct SheetSession {
    pub sheet: tokio::sync::RwLock<Spreadsheet>,
    /// Cached dependency graphs, one per worksheet, maintained incrementally
    /// on edit so recalculation never rebuilds the full topology (#784).
    pub dep_graphs: std::sync::Mutex<Vec<DepGraph>>,
    pub version: AtomicU64,
    pub oplog: Mutex<Vec<SheetOp>>,
    pub last_access: AtomicI64,
    dirty: AtomicBool,
    save_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl SheetSession {
    fn new(sheet: Spreadsheet) -> Arc<Self> {
        let graphs = sheet.worksheets.iter().map(DepGraph::build).collect();
        Arc::new(Self {
            sheet: tokio::sync::RwLock::new(sheet),
            dep_graphs: std::sync::Mutex::new(graphs),
            version: AtomicU64::new(0),
            oplog: Mutex::new(Vec::new()),
            last_access: AtomicI64::new(Utc::now().timestamp()),
            dirty: AtomicBool::new(false),
            save_handle: Mutex::new(None),
        })
    }

    pub fn touch(&self) {
        self.last_access.store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    /// Records an operation and bumps the version. The oplog is bounded to the
    /// most recent 500 entries to keep memory flat for long-lived documents.
    pub fn record_op(&self, user_id: &str, kind: &str, payload: serde_json::Value) -> u64 {
        let seq = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        let op = SheetOp {
            seq,
            user_id: user_id.to_string(),
            kind: kind.to_string(),
            payload,
            at: Utc::now().timestamp_millis(),
        };
        let mut log = match self.oplog.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        log.push(op);
        if log.len() > 500 {
            let excess = log.len() - 500;
            log.drain(..excess);
        }
        seq
    }

    /// Marks the session dirty and schedules a debounced persist. Consecutive
    /// edits within 600 ms collapse into a single Drive write.
    pub fn schedule_save(self: &Arc<Self>, state: &SheetState) {
        self.dirty.store(true, Ordering::SeqCst);
        let mut handle = match self.save_handle.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if handle.is_some() {
            return;
        }
        let session = Arc::clone(self);
        let state = state.clone();
        *handle = Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(600)).await;
                if !session.dirty.swap(false, Ordering::SeqCst) {
                    break;
                }
                if let Err(e) = session.persist_now(&state).await {
                    log::warn!("sheet session persist failed: {e}");
                    session.dirty.store(true, Ordering::SeqCst);
                }
            }
        }));
    }

    /// Writes the current state to Drive; runs the xlsx save-back hook too.
    pub async fn persist_now(&self, state: &SheetState) -> Result<(), String> {
        let owner = {
            let sheet = self.sheet.read().await;
            sheet.owner_id.clone()
        };
        let snapshot = self.sheet.read().await.clone();
        save_sheet_to_drive(state, &owner, &snapshot).await
    }

    /// Returns the ops recorded since `since_seq` (exclusive). A client that
    /// reconnects asks for every op after the last one it applied, making the
    /// oplog the recovery mechanism for collab sessions (#789, #791).
    pub fn ops_since(&self, since_seq: u64) -> Vec<SheetOp> {
        let log = match self.oplog.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        log.iter().filter(|op| op.seq > since_seq).cloned().collect()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

/// Registry of live document sessions, keyed by owner + sheet id (#2127).
///
/// The key is composite on purpose: a session holds the workbook snapshot and
/// the oplog, so handing the same `Arc` to a second user would leak both. With
/// `owner:sheet` a live document is only ever reachable by the user whose
/// namespace loaded it.
pub struct SessionStore {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, Arc<SheetSession>>>>,
    /// Handle set at mount time so the eviction loop can persist dirty sheets.
    state_handle: Arc<tokio::sync::Mutex<Option<Arc<SheetState>>>>,
}

/// Key of a live session: owner id plus sheet id.
fn session_key(user_id: &str, sheet_id: &str) -> String {
    format!("{user_id}:{sheet_id}")
}

/// True when `key` belongs to `sheet_id`, whatever owner it was loaded for.
fn key_targets_sheet(key: &str, sheet_id: &str) -> bool {
    match key.split_once(':') {
        Some((_, id)) => id == sheet_id,
        None => false,
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            state_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

impl Clone for SessionStore {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            state_handle: Arc::clone(&self.state_handle),
        }
    }
}

impl SessionStore {
    pub fn new() -> Self {
        let store = Self::default();
        store.spawn_eviction_loop();
        store
    }

    pub fn set_state_handle(&self, state: Arc<SheetState>) {
        if let Ok(mut handle) = self.state_handle.try_lock() {
            *handle = Some(state);
        }
    }

    fn spawn_eviction_loop(&self) {
        let sessions = Arc::clone(&self.sessions);
        let state_handle = Arc::clone(&self.state_handle);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let cutoff = Utc::now().timestamp() - 30 * 60;
                let expired = {
                    let map = sessions.read().await;
                    map.iter()
                        .filter(|(_, s)| s.last_access.load(Ordering::Relaxed) < cutoff)
                        .map(|(id, s)| (id.clone(), Arc::clone(s)))
                        .collect::<Vec<_>>()
                };
                let state = state_handle.lock().await.clone();
                for (id, session) in expired {
                    if let Some(state) = state.as_ref() {
                        if !session.is_dirty() {
                            continue;
                        }
                        if let Err(e) = session.persist_now(state).await {
                            log::warn!("evict persist failed for {id}: {e}");
                        }
                    }
                    sessions.write().await.remove(&id);
                    log::debug!("evicted idle sheet session {id}");
                }
            }
        });
    }

    /// Returns the live session for a sheet, loading it from Drive on first
    /// access. Access is denied when the user cannot read the sheet (#789).
    ///
    /// Sessions are per user (#2127): a cached session is only returned to the
    /// user id it was created for, so two users can never share the same
    /// workbook snapshot through the cache.
    pub async fn get_or_load(
        &self,
        state: &SheetState,
        user_id: &str,
        sheet_id: &str,
    ) -> Result<Arc<SheetSession>, String> {
        let key = session_key(user_id, sheet_id);
        {
            let map = self.sessions.read().await;
            if let Some(session) = map.get(&key) {
                session.touch();
                return Ok(Arc::clone(session));
            }
        }
        let sheet = super::state::load_sheet_by_id(state, user_id, sheet_id).await?;
        if !super::state::can_read_sheet(user_id, &sheet) {
            return Err("Access denied".to_string());
        }
        let session = SheetSession::new(sheet);
        let mut map = self.sessions.write().await;
        if let Some(existing) = map.get(&key) {
            existing.touch();
            return Ok(Arc::clone(existing));
        }
        map.insert(key, Arc::clone(&session));
        Ok(session)
    }

    /// Records an operation on a session and schedules the debounced save.
    pub fn record(
        &self,
        session: &Arc<SheetSession>,
        state: &SheetState,
        user_id: &str,
        kind: &str,
        payload: serde_json::Value,
    ) -> u64 {
        let seq = session.record_op(user_id, kind, payload);
        session.touch();
        session.schedule_save(state);
        seq
    }

    /// Drops every live session of a sheet immediately, persisting the dirty
    /// ones (used on sheet delete). All owners are closed because deleting the
    /// workbook invalidates each user's snapshot of it (#2127).
    pub async fn close(&self, state: &SheetState, sheet_id: &str) {
        let removed: Vec<Arc<SheetSession>> = {
            let mut map = self.sessions.write().await;
            let keys: Vec<String> = map
                .keys()
                .filter(|key| key_targets_sheet(key.as_str(), sheet_id))
                .cloned()
                .collect();
            keys.into_iter().filter_map(|key| map.remove(&key)).collect()
        };
        for session in removed {
            if session.is_dirty() {
                let _ = session.persist_now(state).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{save_sheet_to_drive, DriveOps};
    use crate::types::{Spreadsheet, Worksheet};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemDrive(Mutex<HashMap<String, Vec<u8>>>);

    #[async_trait::async_trait]
    impl DriveOps for MemDrive {
        async fn put_object(
            &self,
            _bucket: &str,
            key: &str,
            body: Vec<u8>,
            _content_type: &str,
        ) -> Result<(), String> {
            self.0
                .lock()
                .map_err(|_| "poisoned".to_string())?
                .insert(key.to_string(), body);
            Ok(())
        }

        async fn get_object(&self, _bucket: &str, key: &str) -> Result<Vec<u8>, String> {
            self.0
                .lock()
                .map_err(|_| "poisoned".to_string())?
                .get(key)
                .cloned()
                .ok_or_else(|| format!("404 {key}"))
        }

        async fn list_objects(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
            let map = self.0.lock().map_err(|_| "poisoned".to_string())?;
            Ok(map
                .keys()
                .filter(|key| key.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn delete_object(&self, _bucket: &str, key: &str) -> Result<(), String> {
            self.0
                .lock()
                .map_err(|_| "poisoned".to_string())?
                .remove(key);
            Ok(())
        }
    }

    fn sheet(owner: &str) -> Spreadsheet {
        Spreadsheet {
            id: "s1".to_string(),
            name: "Report".to_string(),
            owner_id: owner.to_string(),
            worksheets: vec![Worksheet::default()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            named_ranges: None,
            external_links: None,
            source_bucket: None,
            source_path: None,
            source_bytes: None,
            acl: HashMap::new(),
        }
    }

    fn state() -> SheetState {
        SheetState {
            drive: Some(Arc::new(MemDrive::default())),
            on_save: None,
            sessions: SessionStore::default(),
        }
    }

    #[tokio::test]
    async fn a_cached_session_is_never_handed_to_another_user() {
        let state = state();
        save_sheet_to_drive(&state, "u-alice", &sheet("u-alice"))
            .await
            .expect("alice's sheet is persisted");

        let first = state
            .sessions
            .get_or_load(&state, "u-alice", "s1")
            .await
            .expect("alice loads her sheet");
        let again = state
            .sessions
            .get_or_load(&state, "u-alice", "s1")
            .await
            .expect("alice reuses her live session");
        assert!(
            Arc::ptr_eq(&first, &again),
            "the owner must keep working on the same live document"
        );

        let stolen = state.sessions.get_or_load(&state, "u-bob", "s1").await;
        assert!(
            stolen.is_err(),
            "bob must never receive alice's live workbook session (#2127)"
        );
    }

    #[tokio::test]
    async fn closing_a_sheet_drops_its_live_session() {
        let state = state();
        save_sheet_to_drive(&state, "u-alice", &sheet("u-alice"))
            .await
            .expect("sheet persisted");

        let live = state
            .sessions
            .get_or_load(&state, "u-alice", "s1")
            .await
            .expect("loaded");
        live.sheet.write().await.name = "Edited in memory".to_string();

        state.sessions.close(&state, "s1").await;

        let reloaded = state
            .sessions
            .get_or_load(&state, "u-alice", "s1")
            .await
            .expect("reloaded from drive");
        assert_eq!(
            reloaded.sheet.read().await.name,
            "Report",
            "close must drop the session so the next load reads Drive again"
        );
    }
}
