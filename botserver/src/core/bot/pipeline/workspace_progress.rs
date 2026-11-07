//! User-visible workspace stages (#1807).
//!
//! On a slow machine the agent VM can spend minutes pulling an image,
//! starting the container, installing packages and running the generated
//! program. Those stages were only written to the sidecar log. This module
//! turns them into `STEP_STARTED` / `STEP_FINISHED` frames on the existing
//! chat run stream, with a stable phase/step name, elapsed time and detail.
//!
//! The frames are emitted through `event_sink`, so a chat turn persists them
//! in `run_events` and a reconnect can replay the timeline. The tracker only
//! keeps the start instant in memory; the durable frame already carries the
//! elapsed time once the step finishes.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;

use super::event_sink::{self, RunContext};
use super::sink::ChannelSink;
use super::types::SessionIdentity;

/// Phases from the issue contract: infrastructure, agent generation or a
/// workspace tool (`package_install`, `execute`, `save`).
pub const PHASE_INFRA: &str = "infra";
pub const PHASE_AGENT: &str = "agent";
pub const PHASE_TOOL: &str = "tool";

fn started_steps() -> &'static Mutex<HashMap<String, Instant>> {
    static STARTED: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    STARTED.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

/// Emits workspace stages for one chat turn. `None` when the build or channel
/// cannot carry run frames, so callers do not pay for a no-op pipeline.
pub struct WorkspaceProgress<'a> {
    sink: &'a dyn ChannelSink,
    run: RunContext,
}

impl<'a> WorkspaceProgress<'a> {
    pub fn new(identity: &SessionIdentity<'a>) -> Option<Self> {
        if !event_sink::can_emit(identity.sink) {
            return None;
        }
        let thread_id = identity.session_id.to_string();
        let run = RunContext::new(&thread_id, "chat")
            .with_state(identity.state.clone())
            .with_scope(identity.bot_uuid, identity.user_id, None);
        Some(Self {
            sink: identity.sink,
            run,
        })
    }

    /// Starts a step. Re-starting the same phase/step keeps the original
    /// instant so a retry does not hide how long the stage has been running.
    pub async fn start(&self, phase: &str, step: &str, label: &str, detail: &str) {
        let key = self.key(phase, step);
        if let Ok(mut started) = started_steps().lock() {
            started.entry(key).or_insert_with(Instant::now);
        }
        let extra = json!({
            "phase": phase,
            "step": step,
            "detail": detail,
            "started_at": now_ms(),
        });
        event_sink::step_started_with(
            self.sink,
            &self.run,
            &step_id(phase, step),
            label,
            None,
            extra,
        )
        .await;
    }

    /// Finishes a step with `ok` (or `error`) and the elapsed milliseconds.
    pub async fn finish(&self, phase: &str, step: &str, status: &str, detail: &str) {
        let elapsed = started_steps()
            .lock()
            .ok()
            .and_then(|mut started| started.remove(&self.key(phase, step)))
            .map(|instant| instant.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        let extra = json!({
            "phase": phase,
            "step": step,
            "detail": detail,
            "elapsed_ms": elapsed,
            "finished_at": now_ms(),
            "name": label_for(phase, step),
        });
        event_sink::step_finished_with(
            self.sink,
            &self.run,
            &step_id(phase, step),
            status,
            elapsed,
            None,
            extra,
        )
        .await;
    }

    fn key(&self, phase: &str, step: &str) -> String {
        format!("{}:{}:{}", self.run.run_id, phase, step)
    }
}

fn step_id(phase: &str, step: &str) -> String {
    format!("{phase}:{step}")
}

/// Human label used when a completion frame arrives without the start frame
/// (for example after a reconnect that resumed mid-step).
fn label_for(phase: &str, step: &str) -> &'static str {
    match (phase, step) {
        (PHASE_INFRA, "queued") => "Workspace queued",
        (PHASE_INFRA, "image_pull") => "Pulling the workspace image",
        (PHASE_INFRA, "container_start") => "Starting the workspace",
        (PHASE_INFRA, "bootstrap") => "Preparing the workspace",
        (PHASE_TOOL, "package_install") => "Installing packages",
        (PHASE_TOOL, "execute") => "Running the program",
        (PHASE_TOOL, "save") => "Saving the output",
        (PHASE_TOOL, "publish") => "Publishing to Drive",
        (PHASE_AGENT, "generate") => "Generating the program",
        (_, "done") => "Done",
        (_, "failed") => "Failed",
        _ => "Workspace step",
    }
}
