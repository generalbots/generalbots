//! Contract tests for the PLAY option and content-type parsers (#2064).
//!
//! `PLAY "file.pdf", "page=3,zoom=1.5"` normalises a comma-separated option
//! string into `PlayOptions`, and the extension/MIME pair decides which viewer
//! component the client renders. Both are pure and drive the whole player.

use super::{ContentType, PlayOptions};

#[test]
fn an_empty_option_string_keeps_the_defaults() {
    let options = PlayOptions::from_string("");
    assert!(options.controls, "controls default to on");
    assert!(!options.autoplay);
    assert!(!options.loop_content);
    assert!(!options.fullscreen);
    assert!(!options.muted);
    assert!(options.start_time.is_none());
    assert!(options.theme.is_none());
}

#[test]
fn bare_flags_toggle_their_option() {
    let options = PlayOptions::from_string("autoplay, loop, fullscreen, muted, nocontrols");
    assert!(options.autoplay);
    assert!(options.loop_content);
    assert!(options.fullscreen);
    assert!(options.muted);
    assert!(!options.controls, "nocontrols turns the controls off");
}

#[test]
fn flags_and_keys_are_case_and_space_insensitive() {
    let options = PlayOptions::from_string("  AUTOPLAY ,  Theme = Dark  ");
    assert!(options.autoplay);
    // Keys and values are trimmed; the value itself is lower-cased with the
    // rest of the option string.
    assert_eq!(options.theme.as_deref(), Some("dark"));
}

#[test]
fn numeric_keys_parse_and_unknown_values_are_ignored() {
    let options = PlayOptions::from_string("start=1.5,end=9,width=640,height=480,slide=2,page=7,zoom=1.25");
    assert_eq!(options.start_time, Some(1.5));
    assert_eq!(options.end_time, Some(9.0));
    assert_eq!(options.width, Some(640));
    assert_eq!(options.height, Some(480));
    assert_eq!(options.slide, Some(2));
    assert_eq!(options.page, Some(7));
    assert_eq!(options.zoom, Some(1.25));

    // An unparseable number leaves the option unset instead of defaulting to 0.
    let options = PlayOptions::from_string("page=abc,width=-1");
    assert!(options.page.is_none());
    assert!(options.width.is_none());
}

#[test]
fn highlight_lists_split_on_dashes() {
    // The value is a dash-separated list of line numbers, not a range: "1-3"
    // highlights lines 1 and 3.
    let options = PlayOptions::from_string("highlight=1-3");
    assert_eq!(options.highlight_lines, Some(vec![1, 3]));

    let options = PlayOptions::from_string("highlight=1-3-5");
    assert_eq!(options.highlight_lines, Some(vec![1, 3, 5]));

    // A junk value yields no lines rather than panicking.
    let options = PlayOptions::from_string("highlight=x-y");
    assert_eq!(options.highlight_lines, Some(Vec::new()));
}

#[test]
fn line_numbers_is_a_three_state_option() {
    assert_eq!(
        PlayOptions::from_string("linenumbers").line_numbers,
        Some(true)
    );
    assert_eq!(PlayOptions::from_string("").line_numbers, None);
}

#[test]
fn unknown_keys_are_dropped_without_failing() {
    let options = PlayOptions::from_string("bogus=1,nonsense,theme=light");
    assert_eq!(options.theme.as_deref(), Some("light"));
    assert!(options.controls);
}

#[test]
fn extensions_map_to_the_documented_content_types() {
    let cases = [
        ("mp4", ContentType::Video),
        ("MKV", ContentType::Video),
        ("mp3", ContentType::Audio),
        ("png", ContentType::Image),
        ("SVG", ContentType::Image),
        ("pptx", ContentType::Presentation),
        ("docx", ContentType::Document),
        ("xlsx", ContentType::Spreadsheet),
        ("csv", ContentType::Spreadsheet),
        ("pdf", ContentType::Pdf),
        ("rs", ContentType::Code),
        ("BAS", ContentType::Code),
        ("md", ContentType::Markdown),
        ("html", ContentType::Html),
        ("exe", ContentType::Unknown),
        ("", ContentType::Unknown),
    ];
    for (extension, expected) in cases {
        assert_eq!(
            ContentType::from_extension(extension),
            expected,
            "extension {extension:?}"
        );
    }
}

#[test]
fn mime_types_map_to_the_documented_content_types() {
    let cases = [
        ("video/mp4", ContentType::Video),
        ("audio/ogg", ContentType::Audio),
        ("image/jpeg", ContentType::Image),
        ("application/pdf", ContentType::Pdf),
        (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ContentType::Presentation,
        ),
        (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ContentType::Spreadsheet,
        ),
        (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ContentType::Document,
        ),
        ("text/html; charset=utf-8", ContentType::Html),
        ("text/markdown", ContentType::Markdown),
        ("text/plain", ContentType::Code),
        ("application/octet-stream", ContentType::Unknown),
    ];
    for (mime, expected) in cases {
        assert_eq!(ContentType::from_mime(mime), expected, "mime {mime:?}");
    }
}

#[test]
fn every_content_type_has_its_own_player_component() {
    let all = [
        (ContentType::Video, "video-player"),
        (ContentType::Audio, "audio-player"),
        (ContentType::Image, "image-viewer"),
        (ContentType::Presentation, "presentation-viewer"),
        (ContentType::Document, "document-viewer"),
        (ContentType::Code, "code-viewer"),
        (ContentType::Spreadsheet, "spreadsheet-viewer"),
        (ContentType::Pdf, "pdf-viewer"),
        (ContentType::Markdown, "markdown-viewer"),
        (ContentType::Html, "html-viewer"),
        (ContentType::Iframe, "iframe-viewer"),
        (ContentType::Unknown, "generic-viewer"),
    ];

    let mut components: Vec<&str> = all.iter().map(|(_, component)| *component).collect();
    components.sort_unstable();
    components.dedup();
    assert_eq!(components.len(), all.len(), "two types share a component");

    for (content_type, component) in all {
        assert_eq!(content_type.player_component(), component);
    }
}
