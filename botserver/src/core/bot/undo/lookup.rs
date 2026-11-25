//! Verified-scope lookup for the chat-native "undo my last batch" verb (#2448).
//!
//! The caller passes an [`UndoScope`] derived from authenticated session claims
//! only. These helpers take no prompt-supplied id, so a model (or a prompt
//! injection) cannot name a user, bot, session or batch: the only batch that can
//! ever be resolved is the newest active one owned by the verified scope.

use super::{BatchKey, RevertOperation, UndoError, UndoScope, UndoStore};

/// Resolves the newest active batch key owned by `scope`.
///
/// `UndoStore::latest` filters on the exact `(user, bot, session)` triple, so a
/// foreign session never even sees a candidate batch. The scope is validated
/// before the lookup, rejecting nil identities and empty/oversized sessions.
pub fn latest_batch_key<S: UndoStore>(
    store: &S,
    scope: &UndoScope,
) -> Result<BatchKey, UndoError> {
    scope.validate()?;
    let batch = store.latest(scope)?.ok_or(UndoError::NotFound)?;
    Ok(batch.key)
}

/// The resolved target of a chat-native undo, ready for the shared revert path.
#[derive(Debug, Clone, PartialEq)]
pub struct LatestUndo {
    pub key: BatchKey,
    pub revert_plan: Vec<RevertOperation>,
}

/// Consumes the verified scope's newest active batch exactly once and returns
/// its LIFO revert plan. The `after` state is diagnostic and is never carried
/// into the plan, so restore can only target recorded `before` state.
pub fn consume_latest<S: UndoStore>(
    store: &S,
    scope: &UndoScope,
) -> Result<LatestUndo, UndoError> {
    let key = latest_batch_key(store, scope)?;
    let batch = store.consume(&key)?;
    Ok(LatestUndo {
        key,
        revert_plan: batch.revert_plan(),
    })
}
