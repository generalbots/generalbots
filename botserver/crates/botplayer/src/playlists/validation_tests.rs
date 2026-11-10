//! Offline unit tests for the playlist input validation (#2075).
//!
//! Playlist titles and visibility arrive from two different handlers (create
//! and patch) and used to be validated twice, with different rules. The shared
//! helpers are the single rule now: a title is trimmed and length-limited, a
//! visibility must be one of the three stored values.

use super::{normalize_playlist_name, normalize_visibility, MAX_TITLE_LEN};

#[test]
fn titles_are_trimmed() {
    assert_eq!(normalize_playlist_name("  Favourites  "), Some("Favourites".to_string()));
    assert_eq!(normalize_playlist_name("a"), Some("a".to_string()));
}

#[test]
fn empty_and_blank_titles_are_rejected() {
    for raw in ["", " ", "\t", "\n", "   \t  "] {
        assert_eq!(normalize_playlist_name(raw), None, "{raw:?}");
    }
}

#[test]
fn titles_at_the_limit_are_kept_and_one_past_it_rejected() {
    let at_limit = "x".repeat(MAX_TITLE_LEN);
    assert_eq!(normalize_playlist_name(&at_limit), Some(at_limit.clone()));

    let too_long = "x".repeat(MAX_TITLE_LEN + 1);
    assert_eq!(normalize_playlist_name(&too_long), None);
}

#[test]
fn the_length_limit_counts_characters_not_bytes() {
    // 255 accented characters are 510 bytes but still a valid VARCHAR(255).
    let accented = "é".repeat(MAX_TITLE_LEN);
    assert_eq!(normalize_playlist_name(&accented), Some(accented.clone()));
    assert_eq!(normalize_playlist_name(&"é".repeat(MAX_TITLE_LEN + 1)), None);
}

#[test]
fn only_the_three_visibilities_are_accepted() {
    assert_eq!(normalize_visibility("private"), Some("private".to_string()));
    assert_eq!(normalize_visibility("org"), Some("org".to_string()));
    assert_eq!(normalize_visibility("public"), Some("public".to_string()));
    assert_eq!(normalize_visibility(" public "), Some("public".to_string()));
}

#[test]
fn unknown_visibilities_are_rejected() {
    for raw in ["", " ", "PUBLIC", "shared", "everyone", "org ", "private;drop"] {
        assert_eq!(normalize_visibility(raw), None, "{raw:?}");
    }
}
