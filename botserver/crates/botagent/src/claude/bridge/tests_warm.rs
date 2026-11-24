//! A warm stream-json session across turns, against a scripted Claude Code
//! that takes several user messages (`test_warm_peer.py`): each turn's
//! `can_use_tool` reaches that turn's gate, one written between turns is
//! denied, and a dead process never starts a turn.

use std::sync::Mutex;
use std::time::Duration;

use botlib::security::command_guard::SafeCommand;

use super::drive::{drive_on, OpenSession};
use super::warm::{refuse_idle, ClaudeWarm};
use super::{BridgeGate, ClaudeAction, DriveReport, GateAnswer, ToolOutcome, ToolRequest};
use crate::duplex::{DuplexOptions, DuplexSession};
use crate::warm::{WarmProcess, IDLE_REFUSAL};

const PEER: &str = include_str!("test_warm_peer.py");

struct Gate {
    allow: bool,
    asked: Mutex<Vec<String>>,
    text: Mutex<String>,
    thoughts: Mutex<Vec<String>>,
}

impl Gate {
    fn new(allow: bool) -> Self {
        Self { allow, asked: Mutex::new(Vec::new()), text: Mutex::new(String::new()), thoughts: Mutex::new(Vec::new()) }
    }
}

impl BridgeGate for Gate {
    async fn decide(&self, request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        let ClaudeAction::Command { command } = action else { return GateAnswer::Deny("no".into()) };
        self.asked.lock().expect("lock").push(format!("{} {command}", request.tool_name));
        if self.allow {
            GateAnswer::Allow
        } else {
            GateAnswer::Deny("denied".into())
        }
    }

    async fn finished(&self, _tool_use_id: &str, _outcome: &ToolOutcome) {}

    async fn text(&self, chunk: &str) -> bool {
        self.text.lock().expect("lock").push_str(chunk);
        true
    }

    async fn thinking(&self, chunk: &str) {
        self.thoughts.lock().expect("lock").push(chunk.to_string());
    }

    fn stopped(&self) -> bool {
        false
    }
}

fn spawn(mode: &str) -> ClaudeWarm {
    let program = if cfg!(windows) { "python" } else { "python3" };
    let command = SafeCommand::new(program)
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_arg(PEER))
        .and_then(|c| c.arg(mode))
        .expect("the peer command is accepted");
    let session = DuplexSession::spawn(&command, DuplexOptions::default()).expect("the peer starts");
    let (sender, events) = session.into_parts();
    ClaudeWarm { sender, events, session_id: String::new(), with_folder: false }
}

async fn turn(process: &mut ClaudeWarm, first: bool, gate: &Gate) -> DriveReport {
    let open = OpenSession { sender: &process.sender, events: &mut process.events, initialize: first, keep: true };
    tokio::time::timeout(Duration::from_secs(30), drive_on(open, "do it", &[], gate, None))
        .await
        .expect("the turn ends in time")
}

#[tokio::test]
async fn each_message_on_a_warm_session_asks_its_own_gate() {
    let mut process = spawn("plain");
    let first = Gate::new(true);
    let report = turn(&mut process, true, &first).await;
    assert!(report.kept, "the session stays open after the result");
    assert!(process.alive());
    assert_eq!(first.text.lock().expect("lock").as_str(), "turn 1 allow stray=None");
    assert_eq!(first.thoughts.lock().expect("lock").as_slice(), &["pondering 1".to_string()]);

    let second = Gate::new(false);
    let report = turn(&mut process, false, &second).await;
    assert!(report.kept);
    assert_eq!(second.text.lock().expect("lock").as_str(), "turn 2 deny stray=None");
    assert_eq!(first.asked.lock().expect("lock").as_slice(), &["Bash echo 1".to_string()]);
    assert_eq!(second.asked.lock().expect("lock").as_slice(), &["Bash echo 2".to_string()]);
    process.shut_down();
}

#[tokio::test]
async fn a_request_written_between_turns_is_denied_and_reaches_no_gate() {
    let mut process = spawn("stray");
    let first = Gate::new(true);
    turn(&mut process, true, &first).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(refuse_idle(&mut process), "the session is still usable");

    let second = Gate::new(true);
    turn(&mut process, false, &second).await;
    assert_eq!(second.text.lock().expect("lock").as_str(), "turn 2 allow stray=deny");
    assert_eq!(second.asked.lock().expect("lock").as_slice(), &["Bash echo 2".to_string()]);
    assert!(!first.asked.lock().expect("lock").iter().any(|a| a.contains("rm -rf")));
    assert!(!IDLE_REFUSAL.is_empty());
    process.shut_down();
}

#[tokio::test]
async fn a_dead_session_hears_nothing_so_the_turn_falls_back() {
    let mut process = spawn("plain");
    process.sender.close();
    process.sender.wait_ended().await;
    assert!(!refuse_idle(&mut process));
    let gate = Gate::new(true);
    let report = turn(&mut process, false, &gate).await;
    assert!(!report.heard && !report.kept);
    assert!(gate.asked.lock().expect("lock").is_empty());
}
