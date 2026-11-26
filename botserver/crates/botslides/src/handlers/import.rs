use crate::auth::SlidesUser;
use crate::storage::{
    load_presentation_by_id, load_presentation_pptx, save_presentation_pptx,
    save_presentation_to_drive, DriveOps,
};
use crate::types::Presentation;
use crate::utils::{create_default_theme, slides_from_markdown};
use crate::SlidesState;
use axum::{extract::State, http::StatusCode, Json};
use botcore::{ImportSize, WorkClass};
use botsecurity_core::error_sanitizer::log_and_sanitize_json;
use botsecurity_core::upload_hostile::{macro_strip, password as password_guard};
use botsecurity_core::upload_policy::{self, UploadClass};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Test-only hostile-file gate inspection used by the offline corpus tests.
///
/// `upload_policy::validate_upload` applies this gate centrally to every
/// import; the request path relies on that single pass so the payload is never
/// scanned twice (#2927). This helper keeps the refusal contract assertable
/// without a live scanner and never leaks archive internals to the client.
#[cfg(test)]
fn audit_hostile_upload(
    bytes: &[u8],
    filename: &str,
    headers: &axum::http::HeaderMap,
    context: &str,
    policy: &upload_policy::HostileFilePolicy,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    // Mirror `validate_upload`: only a zip package has OOXML parts. Markdown
    // and JSON decks are plain text and must not be parsed as a zip.
    let is_package = filename
        .rsplit('.')
        .next()
        .and_then(upload_policy::mime_for_extension)
        .is_some_and(upload_policy::is_zip_container);
    if !is_package {
        return Ok(());
    }

    audit_hostile_upload_with_scanner(bytes, filename, headers, context, policy, None)
}

/// Test seam: an explicit scanner makes the fail-closed assertion independent
/// of whatever hook another test installed process-wide.
#[cfg(test)]
fn audit_hostile_upload_with_scanner(
    bytes: &[u8],
    filename: &str,
    headers: &axum::http::HeaderMap,
    context: &str,
    policy: &upload_policy::HostileFilePolicy,
    scanner: Option<&botsecurity_core::upload_hostile::MalwareScanner>,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let report = botsecurity_core::upload_hostile::inspect_hostile_upload_with_scanner(
        bytes, filename, policy, scanner,
    )
    .map_err(|error| {
        log::warn!(
            "{context}: hostile-file gate refused upload: {}",
            error.log_detail()
        );
        upload_policy::rejection_json(&error, headers)
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
    Ok(())
}

/// Deterministic deck id for a Drive source object (FNV-1a 64-bit). A
/// cryptographic hash buys nothing here: the id only needs to be stable and
/// collision-resistant enough for one user's presentations.
fn stable_source_id(bucket: &str, path: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in format!("gb-slides:{bucket}/{path}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("source-{hash:016x}")
}

/// Storage failures are infrastructure errors: log the cause server-side and
/// answer with the shared sanitized envelope, never with the raw Drive error.
fn storage_rejection(error: &str, context: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": log_and_sanitize_json(&error, context, None)
        })),
    )
}

/// The one persistence gate for every accepted import.
///
/// #2927 — the sanitized package used to be written to Drive without writing
/// the presentation metadata row on some paths (or with the failure only
/// logged), so `/api/slides/list` stayed empty and no notice reached the
/// caller. Both the new-deck path and the re-opened-Drive-source path now call
/// this function, which writes the JSON row first, then the sanitized PPTX,
/// and attaches the security notice to the returned presentation.
async fn persist_import<D: DriveOps>(
    drive: Option<&D>,
    user_id: &str,
    presentation: &Presentation,
    pptx_bytes: Option<&[u8]>,
    notices: Vec<crate::import_notice::SecurityNotice>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Some(drive) = drive {
        save_presentation_to_drive(drive, user_id, presentation)
            .await
            .map_err(|error| {
                log::error!("botslides::import: metadata save failed: {error}");
                storage_rejection(&error, "botslides::handlers::import::metadata")
            })?;
        if let Some(bytes) = pptx_bytes {
            save_presentation_pptx(drive, user_id, &presentation.id, bytes)
                .await
                .map_err(|error| {
                    log::error!("botslides::import: package save failed: {error}");
                    storage_rejection(&error, "botslides::handlers::import::package")
                })?;
        }
    }
    crate::import_notice::with_notices(presentation, notices)
}

pub async fn handle_import_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    user: SlidesUser,
    headers: axum::http::HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // #2132 — refuse an oversized body before it is buffered.
    if let Err(error) =
        upload_policy::check_declared_length(UploadClass::PresentationImport, &headers)
    {
        return Err(upload_policy::rejection_json(&error, &headers));
    }

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.pptx".to_string();
    let mut content_type = String::new();
    let mut source_bucket = String::new();
    let mut source_path = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("import.pptx").to_string();
                // A multipart field may legitimately omit its content type;
                // `validate_upload` below inspects the bytes instead.
                content_type = match field.content_type() {
                    Some(content_type) => content_type.to_string(),
                    None => String::new(),
                };
                if let Ok(bytes) = field.bytes().await {
                    file_bytes = Some(bytes.to_vec());
                }
            }
            // The Drive opener sends the object identity so repeated opens of
            // the same file resolve to the same deck (#1710).
            Some("bucket") => {
                source_bucket = match field.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        log::warn!("slides import: unreadable bucket field: {e}");
                        String::new()
                    }
                };
            }
            Some("path") => {
                source_path = match field.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        log::warn!("slides import: unreadable path field: {e}");
                        String::new()
                    }
                };
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    // #2530 — pure, env-configurable budget check before any converter
    // materialises the deck. Only the byte length is known at this point; the
    // slide count is checked again inside `load_pptx` once the package order
    // has been read.
    let budget = crate::handlers::admission::import_budget(WorkClass::SlidesImport);
    if let Err(error) = budget.check(ImportSize {
        bytes: bytes.len() as u64,
        ..ImportSize::default()
    }) {
        log::warn!("botslides::import: budget refused: {}", error.code());
        return Err((error.status_code(), Json(error.json_body())));
    }

    // #2522 — strip executable parts *before* the shared gate so a
    // macro-enabled deck imports with macros removed and the caller receives a
    // machine-readable notice. The original bytes are never modified.
    let stripped = match macro_strip::strip_executable_parts(&bytes) {
        Ok(result) => result,
        Err(error) => {
            log::warn!(
                "botslides::import: macro stripping failed: {}",
                error.log_detail()
            );
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };
    let mut notices = Vec::new();
    let effective: &[u8] = match &stripped {
        Some(result) => {
            log::warn!(
                "botslides::import: removed {} executable part(s) before conversion",
                result.removed_count
            );
            notices.push(crate::import_notice::macro_notice(
                result.removed_parts.clone(),
                result.removed_count,
            ));
            &result.bytes
        }
        None => &bytes,
    };

    // #2132 — the extension, the declared type and the magic bytes must agree.
    // `validate_upload` runs the hostile-file gate (including the malware
    // scan) exactly once; auditing it again below used to scan a second time.
    let upload = match upload_policy::validate_upload(
        UploadClass::PresentationImport,
        &filename,
        &content_type,
        effective,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            log::warn!(
                "botslides::import: upload policy refused: {}",
                error.log_detail()
            );
            if password_guard::is_encrypted_rejection(&error) {
                return Err(password_guard::actionable_rejection(&error, &headers));
            }
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };
    log::info!(
        "botslides::import: hostile-file gate passed: bytes={}, malware={:?}",
        upload.size,
        upload.malware_scan
    );
    let filename = upload.filename;

    // #2530 — admit before any format parser materialises the deck. The permit
    // is held for the parse below and released by RAII on every exit.
    let permit = crate::handlers::admission::admit(
        WorkClass::SlidesImport,
        effective.len() as u64,
        Some(user.id.as_str()),
    )
    .await?;
    log::debug!("slides import admitted: class={}", permit.class().as_str());

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let theme = create_default_theme();

    let slides = match ext.as_str() {
        "md" | "markdown" => {
            let content = String::from_utf8_lossy(effective);
            slides_from_markdown(&content)
        }
        "json" => {
            let pres: Result<Presentation, _> = serde_json::from_slice(effective);
            match pres {
                Ok(p) => p.slides,
                Err(e) => {
                    // A parse error carries the reader's offsets and internals;
                    // the client is told which input was rejected. Detail: log.
                    log::warn!("slides import: JSON parse failed: {e}");
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": "Invalid JSON" })),
                    ));
                }
            }
        }
        "pptx" => {
            crate::pptx::load_pptx(effective)
                .map_err(|e| {
                    log::warn!("slides import: PPTX parse failed: {e}");
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": "Invalid PPTX" })),
                    )
                })?
                .slides
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: {ext}") })),
            ))
        }
    };

    let name = filename
        .rsplit('/')
        .next()
        .unwrap_or(&filename)
        .rsplit('.')
        .next_back()
        .unwrap_or(&filename)
        .to_string();

    let user_id = user.owner_id().to_string();
    // Stable per-source identity: opening the same Drive object twice must
    // not create a duplicate deck (and must keep edits made in the first).
    let source_id = if source_bucket.is_empty() || source_path.is_empty() {
        None
    } else {
        Some(stable_source_id(&source_bucket, &source_path))
    };
    // #2522 — this is the sanitized package; the macro-bearing original is
    // never persisted on either the new-deck or the re-open path.
    let pptx_bytes: Option<&[u8]> = if ext == "pptx" { Some(effective) } else { None };

    let mut presentation = Presentation {
        id: source_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        name,
        owner_id: user_id.clone(),
        slides,
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_bucket: Some(source_bucket.clone()).filter(|value| !value.is_empty()),
        source_path: Some(source_path.clone()).filter(|value| !value.is_empty()),
    };

    if let (Some(id), Some(drive)) = (source_id.as_ref(), state.drive.as_ref()) {
        if let Ok(mut existing) = load_presentation_by_id(drive, &user_id, id).await {
            // Decks imported before the source fields existed have no identity:
            // record it on re-open so their next save also writes the sibling
            // .gbslides.pptx (#1710). The save itself happens in
            // `persist_import`, so a failure is a typed error, not a warning.
            if existing.source_bucket.is_none() || existing.source_path.is_none() {
                existing.source_bucket = Some(source_bucket.clone());
                existing.source_path = Some(source_path.clone());
            }
            // #2041 — a deck imported before the original package was kept
            // (JSON only) gets it on the next open, so its first save already
            // preserves theme, notes and media. #2522 — never replace an
            // already sanitized package with the macro-bearing original.
            let stored_pptx = match pptx_bytes {
                Some(bytes)
                    if load_presentation_pptx(drive, &user_id, &existing.id)
                        .await
                        .is_none() =>
                {
                    Some(bytes)
                }
                _ => None,
            };
            presentation = existing;
            return persist_import(
                state.drive.as_ref(),
                &user_id,
                &presentation,
                stored_pptx,
                notices,
            )
            .await;
        }
    }

    persist_import(
        state.drive.as_ref(),
        &user_id,
        &presentation,
        pptx_bytes,
        notices,
    )
    .await
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod hostile_import_tests;
