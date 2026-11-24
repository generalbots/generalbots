//! #2522 — Slides-specific malicious-file corpus.
//!
//! Mirrors `botdocs/src/hostile_corpus_tests.rs` with PPTX part names: eight
//! crafted decks (VBA part, XLM macro sheet, encrypted package, SSRF
//! relationship, XML entity bomb, oversized embedded-media ratio, external
//! `file://` relationship, deeply nested container) that must fail closed at
//! the Slides import boundary. The policy under test is the shared
//! `HostileFilePolicy::default()`, which always requires a scanner, so this
//! file is also CI evidence that the default is fail-closed.
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
    // A highly compressible media payload: 512 KiB of zeros deflates to a few
    // hundred bytes, so uncompressed/compressed blows past the default cap of
    // 100 without relying on the separate uncompressed-size gate. The `.png`
    // name keeps this case on the ratio guard instead of the macro guard,
    // which treats any `ppt/*.bin` part as executable.
    let media_ratio = zip_with(&[
        ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
        ("ppt/media/image1.png", vec![0u8; 512 * 1024]),
    ]);
    // An external relationship whose Target uses the `file://` scheme. The
    // SSRF-strict validator refuses the scheme before any parser resolves it,
    // so no local path is ever read.
    let file_url_rel = zip_with(&[(
        "ppt/slides/_rels/slide1.xml.rels",
        b"<Relationships><Relationship Target=\"file:///etc/passwd\"/></Relationships>".to_vec(),
    )]);
    // Deep nesting: three container layers, each ending in a container
    // extension so the gate sees a nested package and refuses it in pass 2
    // rather than recursing into it.
    let deep_inner = zip_with(&[("ppt/slides/slide3.xml", b"<p:sld/>".to_vec())]);
    let deep_middle = zip_with(&[("ppt/media/middle.pptx", deep_inner)]);
    let deep_nested = zip_with(&[("ppt/media/outer.pptx", deep_middle)]);

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
        SlidesCase {
            name: "ppt-media-ratio",
            filename: "oversized-media.pptx",
            bytes: media_ratio,
            expected: ExpectedRefusal::CompressionRatio,
        },
        SlidesCase {
            name: "ppt-file-url-relationship",
            filename: "file-url-rel.pptx",
            bytes: file_url_rel,
            expected: ExpectedRefusal::Relationship,
        },
        SlidesCase {
            name: "ppt-deep-nesting",
            filename: "deep-nested.pptx",
            bytes: deep_nested,
            expected: ExpectedRefusal::Nested,
        },
    ]
}

/// The shared default, used verbatim by the import boundary, is fail-closed.
fn fail_closed_policy() -> upload_policy::HostileFilePolicy {
    upload_policy::HostileFilePolicy::default()
}

#[test]
fn slides_hostile_corpus_every_case_fails_closed() {
    let policy = fail_closed_policy();
    for case in slides_cases() {
        let result =
            upload_policy::inspect_hostile_upload_with(&case.bytes, case.filename, &policy);
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
fn the_default_policy_keeps_the_corpus_fail_closed_with_no_opt_out() {
    let policy = fail_closed_policy();
    assert!(
        policy.require_malware_scanner,
        "the import boundary must never accept an unscanned package"
    );

    // Structural guards are independent of scanning: every case is refused by
    // the same policy the handler uses, before any parser touches the package.
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
            "corpus case {} must fail closed under the default policy",
            case.name
        );
    }
}

/// The three added PPTX cases must refuse with a sanitized 400 and must not
/// echo any part name, the offending URL or the computed ratio into the body.
/// The generic corpus tests above cover the status/code; this one pins the
/// "no internals leaked" half of the contract for the newly added cases.
#[test]
fn slides_extended_corpus_rejects_sanitized_without_leaking_internals() {
    let policy = fail_closed_policy();
    let added = [
        "ppt-media-ratio",
        "ppt-file-url-relationship",
        "ppt-deep-nesting",
    ];
    let mut checked = 0usize;
    for case in slides_cases() {
        if !added.contains(&case.name) {
            continue;
        }
        checked += 1;
        let result = audit_hostile_upload(
            &case.bytes,
            case.filename,
            &axum::http::HeaderMap::new(),
            "slides-extended-corpus",
            &policy,
        );
        assert!(result.is_err(), "case {} must be refused", case.name);
        if let Err((status, axum::Json(body))) = result {
            assert_eq!(
                status,
                axum::http::StatusCode::BAD_REQUEST,
                "case {} must be a 400",
                case.name
            );
            assert_eq!(
                body["code"], "error-file-archive-unsafe",
                "case {} must use the stable sanitized code",
                case.name
            );
            let rendered = body.to_string();
            for internal in [
                "presentation.xml",
                "image1.png",
                "slide1.xml.rels",
                "outer.pptx",
                "middle.pptx",
                "file:///etc/passwd",
            ] {
                assert!(
                    !rendered.contains(internal),
                    "case {} leaked internal detail {internal:?}: {rendered}",
                    case.name
                );
            }
        }
    }
    assert_eq!(checked, added.len(), "every added case must be exercised");
}
