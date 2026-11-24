//! A warm app server across turns, against the scripted Codex app server:
//! each turn's requests reach that turn's gate, a request sent between turns
//! is refused, and a dead process is never reused.

use std::time::Duration;

use super::bridge::{check_account, handshake, open_thread};
use super::bridge_tests::{start, Script};
use super::turn_loop::run_turn_tracked;
use super::warm::{refuse_idle, CodexWarm};
use crate::duplex::INTERNAL_ERROR;
use crate::warm::WarmProcess;

const WAIT: Duration = Duration::from_secs(30);

async fn opened(mode: &str) -> CodexWarm {
    let (client, inbox) = start(mode);
    let version = handshake(&client).await.expect("initialize");
    check_account(&client).await.expect("signed in");
    let (thread_id, _) = open_thread(&client, None, "/home/codex", "", None).await.expect("thread");
    CodexWarm { client, inbox, thread_id, version }
}

async fn turn(process: &mut CodexWarm, gate: &Script) -> super::turn_loop::TurnEnd {
    let thread = process.thread_id.clone();
    tokio::time::timeout(WAIT, run_turn_tracked(&process.client, &mut process.inbox, &thread, "olá", &[], "", gate))
        .await
        .expect("the turn ends in time")
}

#[tokio::test]
async fn each_turn_on_a_warm_process_asks_its_own_gate() {
    let mut process = opened("allow").await;
    let first = Script::with(&["allow"]);
    let end = turn(&mut process, &first).await;
    assert!(end.started && end.completed, "the first turn completes and leaves the process usable");
    assert_eq!(end.result.expect("answer"), "decision=accept");
    assert!(refuse_idle(&mut process.inbox));
    assert!(process.alive());

    let second = Script::with(&["deny"]);
    let end = turn(&mut process, &second).await;
    assert!(end.completed);
    assert_eq!(end.result.expect("answer"), "decision=decline");
    assert_eq!(first.log().len(), 2, "the first gate saw its ask and its result only");
    assert_eq!(second.log(), vec!["ask command echo olá in /home/codex".to_string()]);
    process.shut_down();
}

#[tokio::test]
async fn a_request_sent_between_turns_is_refused_and_reaches_no_gate() {
    let mut process = opened("stray").await;
    let first = Script::with(&["allow"]);
    let end = turn(&mut process, &first).await;
    assert_eq!(end.result.expect("answer"), "got=olá stray=None");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(refuse_idle(&mut process.inbox), "the process is still usable");

    let second = Script::with(&["allow"]);
    let end = turn(&mut process, &second).await;
    assert_eq!(end.result.expect("answer"), format!("got=olá stray={INTERNAL_ERROR}"));
    assert!(first.log().is_empty() && second.log().is_empty(), "no gate was asked about the stray request");
    process.shut_down();
}

#[tokio::test]
async fn a_process_that_ended_is_not_alive_and_its_turn_never_starts() {
    let mut process = opened("allow").await;
    process.client.sender().close();
    process.client.sender().wait_ended().await;
    assert!(!process.alive());
    assert!(!refuse_idle(&mut process.inbox) || !process.alive());
    let gate = Script::with(&["allow"]);
    let end = turn(&mut process, &gate).await;
    assert!(!end.started, "a turn that never started may fall back to a fresh process");
    assert!(end.result.is_err());
    assert!(gate.log().is_empty());
}
