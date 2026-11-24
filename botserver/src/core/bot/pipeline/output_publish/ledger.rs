//! What a turn may publish: the limits, the published-file ledger (one per
//! conversation, in process memory) and the choice of new or changed files.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use uuid::Uuid;

/// Most artifacts one turn publishes. A program that writes a hundred files
/// must not turn the chat into an upload queue.
pub const MAX_PUBLISH_FILES: usize = 5;
/// Largest single artifact published without asking (25 MB).
pub const MAX_PUBLISH_FILE_BYTES: u64 = 25 * 1024 * 1024;
/// Largest total published in one turn (50 MB).
pub const MAX_PUBLISH_TURN_BYTES: u64 = 50 * 1024 * 1024;
/// Conversations whose ledger is kept before one is evicted to bound memory.
const MAX_LEDGER_SESSIONS: usize = 512;

/// What the ledger remembers about a published artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedFile {
    pub size: u64,
    pub modified: u64,
    pub digest: u64,
}

/// One file found in the conversation's `output/` folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputFile {
    pub name: String,
    pub size: u64,
    /// Modification time in epoch milliseconds, `0` when unknown.
    pub modified: u64,
}

/// One artifact that reached Drive in this turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedArtifact {
    /// File name inside the conversation's `output/`.
    pub name: String,
    /// `bucket/key` returned by the bridge.
    pub drive_key: String,
    /// Markdown link attached to the turn.
    pub link: String,
}

impl PublishedArtifact {
    pub fn new(name: impl Into<String>, drive_key: impl Into<String>, conversation: Uuid) -> Self {
        let name = name.into();
        let drive_key = drive_key.into();
        let link = output_link(&name, conversation);
        Self {
            name,
            drive_key,
            link,
        }
    }

    /// File name at the end of the published key. Must equal [`Self::name`]:
    /// the link the user clicks and the object in Drive are the same file.
    pub fn drive_name(&self) -> &str {
        self.drive_key.rsplit('/').next().unwrap_or("")
    }
}

/// Outcome of one turn's publishing attempt.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublishReport {
    pub folder: String,
    pub published: Vec<PublishedArtifact>,
    /// Links to new files that stay in the conversation, not in Drive.
    pub unsaved: Vec<String>,
}

/// Chat link to a file of the conversation's `output/`.
#[must_use]
pub fn output_link(name: &str, conversation: Uuid) -> String {
    format!("[{name}](vmfile:{conversation}/output/{name})")
}

impl PublishReport {
    pub fn is_empty(&self) -> bool {
        self.published.is_empty() && self.unsaved.is_empty()
    }

    /// Chat message the person reads (pt-BR). Empty when nothing was
    /// published or held back, so the caller never sends an empty bubble.
    pub fn message(&self) -> String {
        let mut message = String::new();
        if !self.published.is_empty() {
            let folder = self.folder.as_str();
            message.push_str(&format!("**Arquivos salvos no seu Drive** (`conversations/{folder}/`)\n"));
            for artifact in &self.published {
                let link = artifact.link.as_str();
                message.push_str(&format!("- {link}\n"));
            }
        }
        if !self.unsaved.is_empty() {
            if !message.is_empty() {
                message.push('\n');
            }
            message.push_str(
                "**Arquivos desta conversa, não salvos no Drive** (use «Salvar no Drive» no painel de arquivos)\n",
            );
            for link in &self.unsaved {
                message.push_str(&format!("- {link}\n"));
            }
        }
        message.trim_end().to_string()
    }
}

/// Cheap content fingerprint (FNV-1a 64). Detects a rewrite that kept the
/// size and the millisecond timestamp, which the listing alone cannot.
pub fn digest(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn ledger() -> &'static Mutex<HashMap<Uuid, HashMap<String, PublishedFile>>> {
    static LEDGER: OnceLock<Mutex<HashMap<Uuid, HashMap<String, PublishedFile>>>> = OnceLock::new();
    LEDGER.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn snapshot(session_id: Uuid) -> HashMap<String, PublishedFile> {
    match ledger().lock() {
        Ok(map) => map.get(&session_id).cloned().unwrap_or_default(),
        Err(error) => {
            botlog::safe_warn!("output publish: ledger lock failed: {error}");
            HashMap::new()
        }
    }
}

pub(super) fn store(session_id: Uuid, entries: HashMap<String, PublishedFile>) {
    let Ok(mut map) = ledger().lock() else {
        botlog::safe_warn!("output publish: ledger lock failed on write");
        return;
    };
    if !map.contains_key(&session_id) && map.len() >= MAX_LEDGER_SESSIONS {
        // Bounded memory: the map has no ordering, so any one conversation is
        // dropped. A dropped conversation re-publishes once; the deterministic
        // Drive key makes that an overwrite, not a duplicate.
        if let Some(victim) = map.keys().next().copied() {
            map.remove(&victim);
        }
    }
    map.insert(session_id, entries);
}

/// True when the listing stamp is the one the ledger already published.
pub fn is_unchanged(
    seen: &HashMap<String, PublishedFile>,
    name: &str,
    size: u64,
    modified: u64,
) -> bool {
    match seen.get(name) {
        Some(published) => published.size == size && published.modified == modified,
        None => false,
    }
}

/// New or changed files, deterministic order, bounded by [`MAX_PUBLISH_FILES`].
pub fn candidates(files: &[OutputFile], seen: &HashMap<String, PublishedFile>) -> Vec<OutputFile> {
    let mut selected: Vec<OutputFile> = files
        .iter()
        .filter(|file| file.size <= MAX_PUBLISH_FILE_BYTES)
        .filter(|file| !is_unchanged(seen, &file.name, file.size, file.modified))
        .cloned()
        .collect();
    selected.sort_by(|left, right| left.name.cmp(&right.name));
    selected.truncate(MAX_PUBLISH_FILES);
    selected
}
