//! The conversation folders kept in the person's own Drive area,
//! `Conversas/<title>-<id8>/` under `users/<id>/` (see
//! `core::bot::person_drive`), beside the older copies in a bot's
//! `conversations/`. The area is the caller's own; a folder is still only a
//! conversation folder when its `<id8>` names one of the caller's sessions.

use std::collections::BTreeMap;
use std::sync::Arc;

use botlib::traits::DriveRepository;
use serde_json::Value;
use uuid::Uuid;

use super::folders::{group_under, owner_of, validate_under, OwnedFolder, OwnedSession};
use crate::core::bot::person_drive::{self, CONVERSATIONS_FOLDER};

/// The folder of `path` when it names a file of a conversation folder in the
/// person's area (`Conversas/<folder>/<file…>`).
pub(super) fn folder_of(path: &str) -> Option<&str> {
    validate_under(path, CONVERSATIONS_FOLDER).ok()
}

fn all_sessions(by_bot: &BTreeMap<Uuid, Vec<OwnedSession>>) -> Vec<OwnedSession> {
    by_bot.values().flatten().cloned().collect()
}

/// Bucket and key of `path` (whose folder is `folder`) in the caller's area,
/// when the folder belongs to one of the caller's conversations.
pub(super) fn file_location(
    user_id: Uuid,
    folder: &str,
    path: &str,
    by_bot: &BTreeMap<Uuid, Vec<OwnedSession>>,
) -> Option<(String, String)> {
    owner_of(folder, &all_sessions(by_bot))?;
    Some((person_drive::bucket(user_id), person_drive::key(user_id, path)))
}

/// Groups keys listed under `listed` (`users/<id>/Conversas/`) into the
/// caller's conversation folders.
pub(super) fn folders_from(
    keys: Vec<(String, u64)>,
    listed: &str,
    by_bot: &BTreeMap<Uuid, Vec<OwnedSession>>,
) -> Vec<OwnedFolder> {
    let relative: Vec<(String, u64)> = keys
        .into_iter()
        .filter_map(|(key, size)| key.strip_prefix(listed).map(|rest| (rest.to_string(), size)))
        .collect();
    group_under(&relative, &all_sessions(by_bot), CONVERSATIONS_FOLDER)
}

/// The caller's conversation folders in their own area, each with the bucket
/// and the key prefix its file paths are relative to.
pub(super) async fn list(
    drive: &Arc<dyn DriveRepository>,
    user_id: Uuid,
    by_bot: &BTreeMap<Uuid, Vec<OwnedSession>>,
) -> Vec<(String, String, OwnedFolder)> {
    let bucket = person_drive::bucket(user_id);
    let key_prefix = person_drive::prefix(user_id);
    let listed = format!("{key_prefix}{CONVERSATIONS_FOLDER}/");
    let objects = match drive.list_objects_with_metadata(&bucket, Some(&listed)).await {
        Ok(objects) => objects,
        Err(error) => {
            // A person who never saved a file has no area yet.
            botlog::safe_debug!("agent drive files: no conversation folder in the person's area: {error}");
            return Vec::new();
        }
    };
    let keys = objects.into_iter().map(|object| (object.key, object.size)).collect();
    folders_from(keys, &listed, by_bot)
        .into_iter()
        .map(|folder| (bucket.clone(), key_prefix.clone(), folder))
        .collect()
}

fn conversation_of(entry: &Value) -> Option<&str> {
    entry.get("conversation_id").and_then(Value::as_str)
}

fn modified_of(file: &Value) -> Option<&str> {
    file.get("modified").and_then(Value::as_str)
}

/// One entry per conversation. A conversation with files both in the
/// person's `Conversas/` and in an older bot `conversations/` folder is listed
/// once, under the first entry's folder (the person's, listed first), with
/// the files of both; a name present in both is kept once, from the first.
/// The entries keep their order; merged files are newest first again.
pub(super) fn merge_same_conversation(entries: Vec<Value>) -> Vec<Value> {
    let mut merged: Vec<Value> = Vec::with_capacity(entries.len());
    for entry in entries {
        let known = conversation_of(&entry)
            .and_then(|id| merged.iter().position(|seen| conversation_of(seen) == Some(id)));
        let Some(index) = known else {
            merged.push(entry);
            continue;
        };
        let extra = entry.get("files").and_then(Value::as_array).cloned().unwrap_or_default();
        if let Some(files) = merged[index].get_mut("files").and_then(Value::as_array_mut) {
            for file in extra {
                if !files.iter().any(|kept| kept.get("name") == file.get("name")) {
                    files.push(file);
                }
            }
            files.sort_by(|left, right| modified_of(right).cmp(&modified_of(left)));
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use uuid::Uuid;

    use super::{file_location, folder_of, folders_from, merge_same_conversation};
    use crate::main_module::routes::agent_drive_files::folders::OwnedSession;

    fn sessions(id: &str) -> BTreeMap<Uuid, Vec<OwnedSession>> {
        let id = Uuid::parse_str(id).unwrap_or_default();
        let bot_id = Uuid::new_v4();
        let session = OwnedSession { id, bot_id, title: "Relatório mensal".into(), last_activity: 10 };
        BTreeMap::from([(bot_id, vec![session])])
    }

    #[test]
    fn the_persons_conversation_folders_are_listed_with_drive_paths() {
        let by_bot = sessions("1a2b3c4d-0000-4000-8000-000000000001");
        let listed = "users/u1/Conversas/";
        let keys = vec![
            ("users/u1/Conversas/Relatorio-1a2b3c4d/relatorio.pdf".to_string(), 2048),
            ("users/u1/Conversas/Alheia-9f9f9f9f/x.pdf".to_string(), 1),
            ("users/u1/Conversas/sem-id/y.pdf".to_string(), 1),
        ];
        let folders = folders_from(keys, listed, &by_bot);
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].folder, "Relatorio-1a2b3c4d");
        assert_eq!(folders[0].owner.title, "Relatório mensal");
        assert_eq!(folders[0].files[0].path, "Conversas/Relatorio-1a2b3c4d/relatorio.pdf");
        assert_eq!(folders[0].files[0].size, 2048);
    }

    #[test]
    fn only_the_callers_conversation_files_are_served_from_their_area() {
        let user = Uuid::parse_str("0b7c2a9e-1111-4222-8333-944455556666").unwrap_or_default();
        let by_bot = sessions("1a2b3c4d-0000-4000-8000-000000000001");
        let path = "Conversas/Relatorio-1a2b3c4d/relatorio.pdf";
        assert_eq!(folder_of(path), Some("Relatorio-1a2b3c4d"));
        assert_eq!(
            file_location(user, "Relatorio-1a2b3c4d", path, &by_bot),
            Some((
                "user-0b7c2a9e-1111-4222-8333-944455556666".to_string(),
                "users/0b7c2a9e-1111-4222-8333-944455556666/Conversas/Relatorio-1a2b3c4d/relatorio.pdf".to_string()
            ))
        );
        assert_eq!(file_location(user, "Alheia-9f9f9f9f", "Conversas/Alheia-9f9f9f9f/x.pdf", &by_bot), None);
        assert_eq!(folder_of("Conversas/../x.pdf"), None);
        assert_eq!(folder_of("conversations/Relatorio-1a2b3c4d/relatorio.pdf"), None);
    }

    #[test]
    fn old_and_new_folders_of_one_conversation_are_one_entry() {
        let entries = vec![
            json!({"folder": "Relatorio-1a2b3c4d", "conversation_id": "c1", "title": "R", "files": [
                {"name": "relatorio.pdf", "path": "Conversas/Relatorio-1a2b3c4d/relatorio.pdf", "size": 2, "modified": "2026-11-24T10:00:00+00:00"},
            ]}),
            json!({"folder": "Outra-2b3c4d5e", "conversation_id": "c2", "title": "O", "files": []}),
            json!({"folder": "Relatorio-1a2b3c4d", "conversation_id": "c1", "title": "R", "files": [
                {"name": "relatorio.pdf", "path": "conversations/Relatorio-1a2b3c4d/relatorio.pdf", "size": 1, "modified": "2026-11-20T10:00:00+00:00"},
                {"name": "dados.csv", "path": "conversations/Relatorio-1a2b3c4d/dados.csv", "size": 1, "modified": "2026-11-25T10:00:00+00:00"},
            ]}),
        ];
        let merged = merge_same_conversation(entries);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0]["conversation_id"], "c1");
        assert_eq!(merged[1]["conversation_id"], "c2");
        let files = merged[0]["files"].as_array().cloned().unwrap_or_default();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0]["path"], "conversations/Relatorio-1a2b3c4d/dados.csv");
        assert_eq!(files[1]["path"], "Conversas/Relatorio-1a2b3c4d/relatorio.pdf");
    }
}
