//! Live check of [`super::open_duplex`] through the real Incus
//! command (on Windows, the WSL wrapper). Ignored by default. It needs a
//! throwaway container with python3, a user `duplex` and the fake peer
//! (`src/duplex/test_peer.py`) at `/usr/local/lib/gb-duplex-peer.py`:
//!
//! `GB_DUPLEX_LIVE_VM=<container> AGENT_INCUS_BIN=<incus> cargo test -p botagent --lib duplex_live -- --ignored --nocapture`

use std::time::{Duration, Instant};

use botlib::security::command_guard::SafeCommand;
use serde_json::json;

use super::open_duplex;
use crate::duplex::{
    DuplexEvent, DuplexOptions, DuplexSession, EndReason, Incoming, RpcClient, RpcOptions,
    METHOD_NOT_FOUND,
};

const WAIT: Duration = Duration::from_secs(60);

fn live_target() -> (String, String) {
    let bin = std::env::var("AGENT_INCUS_BIN").unwrap_or_else(|_| "incus".to_string());
    let vm =
        std::env::var("GB_DUPLEX_LIVE_VM").expect("GB_DUPLEX_LIVE_VM names a throwaway container");
    (bin, vm)
}

fn open(bin: &str, vm: &str, mode: &str) -> DuplexSession {
    open_with(bin, vm, mode, DuplexOptions::default())
}

fn open_with(bin: &str, vm: &str, mode: &str, options: DuplexOptions) -> DuplexSession {
    let script = format!(
        "su duplex -s /bin/sh -c 'exec python3 -u /usr/local/lib/gb-duplex-peer.py {mode}'"
    );
    let args = ["sh".to_string(), "-c".to_string(), script];
    let exec = SafeCommand::new(bin).expect("incus is allowed");
    let reaper = SafeCommand::new(bin).expect("incus is allowed");
    open_duplex(exec, reaper, vm, &args, options).expect("incus exec starts")
}

async fn line(session: &mut DuplexSession) -> String {
    match tokio::time::timeout(WAIT, session.recv()).await {
        Ok(Some(DuplexEvent::Line(line))) => line,
        other => panic!("expected a line, got {other:?}"),
    }
}

fn peers_left(bin: &str, vm: &str, pattern: &str) -> String {
    let output = SafeCommand::new(bin)
        .and_then(|command| command.args(&["exec", vm, "--", "pgrep", "-af"]))
        .and_then(|command| command.trusted_arg(pattern))
        .and_then(|command| command.execute())
        .expect("pgrep runs");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container"]
async fn duplex_live_echo_and_rpc_through_the_wrapper() {
    let (bin, vm) = live_target();

    let started = Instant::now();
    let mut session = open(&bin, &vm, "echo");
    session.send("hello").expect("send");
    assert_eq!(line(&mut session).await, "ECHO hello");
    eprintln!(
        "live: first round trip incl. spawn {} ms",
        started.elapsed().as_millis()
    );

    let text = "ação, coração, São Paulo — 日本語 — 🚀 — tab\tend";
    session.send(text).expect("send");
    assert_eq!(line(&mut session).await, format!("ECHO {text}"));
    let big = text.repeat(1024 * 1024 / text.len() + 1);
    let sent = Instant::now();
    session.send(&big).expect("send");
    assert_eq!(line(&mut session).await, format!("ECHO {big}"));
    eprintln!(
        "live: {} byte line round trip {} ms",
        big.len(),
        sent.elapsed().as_millis()
    );

    let mut rtts = Vec::new();
    for n in 0..50 {
        let sent = Instant::now();
        session
            .send(&format!("{{\"id\":{n},\"t\":\"ç\"}}"))
            .expect("send");
        assert_eq!(
            line(&mut session).await,
            format!("ECHO {{\"id\":{n},\"t\":\"ç\"}}")
        );
        rtts.push(sent.elapsed().as_secs_f64() * 1000.0);
    }
    rtts.sort_by(f64::total_cmp);
    eprintln!(
        "live: 50 round trips p50 {:.2} ms, max {:.2} ms",
        rtts[25], rtts[49]
    );

    let closing = Instant::now();
    let end = session.close().await;
    eprintln!(
        "live: close {:?} exit {:?} in {} ms",
        end.reason,
        end.exit_code,
        closing.elapsed().as_millis()
    );
    assert_eq!(end.reason, EndReason::Closed);
    assert_eq!(end.exit_code, Some(0));

    let (client, mut inbox) = RpcClient::start(
        open(&bin, &vm, "rpc"),
        RpcOptions::accepting(["item/approve"]),
    );
    let echoed = client
        .request("echo", json!({"q": "ação"}), WAIT)
        .await
        .expect("echo");
    assert_eq!(echoed, json!({"q": "ação"}));
    let ask = client.request("ask", json!({"method": "item/approve"}), WAIT);
    let handle = async {
        match tokio::time::timeout(WAIT, inbox.next()).await {
            Ok(Some(Incoming::Request(request))) => {
                request
                    .respond(json!({"decision": "accept"}))
                    .expect("answer");
            }
            other => panic!("expected a server request, got {other:?}"),
        }
    };
    let (seen, ()) = tokio::join!(ask, handle);
    assert_eq!(seen.expect("ask")["result"]["decision"], json!("accept"));
    let refused = client
        .request("ask", json!({"method": "weird/thing"}), WAIT)
        .await
        .expect("ask");
    assert_eq!(refused["error"]["code"], json!(METHOD_NOT_FOUND));
    let end = client.close().await;
    eprintln!("live: rpc close {:?} exit {:?}", end.reason, end.exit_code);
    assert_eq!(end.reason, EndReason::Closed);

    tokio::time::sleep(Duration::from_secs(1)).await;
    let left = peers_left(&bin, &vm, "gb-duplex-peer.py [er]");
    eprintln!(
        "live: peer processes left in the container: {}",
        if left.is_empty() { "none" } else { &left }
    );
    assert!(left.is_empty());
}

#[tokio::test]
#[ignore = "needs a throwaway Incus container"]
async fn duplex_live_deadline_kills_inside_the_container() {
    let (bin, vm) = live_target();
    // The deadline counts from spawn, a WSL cold start included: warm up first.
    peers_left(&bin, &vm, "gb-duplex-peer.py stubborn");
    let options = DuplexOptions {
        deadline: Duration::from_secs(5),
        ..DuplexOptions::default()
    };
    let mut session = open_with(&bin, &vm, "stubborn /home/duplex/beat", options);
    assert_eq!(line(&mut session).await, "ready");
    let end = match tokio::time::timeout(WAIT, session.recv()).await {
        Ok(Some(DuplexEvent::Ended(end))) => end,
        other => panic!("expected the end, got {other:?}"),
    };
    eprintln!(
        "live: deadline end {:?} exit {:?} after {} ms",
        end.reason,
        end.exit_code,
        end.elapsed.as_millis()
    );
    assert_eq!(end.reason, EndReason::Deadline);
    let left = peers_left(&bin, &vm, "gb-duplex-peer.py stubborn");
    eprintln!(
        "live: peer processes left when Ended arrived: {}",
        if left.is_empty() { "none" } else { &left }
    );
    assert!(left.is_empty());
}
