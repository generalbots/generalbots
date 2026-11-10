//! Offline unit tests for the media type/format helpers (#2070).
//!
//! The player serves drive bytes through the browser, so the `Content-Type`
//! and the label shown in the UI are both derived from the file name. Both
//! helpers are pure string operations; the drive read and the playlist store
//! are covered by the integration suite.

use super::{get_format, get_mime_type};

#[test]
fn video_extensions_map_to_video_types() {
    assert_eq!(get_mime_type("clip.mp4"), "video/mp4");
    assert_eq!(get_mime_type("clip.webm"), "video/webm");
    assert_eq!(get_mime_type("clip.ogv"), "video/ogg");
}

#[test]
fn audio_extensions_map_to_audio_types() {
    assert_eq!(get_mime_type("track.mp3"), "audio/mpeg");
    assert_eq!(get_mime_type("track.wav"), "audio/wav");
    assert_eq!(get_mime_type("track.ogg"), "audio/ogg");
    assert_eq!(get_mime_type("track.m4a"), "audio/mp4");
    assert_eq!(get_mime_type("track.flac"), "audio/flac");
}

#[test]
fn document_and_image_extensions_are_served() {
    assert_eq!(get_mime_type("doc.pdf"), "application/pdf");
    assert_eq!(get_mime_type("img.png"), "image/png");
    assert_eq!(get_mime_type("img.jpg"), "image/jpeg");
    assert_eq!(get_mime_type("img.jpeg"), "image/jpeg");
    assert_eq!(get_mime_type("img.gif"), "image/gif");
    assert_eq!(get_mime_type("img.svg"), "image/svg+xml");
    assert_eq!(get_mime_type("img.webp"), "image/webp");
}

#[test]
fn unknown_and_missing_extensions_fall_back_to_octet_stream() {
    assert_eq!(get_mime_type("archive.tar.gz"), "application/octet-stream");
    assert_eq!(get_mime_type("README"), "application/octet-stream");
    assert_eq!(get_mime_type(""), "application/octet-stream");
    assert_eq!(get_mime_type("no-ext."), "application/octet-stream");
}

#[test]
fn extension_matching_is_case_insensitive() {
    assert_eq!(get_mime_type("CLIP.MP4"), "video/mp4");
    assert_eq!(get_mime_type("Track.Mp3"), "audio/mpeg");
    assert_eq!(get_mime_type("IMG.JPEG"), "image/jpeg");
}

#[test]
fn only_the_last_segment_of_a_path_is_considered() {
    assert_eq!(get_mime_type("bot.gbai/bot.gbdrive/media/clip.mp4"), "video/mp4");
    // A dot in a directory name must not leak into the extension.
    assert_eq!(get_mime_type("v1.2/clip.mp4"), "video/mp4");
    assert_eq!(get_mime_type("v1.2/README"), "application/octet-stream");
}

#[test]
fn format_is_the_uppercased_extension() {
    assert_eq!(get_format("clip.mp4"), "MP4");
    assert_eq!(get_format("clip.Mkv"), "MKV");
    assert_eq!(get_format("archive.tar.gz"), "GZ");
    assert_eq!(get_format("bot.gbai/media/clip.webm"), "WEBM");
}

#[test]
fn format_falls_back_to_the_whole_name_when_there_is_no_dot() {
    // `rsplit('.').next()` on a dotless name returns the name itself; the
    // player shows it upper-cased rather than an empty label.
    assert_eq!(get_format("README"), "README");
    assert_eq!(get_format(""), "");
}
