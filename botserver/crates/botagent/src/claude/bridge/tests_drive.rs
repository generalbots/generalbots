//! The driver against a scripted Claude Code (`test_peer.py`) over a real
//! duplex session.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use botlib::security::command_guard::SafeCommand;

use super::{drive, interpret, BridgeGate, ClaudeAction, DriveReport, GateAnswer, ToolOutcome, ToolRequest, UNKNOWN_TOOL};
use crate::duplex::{DuplexOptions, DuplexSession};

const PEER: &str = include_str!("test_peer.py");

#[derive(Default)]
struct Recorder {
    deny: Option<String>,
    stop_after_text: bool,
    stopped: AtomicBool,
    decided: Mutex<Vec<ClaudeAction>>,
    finished: Mutex<Vec<(String, ToolOutcome)>>,
    text: Mutex<String>,
}

impl BridgeGate for Recorder {
    async fn decide(&self, _request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        self.decided.lock().expect("lock").push(action.clone());
        match &self.deny {
            Some(message) => GateAnswer::Deny(message.clone()),
            None => GateAnswer::Allow,
        }
    }

    async fn finished(&self, tool_use_id: &str, outcome: &ToolOutcome) {
        self.finished.lock().expect("lock").push((tool_use_id.to_string(), outcome.clone()));
    }

    async fn text(&self, chunk: &str) -> bool {
        self.text.lock().expect("lock").push_str(chunk);
        if self.stop_after_text {
            self.stopped.store(true, Ordering::SeqCst);
        }
        true
    }

    fn stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}

async fn run(mode: &str, gate: &Recorder) -> DriveReport {
    let program = if cfg!(windows) { "python" } else { "python3" };
    let command = SafeCommand::new(program)
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_arg(PEER))
        .and_then(|c| c.arg(mode))
        .expect("the peer command is accepted");
    let session = DuplexSession::spawn(&command, DuplexOptions::default()).expect("the peer starts");
    tokio::time::timeout(Duration::from_secs(30), drive(session, "list the folder", gate))
        .await
        .expect("the turn ends in time")
}

#[tokio::test]
async fn an_allowed_command_runs_and_its_result_is_reported() {
    let gate = Recorder::default();
    let report = run("flow", &gate).await;
    assert_eq!(gate.decided.lock().expect("lock").as_slice(), &[ClaudeAction::Command { command: "ls /nope".into() }]);
    let finished = gate.finished.lock().expect("lock").clone();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].0, "toolu_1");
    assert_eq!(finished[0].1, ToolOutcome { ok: false, exit_code: Some(2), output: "ls: cannot access '/nope'".into() });
    assert_eq!(gate.text.lock().expect("lock").as_str(), "Working on: list the folder\n\nDone.");
    assert_eq!(report.session_id.as_deref(), Some("sess-1"));
    assert!(report.unasked.is_empty());
    let answer = interpret(report).expect("a good turn");
    assert_eq!(answer.answer, "Working on: list the folder\n\nDone.");
}

#[tokio::test]
async fn a_denied_command_tells_the_model_and_reports_nothing() {
    let gate = Recorder { deny: Some("PERMISSION DENIED by the person".into()), ..Recorder::default() };
    let report = run("flow", &gate).await;
    assert!(gate.finished.lock().expect("lock").is_empty());
    assert!(report.answer.ends_with("Denied: PERMISSION DENIED by the person"));
    assert!(interpret(report).is_ok());
}

#[tokio::test]
async fn unknown_control_requests_and_tools_are_refused_without_asking() {
    let gate = Recorder::default();
    let report = run("unknown", &gate).await;
    assert!(gate.decided.lock().expect("lock").is_empty());
    assert_eq!(report.answer, "peer saw: error/h1/deny");
    assert_eq!(report.asked, vec!["CronCreate".to_string()]);
    assert!(UNKNOWN_TOOL.starts_with("NOT RUN"));
}

#[tokio::test]
async fn a_session_that_ends_mid_request_closes_the_allowed_step_and_fails() {
    let gate = Recorder::default();
    let report = run("midway", &gate).await;
    assert_eq!(
        gate.decided.lock().expect("lock").as_slice(),
        &[ClaudeAction::WriteFile { path: "/home/claude/notes.txt".into(), edit: false }]
    );
    let finished = gate.finished.lock().expect("lock").clone();
    assert_eq!(finished.len(), 1);
    assert!(!finished[0].1.ok);
    assert!(report.result.is_none());
    assert_eq!(interpret(report).map(|_| ()).unwrap_err().error, "internal_error");
}

#[tokio::test]
async fn a_tool_that_ran_without_asking_is_recorded() {
    let gate = Recorder::default();
    let report = run("unasked", &gate).await;
    assert_eq!(report.unasked, vec!["Bash".to_string()]);
    assert!(!super::selfcheck::verdict(false, &report.unasked));
}

#[tokio::test]
async fn stop_sends_an_interrupt_and_ends_the_turn_quietly() {
    let gate = Recorder { stop_after_text: true, ..Recorder::default() };
    let report = run("interrupt", &gate).await;
    assert!(report.stopped);
    assert_eq!(report.result.as_ref().map(|r| r.subtype.as_str()), Some("error_during_execution"));
    assert!(interpret(report).expect("a stopped turn is not an error").stopped);
}

#[tokio::test]
async fn the_cli_sign_in_message_is_not_streamed_and_becomes_not_signed_in() {
    let gate = Recorder::default();
    let report = run("synthetic", &gate).await;
    assert!(gate.text.lock().expect("lock").is_empty());
    let error = interpret(report).map(|_| ()).unwrap_err();
    assert_eq!(error.error, "unauthorized");
}
