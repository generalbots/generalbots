//! Offline unit tests for the system/terminal helpers (#2073).
//!
//! `parse_meminfo_kb` (in `system.rs`) and `guess_content_type` (in the
//! `system_setup` child module) are the two pure helpers: the first turns one
//! `/proc/meminfo` line into a number, the second picks the `Content-Type` for
//! a file served by the terminal app. The #2747 tests below pin the
//! parameterized setup INSERT and slug normalization against injection
//! payloads. The `/proc` read itself and the directory walk stay in the shell.

use super::system_setup::{guess_content_type, safe_bot_slug, BOT_SETUP_INSERT_SQL};
use super::parse_meminfo_kb;
use std::path::Path;

#[test]
fn meminfo_line_yields_kilobytes() {
    assert_eq!(parse_meminfo_kb("MemTotal:       16316184 kB", "MemTotal"), Some(16_316_184));
    assert_eq!(parse_meminfo_kb("MemFree:          123456 kB", "MemFree"), Some(123_456));
    assert_eq!(parse_meminfo_kb("MemAvailable:    8192000 kB", "MemAvailable"), Some(8_192_000));
}

#[test]
fn meminfo_tolerates_spacing_variants() {
    // No space after the colon is accepted.
    assert_eq!(parse_meminfo_kb("MemTotal:1024 kB", "MemTotal"), Some(1024));
    // Leading whitespace after the colon is trimmed.
    assert_eq!(parse_meminfo_kb("MemTotal:\t2048 kB", "MemTotal"), Some(2048));
    assert_eq!(parse_meminfo_kb("MemTotal: 0 kB", "MemTotal"), Some(0));
}

#[test]
fn meminfo_rejects_a_different_key() {
    assert_eq!(parse_meminfo_kb("MemTotal:       1024 kB", "MemFree"), None);
    // Prefix stripping is literal: "Mem" leaves "Total: 1024 kB", which is not a
    // number, so the line is rejected rather than mis-parsed.
    assert_eq!(parse_meminfo_kb("MemTotal: 1024 kB", "Mem"), None);
    assert_eq!(parse_meminfo_kb("SwapTotal: 1024 kB", "MemTotal"), None);
}

#[test]
fn meminfo_rejects_lines_without_the_kb_unit() {
    assert_eq!(parse_meminfo_kb("MemTotal: 1024", "MemTotal"), None);
    assert_eq!(parse_meminfo_kb("MemTotal: 1024 MB", "MemTotal"), None);
    assert_eq!(parse_meminfo_kb("", "MemTotal"), None);
}

#[test]
fn meminfo_rejects_non_numeric_values() {
    assert_eq!(parse_meminfo_kb("MemTotal: plenty kB", "MemTotal"), None);
    assert_eq!(parse_meminfo_kb("MemTotal: -1 kB", "MemTotal"), None);
    assert_eq!(parse_meminfo_kb("MemTotal: 1.5 kB", "MemTotal"), None);
}

#[test]
fn content_type_maps_the_served_extensions() {
    let cases = [
        ("index.html", "text/html"),
        ("app.css", "text/css"),
        ("app.js", "application/javascript"),
        ("data.json", "application/json"),
        ("logo.png", "image/png"),
        ("photo.jpg", "image/jpeg"),
        ("photo.jpeg", "image/jpeg"),
        ("icon.svg", "image/svg+xml"),
        ("favicon.ico", "image/x-icon"),
        ("manual.pdf", "application/pdf"),
        ("tool.bas", "text/plain"),
        ("tool.basic", "text/plain"),
        ("kb.gbkb", "application/octet-stream"),
        ("script.ast", "application/octet-stream"),
        ("rows.csv", "text/csv"),
        ("notes.md", "text/markdown"),
        ("conf.yaml", "application/yaml"),
        ("conf.yml", "application/yaml"),
        ("feed.xml", "application/xml"),
        ("bundle.zip", "application/zip"),
    ];
    for (name, expected) in cases {
        assert_eq!(
            guess_content_type(Path::new(name)),
            expected,
            "wrong content type for {name}"
        );
    }
}

#[test]
fn content_type_falls_back_to_octet_stream() {
    assert_eq!(guess_content_type(Path::new("README")), "application/octet-stream");
    assert_eq!(guess_content_type(Path::new("archive.tar.gz")), "application/octet-stream");
    assert_eq!(guess_content_type(Path::new("unknown.bin")), "application/octet-stream");
    // Directories end in a separator, so there is no extension.
    assert_eq!(guess_content_type(Path::new("some/dir/")), "application/octet-stream");
}

#[test]
fn content_type_is_case_sensitive_on_the_extension() {
    // Documented behaviour: an upper-case extension is not served as its type.
    // Callers normalise names before serving; this test pins the fallback.
    assert_eq!(guess_content_type(Path::new("PHOTO.JPG")), "application/octet-stream");
}

#[test]
fn content_type_uses_the_last_extension_only() {
    assert_eq!(guess_content_type(Path::new("backup.html.txt")), "application/octet-stream");
    assert_eq!(guess_content_type(Path::new("bundle.min.js")), "application/javascript");
}

/// #2747 — the wizard INSERT must never carry request text in its SQL.
#[test]
fn setup_insert_sql_uses_only_bind_placeholders() {
    for index in 1..=6 {
        let placeholder = format!("${index}");
        assert_eq!(
            BOT_SETUP_INSERT_SQL.matches(&placeholder).count(),
            1,
            "expected exactly one {placeholder} bind placeholder"
        );
    }
    // The only literal quoted strings are the two static empty JSON objects.
    assert_eq!(BOT_SETUP_INSERT_SQL.matches("'{}'").count(), 2);
    assert!(!BOT_SETUP_INSERT_SQL.contains("format!"));
    for payload in [
        "' OR '1'='1",
        "'; DROP TABLE bots; --",
        "x'), ('injected",
        "1; UPDATE organizations SET slug='hacked'",
    ] {
        assert!(
            !BOT_SETUP_INSERT_SQL.contains(payload),
            "SQL statement contains injected payload {payload}"
        );
    }
}

/// #2747 — defense-in-depth: the slug derived from a hostile bot name stays
/// inert and usable as a Drive object-key prefix.
#[test]
fn bot_slug_neutralizes_injection_payloads() {
    for payload in [
        "' OR 1=1 --",
        "a'; DROP TABLE bots;--",
        "../etc/passwd",
        "<script>alert(1)</script>",
        "bot\x00name",
    ] {
        let slug = safe_bot_slug(payload);
        assert!(
            slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "slug {slug:?} kept a dangerous character from {payload:?}"
        );
        assert!(!slug.contains('\''));
        assert!(!slug.contains(';'));
        assert!(!slug.contains('/'));
        assert!(!slug.contains('\0'));
    }
    assert_eq!(safe_bot_slug("My Assistant"), "my-assistant");
    assert_eq!(safe_bot_slug("  !!  "), "assistant");
}
