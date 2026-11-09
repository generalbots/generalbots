//! #2042/#2044 — durable source package for DOCX documents.
//!
//! Import keeps the uploaded package next to the document JSON; a save patches
//! the edited text into it and an untouched save returns it byte for byte.
//! Split from `storage_drive` so that file stays under the size limit.

use crate::state::DocState;
use crate::storage_core::{
    cache_document_bytes, get_cached_document_bytes, get_user_docs_path, html_to_paragraphs,
};
use crate::storage_docx::convert_html_to_docx;

async fn original_docx(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
) -> Option<Vec<u8>> {
    if let Some(bytes) = get_cached_document_bytes(doc_id).await {
        return Some(bytes);
    }
    let docx_path = format!("{}/{}.docx", get_user_docs_path(user_identifier), doc_id);
    match state
        .drive
        .get_object(&state.bucket_name, &docx_path)
        .await
    {
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

/// #2042 — patch the edited text into the original package when it exists,
/// otherwise rebuild. The caller decides whether to persist the result; the
/// second value is the fidelity it achieved so callers can report it.
pub async fn docx_bytes_with_fidelity(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<(Vec<u8>, &'static str), String> {
    if let Some(original_bytes) = original_docx(state, user_identifier, doc_id).await {
        // #2044 — untouched save: the HTML is exactly what the import produced,
        // so return the source package instead of rewriting its XML.
        if unchanged_since_import(state, user_identifier, doc_id, content).await {
            return Ok((original_bytes, "preserved"));
        }
        let paragraphs = html_to_paragraphs(content);
        match crate::ooxml::update_docx_text(&original_bytes, &paragraphs) {
            Ok(patched) => return Ok((patched, "preserved")),
            Err(error) => {
                log::warn!(
                    "docs: could not patch text into the original package for {doc_id}: {error}"
                );
            }
        }
    }
    Ok((convert_html_to_docx(title, content)?, "regenerated"))
}

/// Stable 64-bit FNV-1a fingerprint of the HTML the import produced. Used to
/// tell "the user changed nothing" from "the text changed" without comparing
/// normalised paragraphs, which differ from the package by design.
fn content_fingerprint(content: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn source_fingerprint_path(user_identifier: &str, doc_id: &str) -> String {
    format!(
        "{}/{}.docx.source.json",
        get_user_docs_path(user_identifier),
        doc_id
    )
}

/// #2044 — whether the current HTML still matches the one the original package
/// was imported with, i.e. the save is untouched and can return the package
/// byte for byte.
async fn unchanged_since_import(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    content: &str,
) -> bool {
    let path = source_fingerprint_path(user_identifier, doc_id);
    let Ok(bytes) = state.drive.get_object(&state.bucket_name, &path).await else {
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

/// #2042 — persist the uploaded/original package so the first save (and every
/// save after a restart) can patch it instead of rebuilding.
pub async fn persist_original_docx(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    bytes: &[u8],
    source_content: &str,
) -> Result<(), String> {
    let docx_path = format!("{}/{}.docx", get_user_docs_path(user_identifier), doc_id);
    state
        .drive
        .put_object(
            &state.bucket_name,
            &docx_path,
            bytes.to_vec(),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        )
        .await
        .map_err(|e| format!("Failed to store the original DOCX: {e}"))?;
    cache_document_bytes(doc_id, bytes.to_vec()).await;
    let fingerprint_path = source_fingerprint_path(user_identifier, doc_id);
    let fingerprint = serde_json::json!({ "content_hash": content_fingerprint(source_content) });
    state
        .drive
        .put_object(
            &state.bucket_name,
            &fingerprint_path,
            fingerprint.to_string().into_bytes(),
            Some("application/json"),
        )
        .await
        .map_err(|e| format!("Failed to store the DOCX fingerprint: {e}"))?;
    Ok(())
}

