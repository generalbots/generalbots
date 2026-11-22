//! Live check of the bridge against the real `codex app-server` in a
//! throwaway container, with the self-check's loopback stand-in model (no
//! account, nothing leaves the container) and a scripted gate standing in
//! for the person. Ignored by default. The container needs the `codex` user
//! and the pinned CLI:
//!
//! `GB_CODEX_LIVE_VM=<container> AGENT_INCUS_BIN=<incus> cargo test -p botagent --lib codex_live -- --ignored --nocapture`

use std::time::Duration;

use botlib::security::command_guard::SafeCommand;

use super::app_server::HANDLED_REQUESTS;
use super::bridge::{check_account, handshake, open_thread, run_turn, BridgeError};
use super::bridge_turn::{launch_script, store_script, stored_thread};
use super::bridge_tests::Script;
use super::selfcheck::{probe, session_script, stage_script};
use crate::duplex::{DuplexOptions, DuplexSession, RpcClient, RpcInbox, RpcOptions};

const WAIT: Duration = Duration::from_secs(120);
const WORK: &str = "/home/codex/.gb-selfcheck/work";

fn target() -> (String, String) {
    let bin = std::env::var("AGENT_INCUS_BIN").unwrap_or_else(|_| "incus".to_string());
    let vm = std::env::var("GB_CODEX_LIVE_VM").expect("GB_CODEX_LIVE_VM names a throwaway container");
    (bin, vm)
}

fn incus(bin: &str, vm: &str, flags: &[&str], script: &str) -> SafeCommand {
    SafeCommand::new(bin)
        .and_then(|command| command.args(&["exec", vm]))
        .and_then(|command| command.args(flags))
        .and_then(|command| command.args(&["--", "sh", "-c"]))
        .and_then(|command| command.trusted_shell_script_arg(script))
        .expect("incus is allowed")
}

fn run(bin: &str, vm: &str, script: &str) -> String {
    let output = incus(bin, vm, &[], script).execute().expect("incus exec runs");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn open(bin: &str, vm: &str) -> (RpcClient, RpcInbox) {
    open_script(bin, vm, &session_script())
}

fn open_script(bin: &str, vm: &str, script: &str) -> (RpcClient, RpcInbox) {
    let session = DuplexSession::spawn(&incus(bin, vm, &["-T"], script), DuplexOptions::default())
        .expect("incus exec starts");
    RpcClient::start(session, RpcOptions::accepting(HANDLED_REQUESTS))
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container with the pinned Codex CLI"]
async fn codex_live_bridge_allows_then_denies() {
    let (bin, vm) = target();
    assert_eq!(run(&bin, &vm, &format!("{} && echo STAGED", stage_script())), "STAGED");
    run(&bin, &vm, &format!("rm -f {WORK}/denied.txt"));

    let (client, mut inbox) = open(&bin, &vm);
    let checked = tokio::time::timeout(WAIT, probe(&client, &mut inbox)).await.expect("self-check in time");
    println!("self-check: {checked:?}");
    assert_eq!(checked.map(|(_, asked)| asked), Ok(true), "Codex asks before running a command");
    client.close().await;

    let (client, mut inbox) = open(&bin, &vm);
    let version = handshake(&client).await.expect("initialize");
    let (thread, new) = open_thread(&client, None, WORK, "", None).await.expect("thread/start");
    assert!(new);
    let gate = Script::with(&["allow", "deny"]);
    let allowed = tokio::time::timeout(WAIT, run_turn(&client, &mut inbox, &thread, "CMD: echo live-ok-ç", "", &gate))
        .await
        .expect("first turn in time");
    println!("version {version}, allowed turn: {allowed:?}");
    let denied = tokio::time::timeout(
        WAIT,
        run_turn(&client, &mut inbox, &thread, &format!("CMD: touch {WORK}/denied.txt"), "", &gate),
    )
    .await
    .expect("second turn in time");
    println!("denied turn: {denied:?}");
    for line in gate.log() {
        println!("gate: {line}");
    }
    client.close().await;
    let log = gate.log();
    assert!(allowed.is_ok() && denied.is_ok());
    assert!(log[0].starts_with("ask command ") && log[0].contains("echo live-ok-ç"));
    assert!(log[1].contains("ok=true exit=Some(0) out=live-ok-ç"), "{}", log[1]);
    assert!(log[2].starts_with("ask command ") && log[2].contains("denied.txt"));
    assert_eq!(log.len(), 3, "a denied command reports nothing");
    assert_eq!(run(&bin, &vm, &format!("test -e {WORK}/denied.txt && echo RAN || echo NOT-RUN")), "NOT-RUN");

    let (client, mut inbox) = open(&bin, &vm);
    handshake(&client).await.expect("initialize");
    let resumed = open_thread(&client, Some(&thread), WORK, "", None).await.expect("thread/resume");
    println!("resumed: {resumed:?}");
    assert_eq!(resumed, (thread.clone(), false));
    let gate = Script::with(&[]);
    let again = tokio::time::timeout(WAIT, run_turn(&client, &mut inbox, &thread, "olá", "", &gate))
        .await
        .expect("resumed turn in time");
    println!("resumed turn: {again:?}");
    assert!(again.is_ok());
    client.close().await;
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container with the pinned Codex CLI"]
async fn codex_live_chat_session_reads_its_thread_and_account() {
    let (bin, vm) = target();
    assert_eq!(run(&bin, &vm, &format!("{} && echo STORED", store_script("live-conv", "01a2-live"))), "STORED");
    let (client, mut inbox) = open_script(&bin, &vm, &launch_script("live-conv"));
    let version = handshake(&client).await.expect("initialize");
    assert_eq!(stored_thread(&mut inbox).as_deref(), Some("01a2-live"));
    let account = check_account(&client).await;
    println!("version {version}, account without sign-in: {account:?}");
    assert_eq!(account, Err(BridgeError::NotSignedIn));
    client.close().await;
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container with the pinned Codex CLI"]
async fn codex_live_file_change_asks_per_path() {
    let (bin, vm) = target();
    assert_eq!(run(&bin, &vm, &format!("{} && rm -f {WORK}/patched.txt && echo STAGED", stage_script())), "STAGED");
    let (client, mut inbox) = open(&bin, &vm);
    handshake(&client).await.expect("initialize");
    let (thread, _) = open_thread(&client, None, WORK, "", None).await.expect("thread/start");
    let gate = Script::with(&["allow"]);
    let prompt = format!("PATCH: {WORK}/patched.txt");
    let done = tokio::time::timeout(WAIT, run_turn(&client, &mut inbox, &thread, &prompt, "", &gate))
        .await
        .expect("patch turn in time");
    println!("patch turn: {done:?}");
    for line in gate.log() {
        println!("gate: {line}");
    }
    client.close().await;
    let log = gate.log();
    assert!(log[0].starts_with("ask write ") && log[0].ends_with("patched.txt"), "{}", log[0]);
    assert!(log[1].contains("ok=true") && log[1].contains("patched.txt"), "{}", log[1]);
    assert_eq!(run(&bin, &vm, &format!("cat {WORK}/patched.txt")), "patched");
}
