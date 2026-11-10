//! Inbound Telegram delivery tests (TG-1..TG-3 in #2070).
//!
//! Covers the three decisions the webhook makes before any database work:
//! what the message *means* (`extract_message_content`), whether the delivery
//! is authentic (`secret_matches`), and whether it was already handled
//! (`DedupRegistry`, the redelivery state machine).

use super::*;
use crate::types::{
    TelegramAudio, TelegramChat, TelegramContact, TelegramDocument, TelegramLocation,
    TelegramMessage, TelegramPhotoSize, TelegramVideo, TelegramVoice,
};
use std::time::{Duration, Instant};

fn message() -> TelegramMessage {
    TelegramMessage {
        message_id: 1,
        from: None,
        chat: TelegramChat {
            id: -100_200,
            chat_type: "private".to_string(),
            title: None,
            username: None,
            first_name: None,
            last_name: None,
        },
        date: 1_700_000_000,
        text: None,
        photo: None,
        document: None,
        voice: None,
        audio: None,
        video: None,
        location: None,
        contact: None,
        caption: None,
    }
}

fn photo() -> Vec<TelegramPhotoSize> {
    vec![TelegramPhotoSize {
        file_id: "photo-1".to_string(),
        file_unique_id: "unique-1".to_string(),
        width: 1280,
        height: 720,
        file_size: Some(1024),
    }]
}

fn document() -> TelegramDocument {
    TelegramDocument {
        file_id: "doc-1".to_string(),
        file_unique_id: "doc-unique-1".to_string(),
        file_name: Some("contrato.pdf".to_string()),
        mime_type: Some("application/pdf".to_string()),
        file_size: Some(2048),
    }
}

#[test]
fn text_wins_over_every_attachment() {
    let mut update = message();
    update.text = Some("olá".to_string());
    update.caption = Some("legenda".to_string());
    update.photo = Some(photo());
    update.document = Some(document());

    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &update), "olá");
}

#[test]
fn caption_is_used_when_the_text_is_absent() {
    let mut update = message();
    update.caption = Some("legenda da foto".to_string());
    update.photo = Some(photo());

    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &update), "legenda da foto");
}

#[test]
fn an_empty_text_falls_back_to_the_caption() {
    let mut update = message();
    update.text = Some(String::new());
    update.caption = Some("legenda".to_string());

    // Pinned: the empty text is returned verbatim, so the caller sees "" and
    // drops the turn instead of routing the caption. Telegram never sends an
    // empty `text`, so this only documents the current branch order.
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &update), "");
}

#[test]
fn each_media_kind_maps_to_its_marker() {
    let mut photo_update = message();
    photo_update.photo = Some(photo());
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &photo_update), "[Photo received]");

    let mut document_update = message();
    document_update.document = Some(document());
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &document_update), "[Document received]");

    let mut voice_update = message();
    voice_update.voice = Some(TelegramVoice {
        file_id: "voice-1".to_string(),
        file_unique_id: "voice-unique-1".to_string(),
        duration: 3,
        mime_type: Some("audio/ogg".to_string()),
        file_size: Some(512),
    });
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &voice_update), "[Voice message received]");

    let mut audio_update = message();
    audio_update.audio = Some(TelegramAudio {
        file_id: "audio-1".to_string(),
        file_unique_id: "audio-unique-1".to_string(),
        duration: 180,
        performer: Some("Ana".to_string()),
        title: Some("Song".to_string()),
        mime_type: Some("audio/mpeg".to_string()),
        file_size: Some(4096),
    });
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &audio_update), "[Audio message received]");

    let mut video_update = message();
    video_update.video = Some(TelegramVideo {
        file_id: "video-1".to_string(),
        file_unique_id: "video-unique-1".to_string(),
        width: 640,
        height: 480,
        duration: 12,
        mime_type: Some("video/mp4".to_string()),
        file_size: Some(8192),
    });
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &video_update), "[Video received]");
}

#[test]
fn photo_wins_over_document_when_both_are_present() {
    let mut update = message();
    update.photo = Some(photo());
    update.document = Some(document());

    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &update), "[Photo received]");
}

#[test]
fn location_and_contact_render_their_fields() {
    let mut location_update = message();
    location_update.location = Some(TelegramLocation {
        latitude: -23.5505,
        longitude: -46.6333,
    });
    assert_eq!(
        extract_message_content(&botlib::i18n::Locale::default(), &location_update),
        "[Location: -23.5505, -46.6333]"
    );

    let mut contact_update = message();
    contact_update.contact = Some(TelegramContact {
        phone_number: "+5511987654321".to_string(),
        first_name: "Ana".to_string(),
        last_name: Some("Souza".to_string()),
        user_id: Some(42),
    });
    // Pinned: only the first name is rendered, the last name is dropped.
    assert_eq!(
        extract_message_content(&botlib::i18n::Locale::default(), &contact_update),
        "[Contact: Ana +5511987654321]"
    );
}

#[test]
fn a_message_without_content_yields_an_empty_string() {
    assert_eq!(extract_message_content(&botlib::i18n::Locale::default(), &message()), "");
}

#[test]
fn unsigned_delivery_is_accepted_when_no_secret_is_configured() {
    assert!(secret_matches("", ""));
    assert!(secret_matches("   ", ""));
    assert!(secret_matches("", "anything"));
}

#[test]
fn configured_secret_requires_the_exact_header_value() {
    assert!(secret_matches("s3cret", "s3cret"));
    assert!(secret_matches(" s3cret ", "s3cret"));
    // Both sides are trimmed: a delivery that pads the header still matches.
    assert!(secret_matches("s3cret", "s3cret "));
    assert!(!secret_matches("s3cret", ""));
    assert!(!secret_matches("s3cret", "s3cre"));
    assert!(!secret_matches("s3cret", "S3CRET"));
    assert!(!secret_matches("s3cret", "s3cret extra"));
}

#[test]
fn secret_header_name_matches_the_bot_api() {
    assert_eq!(SECRET_HEADER, "x-telegram-bot-api-secret-token");
}

#[test]
fn constant_time_eq_compares_content_not_identity() {
    assert!(constant_time_eq(b"", b""));
    assert!(constant_time_eq(b"abc", b"abc"));
    // Length mismatch is rejected without indexing out of bounds.
    assert!(!constant_time_eq(b"abc", b"abcd"));
    // Same length, last byte differs: the comparison must still walk the whole
    // slice (a short-circuit would return at the first byte).
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"xbc"));
    assert!(!constant_time_eq(b"abc", b""));
}

fn registry() -> DedupRegistry {
    DedupRegistry::new(Duration::from_secs(600))
}

#[test]
fn the_first_delivery_is_accepted_and_the_redelivery_dropped() {
    let mut dedup = registry();
    let now = Instant::now();

    assert!(dedup.register_at(7, now));
    assert!(!dedup.register_at(7, now));
    assert!(!dedup.register_at(7, now + Duration::from_secs(1)));
    assert_eq!(dedup.len(), 1);
}

#[test]
fn independent_update_ids_are_all_accepted() {
    let mut dedup = registry();
    let now = Instant::now();

    assert!(dedup.register_at(1, now));
    assert!(dedup.register_at(2, now));
    assert!(dedup.register_at(3, now));
    assert_eq!(dedup.len(), 3);
}

#[test]
fn the_extreme_update_ids_are_valid_keys() {
    let mut dedup = registry();
    let now = Instant::now();

    assert!(dedup.register_at(i64::MIN, now));
    assert!(dedup.register_at(i64::MAX, now));
    assert!(!dedup.register_at(i64::MIN, now));
    assert!(!dedup.register_at(i64::MAX, now));
}

#[test]
fn an_id_ages_out_exactly_at_the_window_boundary() {
    let mut dedup = registry();
    let start = Instant::now();

    assert!(dedup.register_at(11, start));
    // One nanosecond before the window: still a duplicate.
    assert!(!dedup.register_at(11, start + Duration::from_secs(599)));
    // At the window the entry is evicted, so the id is accepted again.
    assert!(dedup.register_at(11, start + Duration::from_secs(600)));
}

#[test]
fn eviction_drops_every_entry_older_than_the_window() {
    let mut dedup = registry();
    let start = Instant::now();

    assert!(dedup.register_at(21, start));
    assert!(dedup.register_at(22, start + Duration::from_secs(10)));
    assert_eq!(dedup.len(), 2);

    // At +601 s the first id (601 s old) is evicted while the second (591 s
    // old) is still inside the window, so only it plus the new id remain.
    assert!(dedup.register_at(23, start + Duration::from_secs(601)));
    assert_eq!(dedup.len(), 2);
    assert!(!dedup.register_at(22, start + Duration::from_secs(602)));
    // The evicted id is accepted again: dedup is best effort, not permanent.
    assert!(dedup.register_at(21, start + Duration::from_secs(602)));
}

#[test]
fn a_clock_that_goes_backwards_does_not_panic() {
    let mut dedup = registry();
    let start = Instant::now() + Duration::from_secs(10);

    assert!(dedup.register_at(31, start));
    // saturating_duration_since: a non-monotonic `now` keeps the entry instead
    // of panicking inside the handler.
    assert!(!dedup.register_at(31, Instant::now()));
}
