//! #2045 — budget/timeout message tests for `vm_exec`.
//!
//! Split out of `vm_exec.rs` so that file stays under the size limit.

use super::*;


#[test]
fn empty_reason_keeps_a_short_call_to_action() {
    let message = workspace_failure_message("   ", 60_000);
    assert_eq!(
        message,
        "The workspace could not run this program. Start a new message and try again."
    );
}

#[test]
fn the_reason_is_shown_so_the_cause_stays_actionable() {
    let message = workspace_failure_message(
        "container ag-uf4b70eb7-7e97 did not reach RUNNING within 60s; instance log: bpf_program_load_kernel: Operation not permitted",
        60_000,
    );
    assert!(message.contains("container ag-uf4b70eb7-7e97"));
    assert!(message.contains("Operation not permitted"));
    assert!(message.starts_with("The workspace could not run this program: "));
}

#[test]
fn a_cold_start_reason_is_not_framed_as_a_dead_host() {
    let message = workspace_failure_message(
        "the workspace container ag-u58d054e0-619e is still starting after 900s; the first run downloads and unpacks the Incus image. Retry in a moment",
        60_000,
    );
    assert!(message.contains("still starting"));
    assert!(!message.contains("restarting the app"));
    assert!(message.ends_with("Start a new message to retry."));
}

#[test]
fn a_budget_timeout_gets_timeout_guidance_not_host_guidance() {
    let message = workspace_failure_message(
        "the run budget elapsed before the workspace answered",
        60_000,
    );
    assert!(message.contains("180-second budget"));
    assert!(message.contains("60000 ms program timeout"));
    assert!(message.contains("120 s of workspace overhead"));
    assert!(message.contains("Retry"));
    assert!(message.contains("split it into smaller steps"));
    assert!(message.contains("check the budget"));
    assert!(message.contains("`timeout_ms`"));
    // #2045 — the message names the configurable knobs instead of a cap the
    // caller already hit.
    assert!(message.contains("GB_WORKSPACE_TIMEOUT_MAX_MS"));
    assert!(message.contains("GB_WORKSPACE_OVERHEAD_SECS"));
    assert!(message.contains("GB_WORKSPACE_BUDGET_MS"));
    assert!(!message.contains("restart"));
    assert!(!message.contains("Start a new message once"));
    assert!(!message.contains("wait for"));
}

#[test]
fn the_budget_message_names_the_stage_that_consumed_the_time() {
    // #2045 — the reason carries the stage, so the user learns where the
    // budget went instead of only the total.
    let message = workspace_failure_message(
        "the run budget elapsed during 'install packages' before the workspace answered",
        60_000,
    );
    assert!(
        message.contains("while in 'install packages'"),
        "stage missing from {message}"
    );
    assert!(message.contains("180-second budget"));
}

#[test]
fn a_stage_less_reason_defaults_to_the_workspace_step() {
    let message = workspace_failure_message("the run budget elapsed", 60_000);
    assert!(
        message.contains("while in 'execute'"),
        "a reason without a stage must still name one: {message}"
    );
}

#[test]
fn the_default_timeout_cap_allows_a_real_build() {
    // #2045 — 120 s could not cover a cold image pull plus a build.
    assert!(DEFAULT_TIMEOUT_CAP_MS >= 600_000);
    assert!(DEFAULT_WORKSPACE_OVERHEAD_SECS >= 120);
}

#[test]
fn env_overrides_are_parsed_and_ignored_when_malformed() {
    assert_eq!(parse_positive_u64(Some("300")), Some(300));
    assert_eq!(parse_positive_u64(Some(" 45 ")), Some(45));
    assert_eq!(parse_positive_u64(Some("")), None);
    assert_eq!(parse_positive_u64(Some("0")), None);
    assert_eq!(parse_positive_u64(Some("soon")), None);
    assert_eq!(parse_positive_u64(None), None);
}

#[test]
fn the_budget_is_the_timeout_plus_overhead_and_honours_a_floor() {
    assert_eq!(budget_from(60_000, 120, None), 180_000);
    assert_eq!(budget_from(600_000, 300, None), 900_000);
    // An absolute floor only ever raises the budget.
    assert_eq!(budget_from(60_000, 120, Some(3_600_000)), 3_600_000);
    assert_eq!(budget_from(600_000, 120, Some(60_000)), 720_000);
    // Saturating arithmetic must not wrap.
    assert_eq!(budget_from(u64::MAX, 120, None), u64::MAX);
}

#[test]
fn a_reason_far_beyond_the_limit_is_clipped_with_an_ellipsis() {
    let long = "x".repeat(600);
    let message = workspace_failure_message(&long, 60_000);
    assert!(message.contains('…'), "clip marker present");
    assert!(message.chars().count() < 500, "message stays readable");
    assert!(message.ends_with("Start a new message to retry."));
}
