//! `agy acp` kept warm between the turns of one conversation: the process
//! and its open session (already in the asking mode, model selected). The
//! next turn of the same conversation sends `session/prompt` on it instead
//! of starting the CLI and resuming the session again.
//!
//! Every `session/request_permission` is still read and answered inside a
//! turn, by that turn's gate: one sent while the process waited between
//! turns is refused before the next prompt.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::codex::app_server::ActionOutcome;
use crate::codex::Verdict;
use crate::duplex::{RpcClient, RpcInbox};
use crate::warm::observe::TurnClock;
use crate::warm::{self, Warm, WarmKey, WarmPool, WarmProcess};

use super::proto::{AcpAction, AgentInfo};
use super::turn::{self, AcpGate, PromptEnd};
use super::{check, AcpError, AcpTurn, BRIDGE_DEADLINE};

/// Longest life of one `agy acp` process (its session deadline).
pub(super) const LIFETIME: Duration = Duration::from_secs(2 * 60 * 60);

/// One warm `agy acp` with its session open.
pub struct AcpWarm {
    pub(super) client: RpcClient,
    pub(super) inbox: RpcInbox,
    pub(super) session_id: String,
    pub(super) info: AgentInfo,
}

impl WarmProcess for AcpWarm {
    fn alive(&self) -> bool {
        self.client.sender().ended().is_none()
    }

    fn shut_down(self) {
        let Self { client, .. } = self;
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    client.close().await;
                });
            }
            Err(_) => drop(client),
        }
    }
}

static POOL: LazyLock<WarmPool<AcpWarm>> =
    LazyLock::new(|| WarmPool::new("antigravity", warm::limits_for(LIFETIME, BRIDGE_DEADLINE)));

/// The Antigravity warm processes.
pub fn warm_pool() -> &'static WarmPool<AcpWarm> {
    &POOL
}

pub(super) fn fingerprint(turn: &AcpTurn<'_>) -> String {
    warm::fingerprint(&[&crate::cli_turn::cli_model(turn.model)])
}

/// Runs the turn on the conversation's warm process. `None` when there is
/// none, or when it failed before the agent said anything (the caller
/// starts a fresh one).
pub(super) async fn run_warm<H: AcpGate>(vm_name: &str, key: &WarmKey, turn: &AcpTurn<'_>, handler: &H) -> Option<Result<String, AcpError>> {
    let fingerprint = fingerprint(turn);
    let mut process = warm_pool().take(key, &fingerprint, Instant::now())?;
    if !warm::refuse_idle_rpc(&mut process.process.inbox, "antigravity") {
        process.process.shut_down();
        return None;
    }
    if let Err(error) = check::ensure(vm_name, &process.process.info) {
        process.process.shut_down();
        return Some(Err(error));
    }
    let clock = TurnClock::start("antigravity", true);
    let text = if turn.latest.trim().is_empty() { turn.transcript } else { turn.latest };
    let end = prompt(&mut process.process, text, turn, handler, &clock).await;
    if end.result.is_err() && !end.heard && !end.answered && !clock.touched() {
        tracing::info!("antigravity warm: the warm process failed before the turn began; starting a fresh one");
        process.process.shut_down();
        return None;
    }
    Some(settle(end, process, key.clone(), fingerprint, vm_name))
}

/// One prompt on the open session, within [`BRIDGE_DEADLINE`].
pub(super) async fn prompt<H: AcpGate>(
    process: &mut AcpWarm,
    text: &str,
    turn: &AcpTurn<'_>,
    handler: &H,
    clock: &TurnClock,
) -> PromptEnd {
    let observed = Observed { inner: handler, clock };
    let input = turn::PromptInput {
        session_id: &process.session_id,
        text,
        images: turn.images,
        resources: turn.resources,
        info: &process.info,
    };
    clock.prompted();
    let run = turn::run_prompt_tracked(&process.client, &mut process.inbox, input, &observed);
    match tokio::time::timeout(BRIDGE_DEADLINE, run).await {
        Ok(end) => end,
        Err(_) => PromptEnd {
            result: Err(AcpError::Failed(
                axum::http::StatusCode::GATEWAY_TIMEOUT,
                "Antigravity passed its time limit".to_string(),
            )),
            heard: true,
            answered: false,
        },
    }
}

/// Keeps the process when the prompt answered well, ends it otherwise; an
/// action that ran unasked marks the version unsafe.
pub(super) fn settle(
    end: PromptEnd,
    process: Warm<AcpWarm>,
    key: WarmKey,
    fingerprint: String,
    vm_name: &str,
) -> Result<String, AcpError> {
    match end.result {
        Ok(text) if end.answered => {
            warm_pool().put(key, fingerprint, process, Instant::now());
            warm::keep_reaped(warm_pool());
            Ok(text)
        }
        Err(AcpError::Unasked) => {
            let version = process.process.info.version.clone();
            process.process.shut_down();
            check::mark_unsafe(vm_name, &version);
            Err(AcpError::Unsafe { version })
        }
        other => {
            process.process.shut_down();
            other
        }
    }
}

/// The turn's own gate, with the turn's timing.
struct Observed<'a, H> {
    inner: &'a H,
    clock: &'a TurnClock,
}

impl<H: AcpGate> AcpGate for Observed<'_, H> {
    type Ticket = H::Ticket;

    async fn decide(&self, action: &AcpAction) -> Verdict<H::Ticket> {
        self.clock.touch("permission request");
        self.inner.decide(action).await
    }

    async fn finished(&self, ticket: H::Ticket, outcome: ActionOutcome) {
        self.inner.finished(ticket, outcome).await;
    }

    async fn noted(&self, action: &AcpAction, ok: bool) {
        self.clock.touch("step");
        self.inner.noted(action, ok).await;
    }

    async fn text(&self, chunk: &str) {
        self.clock.touch("text");
        self.inner.text(chunk).await;
    }

    async fn thought(&self, chunk: &str) {
        self.clock.touch("thought");
        self.inner.thought(chunk).await;
    }

    fn stopped(&self) -> bool {
        self.inner.stopped()
    }
}
