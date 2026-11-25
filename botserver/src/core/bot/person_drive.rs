//! The person's own Drive area, the one Drive lists as "Meus arquivos"
//! (`scope=user`): bucket `user-<id>` (`botdrive::user_scope::get_user_bucket`)
//! with keys under `users/<id>/` (`resolve_key_prefix`).
//!
//! Files a conversation produces are kept there as the one copy, in
//! `Conversas/<title>-<id8>/<name>`: the person renames, moves and deletes
//! them like any other file of theirs, and they count in the person's space.
//! `<id8>` (the first eight hex digits of the session id) ties a folder to the
//! conversation, the same way the older bot-area folders were named.

use std::sync::Arc;

use botcore::shared::state::AppState;
use botdrive::drive_types::FileScope;
use botdrive::user_scope;
use uuid::Uuid;

use super::drive_bridge;

/// Top folder of the conversation files in the person's Drive.
pub const CONVERSATIONS_FOLDER: &str = "Conversas";

/// Bucket of the person's area.
#[must_use]
pub fn bucket(user_id: Uuid) -> String {
    user_scope::get_user_bucket(&user_id.to_string())
}

/// Key prefix of the person's area inside [`bucket`]: `users/<id>/`.
#[must_use]
pub fn prefix(user_id: Uuid) -> String {
    user_scope::resolve_key_prefix(&FileScope::User, &user_id.to_string())
}

/// Object key of `path`, a path as the person's Drive shows it.
#[must_use]
pub fn key(user_id: Uuid, path: &str) -> String {
    user_scope::build_object_key(&FileScope::User, &user_id.to_string(), path)
}

/// `Conversas/<folder>/<name>`: where a conversation's file lives in the
/// person's Drive, and the `drive_path` the chat receives.
#[must_use]
pub fn conversation_path(folder: &str, name: &str) -> String {
    format!("{CONVERSATIONS_FOLDER}/{folder}/{name}")
}

/// Writes a conversation's file into the person's area and returns its
/// [`conversation_path`]; `None` when Drive is unavailable or the write fails.
/// The key is deterministic, so saving the same file again overwrites it.
pub async fn save_conversation_file(
    state: &Arc<AppState>,
    user_id: Uuid,
    folder: &str,
    name: &str,
    bytes: Vec<u8>,
) -> Option<String> {
    let drive = state.drive.clone()?;
    let bucket = bucket(user_id);
    let path = conversation_path(folder, name);
    let key = key(user_id, &path);
    // A person who never opened Drive has no bucket yet.
    if let Err(error) = drive.create_bucket_if_not_exists(&bucket).await {
        botlog::safe_warn!("person drive: preparing the person's area failed: {error}");
    }
    match drive.put_object(&bucket, &key, bytes, Some(drive_bridge::content_type(name))).await {
        Ok(()) => Some(path),
        Err(error) => {
            botlog::safe_warn!("person drive: saving a conversation file failed: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bucket, conversation_path, key, prefix};
    use uuid::Uuid;

    #[test]
    fn a_conversation_file_lands_in_the_persons_own_area() {
        let user = Uuid::parse_str("0b7c2a9e-1111-4222-8333-944455556666").unwrap_or_default();
        let path = conversation_path("Relatório-1a2b3c4d", "relatorio.pdf");
        assert_eq!(path, "Conversas/Relatório-1a2b3c4d/relatorio.pdf");
        assert_eq!(bucket(user), "user-0b7c2a9e-1111-4222-8333-944455556666");
        assert_eq!(prefix(user), "users/0b7c2a9e-1111-4222-8333-944455556666/");
        assert_eq!(
            key(user, &path),
            "users/0b7c2a9e-1111-4222-8333-944455556666/Conversas/Relatório-1a2b3c4d/relatorio.pdf"
        );
    }
}
