use super::symlink_target_is_contained;
use std::path::Path;

#[test]
fn symlink_targets_staying_in_base_are_allowed() {
    let base = Path::new("/opt/gbo/bin");
    assert!(symlink_target_is_contained(base, base, "libggml.so.0"));
    let sub = base.join("lib");
    assert!(symlink_target_is_contained(base, &sub, "libggml.so"));
    assert!(symlink_target_is_contained(base, &sub, "../lib/libggml.so.0"));
}
#[test]
fn symlink_targets_escaping_base_are_refused() {
    let base = Path::new("/opt/gbo/bin");
    assert!(!symlink_target_is_contained(base, base, "/etc/passwd"));
    assert!(!symlink_target_is_contained(base, base, "../../etc/passwd"));
    let sub = base.join("a/b");
    assert!(!symlink_target_is_contained(base, &sub, "../../../outside"));
}
#[test]
fn enclosed_name_never_escapes_the_destination() {
    // enclosed_name() returns None for `../`; absolute names normalise
    // inside the base — both stay contained by the starts_with guard.
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    use zip::write::SimpleFileOptions;
    archive.start_file("../escape.txt", SimpleFileOptions::default()).unwrap();
    archive.start_file("/absolute.txt", SimpleFileOptions::default()).unwrap();
    archive.start_file("safe/lib.so", SimpleFileOptions::default()).unwrap();
    let bytes = archive.finish().unwrap().into_inner();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let names: Vec<_> = (0..zip.len()).map(|i| zip.by_index(i).unwrap().enclosed_name().map(|p| p.to_path_buf())).collect();
    assert_eq!(names.len(), 3);
    assert!(names[0].is_none(), "traversal entry must be refused");
    // Absolute entries are refused or normalised inside the base.
    if let Some(p) = &names[1] { assert!(p.is_relative(), "absolute entry must not stay absolute"); }
    assert_eq!(names[2].as_deref(), Some(Path::new("safe/lib.so")));
}
