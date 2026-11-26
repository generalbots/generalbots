//! Session tests against a fake peer: a small Python program spawned
//! through `SafeCommand` instead of `incus`, so no workspace is needed.

use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use botlib::security::command_guard::SafeCommand;

use super::supervise::{read_line_capped, LineRead, StderrTail};
use super::{DuplexError, DuplexEvent, DuplexOptions, DuplexSession, EndReason, SessionEnd};

const PEER: &str = include_str!("test_peer.py");
const WAIT: Duration = Duration::from_secs(20);

pub(super) fn peer_command(mode: &str, arg: Option<&str>) -> SafeCommand {
    let program = if cfg!(windows) { "python" } else { "python3" };
    let command = SafeCommand::new(program)
        .and_then(|command| command.arg("-c"))
        .and_then(|command| command.trusted_arg(PEER))
        .and_then(|command| command.arg(mode));
    match arg {
        Some(arg) => command.and_then(|command| command.trusted_arg(arg)),
        None => command,
    }
    .expect("the test peer command is accepted by the guard")
}

pub(super) fn spawn_peer(mode: &str, arg: Option<&str>, options: DuplexOptions) -> DuplexSession {
    DuplexSession::spawn(&peer_command(mode, arg), options).expect("the test peer starts")
}

async fn next(session: &mut DuplexSession) -> DuplexEvent {
    tokio::time::timeout(WAIT, session.recv())
        .await
        .expect("an event in time")
        .expect("an event before the end")
}

async fn line(session: &mut DuplexSession) -> String {
    match next(session).await {
        DuplexEvent::Line(line) => line,
        DuplexEvent::Ended(end) => panic!("expected a line, the session ended: {end:?}"),
    }
}

async fn ended(session: &mut DuplexSession) -> SessionEnd {
    match next(session).await {
        DuplexEvent::Ended(end) => end,
        DuplexEvent::Line(line) => panic!("expected the end, got a line of {} bytes", line.len()),
    }
}

fn heartbeat_path() -> PathBuf {
    std::env::temp_dir().join(format!("gb-duplex-beat-{}", uuid::Uuid::new_v4()))
}

fn size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

/// The peer wrote its heartbeat and has now stopped writing it.
async fn assert_heartbeat_stops(path: &Path) {
    let give_up = Instant::now() + Duration::from_secs(8);
    loop {
        let before = size(path);
        tokio::time::sleep(Duration::from_millis(400)).await;
        if size(path) == before {
            assert!(before > 0, "the peer never wrote its heartbeat");
            let _ = std::fs::remove_file(path);
            return;
        }
        assert!(Instant::now() < give_up, "the peer is still running");
    }
}

fn small(cap: usize, grace_ms: u64) -> DuplexOptions {
    DuplexOptions {
        max_line_bytes: cap,
        close_grace: Duration::from_millis(grace_ms),
        ..DuplexOptions::default()
    }
}

#[test]
fn capped_reader_splits_lines_across_chunks() {
    let mut reader = BufReader::with_capacity(2, Cursor::new(b"one\r\ntwo\nlast".to_vec()));
    let mut buffer = Vec::new();
    let mut lines = Vec::new();
    while read_line_capped(&mut reader, &mut buffer, 16).expect("read") == LineRead::Line {
        lines.push(String::from_utf8(buffer.clone()).expect("utf-8"));
    }
    assert_eq!(lines, ["one\r", "two", "last"]);
}

#[test]
fn capped_reader_accepts_the_cap_and_refuses_one_more_byte() {
    let mut buffer = Vec::new();
    let mut at_cap = BufReader::with_capacity(3, Cursor::new(b"abcd\n".to_vec()));
    assert_eq!(
        read_line_capped(&mut at_cap, &mut buffer, 4).expect("read"),
        LineRead::Line
    );
    assert_eq!(buffer, b"abcd");
    let mut over = BufReader::with_capacity(3, Cursor::new(b"abcde\n".to_vec()));
    assert_eq!(
        read_line_capped(&mut over, &mut buffer, 4).expect("read"),
        LineRead::TooLong
    );
    assert!(buffer.len() <= 4, "never buffered past the cap");
}

#[test]
fn stderr_tail_is_bounded_and_starts_on_a_line() {
    let mut tail = StderrTail::new(12);
    tail.push(b"first line\nsecond\nthird\n");
    assert_eq!(tail.text(), "third");
    let mut short = StderrTail::new(64);
    short.push("erro: ação\n".as_bytes());
    assert_eq!(short.text(), "erro: ação");
}

#[tokio::test]
async fn round_trip_keeps_order_and_utf8_bytes() {
    let mut session = spawn_peer("echo", None, DuplexOptions::default());
    let text = "ação, coração, São Paulo — 日本語 — 🚀 — tab\tend";
    let big: String = text.repeat(1024 * 1024 / text.len() + 1);
    session.send("hello").expect("send");
    session.send(text).expect("send");
    session.send(&big).expect("send");
    session
        .send_json(&serde_json::json!({"q": "ç"}))
        .expect("send");
    assert_eq!(line(&mut session).await, "ECHO hello");
    assert_eq!(line(&mut session).await, format!("ECHO {text}"));
    assert_eq!(line(&mut session).await, format!("ECHO {big}"));
    assert_eq!(line(&mut session).await, "ECHO {\"q\":\"ç\"}");
    let end = session.close().await;
    assert_eq!(end.reason, EndReason::Closed);
    assert_eq!(end.exit_code, Some(0));
    assert!(
        end.stderr_tail.contains("stdin closed"),
        "{}",
        end.stderr_tail
    );
}

#[tokio::test]
async fn outgoing_line_with_a_newline_is_refused() {
    let session = spawn_peer("echo", None, small(16, 300));
    assert!(matches!(
        session.send("a\nb"),
        Err(DuplexError::InvalidLine(_))
    ));
    assert!(matches!(
        session.send("a\rb"),
        Err(DuplexError::InvalidLine(_))
    ));
    assert!(matches!(
        session.send(&"x".repeat(17)),
        Err(DuplexError::InvalidLine(_))
    ));
    assert_eq!(session.close().await.reason, EndReason::Closed);
}

#[tokio::test]
async fn line_at_the_cap_is_delivered() {
    let cap = 64 * 1024;
    let mut session = spawn_peer("big", Some(&cap.to_string()), small(cap, 300));
    assert_eq!(line(&mut session).await.len(), cap);
    assert_eq!(session.close().await.reason, EndReason::Closed);
}

#[tokio::test]
async fn line_over_the_cap_ends_the_session() {
    let cap = 64 * 1024;
    let mut session = spawn_peer("big", Some(&(cap + 1).to_string()), small(cap, 300));
    let end = ended(&mut session).await;
    assert_eq!(end.reason, EndReason::LineTooLong);
    assert!(session.recv().await.is_none(), "nothing after the end");
}

#[tokio::test]
async fn peer_exit_is_signalled_with_code_and_stderr() {
    let mut session = spawn_peer("exit3", None, DuplexOptions::default());
    let sender = session.sender();
    assert_eq!(line(&mut session).await, "bye");
    let end = ended(&mut session).await;
    assert_eq!(end.reason, EndReason::PeerExited);
    assert_eq!(end.exit_code, Some(3));
    assert!(
        end.stderr_tail.contains("failing on purpose"),
        "{}",
        end.stderr_tail
    );
    assert_eq!(
        sender.ended().map(|end| end.reason),
        Some(EndReason::PeerExited)
    );
    assert_eq!(sender.wait_ended().await.exit_code, Some(3));
    assert_eq!(
        sender.send("late"),
        Err(DuplexError::Ended(EndReason::PeerExited))
    );
}

#[tokio::test]
async fn deadline_kills_the_peer() {
    let beat = heartbeat_path();
    let options = DuplexOptions {
        deadline: Duration::from_millis(1500),
        ..DuplexOptions::default()
    };
    let mut session = spawn_peer("stubborn", beat.to_str(), options);
    assert_eq!(line(&mut session).await, "ready");
    let end = ended(&mut session).await;
    assert_eq!(end.reason, EndReason::Deadline);
    assert_eq!(end.exit_code, None);
    assert!(end.elapsed >= Duration::from_millis(1500));
    assert_heartbeat_stops(&beat).await;
}

#[tokio::test]
async fn close_kills_a_peer_that_ignores_end_of_file() {
    let beat = heartbeat_path();
    let mut session = spawn_peer("stubborn", beat.to_str(), small(1024, 300));
    assert_eq!(line(&mut session).await, "ready");
    let closing = Instant::now();
    let end = session.close().await;
    assert_eq!(end.reason, EndReason::Closed);
    assert_eq!(end.exit_code, None, "killed, not exited");
    assert!(closing.elapsed() >= Duration::from_millis(300));
    assert_heartbeat_stops(&beat).await;
}

#[tokio::test]
async fn dropping_the_session_ends_the_peer() {
    let beat = heartbeat_path();
    let mut session = spawn_peer("stubborn", beat.to_str(), small(1024, 300));
    assert_eq!(line(&mut session).await, "ready");
    drop(session);
    assert_heartbeat_stops(&beat).await;
}

#[cfg(windows)]
#[tokio::test]
async fn kill_takes_the_whole_process_tree() {
    let beat = heartbeat_path();
    let mut session = spawn_peer("tree", beat.to_str(), small(1024, 300));
    assert_eq!(line(&mut session).await, "ready");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(session.close().await.reason, EndReason::Closed);
    assert_heartbeat_stops(&beat).await;
}

#[tokio::test]
async fn cleanup_runs_before_the_end_is_published() {
    let marker = heartbeat_path();
    let program = if cfg!(windows) { "python" } else { "python3" };
    let cleanup = SafeCommand::new(program)
        .and_then(|command| command.arg("-c"))
        .and_then(|command| {
            command.trusted_arg("import sys; open(sys.argv[1], 'w').write('reaped')")
        })
        .and_then(|command| command.trusted_arg(marker.to_str().unwrap_or_default()))
        .expect("cleanup command");
    let mut session = DuplexSession::spawn_with_cleanup(
        &peer_command("exit3", None),
        DuplexOptions::default(),
        cleanup,
    )
    .expect("the test peer starts");
    assert_eq!(line(&mut session).await, "bye");
    assert_eq!(ended(&mut session).await.reason, EndReason::PeerExited);
    assert_eq!(
        std::fs::read_to_string(&marker).ok().as_deref(),
        Some("reaped")
    );
    let _ = std::fs::remove_file(&marker);
}
