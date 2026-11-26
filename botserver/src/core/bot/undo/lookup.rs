//! Verified-scope lookup for the chat-native "undo my last batch" verb (#2448).
//!
//! The caller passes an [`UndoScope`] derived from authenticated session claims
//! only. These helpers take no prompt-supplied id, so a model (or a prompt
//! injection) cannot name a user, bot, session or batch: the only batch that can
//! ever be resolved is the newest active one owned by the verified scope.

use super::{BatchKey, RevertOperation, UndoBatch, UndoError, UndoScope, UndoStore};

/// Resolves the newest active batch key owned by `scope`.
///
/// `UndoStore::latest` filters on the exact `(user, bot, session)` triple, so a
/// foreign session never even sees a candidate batch. The scope is validated
/// before the lookup, rejecting nil identities and empty/oversized sessions.
pub fn latest_batch_key<S: UndoStore>(store: &S, scope: &UndoScope) -> Result<BatchKey, UndoError> {
    scope.validate()?;
    let batch = store.latest(scope)?.ok_or(UndoError::NotFound)?;
    Ok(batch.key)
}

/// Read-only counterpart of `UndoStore::consume`: returns the active batch
/// without changing its status. A failed restore can therefore leave the batch
/// active and retryable instead of losing it.
pub fn peek<S: UndoStore>(store: &S, key: &BatchKey) -> Result<UndoBatch, UndoError> {
    store.find(key)?.ok_or(UndoError::NotFound)
}

/// Resolves the newest active batch of `scope` without consuming it.
///
/// The scope is checked before the lookup, so a foreign session never sees a
/// candidate; the returned key still goes through the store's ownership check
/// when it is finally consumed.
pub fn peek_latest<S: UndoStore>(store: &S, scope: &UndoScope) -> Result<LatestUndo, UndoError> {
    let key = latest_batch_key(store, scope)?;
    let batch = peek(store, &key)?;
    Ok(LatestUndo {
        key,
        revert_plan: batch.revert_plan(),
    })
}

/// Consumes a batch only after `restore` has succeeded.
///
/// `restore` is evaluated first: any failure is returned unchanged and the
/// batch stays active for a retry. The store's conditional consume keeps the
/// single-use guarantee once restore succeeded. Callers map the store error to
/// their own HTTP/error type through `map_store_error`.
pub fn consume_after_restore<S, E, M>(
    store: &S,
    key: &BatchKey,
    restore: Result<Vec<String>, E>,
    map_store_error: M,
) -> Result<Vec<String>, E>
where
    S: UndoStore,
    M: FnOnce(UndoError) -> E,
{
    let restored = restore?;
    store.consume(key).map_err(map_store_error)?;
    Ok(restored)
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
pub fn consume_latest<S: UndoStore>(store: &S, scope: &UndoScope) -> Result<LatestUndo, UndoError> {
    let key = latest_batch_key(store, scope)?;
    let batch = store.consume(&key)?;
    Ok(LatestUndo {
        key,
        revert_plan: batch.revert_plan(),
    })
}
