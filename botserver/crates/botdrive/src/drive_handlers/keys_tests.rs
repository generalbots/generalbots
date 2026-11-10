//! Unit tests for the drive key arithmetic and MIME table.
//!
//! These functions decide which object a request touches and what the Drive UI
//! shows for it. They are pure string/`HashMap` work, so the tenant-boundary
//! rules can be pinned without MinIO.

use super::cached_quota::{build_file_list_items, normalize_path, scoped_key};
use super::list_files::guess_mime;
use super::public_link_url::{format_link_time, public_link_url, scope_label};
use crate::drive_types::FileScope;
use std::collections::HashMap;

fn sizes(pairs: &[(&str, u64)]) -> HashMap<String, u64> {
    pairs
        .iter()
        .map(|(key, size)| ((*key).to_string(), *size))
        .collect()
}

fn names(items: &[crate::drive_types::FileListItem]) -> Vec<&str> {
    items.iter().map(|item| item.name.as_str()).collect()
}

#[test]
fn normalize_path_drops_empty_and_dot_segments() {
    let cases = [
        ("../../etc/passwd", "etc/passwd"),
        ("a//b/", "a/b"),
        ("/a/./b", "a/b"),
        ("", ""),
        ("/", ""),
        (".", ""),
        ("a/./b/../c", "a/b/c"),
        ("a/b/c", "a/b/c"),
        ("a/", "a"),
    ];
    for (input, expected) in cases {
        assert_eq!(normalize_path(input), expected, "input={input:?}");
    }
}

#[test]
fn normalize_path_keeps_dot_prefixed_names_and_backslashes() {
    // `..b` is a legitimate filename, and `\` is not a separator on S3.
    assert_eq!(normalize_path("a/..b/c"), "a/..b/c");
    assert_eq!(normalize_path("a\\..\\b"), "a\\..\\b");
}

#[test]
fn scoped_key_never_doubles_an_already_prefixed_path() {
    let prefix = "users/u1/";
    assert_eq!(scoped_key(prefix, "doc.pdf"), "users/u1/doc.pdf");
    assert_eq!(scoped_key(prefix, "users/u1/doc.pdf"), "users/u1/doc.pdf");
    assert_eq!(
        scoped_key(prefix, "users/u1/sub/doc.pdf"),
        "users/u1/sub/doc.pdf"
    );
    // The historical regression: users/u1/users/u1/x
    assert!(!scoped_key(prefix, "users/u1/x").contains("users/u1/users/u1"));
}

#[test]
fn scoped_key_without_a_prefix_returns_the_normalized_path() {
    assert_eq!(scoped_key("", "a//b"), "a/b");
    assert_eq!(scoped_key("", "docs/a.txt"), "docs/a.txt");
    assert_eq!(scoped_key("", "/"), "");
}

#[test]
fn scoped_key_normalizes_before_joining() {
    assert_eq!(scoped_key("users/u1/", "/docs/./a.txt"), "users/u1/docs/a.txt");
    assert_eq!(scoped_key("users/u1/", "../docs/a.txt"), "users/u1/docs/a.txt");
}

#[test]
fn build_file_list_items_dedupes_directories_and_sorts_dirs_before_files() {
    let keys = vec![
        "users/u1/a.txt".to_string(),
        "users/u1/sub/b.txt".to_string(),
        "users/u1/sub/deep/c.txt".to_string(),
        "users/u1/sub/other.txt".to_string(),
    ];
    let items = build_file_list_items("users/u1/", &keys, &sizes(&[("users/u1/a.txt", 7)]));

    assert_eq!(names(&items), vec!["sub", "a.txt"]);
    assert!(items[0].is_dir);
    assert_eq!(items[0].path, "users/u1/sub");
    assert_eq!(items[0].size, 0);
    assert!(!items[1].is_dir);
    assert_eq!(items[1].path, "users/u1/a.txt");
    assert_eq!(items[1].size, 7);
}

#[test]
fn build_file_list_items_sorts_siblings_alphabetically() {
    let keys = vec![
        "p/zeta.txt".to_string(),
        "p/alpha.txt".to_string(),
        "p/beta/x.txt".to_string(),
        "p/Alpha/x.txt".to_string(),
    ];
    let items = build_file_list_items("p/", &keys, &sizes(&[]));
    assert_eq!(names(&items), vec!["Alpha", "beta", "alpha.txt", "zeta.txt"]);
}

#[test]
fn build_file_list_items_reports_a_size_of_zero_on_a_metadata_miss() {
    let keys = vec!["p/missing.bin".to_string()];
    let items = build_file_list_items("p/", &keys, &sizes(&[("p/other.bin", 99)]));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].size, 0);
}

#[test]
fn build_file_list_items_flags_kb_objects_by_key_substring() {
    let keys = vec![
        "bot.gbai/bot.gbkb".to_string(),
        "bot.gbai/readme.txt".to_string(),
    ];
    let items = build_file_list_items("bot.gbai/", &keys, &sizes(&[]));
    assert_eq!(names(&items), vec!["bot.gbkb", "readme.txt"]);
    assert!(items[0].is_kb);
    assert!(!items[1].is_kb);
    assert!(items.iter().all(|item| !item.is_dir));
}

#[test]
fn build_file_list_items_accepts_a_prefix_without_a_trailing_slash() {
    let keys = vec!["users/u1/a.txt".to_string(), "users/u1/sub/b.txt".to_string()];
    let items = build_file_list_items("users/u1", &keys, &sizes(&[]));
    assert_eq!(names(&items), vec!["sub", "a.txt"]);
    assert_eq!(items[1].path, "users/u1/a.txt");
}

#[test]
fn build_file_list_items_treats_a_trailing_slash_key_as_a_directory() {
    let keys = vec!["users/u1/empty/".to_string()];
    let items = build_file_list_items("users/u1/", &keys, &sizes(&[]));
    assert_eq!(names(&items), vec!["empty"]);
    assert!(items[0].is_dir);
}

#[test]
fn build_file_list_items_skips_keys_equal_to_the_prefix() {
    let keys = vec!["users/u1/".to_string(), "users/u1/a.txt".to_string()];
    let items = build_file_list_items("users/u1/", &keys, &sizes(&[]));
    assert_eq!(names(&items), vec!["a.txt"]);
}

#[test]
fn build_file_list_items_with_no_keys_is_empty() {
    assert!(build_file_list_items("users/u1/", &[], &sizes(&[])).is_empty());
}

#[test]
fn guess_mime_is_case_insensitive_and_unknown_extensions_fall_back() {
    let cases = [
        ("a.PDF", "application/pdf"),
        ("report.Pdf", "application/pdf"),
        ("photo.JPEG", "image/jpeg"),
        ("noext", "application/octet-stream"),
        (".hidden", "application/octet-stream"),
        ("x.tar.gz", "application/octet-stream"),
        ("", "application/octet-stream"),
        ("a.txt", "text/plain; charset=utf-8"),
        ("a.md", "text/plain; charset=utf-8"),
        ("a.html", "text/html; charset=utf-8"),
        ("a.mjs", "application/javascript"),
        ("a.docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        ("a.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        ("a.pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        ("a.zip", "application/zip"),
        ("a.svg", "image/svg+xml"),
        ("a.mp4", "video/mp4"),
    ];
    for (name, expected) in cases {
        assert_eq!(guess_mime(name), expected, "name={name:?}");
    }
}

#[test]
fn guess_mime_covers_every_declared_arm() {
    // A dropped arm silently serves the wrong content type to the browser.
    let arms = [
        ("pdf", "application/pdf"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        ("webp", "image/webp"),
        ("svg", "image/svg+xml"),
        ("mp3", "audio/mpeg"),
        ("wav", "audio/wav"),
        ("ogg", "audio/ogg"),
        ("mp4", "video/mp4"),
        ("webm", "video/webm"),
        ("txt", "text/plain; charset=utf-8"),
        ("log", "text/plain; charset=utf-8"),
        ("md", "text/plain; charset=utf-8"),
        ("html", "text/html; charset=utf-8"),
        ("htm", "text/html; charset=utf-8"),
        ("css", "text/css; charset=utf-8"),
        ("js", "application/javascript"),
        ("mjs", "application/javascript"),
        ("json", "application/json"),
        ("xml", "application/xml"),
        ("zip", "application/zip"),
        ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        ("doc", "application/msword"),
        ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        ("xls", "application/vnd.ms-excel"),
        ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        ("ppt", "application/vnd.ms-powerpoint"),
    ];
    for (ext, expected) in arms {
        assert_eq!(guess_mime(&format!("file.{ext}")), expected, "ext={ext}");
    }
}

#[test]
fn scope_label_is_the_documented_wire_value() {
    assert_eq!(scope_label(&FileScope::User), "user");
    assert_eq!(scope_label(&FileScope::Bot), "bot");
}

#[test]
fn format_link_time_keeps_none_and_renders_rfc3339() {
    assert_eq!(format_link_time(None), None);
    let stamp = chrono::DateTime::parse_from_rfc3339("2026-01-02T03:04:05Z")
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);
    assert_eq!(
        format_link_time(Some(stamp)),
        Some("2026-01-02T03:04:05+00:00".to_string())
    );
}

#[test]
fn public_link_url_uses_the_declared_public_base_or_a_relative_path() {
    // The base comes from `PUBLIC_BASE_URL` / the product config; when it is
    // absent the API keeps returning the path-relative link it always has.
    std::env::remove_var("PUBLIC_BASE_URL");
    assert_eq!(public_link_url("tok"), "/api/files/public/tok");

    std::env::set_var("PUBLIC_BASE_URL", "https://drive.example.com/");
    assert_eq!(
        public_link_url("tok"),
        "https://drive.example.com/api/files/public/tok"
    );
    std::env::remove_var("PUBLIC_BASE_URL");
    assert_eq!(public_link_url("tok"), "/api/files/public/tok");
}
