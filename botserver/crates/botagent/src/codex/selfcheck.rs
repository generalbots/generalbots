//! The approval self-check: before the first bridged turn of a CLI version
//! in a workspace, a scratch Codex home with a loopback stand-in model
//! proposes the harmless command `true` under the bridge's approval policy.
//! Codex must ask before running it. If it runs it unasked, that version is
//! refused for chat turns (`BridgeError::Unsafe`) instead of acting without
//! the person's decision.
//!
//! The check needs no account, sends nothing outside the workspace and
//! never touches the person's own Codex home or threads.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use base64::Engine;
use botsecurity_core::error_sanitizer::SafeErrorResponse;
use serde_json::Value;

use crate::duplex::{DuplexOptions, Incoming, RpcClient, RpcInbox, RpcOptions};
use crate::{vm, AgentService};

use super::app_server::{self as proto, Answer, COMMAND_APPROVAL, HANDLED_REQUESTS};
use super::bridge::{handshake, BridgeError};
use super::paths::CODEX_USER;

const MODEL_SCRIPT: &str = include_str!("selfcheck_model.py");
const MODEL_PATH: &str = "/usr/local/lib/gb/codex-selfcheck-model.py";
const CHECK_HOME: &str = "/home/codex/.gb-selfcheck";
const CHECK_CWD: &str = "/home/codex/.gb-selfcheck/work";
const CHECK_CONFIG: &str = "model = \"mock-model\"\n\
model_provider = \"gbcheck\"\n\
\n\
[model_providers.gbcheck]\n\
name = \"gbcheck\"\n\
base_url = \"http://127.0.0.1:18766/v1\"\n\
wire_api = \"responses\"\n\
request_max_retries = 0\n\
stream_max_retries = 0\n";
const PROBE_COMMAND: &str = "CMD: true";
const PROBE_WAIT: Duration = Duration::from_secs(120);
const STAGE_TIMEOUT: Duration = Duration::from_secs(60);
const REQUEST_WAIT: Duration = Duration::from_secs(60);

/// `vm name → (version, asks)`, for this server process.
fn results() -> &'static Mutex<HashMap<String, (String, bool)>> {
    static RESULTS: OnceLock<Mutex<HashMap<String, (String, bool)>>> = OnceLock::new();
    RESULTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn running() -> &'static tokio::sync::Mutex<()> {
    static RUNNING: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    RUNNING.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn cached(vm_name: &str, version: &str) -> Option<bool> {
    let results = results().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    results
        .get(vm_name)
        .filter(|(checked, _)| checked == version)
        .map(|(_, asks)| *asks)
}

fn remember(vm_name: &str, version: &str, asks: bool) {
    results()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(vm_name.to_string(), (version.to_string(), asks));
}

/// A bridged turn saw an action run unasked: refuse this version from now on.
pub fn mark_unsafe(vm_name: &str, version: &str) {
    remember(vm_name, version, false);
}

fn verdict(asks: bool, version: &str) -> Result<(), BridgeError> {
    if asks {
        Ok(())
    } else {
        Err(BridgeError::Unsafe { version: version.to_string() })
    }
}

/// Passes when `version` in `vm_name` asks before acting; runs the check
/// once per workspace and version.
pub async fn ensure(state: &AgentService, vm_name: &str, version: &str) -> Result<(), BridgeError> {
    if let Some(asks) = cached(vm_name, version) {
        return verdict(asks, version);
    }
    let _one_at_a_time = running().lock().await;
    if let Some(asks) = cached(vm_name, version) {
        return verdict(asks, version);
    }
    let (checked, asks) = run_check(state, vm_name).await?;
    if checked != version && asks {
        return Err(inconclusive(&format!("checked {checked}, session runs {version}")));
    }
    remember(vm_name, version, asks);
    verdict(asks, version)
}

/// Runs the check ahead of the first turn (after a server start or a sign-in)
/// and keeps the verdict for the version it found, so that turn does not wait
/// for it. Does nothing when this workspace was already checked.
pub async fn precheck(state: &AgentService, vm_name: &str) -> Result<(), BridgeError> {
    if checked_any(vm_name) {
        return Ok(());
    }
    let _one_at_a_time = running().lock().await;
    if checked_any(vm_name) {
        return Ok(());
    }
    let (checked, asks) = run_check(state, vm_name).await?;
    remember(vm_name, &checked, asks);
    verdict(asks, &checked)
}

fn checked_any(vm_name: &str) -> bool {
    results()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .contains_key(vm_name)
}

/// One run of the check: the version the scratch session reported and
/// whether it asked before running the probe command.
async fn run_check(state: &AgentService, vm_name: &str) -> Result<(String, bool), BridgeError> {
    stage(state, vm_name).await?;
    let options = DuplexOptions { deadline: PROBE_WAIT + Duration::from_secs(30), ..DuplexOptions::default() };
    let session = vm::exec_in_vm_duplex(state, vm_name, &["sh".into(), "-c".into(), session_script()], options)
        .await
        .map_err(BridgeError::Failed)?;
    let (client, mut inbox) = RpcClient::start(session, RpcOptions::accepting(HANDLED_REQUESTS));
    let probed = tokio::time::timeout(PROBE_WAIT, probe(&client, &mut inbox)).await;
    client.close().await;
    match probed {
        Ok(Ok((checked, asks))) => {
            tracing::info!("codex self-check: version {checked} asks before acting: {asks}");
            Ok((checked, asks))
        }
        Ok(Err(why)) => Err(inconclusive(&why)),
        Err(_) => Err(inconclusive("no result in time")),
    }
}

fn inconclusive(why: &str) -> BridgeError {
    tracing::warn!("codex self-check inconclusive: {why}");
    BridgeError::Failed(SafeErrorResponse::new(
        "service_unavailable",
        "Could not verify that Codex asks before acting in the workspace",
    ))
}

pub(crate) fn stage_script() -> String {
    let b64 = |text: &str| base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    format!(
        "umask 022 && mkdir -p /usr/local/lib/gb && printf '%s' {model} | base64 -d > {MODEL_PATH} && \
         umask 077 && mkdir -p {CHECK_CWD} && printf '%s' {config} | base64 -d > {CHECK_HOME}/config.toml && \
         chown -R {CODEX_USER}:{CODEX_USER} {CHECK_HOME}",
        model = b64(MODEL_SCRIPT),
        config = b64(CHECK_CONFIG),
    )
}

pub(crate) fn session_script() -> String {
    format!(
        "exec su {CODEX_USER} -s /bin/sh -c 'cd {CHECK_CWD} && export HOME=/home/codex CODEX_HOME={CHECK_HOME} && \
         (python3 {MODEL_PATH} >/dev/null 2>&1 &) && exec codex app-server'"
    )
}

async fn stage(state: &AgentService, vm_name: &str) -> Result<(), BridgeError> {
    match vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), stage_script()], STAGE_TIMEOUT).await {
        Ok((true, _, _)) => Ok(()),
        Ok(_) => Err(inconclusive("could not stage the check")),
        Err((_, message)) => Err(BridgeError::failed(message)),
    }
}

/// Drives the check session: `Ok((version, asked))`, where `asked` is
/// false when the command ran without an approval request.
pub(crate) async fn probe(client: &RpcClient, inbox: &mut RpcInbox) -> Result<(String, bool), String> {
    let version = handshake(client).await.map_err(|e| format!("{e:?}"))?;
    let thread = client
        .request("thread/start", proto::thread_start_params(CHECK_CWD, "", true), REQUEST_WAIT)
        .await
        .map_err(|e| format!("thread/start: {e}"))?;
    let thread_id = proto::thread_id_of(&thread).ok_or("thread/start returned no thread id")?;
    client
        .request("turn/start", proto::turn_start_params(&thread_id, PROBE_COMMAND, ""), REQUEST_WAIT)
        .await
        .map_err(|e| format!("turn/start: {e}"))?;
    let mut asked = false;
    while let Some(message) = inbox.next().await {
        match message {
            Incoming::Request(request) => {
                asked |= request.method == COMMAND_APPROVAL;
                if let Err(error) = request.respond(Answer::Decline.result()) {
                    return Err(format!("answer not delivered: {error}"));
                }
            }
            Incoming::Notification { method, params } => {
                let item = params.get("item").unwrap_or(&Value::Null);
                let kind = item.get("type").and_then(Value::as_str);
                let status = item.get("status").and_then(Value::as_str).unwrap_or("");
                if method == "item/completed" && kind == Some("commandExecution") && !asked && status != "declined" {
                    return Ok((version, false));
                }
                if method == "turn/completed" {
                    return if asked { Ok((version, true)) } else { Err("no command was proposed".to_string()) };
                }
            }
            Incoming::Unparsed(_) => {}
            Incoming::Ended(_) => return Err("the check session ended early".to_string()),
        }
    }
    Err("the check session ended early".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_fits_the_command_guard_and_stays_off_tmp() {
        assert!(stage_script().len() < 16_384);
        assert!(!stage_script().contains("/tmp") && !session_script().contains("/tmp"));
        assert!(session_script().contains("CODEX_HOME=/home/codex/.gb-selfcheck"));
        assert!(!session_script().contains("su -"));
    }

    #[test]
    fn a_failed_check_is_remembered_per_version() {
        mark_unsafe("vm-test-a", "9.9.9");
        assert_eq!(cached("vm-test-a", "9.9.9"), Some(false));
        assert_eq!(cached("vm-test-a", "9.9.10"), None);
        assert!(checked_any("vm-test-a"));
        assert!(!checked_any("vm-test-never-checked"));
        assert_eq!(verdict(false, "9.9.9"), Err(BridgeError::Unsafe { version: "9.9.9".to_string() }));
    }
}
