//! Which conversation folders of a bot's Drive belong to the caller, and
//! which paths the file endpoint accepts. Pure: no database, no Drive.
//!
//! A conversation folder is `conversations/<slug>-<id8>/` (`<id8>` is the
//! first eight hex digits of the session id). Older folders carry the slug
//! of the conversation title only; they are never matched, since a title
//! can be shared by several people's conversations. Only the caller's
//! own sessions are ever compared, so a folder no session of the
//! caller explains is never listed or served.

use std::collections::HashMap;

use uuid::Uuid;


/// Top folder of the conversation copies inside `{bot}.gbdrive/`.
pub const CONVERSATIONS: &str = "conversations";

/// One session of the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedSession {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    /// Last activity, epoch seconds.
    pub last_activity: i64,
}

/// The caller's session a folder belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderOwner {
    /// `None` when several of the caller's conversations share the title of
    /// an old title-only folder.
    pub conversation_id: Option<Uuid>,
    pub title: String,
    pub last_activity: i64,
}

/// One file of a conversation folder, before its modification time is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderFile {
    pub name: String,
    /// Path relative to `{bot}.gbdrive/`.
    pub path: String,
    pub size: u64,
}

/// A conversation folder of the caller with its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedFolder {
    pub folder: String,
    pub owner: FolderOwner,
    pub files: Vec<FolderFile>,
}

fn id8(folder: &str) -> Option<&str> {
    let (_, suffix) = folder.rsplit_once('-')?;
    let hex = suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
    hex.then_some(suffix)
}

/// The caller's session `folder` belongs to, among `sessions` (the caller's
/// sessions of the bot whose Drive holds the folder).
#[must_use]
pub fn owner_of(folder: &str, sessions: &[OwnedSession]) -> Option<FolderOwner> {
    if let Some(short) = id8(folder) {
        let found = sessions
            .iter()
            .filter(|session| session.id.to_string().starts_with(short))
            .max_by_key(|session| session.last_activity);
        if let Some(session) = found {
            return Some(FolderOwner {
                conversation_id: Some(session.id),
                title: display_title(&session.title, folder),
                last_activity: session.last_activity,
            });
        }
    }
    // Old title-only folders are not matched: on a bot shared by several
    // people, another person's conversation with the same title would be
    // listed and served to the caller. Only the conversation id proves
    // ownership.
    None
}

fn display_title(title: &str, folder: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        folder.to_string()
    } else {
        title.to_string()
    }
}

/// Groups object keys relative to `conversations/` (`<folder>/<file…>`) into
/// the caller's folders, newest conversation first. Keys of folders the
/// caller does not own are dropped.
#[must_use]
pub fn group(objects: &[(String, u64)], sessions: &[OwnedSession]) -> Vec<OwnedFolder> {
    let mut folders: HashMap<String, OwnedFolder> = HashMap::new();
    let mut strangers: Vec<String> = Vec::new();
    for (relative, size) in objects {
        let Some((folder, rest)) = relative.split_once('/') else {
            continue;
        };
        let name = rest.rsplit('/').next().unwrap_or("");
        if folder.is_empty() || name.is_empty() || strangers.iter().any(|known| known == folder) {
            continue;
        }
        if !folders.contains_key(folder) {
            match owner_of(folder, sessions) {
                Some(owner) => {
                    folders.insert(
                        folder.to_string(),
                        OwnedFolder { folder: folder.to_string(), owner, files: Vec::new() },
                    );
                }
                None => {
                    strangers.push(folder.to_string());
                    continue;
                }
            }
        }
        if let Some(entry) = folders.get_mut(folder) {
            entry.files.push(FolderFile {
                name: name.to_string(),
                path: format!("{CONVERSATIONS}/{relative}"),
                size: *size,
            });
        }
    }
    let mut out: Vec<OwnedFolder> = folders.into_values().collect();
    out.sort_by(|left, right| {
        right.owner.last_activity.cmp(&left.owner.last_activity).then_with(|| left.folder.cmp(&right.folder))
    });
    out
}

/// Why a requested path was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathRefusal {
    Empty,
    Absolute,
    Traversal,
    OutsideConversations,
}

/// A path relative to `{bot}.gbdrive/` that names a file of a conversation
/// folder: `conversations/<folder>/<file…>`, no `.`/`..` or empty segment,
/// no drive letter, backslash or control character. Returns the folder.
pub fn validate_path(raw: &str) -> Result<&str, PathRefusal> {
    if raw.trim().is_empty() {
        return Err(PathRefusal::Empty);
    }
    if raw.starts_with('/') || raw.starts_with('\\') || raw.contains(':') {
        return Err(PathRefusal::Absolute);
    }
    if raw.contains('\\') || raw.chars().any(char::is_control) {
        return Err(PathRefusal::Traversal);
    }
    let segments: Vec<&str> = raw.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty() || *segment == "." || *segment == "..") {
        return Err(PathRefusal::Traversal);
    }
    if segments.len() < 3 || segments[0] != CONVERSATIONS {
        return Err(PathRefusal::OutsideConversations);
    }
    Ok(segments[1])
}

#[cfg(test)]
#[path = "folders_tests.rs"]
mod tests;
