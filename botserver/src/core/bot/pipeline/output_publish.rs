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
//! The ledger lives in process memory. After a restart (or eviction) the next
//! turn republishes once; `person_drive::save_conversation_file` uses a deterministic key,
//! so that is an overwrite of the same object, never a duplicate.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use botcore::shared::state::AppState;
use uuid::Uuid;

use super::doc_delta;
use super::native_tools::{persist, send};
use super::types::SessionIdentity;
use crate::core::bot::{agent_vm_hook, drive_bridge, person_drive};

mod ledger;
mod offer;
mod produced;
use ledger::{snapshot, store};
pub use ledger::{
    candidates, digest, is_unchanged, output_link, OutputFile, PublishReport, PublishedArtifact,
    PublishedFile, MAX_PUBLISH_FILES, MAX_PUBLISH_FILE_BYTES, MAX_PUBLISH_TURN_BYTES,
};
use super::vm_consent::{self, VmAction, VmActionKind, VmDecision, VmDenial, VmOutcome, VmReceipt};

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

/// Reads from the conversation's `output/` and writes into the person's own
/// Drive area (`Conversas/<folder>/`, see [`person_drive`]).
struct DrivePublisher {
    state: Arc<AppState>,
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
        person_drive::save_conversation_file(&self.state, self.user_id, &self.folder, name, bytes).await
    }
}

/// New or changed files of `output/` nobody declined; `None` when the
/// workspace cannot be listed.
async fn fresh_output(identity: &SessionIdentity<'_>) -> Option<Vec<OutputFile>> {
    let SessionIdentity { user_id, session_id, .. } = *identity;
    let listing: Vec<OutputFile> = agent_vm_hook::list_session_folder_meta(user_id, session_id, "output")
        .await?
        .into_iter()
        .map(|(name, size, modified)| OutputFile { name, size, modified })
        .collect();
    let seen = snapshot(session_id);
    Some(offer::fresh(session_id, candidates(&listing, &seen)))
}

fn drive_save_action(files: &[OutputFile]) -> VmAction {
    let names = files.iter().map(|file| file.name.clone()).collect();
    VmAction::native(VmActionKind::DriveSave { files: names, folder: None })
}

/// Copies `files` to Drive, advances the ledger and closes the step of
/// `receipt`. Returns the conversation folder and what reached Drive.
async fn copy_to_drive(
    identity: &SessionIdentity<'_>,
    receipt: &VmReceipt,
    files: Vec<OutputFile>,
) -> (String, Vec<PublishedArtifact>) {
    let SessionIdentity { state, user_id, session_id, .. } = *identity;
    let names: Vec<String> = files.iter().map(|file| file.name.clone()).collect();
    let mut seen = snapshot(session_id);
    let folder = drive_bridge::conversation_title(state, session_id).await;
    let publisher = DrivePublisher {
        state: state.clone(),
        user_id,
        session_id,
        folder: folder.clone(),
    };
    let published = publish_candidates(session_id, files, &mut seen, &publisher).await;
    // Store even when every upload failed: successful and identical files
    // advanced, failed ones kept their old stamp and stay candidates.
    store(session_id, seen);
    let outcome = VmOutcome { ok: !published.is_empty(), files: names, ..Default::default() };
    vm_consent::report(identity, receipt, &outcome).await;
    (folder, published)
}

/// What the model reads when the person declined copying files to Drive.
const DECLINED_FOR_MODEL: &str = "The person did not allow copying the new files of output/ to \
    Drive. They stay in this conversation; do not save them another way. Tell the person, in \
    their language, that the files can be saved with the \"Save to Drive\" button of the file \
    panel (\"Salvar no Drive\" in Portuguese).";

/// Asks, inside the turn, whether the new files a step just left in
/// `output/` may be copied to Drive. Called when a `vm_exec` step closes; a
/// stored grant answers without a card. Allowed files are copied now and
/// linked at the end of the turn; declined ones stay in the conversation.
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

/// The turn's end: what was copied during the turn, plus the new files not
/// asked about yet. Those are copied only on a grant this conversation
/// already holds; this never raises a card and never waits on the person.
/// Without a grant they stay in `output/` and are listed as not saved.
pub async fn publish_turn_output(identity: &SessionIdentity<'_>) -> PublishReport {
    let session_id = identity.session_id;
    let kept = offer::take_turn(session_id);
    let mut report = PublishReport {
        folder: kept.folder.unwrap_or_default(),
        published: kept.published,
        unsaved: kept.unsaved.iter().map(|name| output_link(name, session_id)).collect(),
    };
    let Some(files) = fresh_output(identity).await else {
        return report;
    };
    if files.is_empty() {
        return report;
    }
    let receipt = vm_consent::ask_if_granted(identity, &drive_save_action(&files))
        .await
        .map(|ticket| ticket.receipt());
    let SessionIdentity { state, user_id, .. } = *identity;
    let names: Vec<String> = files.iter().map(|file| file.name.clone()).collect();
    let mut seen = snapshot(session_id);
    let folder = drive_bridge::conversation_title(state, session_id).await;
    let publisher = DrivePublisher { state: state.clone(), user_id, session_id, folder: folder.clone() };
    let (published, unsaved) =
        offer::settle_turn_end(session_id, files, receipt.is_some(), &mut seen, &publisher).await;
    if let Some(receipt) = receipt {
        store(session_id, seen);
        let outcome = VmOutcome { ok: !published.is_empty(), files: names, ..Default::default() };
        vm_consent::report(identity, &receipt, &outcome).await;
        report.folder = folder;
    }
    report.published.extend(published);
    report.unsaved.extend(unsaved.iter().map(|name| output_link(name, session_id)));
    report
}

/// Ceiling on one turn's publishing work (listing + reads + uploads). The
/// answer has already been delivered, so a slow Drive must not hold the turn
/// task open; a timeout leaves the ledger untouched and the next turn retries.
const PUBLISH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

/// #2235 F1 — publishes the content ops of a text artifact before `finalize`.
///
/// The bytes are read back from the conversation's `output/` folder (the same
/// file the publisher just uploaded); a read failure is not fatal, the turn
/// still finalizes so the Drive link is never delayed by a delta.
///
/// Ops are emitted as separate envelopes with increasing revisions so the
/// client can render blocks incrementally (streaming) instead of receiving the
/// whole document in one frame. Returns the last emitted revision, or `0` when
/// nothing was emitted (caller should fall back to a finalize-only envelope).
async fn emit_text_content(
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    name: &str,
    app: &str,
) -> u64 {
    let relative = format!("output/{name}");
    let bytes = match agent_vm_hook::read_session_file(user_id, session_id, &relative).await {
        Ok(bytes) => bytes,
        Err(error) => {
            botlog::safe_warn!("doc delta: cannot read {relative} for {session_id}: {error}");
            return 0;
        }
    };
    let ops = doc_delta::text_block_ops(&bytes);
    if ops.is_empty() {
        return 0;
    }
    let total = ops.len();
    let mut rev: u64 = 1;
    for op in ops {
        let envelope = doc_delta::DocDeltaEnvelope::new(name, app, rev, vec![op]);
        doc_delta::emit(state, session_id, &envelope).await;
        rev += 1;
    }
    let last_rev = rev.saturating_sub(1);
    botlog::safe_debug!(
        "doc delta: streamed {total} op envelope(s) for {name} (app={app}, last_rev={last_rev})"
    );
    last_rev
}

/// Publishes the turn's artifacts and attaches their links to the chat.
///
/// `vmfile:` is a chat link, so only the web channel receives the message;
/// every other channel still gets the durable Drive copy.
pub async fn publish_and_attach(identity: &SessionIdentity<'_>) {
    let SessionIdentity {
        state,
        sink,
        session_id,
        user_id,
        ..
    } = *identity;
    let report = match tokio::time::timeout(
        PUBLISH_TIMEOUT,
        publish_turn_output(identity),
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
    let listing: Vec<OutputFile> = agent_vm_hook::list_session_folder_meta(user_id, session_id, "output")
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(name, size, modified)| OutputFile { name, size, modified })
        .collect();
    let produced = produced::files_frame(&report, &listing);
    if let Some(frame) = &produced {
        if let Err(error) = sink.send_raw_json(frame).await {
            botlog::safe_warn!("output publish: files frame not sent for {session_id}: {error}");
        }
    }
    let count = report.published.len();
    botlog::safe_info!("output publish: {count} file(s) for {session_id}");
    // #2172 — signal finalize so a live-streaming app can reconcile to the
    // canonical Drive file instead of reloading a conversion.
    // #2235 F1 — before finalizing, a text artifact also publishes the content
    // ops its renderer understands (paper), so the app window shows the
    // document instead of opening empty on a bare finalize.
    for item in &report.published {
        let app = doc_delta::app_for_filename(&item.name);
        let mut finalize_rev: u64 = 2;
        if doc_delta::is_text_artifact(&item.name) {
            let last_rev = emit_text_content(state, session_id, user_id, &item.name, app).await;
            if last_rev > 0 {
                finalize_rev = last_rev.saturating_add(1);
            }
        }
        let env = doc_delta::DocDeltaEnvelope::finalize(item.name.clone(), app, finalize_rev);
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
    // The files frame stays with this message, so a reload shows the cards.
    let stored = match produced {
        Some(frame) => {
            let mut meta = super::llm::message_meta::MessageMeta::default();
            meta.events.insert(super::turn_events::FILES_PRODUCED.to_string(), frame);
            super::llm::message_meta::encode(&message, &meta)
        }
        None => message,
    };
    persist(state, session_id, user_id, &stored).await;
}

#[cfg(test)]
#[path = "output_publish_tests.rs"]
mod tests;
