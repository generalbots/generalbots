//! Bridge tests against a scripted Codex app server
//! (`test_app_server.py`), spawned through `SafeCommand`: no workspace.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;

use super::app_server::{ActionOutcome, HANDLED_REQUESTS};
use super::bridge::{check_account, handshake, open_thread, run_turn, Approvals, BridgeError, Verdict};
use crate::duplex::{DuplexOptions, DuplexSession, RpcClient, RpcInbox, RpcOptions};

const PEER: &str = include_str!("test_app_server.py");
const WAIT: Duration = Duration::from_secs(30);

/// The script is longer than one guarded argument, so it runs from a file.
fn peer_path() -> &'static std::path::Path {
    static PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("gb-codex-peer-{}.py", std::process::id()));
        std::fs::write(&path, PEER).expect("the fake app server is written");
        path
    })
}

fn start(mode: &str) -> (RpcClient, RpcInbox) {
    let program = if cfg!(windows) { "python" } else { "python3" };
    let command = SafeCommand::new(program)
        .and_then(|command| command.path_arg(peer_path()))
        .and_then(|command| command.arg(mode))
        .expect("the fake app server command is accepted by the guard");
    let session = DuplexSession::spawn(&command, DuplexOptions::default()).expect("the fake app server starts");
    RpcClient::start(session, RpcOptions::accepting(HANDLED_REQUESTS))
}

/// A gate that answers from a script and records what it was asked.
#[derive(Default)]
pub(super) struct Script {
    verdicts: Mutex<VecDeque<&'static str>>,
    log: Mutex<Vec<String>>,
    pub(super) text: Mutex<String>,
    waiting_dropped: Arc<AtomicBool>,
}

struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl Script {
    pub(super) fn with(verdicts: &[&'static str]) -> Self {
        Self { verdicts: Mutex::new(verdicts.iter().copied().collect()), ..Self::default() }
    }

    pub(super) fn log(&self) -> Vec<String> {
        self.log.lock().expect("log").clone()
    }

    async fn decide(&self, asked: String) -> Verdict<String> {
        self.log.lock().expect("log").push(format!("ask {asked}"));
        let next = self.verdicts.lock().expect("verdicts").pop_front().unwrap_or("deny");
        match next {
            "allow" => Verdict::Allow(asked),
            "cancel" => Verdict::Cancel,
            "hang" => {
                let _flag = DropFlag(self.waiting_dropped.clone());
                std::future::pending::<()>().await;
                Verdict::Cancel
            }
            _ => Verdict::Decline,
        }
    }
}

impl Approvals for Script {
    type Ticket = String;

    async fn command(&self, command: &str, cwd: Option<&str>) -> Verdict<String> {
        self.decide(format!("command {command} in {}", cwd.unwrap_or("-"))).await
    }

    async fn file(&self, path: &str, delete: bool) -> Verdict<String> {
        self.decide(format!("{} {path}", if delete { "delete" } else { "write" })).await
    }

    async fn finished(&self, ticket: String, outcome: ActionOutcome) {
        self.log.lock().expect("log").push(format!(
            "finished {ticket} ok={} exit={:?} out={} files={}",
            outcome.ok,
            outcome.exit_code,
            outcome.output.trim(),
            outcome.files.join(",")
        ));
    }

    async fn text(&self, chunk: &str) {
        self.text.lock().expect("text").push_str(chunk);
    }

    async fn noted(&self, kind: &str, text: &str, ok: bool) {
        self.log.lock().expect("log").push(format!("noted {kind} {text} ok={ok}"));
    }

    fn stopped(&self) -> bool {
        false
    }
}

async fn one_turn(mode: &str, gate: &Script) -> Result<String, BridgeError> {
    let (client, mut inbox) = start(mode);
    assert_eq!(handshake(&client).await.expect("initialize"), "0.171.0");
    check_account(&client).await.expect("signed in");
    let (thread, new) = open_thread(&client, None, "/home/codex", "", None).await.expect("thread");
    assert!(new);
    let result = tokio::time::timeout(WAIT, run_turn(&client, &mut inbox, &thread, "olá", "", gate))
        .await
        .expect("the turn ends in time");
    client.close().await;
    result
}

#[tokio::test]
async fn an_allowed_command_runs_and_its_result_is_reported() {
    let gate = Script::with(&["allow"]);
    let answer = one_turn("allow", &gate).await.expect("answer");
    assert_eq!(answer, "decision=accept");
    assert_eq!(*gate.text.lock().expect("text"), "decision=accept", "deltas stream once");
    let ask = "command echo olá in /home/codex";
    assert_eq!(gate.log(), vec![format!("ask {ask}"), format!("finished {ask} ok=true exit=Some(0) out=olá files=")]);
}

#[tokio::test]
async fn a_denied_command_is_declined_and_not_reported() {
    let gate = Script::with(&["deny"]);
    assert_eq!(one_turn("allow", &gate).await.expect("answer"), "decision=decline");
    assert_eq!(gate.log().len(), 1);
}

#[tokio::test]
async fn an_unanswered_card_cancels_the_turn() {
    let gate = Script::with(&["cancel"]);
    let result = one_turn("allow", &gate).await;
    assert_eq!(result, Ok("decision=cancel".to_string()));
}

#[tokio::test]
async fn every_path_of_a_file_change_asks() {
    let gate = Script::with(&["allow", "allow"]);
    assert_eq!(one_turn("file", &gate).await.expect("answer"), "decision=accept");
    let log = gate.log();
    assert_eq!(log[0], "ask write /home/codex/a.txt");
    assert_eq!(log[1], "ask delete /home/codex/b.txt");
    assert_eq!(log.iter().filter(|l| l.starts_with("finished")).count(), 2);
    assert!(log[2].contains("ok=true") && log[2].contains("files=/home/codex/a.txt,/home/codex/b.txt"));
}

#[tokio::test]
async fn one_refused_path_declines_the_whole_change() {
    let gate = Script::with(&["allow", "deny"]);
    assert_eq!(one_turn("file", &gate).await.expect("answer"), "decision=decline");
    let log = gate.log();
    assert_eq!(log.len(), 3);
    assert!(log[2].starts_with("finished write /home/codex/a.txt ok=false"));
}

#[tokio::test]
async fn an_unknown_server_request_gets_an_error() {
    let gate = Script::default();
    assert_eq!(one_turn("unknown", &gate).await.expect("answer"), "reply=-32601");
    assert!(gate.log().is_empty());
}

#[tokio::test]
async fn a_session_that_ends_mid_approval_withdraws_the_card() {
    let gate = Script::with(&["hang"]);
    match one_turn("vanish", &gate).await {
        Err(BridgeError::Failed(_, message)) => assert!(message.contains("ended"), "{message}"),
        other => panic!("expected a failure, got {other:?}"),
    }
    assert!(gate.waiting_dropped.load(Ordering::SeqCst), "the open card's wait was dropped");
}

#[tokio::test]
async fn a_spent_plan_becomes_the_plan_limit_error() {
    match one_turn("limit", &Script::default()).await {
        Err(BridgeError::Failed(code, message)) => {
            assert_eq!(code, StatusCode::TOO_MANY_REQUESTS);
            assert!(message.starts_with(crate::plan_limit::MARKER), "{message}");
        }
        other => panic!("expected the plan limit, got {other:?}"),
    }
}

#[tokio::test]
async fn an_action_that_ran_unasked_stops_the_turn() {
    assert_eq!(one_turn("unasked", &Script::default()).await, Err(BridgeError::Unasked));
}

#[tokio::test]
async fn no_account_is_not_signed_in() {
    let (client, _inbox) = start("noauth");
    handshake(&client).await.expect("initialize");
    assert_eq!(check_account(&client).await, Err(BridgeError::NotSignedIn));
}

#[tokio::test]
async fn the_stored_thread_is_resumed_and_a_lost_one_replaced() {
    let (client, mut inbox) = start("resume");
    handshake(&client).await.expect("initialize");
    let stored = super::bridge_turn::stored_thread(&mut inbox);
    assert_eq!(stored.as_deref(), Some("01a2-stored"));
    let resumed = open_thread(&client, stored.as_deref(), "/home/codex", "", None).await.expect("resume");
    assert_eq!(resumed, ("01a2-stored".to_string(), false));
    let replaced = open_thread(&client, Some("01a2-gone"), "/home/codex", "", None).await.expect("start");
    assert_eq!(replaced, ("01a2-new".to_string(), true));
    let answer = run_turn(&client, &mut inbox, "01a2-stored", "só a última", "", &Script::default()).await;
    assert_eq!(answer, Ok("got=só a última".to_string()));
}

#[tokio::test]
async fn the_self_check_tells_asking_from_unasked() {
    let (client, mut inbox) = start("allow");
    assert_eq!(super::selfcheck::probe(&client, &mut inbox).await, Ok(("0.171.0".to_string(), true)));
    let (client, mut inbox) = start("unasked");
    assert_eq!(super::selfcheck::probe(&client, &mut inbox).await, Ok(("0.171.0".to_string(), false)));
}
