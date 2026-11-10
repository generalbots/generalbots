//! Unit tests for the DriveMonitor file classification, KB path parsing and
//! ETag normalisation.
//!
//! Classification decides whether a synced object is compiled as a script,
//! indexed as knowledge or left alone; a wrong answer is silent (the file is
//! simply never processed), so every branch is pinned here.

use super::{classify_file, is_kb_extension, normalize_etag};

#[test]
fn script_files_are_classified_as_bas() {
    assert_eq!(classify_file("bot.gbai/bot.gbdialog/start.bas"), "bas");
    assert_eq!(classify_file("SALES.BAS"), "bas");
}

#[test]
fn knowledge_documents_are_classified_as_kb() {
    let cases = [
        "b.gbkb/docs/a.PDF",
        "b.gbkb/docs/notes.md",
        "b.gbkb/docs/table.csv",
        "b.gbkb/docs/data.json",
    ];
    for key in cases {
        assert_eq!(classify_file(key), "kb", "key={key}");
    }
}

#[test]
fn an_unknown_extension_inside_a_kb_folder_is_not_knowledge() {
    // `.gbkb` is a location, not a promise: binaries must not be indexed.
    assert_eq!(classify_file("b.gbkb/docs/a.exe"), "other");
    assert_eq!(classify_file("b.gbkb/docs/a.zip"), "other");
}

#[test]
fn a_script_keeps_precedence_over_its_folder() {
    // `.bas` is checked first: a script must always be compiled, even when a
    // stray copy sits inside a knowledge folder.
    assert_eq!(classify_file("b.gbkb/docs/start.bas"), "bas");
}

#[test]
fn bot_config_and_prompts_are_classified_separately() {
    assert_eq!(classify_file("b.gbot/config.csv"), "config");
    assert_eq!(classify_file("b.gbot/PROMPT.md"), "prompt");
    assert_eq!(classify_file("b.gbot/prompt.txt"), "prompt");
    // A config outside `.gbot/` is just a file.
    assert_eq!(classify_file("b.gbdrive/config.csv"), "other");
    // A prompt with another extension is not a prompt.
    assert_eq!(classify_file("b.gbot/PROMPT.html"), "other");
}

#[test]
fn everything_else_is_other() {
    for key in [
        "b.gbai/readme",
        "",
        "b.gbdrive/photo.png",
        "b.gbdialog/notes.txt",
    ] {
        assert_eq!(classify_file(key), "other", "key={key:?}");
    }
}

#[test]
fn classification_is_case_insensitive() {
    assert_eq!(classify_file("B.GBKB/DOCS/A.DOCX"), "kb");
    assert_eq!(classify_file("B.GBOT/CONFIG.CSV"), "config");
    assert_eq!(classify_file("B.GBOT/PROMPT.MD"), "prompt");
}

#[test]
fn is_kb_extension_accepts_the_documented_set() {
    for ext in ["txt", "md", "pdf", "docx", "csv", "html", "json", "yaml", "log", "sql"] {
        assert!(is_kb_extension(&format!("a.{ext}")), "ext={ext}");
    }
    for ext in ["exe", "zip", "png", "mp4", "bas", ""] {
        assert!(!is_kb_extension(&format!("a.{ext}")), "ext={ext}");
    }
    assert!(is_kb_extension("A.PDF"));
    assert!(!is_kb_extension("no-extension"));
}

#[test]
fn normalize_etag_strips_the_quotes_s3_adds() {
    assert_eq!(normalize_etag("\"abc\""), "abc");
    assert_eq!(normalize_etag("abc"), "abc");
    assert_eq!(normalize_etag("\"\""), "");
    assert_eq!(normalize_etag(""), "");
    // Idempotent: an already-normalised value is unchanged.
    assert_eq!(normalize_etag(&normalize_etag("\"abc\"")), "abc");
}

#[cfg(any(feature = "research", feature = "llm"))]
mod kb_path {
    use super::super::parse_kb_path;

    #[test]
    fn a_kb_key_splits_into_kb_name_and_file() {
        let parts = parse_kb_path("b.gbkb/docs/a.pdf").expect("parsed");
        assert_eq!(parts.kb_name, "docs");
        assert_eq!(parts.file_name, "a.pdf");
        assert_eq!(parts.relative_path, "docs/a.pdf");
    }

    #[test]
    fn a_nested_kb_file_keeps_its_subpath() {
        let parts = parse_kb_path("b.gbkb/d/x/y.txt").expect("parsed");
        assert_eq!(parts.kb_name, "d");
        assert_eq!(parts.file_name, "x/y.txt");
        assert_eq!(parts.relative_path, "d/x/y.txt");
    }

    #[test]
    fn a_key_without_a_kb_name_or_file_is_rejected() {
        assert!(parse_kb_path("b.gbkb").is_none());
        assert!(parse_kb_path("b.gbkb/docs/").is_none());
        assert!(parse_kb_path("b.gbai/docs/a.pdf").is_none());
        assert!(parse_kb_path("").is_none());
    }
}
