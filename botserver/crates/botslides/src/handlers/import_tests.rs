//! #2522 — hostile-file gate, macro stripping and fail-closed policy tests for
//! the slides import boundary. Kept in a separate file so the handler stays
//! under the repository's 450-line limit.

use super::*;
use std::io::{Cursor, Write};

fn zip_with(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(*name, options).expect("test entry");
        writer.write_all(body).expect("test body");
    }
    writer.finish().expect("test zip").into_inner()
}

fn is_sanitized_rejection(
    result: Result<(), (StatusCode, Json<serde_json::Value>)>,
    code: &str,
) -> bool {
    match result {
        Err((status, Json(body))) => {
            status == StatusCode::BAD_REQUEST
                && body["code"] == code
                && body.get("presentation.xml").is_none()
        }
        Ok(()) => false,
    }
}

#[test]
fn a_compression_ratio_bomb_is_refused_before_conversion() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", vec![b' '; 512 * 1024])]);
    let result = audit_hostile_upload(
        &bytes,
        "bomb.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn an_oversized_uncompressed_entry_is_refused_before_conversion() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", vec![b'x'; 8192])]);
    let policy = upload_policy::HostileFilePolicy {
        max_uncompressed_bytes: 1024,
        ..upload_policy::HostileFilePolicy::default()
    };
    let result = audit_hostile_upload(
        &bytes,
        "bomb.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &policy,
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_macro_part_is_refused_by_the_gate_before_stripping() {
    let bytes = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let result = audit_hostile_upload(
        &bytes,
        "macro.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_plain_format_is_not_mistaken_for_a_zip_package() {
    let result = audit_hostile_upload(
        b"# Slide\nplain markdown",
        "deck.md",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn a_macro_deck_is_stripped_and_noticed_before_conversion() {
    let bytes = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let stripped = match macro_strip::strip_executable_parts(&bytes) {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            log::warn!("unexpected strip failure: {}", error.log_detail());
            return;
        }
    };
    let upload = match upload_policy::validate_upload(
        UploadClass::PresentationImport,
        "macro.pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        &stripped.bytes,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            log::warn!("sanitized deck refused: {}", error.log_detail());
            return;
        }
    };
    let result = audit_hostile_upload(
        &stripped.bytes,
        &upload.filename,
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok(), "no executable parts may remain");
    let notice =
        crate::import_notice::macro_notice(stripped.removed_parts.clone(), stripped.removed_count);
    assert_eq!(notice.code, "file-macros-removed");
    assert_eq!(notice.message_key, "file-macros-removed");
    assert_eq!(notice.count, 1);
    assert!(notice.parts.iter().any(|part| part == "ppt/vbaproject.bin"));
}

#[test]
fn the_default_policy_requires_a_scanner_with_no_opt_out() {
    assert!(
        upload_policy::HostileFilePolicy::default().require_malware_scanner,
        "Slides imports must never be accepted unscanned"
    );
}

#[test]
fn a_missing_scanner_refuses_a_clean_deck_under_fail_closed_policy() {
    let bytes = zip_with(&[("ppt/slides/slide1.xml", b"<p:sld/>".to_vec())]);
    let result = audit_hostile_upload(
        &bytes,
        "clean.pptx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-upload-failed"));
}

// #2522 — Slides-specific hostile corpus (PPTX paths). Compiled from this
// test module because the gate helpers it reuses are private to `import`.
#[path = "../hostile_corpus_tests.rs"]
mod slides_hostile_corpus;
