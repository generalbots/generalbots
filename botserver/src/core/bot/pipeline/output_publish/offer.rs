//! When the person decides about copying `output/` to Drive.
//!
//! The decision is asked inside the turn, the first time a step leaves new
//! files in `output/` ([`offer_in_turn`], called when a `vm_exec` step
//! closes), on the normal card. Allowed files are copied right away; their
//! links wait for the end of the turn so the answer is not split. The turn's
//! end ([`super::publish_turn_output`]) never raises a card: it copies only on
//! a grant the conversation already holds and otherwise lists the files as
//! not saved, to be saved from the file panel.
//!
//! What the person declined, or was told was not saved, is remembered by its
//! listing stamp, so the same file is neither asked about nor listed again
//! until it changes. The memory is per conversation, in process memory and
//! bounded like the ledger.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use uuid::Uuid;

use super::{publish_candidates, ArtifactPublisher, OutputFile, PublishedArtifact, PublishedFile};

const MAX_OFFER_SESSIONS: usize = 512;

/// Stamp of a file the person did not have saved: `(size, modified)`.
pub type Held = HashMap<String, (u64, u64)>;

#[derive(Default)]
struct Pending {
    held: Held,
    folder: Option<String>,
    published: Vec<PublishedArtifact>,
    unsaved: Vec<String>,
}

/// What the turn's end shows: files copied during the turn and files the
/// person declined during it.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TurnPending {
    pub folder: Option<String>,
    pub published: Vec<PublishedArtifact>,
    pub unsaved: Vec<String>,
}

fn pending() -> &'static Mutex<HashMap<Uuid, Pending>> {
    static PENDING: OnceLock<Mutex<HashMap<Uuid, Pending>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

fn with_pending<T>(session_id: Uuid, apply: impl FnOnce(&mut Pending) -> T) -> Option<T> {
    let Ok(mut map) = pending().lock() else {
        botlog::safe_warn!("output publish: offer lock failed");
        return None;
    };
    if !map.contains_key(&session_id) && map.len() >= MAX_OFFER_SESSIONS {
        if let Some(victim) = map.keys().next().copied() {
            map.remove(&victim);
        }
    }
    Some(apply(map.entry(session_id).or_default()))
}

/// Candidates the person has not already declined at this exact stamp.
#[must_use]
pub fn not_held(candidates: Vec<OutputFile>, held: &Held) -> Vec<OutputFile> {
    candidates
        .into_iter()
        .filter(|file| held.get(&file.name) != Some(&(file.size, file.modified)))
        .collect()
}

/// Remembers `files` at their current stamp as not saved.
pub fn hold(held: &mut Held, files: &[OutputFile]) {
    for file in files {
        held.insert(file.name.clone(), (file.size, file.modified));
    }
}

/// `candidates` minus what this conversation already holds back.
pub(super) fn fresh(session_id: Uuid, candidates: Vec<OutputFile>) -> Vec<OutputFile> {
    let held = with_pending(session_id, |entry| entry.held.clone()).unwrap_or_else(|| { log::debug!("using default for held files"); Default::default() });
    not_held(candidates, &held)
}

/// Holds `files` back; when `listed`, they are also shown as not saved at
/// the end of this turn.
pub(super) fn hold_back(session_id: Uuid, files: &[OutputFile], listed: bool) {
    with_pending(session_id, |entry| {
        hold(&mut entry.held, files);
        if listed {
            for file in files {
                if !entry.unsaved.contains(&file.name) {
                    entry.unsaved.push(file.name.clone());
                }
            }
        }
    });
}

/// Keeps what was copied during the turn for the turn's closing message.
pub(super) fn keep_published(session_id: Uuid, folder: String, published: Vec<PublishedArtifact>) {
    with_pending(session_id, |entry| {
        entry.folder = Some(folder);
        for artifact in published {
            entry.published.retain(|kept| kept.name != artifact.name);
            entry.published.push(artifact);
        }
    });
}

/// What the turn left for its closing message; the held stamps stay.
pub(super) fn take_turn(session_id: Uuid) -> TurnPending {
    with_pending(session_id, |entry| TurnPending {
        folder: entry.folder.take(),
        published: std::mem::take(&mut entry.published),
        unsaved: std::mem::take(&mut entry.unsaved),
    })
    .unwrap_or_else(|| { log::debug!("using default for turn pending"); Default::default() })
}

/// The turn's end for `files`, given the answer of the stored-grant lookup
/// (which cannot raise a card): copies them on a grant, otherwise holds them
/// back. Returns what reached Drive and the names left in the conversation.
pub(super) async fn settle_turn_end(
    session_id: Uuid,
    files: Vec<OutputFile>,
    granted: bool,
    seen: &mut HashMap<String, PublishedFile>,
    publisher: &dyn ArtifactPublisher,
) -> (Vec<PublishedArtifact>, Vec<String>) {
    if files.is_empty() {
        return (Vec::new(), Vec::new());
    }
    if granted {
        return (publish_candidates(session_id, files, seen, publisher).await, Vec::new());
    }
    hold_back(session_id, &files, false);
    (Vec::new(), files.into_iter().map(|file| file.name).collect())
}

#[cfg(test)]
#[path = "offer_tests.rs"]
mod tests;
