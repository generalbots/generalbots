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

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

/// Registry of live document sessions, keyed by sheet id.
pub struct SessionStore {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, Arc<SheetSession>>>>,
    /// Handle set at mount time so the eviction loop can persist dirty sheets.
    state_handle: Arc<tokio::sync::Mutex<Option<Arc<SheetState>>>>,
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
    pub async fn get_or_load(
        &self,
        state: &SheetState,
        user_id: &str,
        sheet_id: &str,
    ) -> Result<Arc<SheetSession>, String> {
        {
            let map = self.sessions.read().await;
            if let Some(session) = map.get(sheet_id) {
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
        if let Some(existing) = map.get(sheet_id) {
            existing.touch();
            return Ok(Arc::clone(existing));
        }
        map.insert(sheet_id.to_string(), Arc::clone(&session));
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

    /// Drops a session immediately, persisting if dirty (used on sheet delete).
    pub async fn close(&self, state: &SheetState, sheet_id: &str) {
        if let Some(session) = self.sessions.write().await.remove(sheet_id) {
            if session.is_dirty() {
                let _ = session.persist_now(state).await;
            }
        }
    }
}
