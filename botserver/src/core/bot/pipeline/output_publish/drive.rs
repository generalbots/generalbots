use super::*;
use botcore::shared::state::AppState;
use std::sync::Arc;
use uuid::Uuid;

use super::super::doc_delta;
use super::super::native_tools::{persist, send};
use super::super::types::SessionIdentity;
use crate::core::bot::{agent_vm_hook, drive_bridge};

pub(super) struct DrivePublisher {
    state: Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    folder: String,
}

/// Publisher for the in-turn Drive offer: the same guarded upload the turn's
/// end uses, with the conversation folder chosen by the consent step.
pub(super) fn publisher(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    folder: String,
) -> DrivePublisher {
    DrivePublisher {
        state: state.clone(),
        bot_uuid,
        user_id,
        session_id,
        folder,
    }
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
    let mut seen = load_durable(state, session_id).await;
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
    // advanced, failed ones kept their old stamp and stay candidates. The
    // mirrored copy survives a restart; the in-memory copy is the fallback.
    save_durable(state, session_id, &seen).await;
    PublishReport { folder, published, unsaved: Vec::new() }
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
    persist(state, session_id, user_id, &message).await;
}
