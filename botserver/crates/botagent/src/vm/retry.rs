//! Retry/backoff helpers for cold Incus starts (issue #1798).
//!
//! The first launch of an agent container downloads and unpacks the image,
//! which can take minutes. During that window the daemon is busy and a
//! container may not exist yet, so transient failures must be retried instead
//! of surfacing as a terminal "Agent VM operation failed".

use axum::http::StatusCode;
use std::time::Duration as StdDuration;

use crate::AgentService;

use super::incus::run_incus;

/// Transient Incus failures during a cold start (daemon busy while the image is
/// unpacked, instance not yet created, a concurrent `start`) are retried with
/// exponential backoff instead of surfacing as a terminal workspace error.
const RETRY_ATTEMPTS: u32 = 5;
const RETRY_MAX_DELAY_SECS: u64 = 30;

/// Delay before retry `attempt` (0-based): 1, 2, 4, 8, 16, 30, 30… seconds.
fn retry_backoff(attempt: u32) -> StdDuration {
    let secs = 1u64.checked_shl(attempt.min(5)).unwrap_or(32);
    StdDuration::from_secs(secs.min(RETRY_MAX_DELAY_SECS))
}

/// An Incus command that failed only because the target already reached the
/// requested state is not an error: `launch` on an existing instance and
/// `start` on a running one both mean "the container is there".
fn is_idempotent_outcome(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    lower.contains("already exists")
        || lower.contains("already in use")
        || lower.contains("already running")
}

/// Cold-start failures are retryable: the first launch downloads and unpacks
/// the image, so "not created yet" and a busy daemon are expected states, not
/// permanent host failures.
fn is_transient_incus_failure(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    is_idempotent_outcome(detail)
        || lower.contains("is not ready")
        || lower.contains("not created yet")
        || lower.contains("instance is being created")
        || lower.contains("database is locked")
        || lower.contains("temporarily unavailable")
        || lower.contains("try again")
        || lower.contains("connection reset")
        || lower.contains("broken pipe")
}

/// Actionable text for a container that never reached RUNNING inside the cold
/// budget. The previous generic "Agent VM operation failed" made a slow first
/// image pull look like a permanently broken host.
pub(super) fn cold_start_timeout_message(vm_name: &str, wait_secs: u64) -> String {
    format!(
        "the container {vm_name} is still starting after {wait_secs}s; \
         the first run downloads and unpacks the Incus image, which can take several \
         minutes on a cold host. Retry in a moment — the container becomes usable as \
         soon as the image pull finishes"
    )
}

/// Retries one Incus operation with exponential backoff. Setup errors (missing
/// daemon or socket permission) are returned immediately: retrying cannot fix
/// a host without Incus. Every other failure is retried and the last one is
/// returned when the attempts are exhausted.
pub(super) async fn retry_incus(
    state: &AgentService,
    args: &[String],
    operation: &str,
) -> Result<std::process::Output, (StatusCode, String)> {
    let mut last: Option<(StatusCode, String)> = None;
    for attempt in 0..RETRY_ATTEMPTS {
        match run_incus(state, args.to_vec()).await {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(output) => {
                let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let detail = if detail.is_empty() {
                    format!("exit status {}", output.status)
                } else {
                    detail
                };
                if is_idempotent_outcome(&detail) {
                    tracing::info!("incus {operation} reached the desired state: {detail}");
                    return Ok(output);
                }
                tracing::warn!(
                    "incus {operation} attempt {}/{} failed: {detail}",
                    attempt + 1,
                    RETRY_ATTEMPTS
                );
                last = Some((StatusCode::INTERNAL_SERVER_ERROR, detail));
            }
            Err(error) if error.0 == StatusCode::SERVICE_UNAVAILABLE => return Err(error),
            Err(error) => {
                tracing::warn!(
                    "incus {operation} attempt {}/{} failed ({}): {}",
                    attempt + 1,
                    RETRY_ATTEMPTS,
                    error.0,
                    error.1
                );
                last = Some(error);
            }
        }
        if attempt + 1 < RETRY_ATTEMPTS {
            tokio::time::sleep(retry_backoff(attempt)).await;
        }
    }
    let (code, detail) = last.unwrap_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("incus {operation} failed without an error detail"),
        )
    });
    Err((
        code,
        format!("{operation} did not succeed after {RETRY_ATTEMPTS} attempts: {detail}"),
    ))
}

/// Starts an existing container, retrying transient failures. `incus start`
/// reports "already running" when another request won the race; that is the
/// desired end state, so it is not treated as an error.
pub(crate) async fn start_container(
    state: &AgentService,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    let args = vec!["start".to_string(), vm_name.to_string()];
    match retry_incus(state, &args, "start").await {
        Ok(_) => Ok(()),
        Err(error) if is_transient_incus_failure(&error.1) => {
            tracing::warn!(
                "container {vm_name} start is still in progress: {}",
                error.1
            );
            Ok(())
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod retry_tests {
    use super::{
        cold_start_timeout_message, is_idempotent_outcome, is_transient_incus_failure,
        retry_backoff,
    };
    use std::time::Duration;

    #[test]
    fn cold_start_timeout_explains_the_first_run_and_retry() {
        let message = cold_start_timeout_message("ag-u58d054e0-619e", 900);
        assert!(message.contains("still starting"));
        assert!(message.contains("downloads and unpacks the Incus image"));
        assert!(message.contains("Retry in a moment"));
        assert!(!message.contains("Agent VM operation failed"));
    }

    #[test]
    fn retry_backoff_grows_and_stays_capped() {
        assert_eq!(retry_backoff(0), Duration::from_secs(1));
        assert_eq!(retry_backoff(1), Duration::from_secs(2));
        assert_eq!(retry_backoff(2), Duration::from_secs(4));
        assert_eq!(retry_backoff(3), Duration::from_secs(8));
        assert_eq!(retry_backoff(4), Duration::from_secs(16));
        assert_eq!(retry_backoff(5), Duration::from_secs(30));
        assert_eq!(retry_backoff(40), Duration::from_secs(30));
    }

    #[test]
    fn an_existing_instance_is_an_idempotent_success() {
        assert!(is_idempotent_outcome(
            "Error: Instance \"ag-u1234\" already exists"
        ));
        assert!(is_idempotent_outcome("Error: Instance is already running"));
        assert!(!is_idempotent_outcome("Error: Image not found"));
    }

    #[test]
    fn transient_cold_start_failures_are_classified() {
        assert!(is_transient_incus_failure("Instance is not ready"));
        assert!(is_transient_incus_failure(
            "Error: Instance is being created"
        ));
        assert!(is_transient_incus_failure("database is locked"));
        assert!(is_transient_incus_failure("connection reset by peer"));
        // A missing host dependency is not transient: retrying cannot help.
        assert!(!is_transient_incus_failure("Image not found"));
        assert!(!is_transient_incus_failure("permission denied"));
    }
}
