//! Claude Code kept warm between the turns of one conversation: the
//! stream-json session stays open after a turn's `result` and the next turn
//! of the same conversation writes its user message to the same stdin.
//!
//! Every `can_use_tool` is still read and answered inside a turn, by that
//! turn's gate: one written while the process waited between turns is
//! denied ([`refuse_idle`]) before the next turn's message is sent.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use botsecurity_core::error_sanitizer::SafeErrorResponse;

use crate::duplex::{DuplexEvents, DuplexOptions, DuplexSender};
use crate::warm::observe::TurnClock;
use crate::warm::{self, Warm, WarmKey, WarmPool, WarmProcess, IDLE_REFUSAL};
use crate::AgentService;

use super::actions::ClaudeAction;
use super::drive::{drive_on, BridgeGate, DriveReport, GateAnswer, OpenSession, ToolOutcome};
use super::protocol::{self, Incoming, ToolRequest};
use super::{command, BridgedTurn};

/// Longest life of one Claude Code process.
const LIFETIME: Duration = Duration::from_secs(2 * 60 * 60);
/// Longest single turn (the person's time on cards counts too).
const TURN_DEADLINE: Duration = Duration::from_secs(30 * 60);

/// One warm `claude -p --input-format stream-json` session.
pub struct ClaudeWarm {
    pub(super) sender: DuplexSender,
    pub(super) events: DuplexEvents,
    pub(super) session_id: String,
    /// Started by a turn with attachments, so its conversation folder was
    /// reachable (`--add-dir`) from the start.
    pub(super) with_folder: bool,
}

impl WarmProcess for ClaudeWarm {
    fn alive(&self) -> bool {
        self.sender.ended().is_none()
    }

    fn shut_down(self) {
        self.sender.close();
    }
}

static POOL: LazyLock<WarmPool<ClaudeWarm>> =
    LazyLock::new(|| WarmPool::new("claude", warm::limits_for(LIFETIME, TURN_DEADLINE)));

/// The Claude Code warm processes.
pub fn warm_pool() -> &'static WarmPool<ClaudeWarm> {
    &POOL
}

/// Denies every permission request written while no turn ran and drops the
/// rest. `false` when the process ended.
pub(super) fn refuse_idle(process: &mut ClaudeWarm) -> bool {
    while let Some(line) = process.events.try_recv_line() {
        for incoming in protocol::parse(&line) {
            let reply = match incoming {
                Incoming::CanUseTool { request_id, .. } => protocol::deny(&request_id, IDLE_REFUSAL),
                Incoming::OtherControl { request_id, .. } => protocol::unsupported(&request_id),
                _ => continue,
            };
            tracing::warn!("claude warm: a control request arrived between turns; refused");
            if let Err(error) = process.sender.send_json(&reply) {
                tracing::debug!("claude warm: could not send the refusal: {error}");
            }
        }
    }
    process.alive()
}

fn fingerprint(turn: &BridgedTurn<'_>) -> String {
    warm::fingerprint(&[&crate::cli_turn::cli_model(turn.model)])
}

/// The conversation's warm process, when it can take this turn.
fn take(vm_name: &str, turn: &BridgedTurn<'_>) -> Option<Warm<ClaudeWarm>> {
    let conversation = turn.conversation?;
    let resume = turn.resume.and_then(command::clean_session_id)?;
    let mut process = warm_pool().take(&WarmKey::new(vm_name, conversation), &fingerprint(turn), Instant::now())?;
    let usable = process.process.session_id == resume
        && (turn.attachments.is_empty() || process.process.with_folder)
        && refuse_idle(&mut process.process);
    if usable {
        return Some(process);
    }
    tracing::info!("claude warm: process not reused (other session, new folder or ended); starting a fresh one");
    process.process.shut_down();
    None
}

/// Keeps the process when the turn ended well with its `result`.
fn settle(vm_name: &str, turn: &BridgedTurn<'_>, mut process: Warm<ClaudeWarm>, report: &DriveReport) {
    let clean = report.kept
        && !report.stopped
        && report.unasked.is_empty()
        && report.result.as_ref().is_some_and(|r| !r.is_error);
    match (clean, turn.conversation, report.session_id.as_deref().and_then(command::clean_session_id)) {
        (true, Some(conversation), Some(id)) => {
            process.process.session_id = id.to_string();
            warm_pool().put(WarmKey::new(vm_name, conversation), fingerprint(turn), process, Instant::now());
            warm::keep_reaped(warm_pool());
        }
        _ => process.process.shut_down(),
    }
}

/// Runs the turn on the conversation's warm process. `None` when there is
/// none, or when it failed before saying anything (the caller starts a
/// fresh one).
pub(super) async fn run_warm(
    vm_name: &str,
    turn: &BridgedTurn<'_>,
    prompt: &str,
    gate: &impl BridgeGate,
) -> Option<DriveReport> {
    let mut process = take(vm_name, turn)?;
    let clock = TurnClock::start("claude", true);
    let observed = Observed { inner: gate, clock: &clock };
    clock.prompted();
    let open = OpenSession { sender: &process.process.sender, events: &mut process.process.events, initialize: false, keep: true };
    let report = drive_on(open, prompt, turn.attachments, &observed, Some(TURN_DEADLINE)).await;
    crate::plan_usage::record_claude(vm_name, &report.rate_limits);
    if !report.heard && !clock.touched() {
        tracing::info!("claude warm: the warm process failed before the turn began; starting a fresh one");
        process.process.shut_down();
        return None;
    }
    settle(vm_name, turn, process, &report);
    Some(report)
}

/// Starts the CLI for this turn and runs it, keeping the process when the
/// turn ends well.
pub(super) async fn run_fresh(
    state: &AgentService,
    vm_name: &str,
    turn: &BridgedTurn<'_>,
    resume: Option<&str>,
    prompt: &str,
    gate: &impl BridgeGate,
) -> Result<DriveReport, SafeErrorResponse> {
    let clock = TurnClock::start("claude", false);
    let script = command::turn_script(turn.model, resume, turn.conversation);
    let args = ["sh".to_string(), "-c".to_string(), script];
    let options = DuplexOptions { deadline: LIFETIME, ..DuplexOptions::default() };
    let session = crate::vm::exec_in_vm_duplex(state, vm_name, &args, options).await?;
    let born = Instant::now();
    clock.ready();
    let (sender, events) = session.into_parts();
    let mut process = ClaudeWarm { sender, events, session_id: String::new(), with_folder: !turn.attachments.is_empty() };
    let observed = Observed { inner: gate, clock: &clock };
    clock.prompted();
    let open = OpenSession { sender: &process.sender, events: &mut process.events, initialize: true, keep: true };
    let report = drive_on(open, prompt, turn.attachments, &observed, Some(TURN_DEADLINE)).await;
    crate::plan_usage::record_claude(vm_name, &report.rate_limits);
    settle(vm_name, turn, Warm { process, born }, &report);
    Ok(report)
}

/// The turn's own gate, with the turn's timing.
struct Observed<'a, G> {
    inner: &'a G,
    clock: &'a TurnClock,
}

impl<G: BridgeGate> BridgeGate for Observed<'_, G> {
    async fn decide(&self, request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        self.clock.touch("permission request");
        self.inner.decide(request, action).await
    }

    async fn finished(&self, tool_use_id: &str, outcome: &ToolOutcome) {
        self.inner.finished(tool_use_id, outcome).await;
    }

    async fn text(&self, chunk: &str) -> bool {
        self.clock.touch("text");
        self.inner.text(chunk).await
    }

    async fn thinking(&self, chunk: &str) {
        self.clock.touch("thought");
        self.inner.thinking(chunk).await;
    }

    fn stopped(&self) -> bool {
        self.inner.stopped()
    }
}
