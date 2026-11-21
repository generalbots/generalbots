use super::*;
use std::time::{Duration, Instant};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|value| value.to_string()).collect()
}

#[cfg(windows)]
fn sleeper() -> SafeCommand {
    SafeCommand::new("cmd")
        .and_then(|command| command.args(&["/C", "ping", "-n", "30", "127.0.0.1"]))
        .unwrap_or_else(|error| panic!("guard rejected the test command: {error}"))
}

#[cfg(not(windows))]
fn sleeper() -> SafeCommand {
    SafeCommand::new("sh")
        .and_then(|command| command.arg("-c"))
        .and_then(|command| command.trusted_shell_script_arg("sleep 30"))
        .unwrap_or_else(|error| panic!("guard rejected the test command: {error}"))
}

#[cfg(windows)]
fn echo() -> SafeCommand {
    SafeCommand::new("cmd")
        .and_then(|command| command.args(&["/C", "echo", "ready"]))
        .unwrap_or_else(|error| panic!("guard rejected the test command: {error}"))
}

#[cfg(not(windows))]
fn echo() -> SafeCommand {
    SafeCommand::new("sh")
        .and_then(|command| command.arg("-c"))
        .and_then(|command| command.trusted_shell_script_arg("echo ready"))
        .unwrap_or_else(|error| panic!("guard rejected the test command: {error}"))
}

#[test]
fn quick_reads_get_the_default_deadline() {
    assert_eq!(base_deadline(&args(&["list", "--format", "json"])), DEFAULT_DEADLINE);
    assert_eq!(base_deadline(&args(&["info", "vm", "--show-log"])), DEFAULT_DEADLINE);
    assert_eq!(base_deadline(&args(&["config", "set", "vm", "k", "v"])), DEFAULT_DEADLINE);
    assert_eq!(base_deadline(&args(&["image", "list", "--format", "json"])), DEFAULT_DEADLINE);
    assert_eq!(base_deadline(&args(&["snapshot", "list", "vm"])), DEFAULT_DEADLINE);
    assert_eq!(base_deadline(&[]), DEFAULT_DEADLINE);
}

#[test]
fn long_operations_get_longer_deadlines() {
    assert_eq!(base_deadline(&args(&["start", "vm"])), POWER_DEADLINE);
    assert_eq!(base_deadline(&args(&["delete", "vm", "--force"])), POWER_DEADLINE);
    assert_eq!(base_deadline(&args(&["snapshot", "create", "vm", "s1"])), DATA_DEADLINE);
    assert_eq!(base_deadline(&args(&["file", "push", "a", "vm/a"])), DATA_DEADLINE);
    assert_eq!(base_deadline(&args(&["exec", "vm", "--", "true"])), EXEC_DEADLINE);
    assert!(base_deadline(&args(&["launch", "images:debian/12", "vm"])) >= Duration::from_secs(60));
}

#[tokio::test]
async fn a_budget_caps_every_call_inside_it() {
    assert_eq!(capped(DEFAULT_DEADLINE), DEFAULT_DEADLINE);
    let inside = within_budget(PROBE_DEADLINE, async { deadline_for(&args(&["start", "vm"])) }).await;
    assert_eq!(inside, Some(PROBE_DEADLINE));
    let nested = within_budget(PROBE_DEADLINE, async {
        within_budget(Duration::from_secs(60), async { capped(EXEC_DEADLINE) }).await
    })
    .await;
    assert_eq!(nested, Some(Some(PROBE_DEADLINE)));
}

#[tokio::test]
async fn a_budget_gives_up_on_a_slow_future() {
    let started = Instant::now();
    let result = within_budget(Duration::from_millis(200), tokio::time::sleep(Duration::from_secs(30))).await;
    assert!(result.is_none());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_command_inside_the_deadline_returns_its_output() {
    let output = output_within(&echo(), Duration::from_secs(20))
        .unwrap_or_else(|error| panic!("echo failed: {error:?}"));
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ready"));
}

#[test]
fn a_hung_command_is_killed_at_the_deadline() {
    let started = Instant::now();
    let result = output_within(&sleeper(), Duration::from_secs(1));
    let elapsed = started.elapsed();
    assert!(matches!(result, Err(RunFailure::TimedOut { .. })), "got {result:?}");
    assert!(elapsed < Duration::from_secs(10), "took {elapsed:?}");
}

/// `cmd` starts `ping` as its own child, like the Incus wrapper starts
/// `wsl.exe`: the pipes only close when the whole tree is gone.
#[cfg(windows)]
#[test]
fn the_kill_reaches_the_grandchild() {
    let result = output_within(&sleeper(), Duration::from_secs(1));
    assert!(
        matches!(result, Err(RunFailure::TimedOut { orphaned: false })),
        "got {result:?}"
    );
}

#[test]
fn the_timeout_error_carries_a_stable_code() {
    let error = timeout_error("list", PROBE_DEADLINE, false);
    assert_eq!(error.0, StatusCode::GATEWAY_TIMEOUT);
    assert!(is_timeout(&error));
    assert!(error.1.contains("5 s"));
    assert!(!is_timeout(&(StatusCode::GATEWAY_TIMEOUT, "other".to_string())));
    assert!(!is_timeout(&(StatusCode::SERVICE_UNAVAILABLE, INCUS_TIMEOUT_CODE.to_string())));
}
