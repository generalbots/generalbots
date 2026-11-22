use crate::auth::DocUser;
use crate::handlers_api::admission::admit;
use crate::state::DocState;
use crate::storage::convert_docx_to_html;
use crate::storage::{create_new_document, load_document_from_drive, save_document};
use crate::types::{
    CompareDocumentsRequest, CompareDocumentsResponse, ComparisonSummary, DocumentComparison,
    DocumentDiff,
};
use crate::utils::{detect_document_format, markdown_to_html, rtf_to_html, strip_html};
use crate::utils_ooxml::odt_zip_to_html;
use axum::{extract::State, http::StatusCode, Json};
use botcore::WorkClass;
use botsecurity_core::upload_hostile::{macro_strip, password as password_guard};
use botsecurity_core::upload_policy::{self, UploadClass};
use chrono::Utc;
use std::sync::Arc;

/// #2522 — run the hostile-file gate explicitly at this import boundary and
/// audit its report before any converter touches the payload.
///
/// `upload_policy::validate_upload` already applies the same gate centrally to
/// every zip container, so this is defence in depth. Its real value is the
/// report: entry count, compression ratio and the malware decision are logged
/// for every accepted import, so "scanned clean" is never inferred merely
/// from "the parse succeeded". Rejections reuse the shared localized,
/// sanitized response; archive internals stay server-side.
fn audit_hostile_upload(
    bytes: &[u8],
    filename: &str,
    headers: &axum::http::HeaderMap,
    context: &str,
    policy: &upload_policy::HostileFilePolicy,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    // Mirror `validate_upload`: only zip packages have OOXML/ODF parts. A
    // PDF/RTF/MD/HTML/TXT upload must not be parsed as a zip and refused.
    let is_package = filename
        .rsplit('.')
        .next()
        .and_then(upload_policy::mime_for_extension)
        .is_some_and(upload_policy::is_zip_container);
    if !is_package {
        return Ok(());
    }

    let report =
        upload_policy::inspect_hostile_upload_with(bytes, filename, policy).map_err(|error| {
            log::warn!(
                "{context}: hostile-file gate refused upload: {}",
                error.log_detail()
            );
            if password_guard::is_encrypted_rejection(&error) {
                password_guard::actionable_rejection(&error, headers)
            } else {
                upload_policy::rejection_json(&error, headers)
            }
        })?;

    if report.malware_scan == upload_policy::MalwareScanDecision::NotConfigured {
        log::warn!(
            "{context}: no malware scanner configured — scan result is NotConfigured, not clean"
        );
    }
    log::info!(
        "{context}: hostile-file gate: entries={}, compressed={}, uncompressed={}, ratio={}, malware={:?}",
        report.entry_count,
        report.compressed_bytes,
        report.uncompressed_bytes,
        report.compression_ratio,
        report.malware_scan,
    );

    // #2522 — second, streaming pass: XML entity limits and relationship/SSRF
    // re-check at the import boundary. `docx_rs`/`ooxmlsdk` parse XML on their
    // own, so passing the archive-size gate is not enough to call a package
    // safe. The scanner keeps constant memory regardless of part size.
    let xml = crate::import_security::inspect_office_package(bytes).map_err(|error| {
        log::warn!(
            "{context}: hostile-file gate refused XML payload: {}",
            error.log_detail()
        );
        upload_policy::rejection_json(&error, headers)
    })?;
    log::info!(
        "{context}: XML safety: parts={}, relationships={}, entity_declarations={}, custom_entity_refs={}",
        xml.parts_checked,
        xml.relationship_targets,
        xml.entity_declarations,
        xml.custom_entity_refs,
    );
    Ok(())
}

pub async fn handle_import_document(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    headers: axum::http::HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // #2132 — refuse an oversized body before it is buffered.
    if let Err(error) = upload_policy::check_declared_length(UploadClass::DocumentImport, &headers)
    {
        return Err(upload_policy::rejection_json(&error, &headers));
    }

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.docx".to_string();
    let mut content_type = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("import.docx").to_string();
            match field.content_type() {
                Some(declared) => content_type = declared.to_string(),
                None => log::warn!(
                    "botdocs::import: multipart file has no content type; relying on magic bytes"
                ),
            }
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    // #2530 — admit before any converter runs. The permit is held for the
    // whole parse/convert/save below and released by RAII on every exit.
    let tenant = user.id().to_string();
    let permit = admit(WorkClass::DocsImport, bytes.len() as u64, Some(&tenant)).await?;
    log::debug!("docs import admitted: class={}", permit.class().as_str());

    // #2522 — strip executable parts *before* the shared gate so a
    // macro-enabled document imports with the macros removed. The original
    // bytes are never modified, and the notice is returned to the caller.
    let stripped = match macro_strip::strip_executable_parts(&bytes) {
        Ok(result) => result,
        Err(error) => {
            log::warn!(
                "botdocs::import: macro stripping failed: {}",
                error.log_detail()
            );
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };
    let mut notices = Vec::new();
    let effective: &[u8] = match &stripped {
        Some(result) => {
            log::warn!(
                "botdocs::import: removed {} executable part(s) before conversion",
                result.removed_count
            );
            notices.push(crate::import_security::macro_notice(
                result.removed_parts.clone(),
                result.removed_count,
            ));
            &result.bytes
        }
        None => &bytes,
    };

    // #2132 — the extension, the declared type and the magic bytes must agree.
    let upload = match upload_policy::validate_upload(
        UploadClass::DocumentImport,
        &filename,
        &content_type,
        effective,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            log::warn!(
                "botdocs::import: upload policy refused: {}",
                error.log_detail()
            );
            if password_guard::is_encrypted_rejection(&error) {
                return Err(password_guard::actionable_rejection(&error, &headers));
            }
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };
    let filename = upload.filename;

    // #2522 — inspect and audit before any format-specific parser runs. The
    // helper skips plain formats (PDF/RTF/MD/HTML/TXT) on its own.
    audit_hostile_upload(
        effective,
        &filename,
        &headers,
        "botdocs::import",
        &upload_policy::HostileFilePolicy::default(),
    )?;

    let format = detect_import_format(&filename, effective);
    let content = match format.as_str() {
        // #2254 — a PDF is read back into editable paragraphs instead of being
        // rejected as an unsupported format.
        "pdf" => crate::import_pdf::pdf_to_editor_html(effective).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
        })?,
        "docx" => convert_docx_to_html(effective).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
        })?,
        "odt" => odt_zip_to_html(effective).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
        })?,
        "rtf" => rtf_to_html(&String::from_utf8_lossy(effective)),
        "html" => String::from_utf8_lossy(&bytes).to_string(),
        "markdown" => markdown_to_html(&String::from_utf8_lossy(&bytes)),
        "txt" => {
            let text = String::from_utf8_lossy(&bytes);
            format!("<p>{}</p>", text.replace('\n', "</p><p>"))
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: {format}") })),
            ));
        }
    };

    let title = filename
        .rsplit('/')
        .next()
        .unwrap_or(&filename)
        .rsplit('.')
        .next_back()
        .unwrap_or(&filename)
        .to_string();

    let user_id = tenant;
    let mut doc = create_new_document(&user_id);
    doc.title = title;
    doc.content = content;
    doc.owner_id = user_id.clone();

    if let Err(e) = save_document(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::import", None)}),
            ),
        ));
    }

    // #2042 — keep the uploaded package. The HTML model is what the editor
    // edits; the original bytes are what a save patches, so theme, styles,
    // headers/footers, tables and images survive.
    if format == "docx" {
        if let Err(e) = crate::storage::persist_original_docx(
            &state,
            &user_id,
            &doc.id,
            effective,
            &doc.content,
        )
        .await
        {
            log::warn!("docs import: could not store the original DOCX: {e}");
        }
    }

    crate::import_security::with_notices(doc, notices)
}

pub(crate) fn detect_import_format(filename: &str, bytes: &[u8]) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "docx" => return "docx".to_string(),
        "odt" => return "odt".to_string(),
        "rtf" => return "rtf".to_string(),
        "pdf" => return "pdf".to_string(),
        "md" | "markdown" => return "markdown".to_string(),
        "html" | "htm" => return "html".to_string(),
        "txt" => return "txt".to_string(),
        _ => {}
    }
    detect_document_format(bytes).to_string()
}

pub async fn handle_compare_documents(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<CompareDocumentsRequest>,
) -> Result<Json<CompareDocumentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();

    let original = match load_document_from_drive(&state, &user_id, &req.original_doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Original document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::import", None)}),
                ),
            ));
        }
    };

    let modified = match load_document_from_drive(&state, &user_id, &req.modified_doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Modified document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::import", None)}),
                ),
            ));
        }
    };

    let original_text = strip_html(&original.content);
    let modified_text = strip_html(&modified.content);

    let mut differences = Vec::new();
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    let mut modifications = 0u32;

    let original_words: Vec<&str> = original_text.split_whitespace().collect();
    let modified_words: Vec<&str> = modified_text.split_whitespace().collect();

    let mut i = 0;
    let mut j = 0;
    let mut position = 0;

    while i < original_words.len() || j < modified_words.len() {
        if i >= original_words.len() {
            differences.push(DocumentDiff {
                diff_type: "insertion".to_string(),
                position,
                original_text: None,
                modified_text: Some(modified_words[j].to_string()),
                length: modified_words[j].len(),
            });
            insertions += 1;
            j += 1;
        } else if j >= modified_words.len() {
            differences.push(DocumentDiff {
                diff_type: "deletion".to_string(),
                position,
                original_text: Some(original_words[i].to_string()),
                modified_text: None,
                length: original_words[i].len(),
            });
            deletions += 1;
            i += 1;
        } else if original_words[i] == modified_words[j] {
            position += original_words[i].len() + 1;
            i += 1;
            j += 1;
        } else {
            differences.push(DocumentDiff {
                diff_type: "modification".to_string(),
                position,
                original_text: Some(original_words[i].to_string()),
                modified_text: Some(modified_words[j].to_string()),
                length: original_words[i].len().max(modified_words[j].len()),
            });
            modifications += 1;
            position += modified_words[j].len() + 1;
            i += 1;
            j += 1;
        }
    }

    let comparison = DocumentComparison {
        id: uuid::Uuid::new_v4().to_string(),
        original_doc_id: req.original_doc_id,
        modified_doc_id: req.modified_doc_id,
        created_at: Utc::now(),
        differences,
        summary: ComparisonSummary {
            insertions,
            deletions,
            modifications,
            total_changes: insertions + deletions + modifications,
        },
    };

    Ok(Json(CompareDocumentsResponse { comparison }))
}

#[cfg(test)]
#[path = "hostile_import_tests.rs"]
mod hostile_import_tests;
