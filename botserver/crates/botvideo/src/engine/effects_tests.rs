//! Offline unit tests for the ffmpeg/transcription parsers (#2075).
//!
//! `detect_scenes` and `transcribe_audio` shell out (or call the models
//! service) and then parse the answer. The parsers are pure and are the only
//! place a malformed answer can turn into a wrong timeline, so they are tested
//! against real ffmpeg `showinfo` lines and transcription payloads.

use super::{scenes_from_ffmpeg_log, segments_from_transcription_json};
use serde_json::json;

fn showinfo(time: &str) -> String {
    format!(
        "[Parsed_showinfo_1 @ 0x55d0] n:0 pts:120 pts_time:{time} pos:0 fmt:yuv420p\n"
    )
}

#[test]
fn an_empty_log_produces_no_scenes() {
    assert!(scenes_from_ffmpeg_log("", 10_000).is_empty());
    assert!(scenes_from_ffmpeg_log("frame= 10 fps=0.0\n", 10_000).is_empty());
}

#[test]
fn cuts_become_ranges_that_cover_the_tail() {
    let log = format!("{}{}{}", showinfo("1.0"), showinfo("3.5"), showinfo("8.0"));
    let scenes = scenes_from_ffmpeg_log(&log, 12_000);

    // Three cuts, plus the closing range that runs to the end of the media.
    assert_eq!(scenes.len(), 4);
    assert_eq!((scenes[0].start_ms, scenes[0].end_ms), (0, 1000));
    assert_eq!((scenes[1].start_ms, scenes[1].end_ms), (1000, 3500));
    assert_eq!((scenes[2].start_ms, scenes[2].end_ms), (3500, 8000));
    assert_eq!((scenes[3].start_ms, scenes[3].end_ms), (8000, 12_000));
}

#[test]
fn sub_second_cuts_are_folded_into_the_previous_scene() {
    // ffmpeg emits one line per frame around a change; only a cut more than
    // 500 ms after the previous accepted one starts a new scene.
    let log = format!(
        "{}{}{}{}",
        showinfo("1.0"),
        showinfo("1.2"),
        showinfo("1.4"),
        showinfo("2.0")
    );
    let scenes = scenes_from_ffmpeg_log(&log, 5_000);
    assert_eq!(scenes.len(), 3);
    assert_eq!((scenes[0].start_ms, scenes[0].end_ms), (0, 1000));
    assert_eq!((scenes[1].start_ms, scenes[1].end_ms), (1000, 2000));
    assert_eq!((scenes[2].start_ms, scenes[2].end_ms), (2000, 5_000));
}

#[test]
fn malformed_and_partial_lines_are_skipped() {
    let log = "\
[Parsed_showinfo_1 @ 0x1] n:0 pts:1 pts_time:abc pos:0\n\
[Parsed_showinfo_1 @ 0x1] n:1 pts:2 pts_time: pos:0\n\
[Parsed_showinfo_1 @ 0x1] n:2 pts:3 pts_time:1.5\n\
[Parsed_showinfo_1 @ 0x1] n:3 pts:4 pts_time:2.5 pos:0\n\
pts_time:\n";
    let scenes = scenes_from_ffmpeg_log(log, 4_000);
    assert_eq!(scenes.len(), 3, "only 1.5 and 2.5 are usable");
    assert_eq!((scenes[0].start_ms, scenes[0].end_ms), (0, 1500));
    assert_eq!((scenes[1].start_ms, scenes[1].end_ms), (1500, 2500));
    assert_eq!((scenes[2].start_ms, scenes[2].end_ms), (2500, 4_000));
}

#[test]
fn a_cut_at_time_zero_does_not_emit_a_zero_length_scene() {
    // `0.0` is not greater than `last_time + 0.5`, so nothing is pushed and no
    // closing scene is fabricated for a log with no real cut.
    let scenes = scenes_from_ffmpeg_log(&showinfo("0.0"), 9_000);
    assert!(scenes.is_empty());
}

#[test]
fn transcription_segments_are_converted_to_milliseconds() {
    let payload = json!({
        "language": "pt",
        "segments": [
            {"start": 0.0, "end": 1.25, "text": "Ola", "confidence": 0.8},
            {"start": 1.25, "end": 2.0, "text": "tudo bem"}
        ]
    });
    let segments = segments_from_transcription_json(&payload);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].start_ms, 0);
    assert_eq!(segments[0].end_ms, 1250);
    assert_eq!(segments[0].text, "Ola");
    assert!((segments[0].confidence - 0.8).abs() < 0.001);
    // A missing confidence falls back to the documented 0.9.
    assert!((segments[1].confidence - 0.9).abs() < 0.001);
    assert_eq!(segments[1].end_ms, 2000);
}

#[test]
fn a_transcription_without_segments_yields_no_captions() {
    for payload in [json!({}), json!({"segments": null}), json!({"segments": []})] {
        assert!(segments_from_transcription_json(&payload).is_empty(), "{payload}");
    }
    // A non-array `segments` value is ignored rather than panicking.
    assert!(segments_from_transcription_json(&json!({"segments": "none"})).is_empty());
}

#[test]
fn a_partial_segment_keeps_the_fields_it_has() {
    let segments = segments_from_transcription_json(&json!({
        "segments": [{"text": "only text"}]
    }));
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].start_ms, 0);
    assert_eq!(segments[0].end_ms, 0);
    assert_eq!(segments[0].text, "only text");
}
