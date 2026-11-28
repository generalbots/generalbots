//! Tests for the provisioning child's progress parsing and stderr tail.

use super::{is_terminal, parse_progress_line, trim_stderr_tail, STDERR_MAX_BYTES};

#[test]
fn only_gbwizard_json_lines_are_progress_events() {
    let line = r#"GBWIZARD:{"id":"vault","state":"ready","message":"ok","elapsed_ms":12}"#;
    let event = parse_progress_line(line).expect("structured progress line");
    assert_eq!(event.id, "vault");
    assert_eq!(event.state, "ready");
    assert_eq!(event.elapsed_ms, 12);

    assert!(parse_progress_line("starting vault...").is_none());
    assert!(parse_progress_line("GBWIZARD:{not json}").is_none());
    assert!(parse_progress_line("GBWIZARD:").is_none());
    assert!(parse_progress_line("").is_none());
}

#[test]
fn only_the_final_event_ends_the_run() {
    let event = |id: &str, state: &str| super::ProgressEvent {
        id: id.to_string(),
        state: state.to_string(),
        message: String::new(),
        elapsed_ms: 0,
    };
    assert!(is_terminal(&event("components", "ready")));
    assert!(is_terminal(&event("vault", "failed")));
    assert!(!is_terminal(&event("vault", "ready")));
    assert!(!is_terminal(&event("components", "starting")));
    assert!(!is_terminal(&event("botserver", "ready")));
}

#[test]
fn the_stderr_tail_never_splits_a_character() {
    let mut short = String::from("ok\n");
    trim_stderr_tail(&mut short);
    assert_eq!(short, "ok\n");

    // A one-byte prefix plus 5000 two-byte characters: the raw byte cut
    // lands in the middle of a character, so it has to move forward.
    let mut buffer = format!("a{}", "é".repeat(5000));
    let before = buffer.len();
    assert!(before > STDERR_MAX_BYTES);
    trim_stderr_tail(&mut buffer);
    assert!(buffer.len() <= STDERR_MAX_BYTES);
    assert!(buffer.chars().all(|c| c == 'é'));
    assert!(!buffer.is_empty());
}
