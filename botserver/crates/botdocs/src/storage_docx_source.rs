//! #2042/#2044 — durable source package for DOCX documents.
//!
//! Import keeps the uploaded package next to the document JSON; a save patches
//! the edited text into it and an untouched save returns it byte for byte.
//! Split from `storage_drive` so that file stays under the size limit.

use std::sync::Arc;

use botlib::traits::{BoxFutureUnit, BoxFutureVecU8, DriveRepository};

use crate::state::DocState;
use crate::storage_core::{
    cache_document_bytes, get_cached_document_bytes, get_user_docs_path, html_to_paragraphs,
};
use crate::storage_docx::convert_html_to_docx;

const DOCX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

/// How faithfully the bytes handed back to the caller represent the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocxFidelity {
    /// The stored package was patched, so styles, numbering, headers/footers,
    /// tables and images came through untouched.
    Preserved,
    /// No usable stored package, so the DOCX was rebuilt from the flattened
    /// HTML model — anything the HTML model cannot express is lost.
    Regenerated,
}

impl DocxFidelity {
    /// Value used in logs and in the `x-gb-docx-fidelity` export header.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preserved => "preserved",
            Self::Regenerated => "regenerated",
        }
    }
}

/// The object-store surface the source snapshot needs, narrowed from
/// [`DriveRepository`] so the persistence path can be exercised without a live
/// bucket. Keys are full object keys inside the bot bucket.
pub(crate) trait DocxSourceStore: Send + Sync + std::fmt::Debug {
    fn read(&self, key: &str) -> BoxFutureVecU8;
    fn write(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> BoxFutureUnit;
}

/// [`DocxSourceStore`] over the bot's Drive bucket — the durable copy that
/// makes fidelity survive a restart.
#[derive(Debug)]
pub(crate) struct DriveDocxSourceStore {
    drive: Arc<dyn DriveRepository>,
    bucket: String,
}

impl DriveDocxSourceStore {
    pub(crate) fn from_state(state: &DocState) -> Self {
        Self {
            drive: Arc::clone(&state.drive),
            bucket: state.bucket_name.clone(),
        }
    }
}

impl DocxSourceStore for DriveDocxSourceStore {
    fn read(&self, key: &str) -> BoxFutureVecU8 {
        let drive = Arc::clone(&self.drive);
        let bucket = self.bucket.clone();
        let key = key.to_string();
        Box::pin(async move { drive.get_object(&bucket, &key).await })
    }

    fn write(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> BoxFutureUnit {
        let drive = Arc::clone(&self.drive);
        let bucket = self.bucket.clone();
        let key = key.to_string();
        let content_type = content_type.to_string();
        Box::pin(async move {
            drive
                .put_object(&bucket, &key, bytes, Some(&content_type))
                .await
        })
    }
}

fn docx_path(user_identifier: &str, doc_id: &str) -> String {
    format!("{}/{}.docx", get_user_docs_path(user_identifier), doc_id)
}

fn source_fingerprint_path(user_identifier: &str, doc_id: &str) -> String {
    format!(
        "{}/{}.docx.source.json",
        get_user_docs_path(user_identifier),
        doc_id
    )
}

/// Stable 64-bit FNV-1a fingerprint of the HTML the stored package represents.
/// Used to tell "the user changed nothing" from "the text changed" without
/// comparing normalised paragraphs, which differ from the package by design.
fn content_fingerprint(content: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The stored package for a document: the process cache first, then the Drive
/// copy written beside the HTML. The Drive copy is what survives a restart.
async fn original_docx(
    store: &dyn DocxSourceStore,
    user_identifier: &str,
    doc_id: &str,
) -> Option<Vec<u8>> {
    if let Some(bytes) = get_cached_document_bytes(doc_id).await {
        return Some(bytes);
    }
    match store.read(&docx_path(user_identifier, doc_id)).await {
        Ok(bytes) => {
            cache_document_bytes(doc_id, bytes.clone()).await;
            Some(bytes)
        }
        Err(error) => {
            log::debug!("docs: no stored DOCX for {doc_id}: {error}");
            None
        }
    }
}

/// Whether the current HTML still matches the one the stored package was
/// written from. A match means the save is untouched and the package can be
/// returned byte for byte instead of rewriting its XML.
async fn matches_last_saved_content(
    store: &dyn DocxSourceStore,
    user_identifier: &str,
    doc_id: &str,
    content: &str,
) -> bool {
    let path = source_fingerprint_path(user_identifier, doc_id);
    let Ok(bytes) = store.read(&path).await else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    value
        .get("content_hash")
        .and_then(serde_json::Value::as_str)
        .map(|stored| stored == content_fingerprint(content))
        .unwrap_or(false)
}

/// #2042 — patch the edited text into the stored package when it exists,
/// otherwise rebuild. The second value reports the fidelity achieved so callers
/// can log it and surface it in the export response.
pub async fn docx_bytes_with_fidelity(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<(Vec<u8>, DocxFidelity), String> {
    let store = DriveDocxSourceStore::from_state(state);
    bytes_with_fidelity_from(&store, user_identifier, doc_id, title, content).await
}

pub(crate) async fn bytes_with_fidelity_from(
    store: &dyn DocxSourceStore,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<(Vec<u8>, DocxFidelity), String> {
    if let Some(original_bytes) = original_docx(store, user_identifier, doc_id).await {
        // #2044 — untouched save: the HTML is exactly what the import produced,
        // so return the source package instead of rewriting its XML.
        if matches_last_saved_content(store, user_identifier, doc_id, content).await {
            return Ok((original_bytes, DocxFidelity::Preserved));
        }
        let paragraphs = html_to_paragraphs(content);
        match crate::ooxml::update_docx_text(&original_bytes, &paragraphs) {
            Ok(patched) => return Ok((patched, DocxFidelity::Preserved)),
            Err(error) => {
                log::warn!(
                    "docs: could not patch text into the stored package for {doc_id}: {error}"
                );
            }
        }
    }
    Ok((
        convert_html_to_docx(title, content)?,
        DocxFidelity::Regenerated,
    ))
}

/// #2042 — persist the package so the next save (and every save after a
/// restart) can patch it instead of rebuilding. `source_content` is the HTML
/// the package represents; it is fingerprinted so an untouched save can return
/// the package unchanged. Save and import both go through here, which keeps the
/// fingerprint in step with the bytes on disk.
pub async fn persist_original_docx(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    bytes: &[u8],
    source_content: &str,
) -> Result<(), String> {
    let store = DriveDocxSourceStore::from_state(state);
    persist_original_to(&store, user_identifier, doc_id, bytes, source_content).await
}

pub(crate) async fn persist_original_to(
    store: &dyn DocxSourceStore,
    user_identifier: &str,
    doc_id: &str,
    bytes: &[u8],
    source_content: &str,
) -> Result<(), String> {
    store
        .write(
            &docx_path(user_identifier, doc_id),
            bytes.to_vec(),
            DOCX_CONTENT_TYPE,
        )
        .await
        .map_err(|e| format!("Failed to store the original DOCX: {e}"))?;
    cache_document_bytes(doc_id, bytes.to_vec()).await;
    let fingerprint = serde_json::json!({ "content_hash": content_fingerprint(source_content) });
    store
        .write(
            &source_fingerprint_path(user_identifier, doc_id),
            fingerprint.to_string().into_bytes(),
            "application/json",
        )
        .await
        .map_err(|e| format!("Failed to store the DOCX fingerprint: {e}"))?;
    Ok(())
}

#[cfg(test)]
#[path = "storage_docx_source_tests.rs"]
mod tests;
