//! Readiness gate for the Incus daemon behind the Windows WSL wrapper.
//!
//! On Windows every Incus call is `incus.exe` -> `wsl.exe ... incus`, and the
//! first call after WSL went idle boots the distro. A short probe killed at
//! its deadline while the distro was still booting coincided with systemd's
//! `incus.service` losing its process to SIGKILL, which left the unit
//! `activating` with no daemon and every later call hanging until it timed
//! out. Calls now wait here instead of racing the boot: one probe at a time,
//! with a deadline that covers a cold boot, checks that the daemon answers.
//! A unit found stuck is restarted once (logged, and visible to callers as
//! [`SERVICE_RESTARTING_CODE`]); a daemon that still does not answer is
//! reported as [`SERVICE_FAILED_CODE`] for a while instead of being hammered.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use std::process::Output;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

use super::incus_run::{capped, output_within, INCUS_TIMEOUT_CODE};
use crate::AgentService;

/// Error code prefix while the gate restarts a stuck `incus.service`.
pub const SERVICE_RESTARTING_CODE: &str = "incus_service_restarting";
/// Error code prefix when the daemon did not answer, even after a restart.
pub const SERVICE_FAILED_CODE: &str = "incus_service_failed";

/// A daemon that answered this recently is not probed again.
const READY_TTL: Duration = Duration::from_secs(5);
/// A failed check is answered from memory this long before the next check.
const FAILED_TTL: Duration = Duration::from_secs(15);
/// How long a call without a tighter budget waits for the check.
const GATE_WAIT: Duration = Duration::from_secs(90);
/// `incus admin waitready --timeout`, inside the distro.
const WAITREADY_SECS: u64 = 30;
/// The wrapper process of `waitready`: the wait plus a cold WSL boot.
const WAITREADY_DEADLINE: Duration = Duration::from_secs(WAITREADY_SECS + 30);
const STATE_DEADLINE: Duration = Duration::from_secs(30);
const RESTART_DEADLINE: Duration = Duration::from_secs(120);
/// At most one automatic restart in this window.
const RESTART_COOLDOWN: Duration = Duration::from_secs(600);
/// Wrapper verb that runs `systemctl` for `incus.service` in the distro.
const SERVICE_VERB: &str = "__gb-service";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Unknown,
    Checking,
    Restarting,
    Ready(Instant),
    Failed(Instant),
}

impl Phase {
    fn settled(&self) -> bool {
        matches!(self, Self::Ready(_) | Self::Failed(_))
    }

    fn needs_check(&self, now: Instant) -> bool {
        match self {
            Self::Unknown => true,
            Self::Checking | Self::Restarting => false,
            Self::Ready(at) => now.saturating_duration_since(*at) >= READY_TTL,
            Self::Failed(at) => now.saturating_duration_since(*at) >= FAILED_TTL,
        }
    }
}

struct Gate {
    phase: watch::Sender<Phase>,
    last_restart: Mutex<Option<Instant>>,
}

static GATE: LazyLock<Gate> = LazyLock::new(|| Gate {
    phase: watch::Sender::new(Phase::Unknown),
    last_restart: Mutex::new(None),
});

/// The guarded Incus command, once the daemon is known to answer.
pub(super) async fn ready_command(state: &AgentService) -> Result<SafeCommand, (StatusCode, String)> {
    let command = super::incus::incus_command(state)?;
    if cfg!(windows) {
        wait_ready(state.incus_bin()).await?;
    }
    Ok(command)
}

async fn wait_ready(bin: &str) -> Result<(), (StatusCode, String)> {
    let now = Instant::now();
    let mut start = false;
    GATE.phase.send_if_modified(|phase| {
        start = phase.needs_check(now);
        if start {
            *phase = Phase::Checking;
        }
        start
    });
    if start {
        let bin = bin.to_string();
        tokio::spawn(async move {
            let settled = tokio::task::spawn_blocking(move || check(&bin))
                .await
                .unwrap_or_else(|error| {
                    tracing::error!("incus readiness check stopped: {error}");
                    Phase::Failed(Instant::now())
                });
            GATE.phase.send_replace(settled);
        });
    }
    let wait = capped(GATE_WAIT);
    let mut receiver = GATE.phase.subscribe();
    let settled = tokio::time::timeout(wait, async {
        receiver.wait_for(Phase::settled).await.map(|phase| *phase)
    })
    .await;
    match settled {
        Ok(Ok(Phase::Failed(_))) => Err(failed_error()),
        Ok(_) => Ok(()),
        Err(_) => {
            let restarting = *GATE.phase.borrow() == Phase::Restarting;
            Err(waiting_error(restarting, wait))
        }
    }
}

/// A call that finished proves the daemon answers: keep the gate open.
pub(super) fn note_answered() {
    let now = Instant::now();
    GATE.phase.send_if_modified(|phase| match phase {
        Phase::Ready(_) => {
            *phase = Phase::Ready(now);
            true
        }
        _ => false,
    });
}

/// A call that ran out of time sends the next one through a new check.
pub(super) fn note_timed_out() {
    GATE.phase.send_if_modified(|phase| match phase {
        Phase::Ready(_) => {
            *phase = Phase::Unknown;
            true
        }
        _ => false,
    });
}

/// The stable code of an error built here, if it is one.
pub fn service_code(error: &(StatusCode, String)) -> Option<&'static str> {
    [SERVICE_RESTARTING_CODE, SERVICE_FAILED_CODE]
        .into_iter()
        .find(|code| error.0 == StatusCode::SERVICE_UNAVAILABLE && error.1.starts_with(code))
}

fn failed_error() -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!(
            "{SERVICE_FAILED_CODE}: the virtual machine service did not answer; restart the computer or WSL"
        ),
    )
}

fn waiting_error(restarting: bool, wait: Duration) -> (StatusCode, String) {
    if restarting {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{SERVICE_RESTARTING_CODE}: the virtual machine service is restarting"),
        );
    }
    (
        StatusCode::GATEWAY_TIMEOUT,
        format!(
            "{INCUS_TIMEOUT_CODE}: the agent workspace did not answer within {} s",
            wait.as_secs().max(1)
        ),
    )
}

/// One readiness check, run on a blocking thread outside any caller budget.
fn check(bin: &str) -> Phase {
    if waitready(bin) {
        return Phase::Ready(Instant::now());
    }
    let Some(state) = service_state(bin) else {
        tracing::error!(
            "the Incus daemon did not answer within {WAITREADY_SECS}s and the state of \
             incus.service could not be read (the incus wrapper may predate `{SERVICE_VERB}`)"
        );
        return Phase::Failed(Instant::now());
    };
    if !state.stuck() {
        tracing::warn!(
            "incus.service is {} with MainPID={} but the daemon did not answer within {WAITREADY_SECS}s",
            state.active,
            state.main_pid
        );
        return Phase::Failed(Instant::now());
    }
    if !take_restart(Instant::now()) {
        tracing::error!(
            "incus.service is still stuck (ActiveState={}, SubState={}, MainPID={}) after the \
             automatic restart; restart the computer or WSL",
            state.active,
            state.sub,
            state.main_pid
        );
        return Phase::Failed(Instant::now());
    }
    GATE.phase.send_replace(Phase::Restarting);
    tracing::warn!(
        "incus.service is stuck (ActiveState={}, SubState={}, MainPID={}); restarting it once",
        state.active,
        state.sub,
        state.main_pid
    );
    let restarted = run(bin, &[SERVICE_VERB, "restart"], RESTART_DEADLINE)
        .is_some_and(|output| output.status.success());
    if restarted && waitready(bin) {
        tracing::info!("incus.service restarted; the Incus daemon answers again");
        return Phase::Ready(Instant::now());
    }
    tracing::error!("incus.service did not answer after a restart; restart the computer or WSL");
    Phase::Failed(Instant::now())
}

fn waitready(bin: &str) -> bool {
    let timeout = format!("--timeout={WAITREADY_SECS}");
    run(bin, &["admin", "waitready", &timeout], WAITREADY_DEADLINE)
        .is_some_and(|output| output.status.success())
}

fn service_state(bin: &str) -> Option<ServiceState> {
    let output = run(bin, &[SERVICE_VERB, "state"], STATE_DEADLINE)?;
    if !output.status.success() {
        return None;
    }
    parse_service_state(&String::from_utf8_lossy(&output.stdout))
}

/// Records an automatic restart unless one happened within the cooldown.
fn take_restart(now: Instant) -> bool {
    let mut last = GATE
        .last_restart
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let allowed = restart_allowed(*last, now);
    if allowed {
        *last = Some(now);
    }
    allowed
}

fn restart_allowed(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|at| now.saturating_duration_since(at) >= RESTART_COOLDOWN)
}

/// `incus <args>` through the wrapper. Only the wrapper process and what it
/// started are ever killed, at a deadline long past a WSL boot.
fn run(bin: &str, args: &[&str], deadline: Duration) -> Option<Output> {
    let mut command = SafeCommand::new(bin)
        .map_err(|error| tracing::error!("incus readiness command rejected: {error}"))
        .ok()?;
    for arg in args {
        command = command
            .arg(arg)
            .map_err(|error| tracing::error!("incus readiness argument rejected: {error}"))
            .ok()?;
    }
    match output_within(&command, deadline) {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail: String = stderr.trim().chars().take(200).collect();
                tracing::warn!("incus {} exited {}: {detail}", args.join(" "), output.status);
            }
            Some(output)
        }
        Err(failure) => {
            tracing::warn!(
                "incus {} did not finish within {}s: {failure:?}",
                args.join(" "),
                deadline.as_secs()
            );
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ServiceState {
    active: String,
    sub: String,
    main_pid: u32,
}

impl ServiceState {
    /// Not running, or running without its main process (the daemon was
    /// killed while systemd still waits on `ExecStartPost`).
    fn stuck(&self) -> bool {
        self.active != "active" || self.main_pid == 0
    }
}

/// Parses `systemctl show incus.service --property=ActiveState,SubState,MainPID`.
fn parse_service_state(stdout: &str) -> Option<ServiceState> {
    let mut state = ServiceState::default();
    let mut seen = false;
    for line in stdout.lines() {
        let Some((key, value)) = line.trim().split_once('=') else {
            continue;
        };
        match key {
            "ActiveState" => {
                state.active = value.to_string();
                seen = true;
            }
            "SubState" => state.sub = value.to_string(),
            "MainPID" => state.main_pid = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    seen.then_some(state)
}

#[cfg(test)]
#[path = "incus_ready_tests.rs"]
mod tests;
