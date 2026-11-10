//! Offline unit tests for the upload filename guard (#2067).
//!
//! `sanitize_filename` is the only thing standing between a multipart filename
//! and the path the upload is stored at. It must produce a single, flat name:
//! no separators, no `..`, no empty result.

use super::sanitize_filename;

#[test]
fn a_plain_name_is_kept() {
    assert_eq!(sanitize_filename("clip.mp4"), "clip.mp4");
    assert_eq!(sanitize_filename("my-clip_01.mp4"), "my-clip_01.mp4");
}

#[test]
fn separators_are_flattened() {
    let flat = sanitize_filename("dir/sub/clip.mp4");
    assert_eq!(flat, "dir_sub_clip.mp4");
    assert!(!flat.contains('/'));

    let windows = sanitize_filename("dir\\sub\\clip.mp4");
    assert_eq!(windows, "dir_sub_clip.mp4");
    assert!(!windows.contains('\\'));
}

#[test]
fn traversal_segments_cannot_survive() {
    for hostile in ["../../etc/passwd", "..", "a/../../b", "..\\..\\boot.ini"] {
        let flat = sanitize_filename(hostile);
        assert!(!flat.contains(".."), "{hostile:?} kept a traversal: {flat}");
        assert!(!flat.contains('/'), "{hostile:?} kept a separator: {flat}");
        assert!(!flat.contains('\\'), "{hostile:?} kept a separator: {flat}");
        assert!(!flat.is_empty());
    }
}

#[test]
fn unsafe_characters_become_underscores() {
    assert_eq!(sanitize_filename("a b.mp4"), "a_b.mp4");
    assert_eq!(sanitize_filename("a:b?c*.mp4"), "a_b_c_.mp4");
    assert_eq!(sanitize_filename("a\0b.mp4"), "a_b.mp4");
    assert_eq!(sanitize_filename("a;rm -rf.mp4"), "a_rm_-rf.mp4");
}

#[test]
fn unicode_letters_are_preserved() {
    assert_eq!(sanitize_filename("vídeo-ação.mp4"), "vídeo-ação.mp4");
}

#[test]
fn an_empty_or_fully_stripped_name_falls_back() {
    assert_eq!(sanitize_filename(""), "upload.bin");
    // Every character is replaced, so the result is not empty either.
    assert!(!sanitize_filename("???").is_empty());
}
