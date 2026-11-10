//! Empty-file detection. A zero-byte `.bas` in a bot's work directory means
//! the drive copy is missing, and the monitor must re-sync that bot instead of
//! compiling an empty script over a working one.
use super::*;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A unique scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "gb-zero-byte-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("scratch dir");
        Self(path)
    }

    fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &std::path::Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
    fs::write(path, contents).expect("write file");
}

#[test]
fn a_missing_file_counts_as_needing_sync() {
    let scratch = Scratch::new();

    assert!(is_file_empty_or_missing(&scratch.path().join("absent.bas")));
}

#[test]
fn a_zero_byte_file_counts_as_needing_sync() {
    let scratch = Scratch::new();
    let file = scratch.path().join("empty.bas");
    write(&file, "");

    assert!(is_file_empty_or_missing(&file));
}

#[test]
fn a_non_empty_file_is_left_alone() {
    let scratch = Scratch::new();
    let file = scratch.path().join("start.bas");
    write(&file, "TALK \"hi\"");

    assert!(!is_file_empty_or_missing(&file));
}

#[test]
fn a_missing_work_dir_scans_to_nothing() {
    let scratch = Scratch::new();

    assert!(scan_for_empty_files(&scratch.path().join("absent")).is_empty());
}

#[test]
fn a_bot_with_an_empty_script_is_reported() {
    let scratch = Scratch::new();
    write(&scratch.path().join("acme.gbai/acme.gbdialog/start.bas"), "");

    assert_eq!(scan_for_empty_files(scratch.path()), vec!["acme".to_string()]);
}

#[test]
fn a_bot_with_only_filled_scripts_is_not_reported() {
    let scratch = Scratch::new();
    write(
        &scratch.path().join("acme.gbai/acme.gbdialog/start.bas"),
        "TALK \"ready\"",
    );
    write(
        &scratch.path().join("acme.gbai/acme.gbdialog/compile.ast"),
        "binary",
    );

    assert!(scan_for_empty_files(scratch.path()).is_empty());
}

#[test]
fn directories_without_a_dialog_folder_are_ignored() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.path().join("loose.gbai/files")).expect("dirs");
    write(&scratch.path().join("loose.gbai/files/empty.bas"), "");

    assert!(scan_for_empty_files(scratch.path()).is_empty());
}

#[test]
fn each_bot_is_reported_once_even_with_several_empty_scripts() {
    let scratch = Scratch::new();
    write(&scratch.path().join("acme.gbai/acme.gbdialog/start.bas"), "");
    write(&scratch.path().join("acme.gbai/acme.gbdialog/help.bas"), "");

    assert_eq!(scan_for_empty_files(scratch.path()), vec!["acme".to_string()]);
}
