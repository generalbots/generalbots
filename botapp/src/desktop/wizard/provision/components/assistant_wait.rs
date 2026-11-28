//! Waiting for the assistant service while watching its process.
//!
//! The wizard used to poll `/api/product` for a fixed time only, so a server
//! that exited during start-up left the page on "Assistant service" for
//! minutes with no error. The wait now ends as soon as the spawned process
//! exits, and the failure names the reason and the log file.

use crate::desktop::i18n;
use crate::desktop::sidecar::{ChildState, Sidecar};
use std::path::Path;
use std::time::Duration;

/// A cold first boot runs migrations, seeds the vault and starts every
/// service; 120 s was too tight and the wizard declared the assistant dead
/// while it was still starting. A process exit ends the wait at once.
pub(super) const FIRST_BOOT_DEADLINE: Duration = Duration::from_secs(300);
pub(super) const RETRY_DEADLINE: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(500);
/// Bytes read from the end of the log when looking for the last error line.
const LOG_TAIL_BYTES: u64 = 64 * 1024;
const MAX_ERROR_CHARS: usize = 240;

/// How one wait for the assistant ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WaitOutcome {
    Ready,
    /// The process this app spawned is gone, with its exit code when known.
    Exited(Option<i32>),
    TimedOut,
}

/// One poll's decision. `None` means keep waiting.
///
/// `seen_running` covers a handle whose exit was collected by another watcher
/// (the boot splash polls the same child): an owned child that later reads as
/// not owned has exited, since only `stop` releases it otherwise and the
/// wizard never stops it while waiting.
pub(super) fn decide(
    api_ready: bool,
    child: ChildState,
    seen_running: bool,
    elapsed: Duration,
    deadline: Duration,
) -> Option<WaitOutcome> {
    if api_ready {
        return Some(WaitOutcome::Ready);
    }
    match child {
        ChildState::Exited(code) => return Some(WaitOutcome::Exited(code)),
        ChildState::NotOwned if seen_running => return Some(WaitOutcome::Exited(None)),
        ChildState::Running | ChildState::NotOwned => {}
    }
    (elapsed >= deadline).then_some(WaitOutcome::TimedOut)
}

/// Polls `url` until it answers, the spawned process exits, or `deadline`.
pub(super) async fn wait_for_assistant(sidecar: &Sidecar, url: &str, deadline: Duration) -> WaitOutcome {
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(2)).build() {
        Ok(client) => client,
        Err(e) => {
            log::warn!("could not build the readiness client: {e}");
            return WaitOutcome::TimedOut;
        }
    };
    let started = std::time::Instant::now();
    let mut seen_running = false;
    loop {
        let api_ready = matches!(client.get(url).send().await, Ok(resp) if resp.status().is_success());
        let child = sidecar.child_state();
        seen_running |= child == ChildState::Running;
        if let Some(outcome) = decide(api_ready, child, seen_running, started.elapsed(), deadline) {
            log::info!(
                "assistant wait ended: {outcome:?} after {:.1}s",
                started.elapsed().as_secs_f64()
            );
            return outcome;
        }
        tokio::time::sleep(POLL).await;
    }
}

/// The message the wizard's failure view shows for a failed wait.
pub(super) fn failure_reason(
    outcome: WaitOutcome,
    deadline: Duration,
    log: &Path,
    last_error: Option<&str>,
) -> String {
    let log = log.display().to_string();
    let head = match outcome {
        WaitOutcome::Exited(code) => {
            let code = code.map_or_else(|| "?".to_string(), |code| code.to_string());
            i18n::t_args("setup-assistant-exited", &[("code", &code), ("log", &log)])
        }
        WaitOutcome::TimedOut | WaitOutcome::Ready => {
            let seconds = deadline.as_secs().to_string();
            i18n::t_args("setup-assistant-timeout", &[("seconds", &seconds), ("log", &log)])
        }
    };
    match last_error {
        Some(line) => format!(
            "{head} {}",
            i18n::t_args("setup-assistant-last-error", &[("line", line)])
        ),
        None => head,
    }
}

/// The last error line of the log's tail, shortened, or `None`.
pub(super) fn last_error_line(log: &Path) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(log).ok()?;
    let len = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(LOG_TAIL_BYTES))).ok()?;
    let mut raw = Vec::new();
    file.read_to_end(&mut raw).ok()?;
    pick_error_line(&String::from_utf8_lossy(&raw))
}

/// The server log writes `<timestamp> <level> <target>:<message>` and wraps
/// long records onto indented continuation lines.
fn pick_error_line(tail: &str) -> Option<String> {
    let mut records: Vec<String> = Vec::new();
    for line in tail.lines() {
        match records.last_mut() {
            Some(record) if line.starts_with(char::is_whitespace) => record.push_str(line.trim()),
            _ => records.push(line.trim().to_string()),
        }
    }
    records.iter().rev().find_map(|record| error_message(record))
}

fn error_message(record: &str) -> Option<String> {
    let mut rest = record;
    for _ in 0..3 {
        let (token, after) = rest.split_once(char::is_whitespace)?;
        if token.trim_matches(['[', ']']).eq_ignore_ascii_case("error") {
            let message = after.trim();
            return (!message.is_empty()).then(|| message.chars().take(MAX_ERROR_CHARS).collect());
        }
        rest = after.trim_start();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEADLINE: Duration = Duration::from_secs(300);
    const EARLY: Duration = Duration::from_secs(5);

    #[test]
    fn an_answering_api_is_ready_even_if_the_handle_is_gone() {
        assert_eq!(
            decide(true, ChildState::Exited(Some(1)), true, EARLY, DEADLINE),
            Some(WaitOutcome::Ready)
        );
    }

    #[test]
    fn an_exited_process_ends_the_wait_before_the_deadline() {
        assert_eq!(
            decide(false, ChildState::Exited(Some(1)), true, EARLY, DEADLINE),
            Some(WaitOutcome::Exited(Some(1)))
        );
    }

    #[test]
    fn an_exit_collected_by_another_watcher_counts_as_an_exit() {
        assert_eq!(
            decide(false, ChildState::NotOwned, true, EARLY, DEADLINE),
            Some(WaitOutcome::Exited(None))
        );
    }

    #[test]
    fn an_attached_assistant_waits_until_the_deadline() {
        assert_eq!(decide(false, ChildState::NotOwned, false, EARLY, DEADLINE), None);
        assert_eq!(
            decide(false, ChildState::NotOwned, false, DEADLINE, DEADLINE),
            Some(WaitOutcome::TimedOut)
        );
    }

    #[test]
    fn a_running_process_keeps_waiting_until_the_deadline() {
        assert_eq!(decide(false, ChildState::Running, true, EARLY, DEADLINE), None);
        assert_eq!(
            decide(false, ChildState::Running, true, DEADLINE, DEADLINE),
            Some(WaitOutcome::TimedOut)
        );
    }

    #[test]
    fn the_failure_names_the_exit_code_the_log_and_the_last_error() {
        let log = Path::new("stack/logs/botserver-sidecar.log");
        let reason = failure_reason(
            WaitOutcome::Exited(Some(1)),
            DEADLINE,
            log,
            Some("run_axum_server: Refusing to start"),
        );
        assert!(reason.contains('1'), "{reason}");
        assert!(reason.contains(&log.display().to_string()), "{reason}");
        assert!(reason.contains("Refusing to start"), "{reason}");
    }

    #[test]
    fn the_last_error_line_is_picked_from_the_tail() {
        let tail = "20261127235557.050 info boot:starting\n\
                    20261127235557.059 error jwt_secret:not persisted\n\
                    20261127235557.059 error run_axum_server:Refusing to start: no SaaS JWT\n\
                    \x20                        secret configured\n\
                    20261127235557.060 info main:bye\n";
        assert_eq!(
            pick_error_line(tail).as_deref(),
            Some("run_axum_server:Refusing to start: no SaaS JWTsecret configured")
        );
        assert_eq!(
            pick_error_line("2026-11-28T10:00:00Z [ERROR] boot: failed\n").as_deref(),
            Some("boot: failed")
        );
        assert_eq!(pick_error_line("20261127 info errors are words here\n"), None);
    }
}
