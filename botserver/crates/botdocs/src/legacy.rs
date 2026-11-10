//! Legacy shared Docs namespace (#2135).
//!
//! Before #2135 every document was stored under `users/default-user/docs`
//! because `get_current_user_id()` returned that constant. The prefix was
//! therefore shared by every account that could reach the bot bucket: any
//! signed-in user could list, read, edit, export and delete every other user's
//! documents.
//!
//! Ownership now comes from the verified identity
//! (`users/{verified-user}/docs`, see [`crate::auth::DocUser`]), so nothing in
//! this crate writes to or reads from the legacy prefix any more. The objects
//! that are already there are **not** migrated automatically: the prefix records
//! no owner, so assigning them to any account would hand one user's documents
//! to another. They stay orphaned, and [`audit_legacy_namespace`] reports them
//! once at boot so an operator can move them by hand into
//! `users/{user-id}/docs`.

use crate::state::DocState;

/// The shared owner id every document used before #2135.
///
/// Kept as the single place the literal exists: the rest of the crate refers to
/// it through [`is_legacy_owner`], which is what the regression test in this
/// module enforces.
pub const LEGACY_OWNER: &str = "default-user";

/// Whether `identifier` names the legacy shared namespace.
///
/// The comparison ignores case and surrounding whitespace because the storage
/// path lowercases the owner segment, so `Default-User` would land in the same
/// prefix.
pub fn is_legacy_owner(identifier: &str) -> bool {
    identifier.trim().to_ascii_lowercase() == LEGACY_OWNER
}

/// Drive prefix holding the orphaned pre-#2135 documents.
pub fn legacy_docs_prefix() -> String {
    format!("users/{LEGACY_OWNER}/docs/")
}

/// Counts the orphaned legacy objects and logs them once at boot.
///
/// Returns the number of objects found; `0` when the prefix is empty or the
/// listing failed (the failure is logged, never propagated: an audit must not
/// stop the server from starting).
pub async fn audit_legacy_namespace(state: &DocState) -> usize {
    let prefix = legacy_docs_prefix();
    match state
        .drive
        .list_objects(&state.bucket_name, Some(&prefix))
        .await
    {
        Ok(objects) => {
            if objects.is_empty() {
                return 0;
            }
            log::warn!(
                "docs: {} object(s) under the legacy shared namespace {} in bucket {} are orphaned. \
                 No owner can be derived from that prefix, so they are neither migrated nor served. \
                 Move each object into users/{{user-id}}/docs to restore access.",
                objects.len(),
                prefix,
                state.bucket_name
            );
            objects.len()
        }
        Err(error) => {
            log::warn!("docs: legacy namespace audit could not list {prefix}: {error}");
            0
        }
    }
}

#[cfg(test)]
#[path = "legacy_tests.rs"]
mod tests;
