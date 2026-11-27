//! Turn-end publishing of workspace artifacts (#2143 gap D).
//!
//! Before this module, a file the agent produced in the VM's `output/` folder
//! only reached Drive when the user clicked Save/Open or the model remembered
//! to call `drive_save`. Closing the conversation threw the artifact away.
//!
//! When a step leaves new or changed files in `output/`, the person is asked
//! inside the turn ([`offer_in_turn`]); allowed files are written into the
//! person's own Drive area, `Conversas/<title>-<id8>/` ([`person_drive`]). At
//! the end of the turn the resulting `vmfile:` links are attached, so the chat
//! shows the attachment card for a file that now outlives the session folder.
//! The turn's end never asks: it copies only on a stored conversation grant
//! and lists the other new files as not saved (see `offer`).
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
//! The ledger's fast path lives in process memory; committed entries are
//! mirrored into Redis with a bounded TTL (see [`ledger`]). After a restart the
//! durable copy is loaded, so files are not republished. When the cache is
//! unavailable the in-memory fallback republishes once;
//! `drive_bridge::save_object` uses a deterministic key, so that is an overwrite
//! of the same object, never a duplicate. #2541 bounds the in-memory fallback
//! with an LRU/TTL cache (`GB_PUBLISH_LEDGER_CAP` = [`MAX_LEDGER_SESSIONS`],
//! `GB_PUBLISH_LEDGER_CACHE_TTL_MS` = 30 days); eviction only costs one
//! re-upload of an already-published object.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::bot::bounded::{self, BoundedMap};

use super::types::SessionIdentity;
use super::vm_consent::{self, VmAction, VmActionKind, VmDecision, VmDenial, VmOutcome, VmReceipt};
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Links to new files that stay in the conversation, not in Drive.
    pub unsaved: Vec<String>,
}

impl PublishReport {
    pub fn is_empty(&self) -> bool {
        self.published.is_empty() && self.unsaved.is_empty()
    }

    /// Chat message for the turn. Empty when nothing was published, so the
    /// caller never sends an empty bubble.
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
            message.push_str("**Arquivos desta conversa, não salvos no Drive** (use «Salvar no Drive» no painel de arquivos)\n");
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

/// #2541 — LRU + TTL bounded in-memory fallback. The durable mirror in
/// [`ledger`] is the source of truth; evicting here can only cause a harmless
/// re-upload of the same deterministic Drive object.
fn ledger() -> &'static Mutex<BoundedMap<Uuid, HashMap<String, PublishedFile>>> {
    static LEDGER: OnceLock<Mutex<BoundedMap<Uuid, HashMap<String, PublishedFile>>>> =
        OnceLock::new();
    LEDGER.get_or_init(|| {
        Mutex::new(BoundedMap::new(
            bounded::cap_from_env("GB_PUBLISH_LEDGER_CAP", MAX_LEDGER_SESSIONS),
            bounded::ttl_from_env("GB_PUBLISH_LEDGER_CACHE_TTL_MS", 30 * 24 * 60 * 60 * 1000),
        ))
    })
}

pub(super) fn snapshot(session_id: Uuid) -> HashMap<String, PublishedFile> {
    match ledger().lock() {
        Ok(mut map) => map.get(&session_id).cloned().map_or_else(Default::default, |v| v),
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
    // BoundedMap evicts expired and least-recently-used conversations before
    // inserting, so the fallback can never grow past its configured cap.
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
    /// Returns the file's path in the person's Drive, or `None` when Drive is
    /// unavailable.
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

/// New or changed files of `output/` nobody declined; `None` when the
/// workspace cannot be listed.
async fn fresh_output(identity: &SessionIdentity<'_>) -> Option<Vec<OutputFile>> {
    let SessionIdentity {
        user_id, session_id, ..
    } = *identity;
    let listing: Vec<OutputFile> =
        agent_vm_hook::list_session_folder_meta(user_id, session_id, "output")
            .await?
            .into_iter()
            .map(|(name, size, modified)| OutputFile {
                name,
                size,
                modified,
            })
            .collect();
    let seen = snapshot(session_id);
    Some(offer::fresh(session_id, candidates(&listing, &seen)))
}

fn drive_save_action(files: &[OutputFile]) -> VmAction {
    let names = files.iter().map(|file| file.name.clone()).collect();
    VmAction::native(VmActionKind::DriveSave {
        files: names,
        folder: None,
    })
}

/// Copies `files` to Drive, advances the ledger and closes the step of
/// `receipt`. Returns the conversation folder and what reached Drive.
async fn copy_to_drive(
    identity: &SessionIdentity<'_>,
    receipt: &VmReceipt,
    files: Vec<OutputFile>,
) -> (String, Vec<PublishedArtifact>) {
    let SessionIdentity {
        state,
        bot_uuid,
        user_id,
        session_id,
        ..
    } = *identity;
    let names: Vec<String> = files.iter().map(|file| file.name.clone()).collect();
    let mut seen = snapshot(session_id);
    let folder = drive_bridge::conversation_title(state, session_id).await;
    let publisher = drive::publisher(state, bot_uuid, user_id, session_id, folder.clone());
    let published = publish_candidates(session_id, files, &mut seen, &publisher).await;
    // Store even when every upload failed: successful and identical files
    // advanced, failed ones kept their old stamp and stay candidates.
    store(session_id, seen);
    let outcome = VmOutcome {
        ok: !published.is_empty(),
        files: names,
        ..VmOutcome::default()
    };
    vm_consent::report(identity, receipt, &outcome).await;
    (folder, published)
}

/// What the model reads when the person declined copying files to Drive.
const DECLINED_FOR_MODEL: &str = "The person did not allow copying the new files of output/ to \
    Drive. They stay in this conversation; do not save them another way. Tell the person, in \
    their language, that the files can be saved with the \"Save to Drive\" button of the file \
    panel (\"Salvar no Drive\" in Portuguese).";

/// Asks, inside the turn, whether the new files a step just left in `output/`
/// may be copied to Drive. Called when a `vm_exec` step closes; a stored grant
/// answers without a card. Allowed files are copied now and linked at the end
/// of the turn; declined ones stay in the conversation.
pub async fn offer_in_turn(identity: &SessionIdentity<'_>) {
    let Some(files) = fresh_output(identity).await else {
        return;
    };
    if files.is_empty() {
        return;
    }
    let session_id = identity.session_id;
    match vm_consent::ask(identity, &drive_save_action(&files)).await {
        VmDecision::Allowed(ticket) => {
            let receipt = ticket.receipt();
            let (folder, published) = copy_to_drive(identity, &receipt, files).await;
            offer::keep_published(session_id, folder, published);
        }
        VmDecision::Denied(VmDenial::ByPerson) => {
            offer::hold_back(session_id, &files, true);
            super::native_tools::tell_model(session_id, DECLINED_FOR_MODEL);
        }
        // Undecided (stopped turn, no answer): the turn's end lists them.
        VmDecision::Denied(_) => {}
    }
}

mod drive;
pub use drive::{publish_and_attach, publish_turn_output};

pub(super) mod produced;

mod offer;

mod ledger;
pub use ledger::{load_durable, output_link, save_durable, DEFAULT_LEDGER_TTL_DAYS};

#[cfg(test)]
#[path = "../output_publish_tests.rs"]
mod tests;
