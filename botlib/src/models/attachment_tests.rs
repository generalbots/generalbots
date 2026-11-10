//! Contract tests for `Attachment` / `AttachmentType` (#2064).
//!
//! `is_media` decides whether a channel uploads the file inline or only links
//! it, and `size: Some(0)` must stay distinct from "size unknown".

use super::{Attachment, AttachmentType};

#[test]
fn each_constructor_sets_its_own_type() {
    let cases = [
        (Attachment::image("u"), AttachmentType::Image),
        (Attachment::audio("u"), AttachmentType::Audio),
        (Attachment::video("u"), AttachmentType::Video),
        (Attachment::document("u"), AttachmentType::Document),
        (Attachment::file("u"), AttachmentType::File),
    ];
    for (attachment, expected) in cases {
        assert_eq!(attachment.attachment_type, expected);
        assert_eq!(attachment.url, "u");
        assert!(attachment.mime_type.is_none());
        assert!(attachment.size.is_none());
    }
}

#[test]
fn is_media_is_true_only_for_image_audio_video() {
    assert!(Attachment::image("u").is_media());
    assert!(Attachment::audio("u").is_media());
    assert!(Attachment::video("u").is_media());
    assert!(!Attachment::document("u").is_media());
    assert!(!Attachment::file("u").is_media());
}

#[test]
fn is_image_only_matches_images() {
    assert!(Attachment::image("u").is_image());
    for other in [
        Attachment::audio("u"),
        Attachment::video("u"),
        Attachment::document("u"),
        Attachment::file("u"),
    ] {
        assert!(!other.is_image());
    }
}

#[test]
fn builders_chain_without_dropping_earlier_fields() {
    let attachment = Attachment::image("https://x/y.png")
        .with_mime_type("image/png")
        .with_filename("y.png")
        .with_size(10)
        .with_thumbnail("https://x/t.png");

    assert_eq!(attachment.mime_type.as_deref(), Some("image/png"));
    assert_eq!(attachment.filename.as_deref(), Some("y.png"));
    assert_eq!(attachment.size, Some(10));
    assert_eq!(attachment.thumbnail_url.as_deref(), Some("https://x/t.png"));
}

#[test]
fn a_zero_size_is_kept_and_not_treated_as_unknown() {
    let attachment = Attachment::file("u").with_size(0);
    assert_eq!(attachment.size, Some(0));
    assert!(attachment.size.is_some());
}

#[test]
fn the_type_is_serialized_lowercase() {
    for (attachment_type, expected) in [
        (AttachmentType::Image, "image"),
        (AttachmentType::Audio, "audio"),
        (AttachmentType::Video, "video"),
        (AttachmentType::Document, "document"),
        (AttachmentType::File, "file"),
    ] {
        let json = serde_json::to_value(attachment_type).expect("serializable");
        assert_eq!(json, serde_json::json!(expected));
    }
}

#[test]
fn a_full_attachment_round_trips_including_u64_max() {
    let original = Attachment::video("https://x/v.mp4")
        .with_size(u64::MAX)
        .with_mime_type("video/mp4");
    let encoded = serde_json::to_string(&original).expect("serializable");
    let decoded: Attachment = serde_json::from_str(&encoded).expect("deserializable");
    assert_eq!(decoded.attachment_type, AttachmentType::Video);
    assert_eq!(decoded.size, Some(u64::MAX));
    assert_eq!(decoded.mime_type.as_deref(), Some("video/mp4"));
}

#[test]
fn unset_optional_fields_are_omitted_from_json() {
    let json = serde_json::to_value(Attachment::document("u")).expect("serializable");
    assert_eq!(json, serde_json::json!({"attachment_type": "document", "url": "u"}));
}
