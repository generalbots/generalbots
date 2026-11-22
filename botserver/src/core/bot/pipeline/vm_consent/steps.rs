//! The action's row in the steps block and the Permission activity that
//! opens and closes its card.
//!
//! Frames (AG-UI step frames, `data` fields):
//! - `STEP_STARTED` `{step_id, phase:"tool", step:"vm_action", detail,
//!   action:{agent,kind,text,outside_vm,cwd?,language?,packages?,...},
//!   waiting:true}` when the gate starts, and again with `waiting:false,
//!   allowed_by:"once"|"conversation"|"all"|"person"` once allowed;
//! - `STEP_FINISHED` `{step_id, status:"ok"|"error", phase, step,
//!   allowed_by, output:[≤3 lines], files:[…], exit_code?}` after the run, or
//!   `{…, denied:"denied"|"no_answer"|"cancelled"|"unavailable"}`.

use serde_json::{json, Value};

use super::{AllowSource, VmAction, VmAgent, VmDenial, VmReceipt};
use crate::core::bot::pipeline::activity::{self, Activity, ActivityKind};
use crate::core::bot::pipeline::event_sink::{self, RunContext};
use crate::core::bot::pipeline::types::SessionIdentity;

const PHASE: &str = "tool";
const STEP: &str = "vm_action";

fn run_context(identity: &SessionIdentity<'_>) -> RunContext {
    RunContext::new(&identity.session_id.to_string(), "chat")
        .with_state(identity.state.clone())
        .with_scope(identity.bot_uuid, identity.user_id, None)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn label(action: &VmAction) -> String {
    action.preview(160)
}

pub(super) async fn begin(identity: &SessionIdentity<'_>, action: &VmAction, hash: &str) -> String {
    let step_id = format!("{PHASE}:{STEP}:{}:{}", &hash[..12], now_ms());
    if event_sink::can_emit(identity.sink) {
        let extra = json!({
            "phase": PHASE, "step": STEP, "detail": label(action),
            "action": action.step_view(), "waiting": true, "started_at": now_ms(),
        });
        event_sink::step_started_with(identity.sink, &run_context(identity), &step_id, &label(action), None, extra).await;
    }
    step_id
}

pub(super) async fn decided(
    identity: &SessionIdentity<'_>,
    action: &VmAction,
    step_id: &str,
    decided: Result<AllowSource, VmDenial>,
) {
    if !event_sink::can_emit(identity.sink) {
        return;
    }
    let run = run_context(identity);
    match decided {
        Ok(source) => {
            let extra = json!({
                "phase": PHASE, "step": STEP, "detail": label(action),
                "action": action.step_view(), "waiting": false, "allowed_by": source.as_str(),
            });
            event_sink::step_started_with(identity.sink, &run, step_id, &label(action), None, extra).await;
        }
        Err(denial) => {
            let extra = json!({
                "phase": PHASE, "step": STEP, "detail": label(action),
                "action": action.step_view(), "denied": denial.as_str(),
            });
            event_sink::step_finished_with(identity.sink, &run, step_id, "error", None, None, extra).await;
        }
    }
}

/// Emits the Permission activity `needs_input`; the shell turns it into the
/// card (`/api/consent/pending`), also after a reload.
pub(super) async fn card_shown(identity: &SessionIdentity<'_>, action: &VmAction, request_id: &str) -> Activity {
    let run = run_context(identity);
    let started = Activity::started(
        ActivityKind::Permission,
        activity::permission_summary(action.kind_key(), "pending", Some(request_id)),
    )
    .with_run_id(&run.run_id);
    let waiting = started.needs_input(activity::permission_summary(action.kind_key(), "pending", Some(request_id)));
    activity::emit_activity(identity.sink, &run, &waiting).await;
    started
}

/// Closes the card: `completed` when allowed or denied, `failed` when nobody
/// answered.
pub(super) async fn card_closed(
    identity: &SessionIdentity<'_>,
    card: Activity,
    action: &VmAction,
    request_id: &str,
    outcome: Result<AllowSource, VmDenial>,
) {
    let run = run_context(identity);
    let closed = match outcome {
        Ok(source) => card.completed(activity::permission_summary(action.kind_key(), source.as_str(), Some(request_id))),
        Err(VmDenial::ByPerson) => card.completed(activity::permission_summary(action.kind_key(), "denied", Some(request_id))),
        Err(denial) => card.failed(denial.as_str()),
    };
    activity::emit_activity(identity.sink, &run, &closed).await;
}

/// How an allowed action ended, for its step row.
#[derive(Debug, Clone, Default)]
pub struct VmOutcome {
    pub ok: bool,
    pub exit_code: Option<i32>,
    /// Output (or error) text; the row keeps the first three lines.
    pub output: String,
    /// Files the action wrote or sent.
    pub files: Vec<String>,
    /// Why it ended this way, a [`super::reason`] code the shell translates.
    pub reason: Option<&'static str>,
}

impl VmOutcome {
    /// From a buffered program run: `Some((ok, stdout, stderr))`, `None`
    /// when the workspace was not available.
    #[must_use]
    pub fn from_run(run: Option<&(bool, String, String)>, failure: Option<&str>) -> Self {
        match run {
            Some((ok, stdout, stderr)) => Self {
                ok: *ok,
                exit_code: ok.then_some(0),
                output: if stdout.trim().is_empty() { stderr.clone() } else { stdout.clone() },
                files: Vec::new(),
                reason: None,
            },
            // The failure text is English and for the model; the step gets a
            // code. `vm_exec` words a spent budget "run budget elapsed".
            None => Self {
                ok: false,
                reason: Some(if failure.is_some_and(|text| text.contains("run budget elapsed")) {
                    super::reason::BUDGET_EXCEEDED
                } else {
                    super::reason::WORKSPACE_UNAVAILABLE
                }),
                ..Self::default()
            },
        }
    }
}

impl VmOutcome {
    /// Uses the exit code the run reported, when the exec path knew it; the
    /// buffered path only knows success, shown as 0.
    #[must_use]
    pub fn with_exit_code(mut self, exit_code: Option<i32>) -> Self {
        if exit_code.is_some() {
            self.exit_code = exit_code;
        }
        self
    }
}

pub(super) fn first_lines(text: &str, count: usize) -> Vec<String> {
    let redact = botsecurity_crypto::secrets::redact_sensitive_data;
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .take(count)
        .map(|line| redact(&line.chars().take(200).collect::<String>()))
        .collect()
}

/// Finishes the action's step: result, first output lines, files and what
/// allowed it. Call it once per allowed action, after it ran.
pub async fn report(identity: &SessionIdentity<'_>, receipt: &VmReceipt, outcome: &VmOutcome) {
    crate::core::bot::pipeline::turn_progress::busy(&receipt.thread_id.to_string(), false);
    if receipt.step_id.is_empty() || !event_sink::can_emit(identity.sink) {
        return;
    }
    let extra: Value = json!({
        "phase": PHASE, "step": STEP,
        "allowed_by": receipt.source.as_str(),
        "output": first_lines(&outcome.output, 3),
        "files": outcome.files.iter().take(8).collect::<Vec<_>>(),
        "reason": outcome.reason,
        "elapsed_ms": receipt.elapsed_ms(),
    });
    let status = if outcome.ok { "ok" } else { "error" };
    event_sink::step_finished_with(
        identity.sink,
        &run_context(identity),
        &receipt.step_id,
        status,
        Some(receipt.elapsed_ms()),
        outcome.exit_code,
        extra,
    )
    .await;
}

/// A step the agent took without asking (Codex has no approval for a web
/// search or an MCP tool call): a started and a finished frame, `unasked`.
pub async fn noted(identity: &SessionIdentity<'_>, agent: VmAgent, kind: &str, text: &str, ok: bool) {
    crate::core::bot::pipeline::turn_progress::touch(&identity.session_id.to_string());
    if !event_sink::can_emit(identity.sink) {
        return;
    }
    let shown = botsecurity_crypto::secrets::redact_sensitive_data(&text.chars().take(600).collect::<String>());
    let detail: String = shown.chars().take(160).collect();
    let step_id = format!("{PHASE}:{STEP}:{kind}:{}", now_ms());
    let action = json!({
        "agent": agent.as_str(), "kind": kind, "text": shown, "outside_vm": kind == "web_search",
    });
    let started = json!({
        "phase": PHASE, "step": STEP, "detail": detail, "action": action,
        "waiting": false, "unasked": true, "started_at": now_ms(),
    });
    let context = run_context(identity);
    event_sink::step_started_with(identity.sink, &context, &step_id, &detail, None, started).await;
    let finished = json!({ "phase": PHASE, "step": STEP, "unasked": true, "output": [], "files": [] });
    let status = if ok { "ok" } else { "error" };
    event_sink::step_finished_with(identity.sink, &context, &step_id, status, Some(0), None, finished).await;
}
