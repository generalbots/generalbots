//! Identity of the turn whose events are being emitted.

use std::sync::Arc;

use uuid::Uuid;

use botcore::shared::state::AppState;

use super::redactor::observe_persisted_seq;

/// Identity of the turn whose events are being emitted.
#[derive(Debug, Clone)]
pub struct RunContext {
    pub thread_id: String,
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub agent: String,
    pub bot_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    /// When present, every accepted frame is appended to `run_events` before
    /// it is sent. Chat turns set it; tests and transports that only stream
    /// leave it `None`.
    pub state: Option<Arc<AppState>>,
}

impl RunContext {
    #[must_use]
    pub fn new(thread_id: &str, agent: &str) -> Self {
        let run_id = crate::core::bot::runs::bus::bus()
            .current_run_id(thread_id)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        Self {
            thread_id: thread_id.to_string(),
            run_id,
            parent_run_id: None,
            agent: agent.to_string(),
            bot_id: None,
            user_id: None,
            org_id: None,
            state: None,
        }
    }

    /// Rebuilds a context from identifiers already persisted (resume path).
    #[must_use]
    pub fn resumed(thread_id: &str, run_id: &str, agent: &str) -> Self {
        Self {
            thread_id: thread_id.to_string(),
            run_id: run_id.to_string(),
            parent_run_id: None,
            agent: agent.to_string(),
            bot_id: None,
            user_id: None,
            org_id: None,
            state: None,
        }
    }

    #[must_use]
    pub fn with_scope(mut self, bot_id: Uuid, user_id: Uuid, org_id: Option<Uuid>) -> Self {
        self.bot_id = Some(bot_id);
        self.user_id = Some(user_id);
        self.org_id = org_id;
        self.seed_sequencer();
        self
    }

    #[must_use]
    pub fn with_state(mut self, state: Arc<AppState>) -> Self {
        self.state = Some(state);
        self.seed_sequencer();
        self
    }

    /// After a restart the in-process counter starts at 1 while the thread may
    /// already have persisted events. Seed from the log so the next minted
    /// sequence cannot collide with `(thread_id, seq)`.
    fn seed_sequencer(&self) {
        let (Some(state), Some(bot_id)) = (self.state.as_ref(), self.bot_id) else {
            return;
        };
        let Ok(mut conn) = state.conn.get() else {
            return;
        };
        match crate::core::bot::runs::store::last_seq(&mut conn, bot_id, &self.thread_id) {
            Ok(last) if last > 0 => observe_persisted_seq(&self.thread_id, last),
            Ok(_) => {}
            Err(error) => log::warn!("run event sequencer seed failed: {error}"),
        }
    }

    #[must_use]
    pub fn with_parent(mut self, parent_run_id: Option<String>) -> Self {
        self.parent_run_id = parent_run_id;
        self
    }
}
