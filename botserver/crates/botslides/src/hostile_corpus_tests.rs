//! #2522 — Slides-specific malicious-file corpus.
//!
//! Mirrors `botdocs/src/hostile_corpus_tests.rs` with PPTX part names: five
//! crafted decks (VBA part, XLM macro sheet, encrypted package, SSRF
//! relationship, XML entity bomb) that must fail closed at the Slides import
//! boundary. The policy under test is the one the unified
//! `GB_ALLOW_UNSCANNED_IMPORT` switch produces when it is not opted out, so
//! this file is also CI evidence that the switch defaults to fail-closed.
//!
//! Included from `handlers/import_tests.rs`, which owns the private gate
//! helpers (`zip_with`, `audit_hostile_upload`, `is_sanitized_rejection`) via
//! its own `use super::*`. The shared corpus lives in `botsecurity-core`.

use super::*;
use botsecurity_core::upload_hostile::corpus::ExpectedRefusal;

/// A case is refused before any parser touches the package, so the exact
/// `UploadError` kind is stable and can be asserted through `ExpectedRefusal`.
struct SlidesCase {
    name: &'static str,
    filename: &'static str,
    bytes: Vec<u8>,
    expected: ExpectedRefusal,
}

/// Set the encryption bit in both the local header and the central directory
/// so the gate sees a password-protected package it cannot inspect.
fn mark_encrypted(bytes: &mut [u8]) {
    if bytes.starts_with(&[0x50, 0x4b, 0x03, 0x04]) && bytes.len() > 6 {
        bytes[6] |= 0x01;
    }
    if let Some(position) = bytes
        .windows(4)
        .position(|window| window == &[0x50, 0x4b, 0x01, 0x02][..])
    {
        if bytes.len() > position + 9 {
            bytes[position + 8] |= 0x01;
        }
    }
}

/// Every part name below is a Slides (`ppt/`) path; the shared gate is
/// path-agnostic, so these exercise the same guards as the Docs/Sheets corpus
/// while proving the PPTX boundary actually applies them.
fn slides_cases() -> Vec<SlidesCase> {
    let macro_part = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let xlm = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/macrosheets/sheet1.xml", b"<macrosheet/>".to_vec()),
    ]);
    let mut encrypted = zip_with(&[("ppt/presentation.xml", b"<p:presentation/>".to_vec())]);
    mark_encrypted(&mut encrypted);
    let ssrf = zip_with(&[(
        "ppt/_rels/presentation.xml.rels",
        b"<Relationships><Relationship Target=\"http://127.0.0.1:8080/admin\"/></Relationships>"
            .to_vec(),
    )]);
    let xxe = zip_with(&[(
        "ppt/slides/slide1.xml",
        b"<!DOCTYPE d [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><d>&xxe;</d>".to_vec(),
    )]);

    vec![
        SlidesCase {
            name: "ppt-vba-macro-part",
            filename: "macro.pptx",
            bytes: macro_part,
            expected: ExpectedRefusal::Macro,
        },
        SlidesCase {
            name: "ppt-xlm-macro-sheet",
            filename: "xlm.pptx",
            bytes: xlm,
            expected: ExpectedRefusal::Macro,
        },
        SlidesCase {
            name: "ppt-encrypted-package",
            filename: "encrypted.pptx",
            bytes: encrypted,
            expected: ExpectedRefusal::Encrypted,
        },
        SlidesCase {
            name: "ppt-ssrf-relationship",
            filename: "ssrf.pptx",
            bytes: ssrf,
            expected: ExpectedRefusal::Relationship,
        },
        SlidesCase {
            name: "ppt-xxe-entity",
            filename: "xxe.pptx",
            bytes: xxe,
            expected: ExpectedRefusal::ArchiveUnsafe,
        },
    ]
}

/// `policy_for_opt_out(false)` is exactly what the unified switch yields when
/// it is unset or carries an unrecognised value, i.e. the fail-closed default.
fn fail_closed_policy() -> upload_policy::HostileFilePolicy {
    policy_for_opt_out(false)
}

#[test]
fn slides_hostile_corpus_every_case_fails_closed() {
    let policy = fail_closed_policy();
    for case in slides_cases() {
        let result = upload_policy::inspect_hostile_upload_with(&case.bytes, case.filename, &policy);
        assert!(result.is_err(), "corpus case {} must be refused", case.name);
        if let Err(error) = result {
            assert!(
                case.expected.matches(&error),
                "corpus case {}: expected {:?}, got {}",
                case.name,
                case.expected,
                error.log_detail()
            );
        }
    }
}

#[test]
fn slides_hostile_corpus_rejections_are_sanitized() {
    let policy = fail_closed_policy();
    for case in slides_cases() {
        let result = audit_hostile_upload(
            &case.bytes,
            case.filename,
            &axum::http::HeaderMap::new(),
            "slides-corpus",
            &policy,
        );
        assert!(
            is_sanitized_rejection(result, "error-file-archive-unsafe"),
            "corpus case {} must be refused with a sanitized 400",
            case.name
        );
    }
}

#[test]
fn the_unified_scanner_switch_keeps_the_corpus_fail_closed_by_default() {
    let fail_closed = fail_closed_policy();
    let opted_out = policy_for_opt_out(true);
    assert!(fail_closed.require_malware_scanner);
    assert!(!opted_out.require_malware_scanner);

    // Read the switch, never mutate it: tests run in parallel. The live Slides
    // policy must track the shared variable (or its deprecated alias).
    let shared = std::env::var(UNCHECKED_IMPORT_OPT_OUT_ENV).ok();
    let deprecated = std::env::var(DEPRECATED_SLIDES_OPT_OUT).ok();
    assert_eq!(
        slides_hostile_policy().require_malware_scanner,
        !is_import_opted_out(shared.as_deref(), deprecated.as_deref())
    );

    // Structural guards are independent of scanning: every case stays refused
    // even when an operator opts out of the scanner requirement.
    for case in slides_cases() {
        for policy in [&fail_closed, &opted_out] {
            let result = audit_hostile_upload(
                &case.bytes,
                case.filename,
                &axum::http::HeaderMap::new(),
                "slides-corpus",
                policy,
            );
            assert!(
                is_sanitized_rejection(result, "error-file-archive-unsafe"),
                "corpus case {} must fail closed under either switch value",
                case.name
            );
        }
    }
}
