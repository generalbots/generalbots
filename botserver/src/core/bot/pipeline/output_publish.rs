//! Turn-end publishing of workspace artifacts (#2143 gap D).
//!
//! Before this module, a file the agent produced in the VM's `output/` folder
//! only reached Drive when the user clicked Save/Open or the model remembered
//! to call `drive_save`. Closing the conversation threw the artifact away.
//!
//! At the end of a turn that produced files, this module publishes the new or
//! changed ones through [`drive_bridge::save_object`] (the same bridge the
//! Save button uses) and attaches the resulting `vmfile:` link to the turn, so
//! the chat shows the attachment card for a file that now outlives the
//! session folder.
//!
//! Rules:
//! - **Bounded.** At most [`MAX_PUBLISH_FILES`] files, [`MAX_PUBLISH_FILE_BYTES`]
//!   per file and [`MAX_PUBLISH_TURN_BYTES`] per turn.
//! - **Idempotent.** The ledger remembers `(size, modified, digest)` per
//!   conversation. A file whose listing stamp did not change is never read or
//!   uploaded again; a file that was only touched (`size`/`mtime` moved, same
//!   content digest) updates the stamp without a second upload.
//! - **Never fatal.** Drive being unconfigured or unreachable logs a warning
//!   and returns an empty report; the turn has already answered the user, and
//!   the next turn retries because the ledger entry was not advanced.
//!
//! The ledger lives in process memory. After a restart (or eviction) the next
//! turn republishes once; `drive_bridge::save_object` uses a deterministic key,
//! so that is an overwrite of the same object, never a duplicate.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use botcore::shared::state::AppState;
use uuid::Uuid;

use super::doc_delta;
use super::native_tools::{persist, send};
use super::types::SessionIdentity;
use crate::core::bot::{agent_vm_hook, drive_bridge};

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
        let link = format!("[{name}](vmfile:{conversation}/output/{name})");
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
}

impl PublishReport {
    pub fn is_empty(&self) -> bool {
        self.published.is_empty()
    }

    /// Chat message for the turn. Empty when nothing was published, so the
    /// caller never sends an empty bubble.
    pub fn message(&self) -> String {
        if self.published.is_empty() {
            return String::new();
        }
        let folder = self.folder.as_str();
        let mut message = format!("**Files saved to your Drive** (`conversations/{folder}/`)\n");
        for artifact in &self.published {
            let link = artifact.link.as_str();
            message.push_str(&format!("- {link}\n"));
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

fn snapshot(session_id: Uuid) -> HashMap<String, PublishedFile> {
    match ledger().lock() {
        Ok(map) => map.get(&session_id).cloned().unwrap_or_default(),
        Err(error) => {
            botlog::safe_warn!("output publish: ledger lock failed: {error}");
            HashMap::new()
        }
    }
}

fn store(session_id: Uuid, entries: HashMap<String, PublishedFile>) {
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

/// Workspace IO and Drive upload, split out so tests can drive the publisher
/// without a VM or an object store.
#[async_trait]
pub trait ArtifactPublisher: Send + Sync {
    async fn read(&self, name: &str) -> Result<Vec<u8>, String>;
    /// Returns `bucket/key`, or `None` when Drive is unavailable.
    async fn publish(&self, name: &str, bytes: Vec<u8>) -> Option<String>;
}

/// Publishes the candidates and advances the ledger. Returns what reached
/// Drive; failures are logged and left for the next turn.
pub async fn publish_candidates(
    conversation: Uuid,
    candidates: Vec<OutputFile>,
    seen: &mut HashMap<String, PublishedFile>,
    publisher: &dyn ArtifactPublisher,
) -> Vec<PublishedArtifact> {
    let mut published = Vec::new();
    let mut total: u64 = 0;
    for OutputFile {
        name,
        size,
        modified,
    } in candidates
    {
        if total.saturating_add(size) > MAX_PUBLISH_TURN_BYTES {
            botlog::safe_info!("output publish: turn byte budget reached, skipping {name}");
            continue;
        }
        let bytes = match publisher.read(&name).await {
            Ok(bytes) => bytes,
            Err(error) => {
                botlog::safe_warn!("output publish: reading {name} failed: {error}");
                continue;
            }
        };
        let actual = bytes.len() as u64;
        if actual > MAX_PUBLISH_FILE_BYTES {
            botlog::safe_info!("output publish: {name} is {actual} B, over the per-file limit");
            continue;
        }
        if total.saturating_add(actual) > MAX_PUBLISH_TURN_BYTES {
            botlog::safe_info!("output publish: turn byte budget reached, skipping {name}");
            continue;
        }
        let fingerprint = digest(&bytes);
        if let Some(previous) = seen.get(&name) {
            // Touched but byte-identical: keep the new stamp, skip the upload.
            if previous.digest == fingerprint && previous.size == actual {
                seen.insert(
                    name.clone(),
                    PublishedFile {
                        size: actual,
                        modified,
                        digest: fingerprint,
                    },
                );
                continue;
            }
        }
        match publisher.publish(&name, bytes).await {
            Some(key) => {
                let artifact = PublishedArtifact::new(name.as_str(), key, conversation);
                // The link and the object must be the same file; a bridge that
                // renamed the key would hand the user a dead link.
                if artifact.drive_name() != name.as_str() {
                    let key = artifact.drive_key.as_str();
                    botlog::safe_warn!(
                        "output publish: key {key} does not end in {name}; not linking"
                    );
                    continue;
                }
                total = total.saturating_add(actual);
                seen.insert(
                    name.clone(),
                    PublishedFile {
                        size: actual,
                        modified,
                        digest: fingerprint,
                    },
                );
                published.push(artifact);
            }
            None => {
                // Drive unavailable: do not advance the ledger, so the next
                // turn retries. The turn itself already succeeded.
                botlog::safe_warn!("output publish: Drive unavailable for {name}; will retry");
            }
        }
    }
    published
}

struct DrivePublisher {
    state: Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    folder: String,
}

#[async_trait]
impl ArtifactPublisher for DrivePublisher {
    async fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        let relative = format!("output/{name}");
        agent_vm_hook::read_session_file(self.user_id, self.session_id, &relative).await
    }

    async fn publish(&self, name: &str, bytes: Vec<u8>) -> Option<String> {
        drive_bridge::save_object(&self.state, &self.bot_uuid, &self.folder, name, bytes).await
    }
}

/// Publishes `output/` for the turn. Never returns an error: a Drive outage
/// leaves the artifacts in the VM and they are retried on the next turn.
pub async fn publish_turn_output(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
) -> PublishReport {
    let listing: Vec<OutputFile> =
        match agent_vm_hook::list_session_folder_meta(user_id, session_id, "output").await {
            Some(files) => files
                .into_iter()
                .map(|(name, size, modified)| OutputFile {
                    name,
                    size,
                    modified,
                })
                .collect(),
            None => return PublishReport::default(),
        };
    let mut seen = snapshot(session_id);
    let selected = candidates(&listing, &seen);
    if selected.is_empty() {
        return PublishReport::default();
    }
    let folder = drive_bridge::conversation_title(state, session_id).await;
    let publisher = DrivePublisher {
        state: state.clone(),
        bot_uuid,
        user_id,
        session_id,
        folder: folder.clone(),
    };
    let published = publish_candidates(session_id, selected, &mut seen, &publisher).await;
    // Store even when every upload failed: successful and identical files
    // advanced, failed ones kept their old stamp and stay candidates.
    store(session_id, seen);
    PublishReport { folder, published }
}

/// Ceiling on one turn's publishing work (listing + reads + uploads). The
/// answer has already been delivered, so a slow Drive must not hold the turn
/// task open; a timeout leaves the ledger untouched and the next turn retries.
const PUBLISH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

/// Publishes the turn's artifacts and attaches their links to the chat.
///
/// `vmfile:` is a chat link, so only the web channel receives the message;
/// every other channel still gets the durable Drive copy.
pub async fn publish_and_attach(identity: &SessionIdentity<'_>) {
    let SessionIdentity {
        state,
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    let report = match tokio::time::timeout(
        PUBLISH_TIMEOUT,
        publish_turn_output(state, bot_uuid, user_id, session_id),
    )
    .await
    {
        Ok(report) => report,
        Err(_) => {
            botlog::safe_warn!("output publish: timed out for {session_id}; retrying next turn");
            return;
        }
    };
    if report.is_empty() {
        return;
    }
    let count = report.published.len();
    botlog::safe_info!("output publish: {count} file(s) for {session_id}");
    // #2172 — signal finalize so a live-streaming app can reconcile to the
    // canonical Drive file instead of reloading a conversion.
    for item in &report.published {
        let app = doc_delta::app_for_filename(&item.name);
        let env = doc_delta::DocDeltaEnvelope::finalize(item.path.clone(), app, 0);
        doc_delta::emit(state, session_id, &env).await;
    }
    if sink.channel_type() != "web" {
        return;
    }
    let message = report.message();
    if message.is_empty() {
        return;
    }
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

#[cfg(test)]
#[path = "output_publish_tests.rs"]
mod tests;
