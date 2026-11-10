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
//! this crate writes to or reads from the legacy prefix any more.
//!
//! What happens to the objects already there depends on whether an owner can be
//! derived:
//!
//! * **Multi-user deployment** — the prefix records no owner, so assigning it to
//!   any account would hand one user's documents to another. The objects stay
//!   orphaned and [`audit_legacy_namespace`] reports them at boot.
//! * **Single-user install** (`no_security`, desktop edition) — the install
//!   owner is provably the only account that could have written them, so
//!   [`adopt_legacy_namespace`] copies them into that owner's namespace. The
//!   legacy objects are copied, never deleted, so nothing is lost if the guess
//!   is wrong.

use crate::state::DocState;
use crate::storage_core::get_user_docs_path;

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

/// Copies the legacy objects into `owner`'s namespace.
///
/// Only safe for a single-user install, where `owner` is the install owner and
/// therefore the only account that could have written the legacy objects. The
/// source objects are left in place: an adoption that guessed wrong must be
/// recoverable, and Drive copies server-side, so the cost is one copy per object
/// once.
///
/// Returns the number of objects copied; objects already present in the
/// destination are skipped, so repeated boots converge instead of duplicating.
pub async fn adopt_legacy_namespace(state: &DocState, owner: &str) -> usize {
    let prefix = legacy_docs_prefix();
    let objects = match state
        .drive
        .list_objects(&state.bucket_name, Some(&prefix))
        .await
    {
        Ok(objects) => objects,
        Err(error) => {
            log::warn!("docs: legacy namespace adoption could not list {prefix}: {error}");
            return 0;
        }
    };

    let mut adopted = 0;
    for object in objects {
        let Some(destination) = adopt_destination(owner, &object) else {
            continue;
        };
        if matches!(
            state
                .drive
                .object_exists(&state.bucket_name, &destination)
                .await,
            Ok(true)
        ) {
            continue;
        }
        match state
            .drive
            .copy_object(&state.bucket_name, &object, &destination)
            .await
        {
            Ok(()) => adopted += 1,
            Err(error) => log::warn!("docs: legacy object {object} could not be adopted: {error}"),
        }
    }

    if adopted > 0 {
        log::info!(
            "docs: adopted {adopted} legacy object(s) into {}/ in bucket {}",
            get_user_docs_path(owner),
            state.bucket_name
        );
    }
    adopted
}

/// Destination of a legacy object inside `owner`'s namespace.
///
/// `None` for an object outside the legacy prefix or for the prefix itself, so
/// the caller never writes outside the owner's folder.
pub fn adopt_destination(owner: &str, object: &str) -> Option<String> {
    let name = object.strip_prefix(&legacy_docs_prefix())?;
    if name.trim().is_empty() || name.ends_with('/') {
        return None;
    }
    Some(format!("{}/{name}", get_user_docs_path(owner)))
}

#[cfg(test)]
#[path = "legacy_tests.rs"]
mod tests;
