//! Durable, session- and bot-scoped agent-batch undo data model (#2448 #4).
//!
//! A batch is keyed by `(user, bot, authenticated session, batch id)`, holds
//! only operations for which a concrete `before` state was recorded, and can be
//! consumed exactly once. Reverting is expressed as a deterministic, reversed
//! plan of `RevertOperation`s — the data model never guesses a before state
//! from an operation name or a consent request id.
//!
//! The production store ([`PgUndoStore`]) writes to the same `consent_audit`
//! table the consent gate uses, so undo shares its lifecycle and user index.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

mod lookup;
mod restore_plan;
pub mod store;

pub use lookup::{
    consume_after_restore, consume_latest, latest_batch_key, peek, peek_latest, LatestUndo,
};
pub use restore_plan::{
    apply_restore_plan, before_version_id, history_has_newer_restore, RestoreSink,
};
pub use store::{shared_pool_store, InMemoryUndoStore, PgUndoStore, UndoStore};

/// Bounds keep persisted records predictable and stop oversized ids from
/// turning into unbounded JSON blobs in the consent audit table.
pub const MAX_ID_CHARS: usize = 128;
pub const MAX_SESSION_CHARS: usize = 256;
pub const MAX_TARGET_CHARS: usize = 512;

/// Typed failures. Callers map these to sanitized client messages; no storage
/// detail or foreign scope is ever carried in the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UndoError {
    InvalidScope,
    InvalidBatch,
    NotReversible,
    NotFound,
    ScopeMismatch,
    AlreadyRecorded,
    AlreadyUndone,
    Storage,
}

impl std::fmt::Display for UndoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::InvalidScope => "invalid undo scope",
            Self::InvalidBatch => "invalid undo batch",
            Self::NotReversible => "operation has no recorded before-state",
            Self::NotFound => "no active undo batch",
            Self::ScopeMismatch => "undo batch belongs to another scope",
            Self::AlreadyRecorded => "undo batch already recorded",
            Self::AlreadyUndone => "undo batch already undone",
            Self::Storage => "undo store unavailable",
        };
        f.write_str(text)
    }
}

impl std::error::Error for UndoError {}

/// Identity of the authenticated actor and chat session that owns a batch.
/// `thread_id` is informational context, not part of the ownership check:
/// lookup deliberately works without it so a new turn can undo a prior batch.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UndoScope {
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

impl UndoScope {
    #[must_use]
    pub fn new(user_id: Uuid, bot_id: Uuid, session_id: &str) -> Self {
        Self {
            user_id,
            bot_id,
            session_id: session_id.to_string(),
            thread_id: None,
        }
    }

    /// Nil identities and empty session ids are rejected: an unauthenticated
    /// or cross-bot caller must never be able to address a stored batch.
    pub fn validate(&self) -> Result<(), UndoError> {
        if self.user_id.is_nil()
            || self.bot_id.is_nil()
            || self.session_id.trim().is_empty()
            || self.session_id.len() > MAX_SESSION_CHARS
        {
            return Err(UndoError::InvalidScope);
        }
        if let Some(thread) = &self.thread_id {
            if thread.trim().is_empty() || thread.len() > MAX_SESSION_CHARS {
                return Err(UndoError::InvalidScope);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        self.user_id == other.user_id
            && self.bot_id == other.bot_id
            && self.session_id == other.session_id
    }
}

/// Full identity of one reversible batch. `batch_id` must be supplied by the
/// caller (the agent run / op id); it is never derived from a consent request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BatchKey {
    pub batch_id: String,
    pub scope: UndoScope,
}

impl BatchKey {
    pub fn new(scope: UndoScope, batch_id: &str) -> Result<Self, UndoError> {
        scope.validate()?;
        if batch_id.trim().is_empty() || batch_id.len() > MAX_ID_CHARS {
            return Err(UndoError::InvalidBatch);
        }
        Ok(Self {
            batch_id: batch_id.to_string(),
            scope,
        })
    }

    /// Unambiguous in-memory key. `Debug` quotes and escapes string fields, so
    /// no delimiter can be injected through a session or batch id.
    #[must_use]
    pub fn storage_key(&self) -> String {
        format!(
            "{:?}",
            (
                &self.scope.user_id,
                &self.scope.bot_id,
                &self.scope.session_id,
                &self.batch_id
            )
        )
    }
}

/// One operation inside a batch. `before` is the recorded, restore-ready
/// state; `after` is diagnostic only and is never used to revert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UndoOperation {
    pub op_id: String,
    pub app_id: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
    pub before: Value,
}

impl UndoOperation {
    pub fn new(op_id: &str, app_id: &str, target: &str, before: Value) -> Result<Self, UndoError> {
        let operation = Self {
            op_id: op_id.to_string(),
            app_id: app_id.to_string(),
            target: target.to_string(),
            after: None,
            before,
        };
        if !operation.is_reversible() {
            return Err(UndoError::NotReversible);
        }
        Ok(operation)
    }

    /// An operation is revertible only when it has a non-null before state and
    /// all identifiers needed to address the target are present and bounded.
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        !self.before.is_null()
            && !self.op_id.trim().is_empty()
            && self.op_id.len() <= MAX_ID_CHARS
            && !self.target.trim().is_empty()
            && self.target.len() <= MAX_TARGET_CHARS
            && self.app_id.len() <= MAX_ID_CHARS
    }

    pub fn with_after(mut self, after: Value) -> Self {
        self.after = Some(after);
        self
    }
}

/// A batch is all-or-nothing: a single operation without a before state makes
/// the whole batch unrecordable, so undo can never partially revert a run.
pub fn validate_operations(operations: &[UndoOperation]) -> Result<(), UndoError> {
    if operations.is_empty() {
        return Err(UndoError::InvalidBatch);
    }
    let mut seen = std::collections::HashSet::new();
    for operation in operations {
        if !operation.is_reversible() || !seen.insert(operation.op_id.as_str()) {
            return Err(UndoError::NotReversible);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Active,
    Undone,
}

/// Persisted agent batch. Only `Active` batches can be found or consumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UndoBatch {
    pub key: BatchKey,
    pub operations: Vec<UndoOperation>,
    pub created_at: DateTime<Utc>,
    pub status: BatchStatus,
}

impl UndoBatch {
    pub fn new(
        key: BatchKey,
        operations: Vec<UndoOperation>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, UndoError> {
        validate_operations(&operations)?;
        Ok(Self {
            key,
            operations,
            created_at,
            status: BatchStatus::Active,
        })
    }

    #[must_use]
    pub fn matches(&self, key: &BatchKey) -> bool {
        self.key == *key
    }

    /// Last-in-first-out restore plan. The executor maps `app_id`/`target` to
    /// its concrete verb; the model only guarantees the before state is real.
    #[must_use]
    pub fn revert_plan(&self) -> Vec<RevertOperation> {
        self.operations
            .iter()
            .rev()
            .map(|op| RevertOperation {
                op_id: op.op_id.clone(),
                app_id: op.app_id.clone(),
                target: op.target.clone(),
                before: op.before.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevertOperation {
    pub op_id: String,
    pub app_id: String,
    pub target: String,
    pub before: Value,
}

#[cfg(test)]
mod lookup_tests;
#[cfg(test)]
mod tests;
