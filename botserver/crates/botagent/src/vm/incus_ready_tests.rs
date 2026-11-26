use super::*;

/// What `systemctl show` printed for the unit during the incident: the
/// daemon was gone while systemd still waited on `incus admin waitready`.
const STUCK: &str = "ActiveState=activating\nSubState=start-post\nMainPID=0\n";
const HEALTHY: &str = "ActiveState=active\nSubState=running\nMainPID=291\n";

#[test]
fn the_incident_state_is_stuck() {
    let state = parse_service_state(STUCK).expect("state parses");
    assert_eq!(state.active, "activating");
    assert_eq!(state.sub, "start-post");
    assert_eq!(state.main_pid, 0);
    assert!(state.stuck());
}

#[test]
fn a_running_daemon_is_not_stuck() {
    let state = parse_service_state(HEALTHY).expect("state parses");
    assert_eq!(state.main_pid, 291);
    assert!(!state.stuck());
}

#[test]
fn failed_and_inactive_units_are_stuck() {
    for text in [
        "ActiveState=failed\nSubState=failed\nMainPID=0",
        "ActiveState=inactive\nSubState=dead\nMainPID=0",
        "ActiveState=activating\nSubState=start-post\nMainPID=328",
    ] {
        let state = parse_service_state(text).expect("state parses");
        assert!(state.stuck(), "{text}");
    }
}

#[test]
fn output_without_a_unit_state_is_unreadable() {
    assert_eq!(parse_service_state(""), None);
    assert_eq!(parse_service_state("Error: unknown command \"__gb-service\""), None);
}

#[test]
fn a_restart_happens_once_per_cooldown() {
    let now = Instant::now();
    assert!(restart_allowed(None, now));
    assert!(!restart_allowed(Some(now), now + Duration::from_secs(60)));
    assert!(restart_allowed(Some(now), now + RESTART_COOLDOWN));
}

#[test]
fn the_gate_checks_again_only_when_its_answer_is_old() {
    let now = Instant::now();
    assert!(Phase::Unknown.needs_check(now));
    assert!(!Phase::Checking.needs_check(now));
    assert!(!Phase::Restarting.needs_check(now));
    assert!(!Phase::Ready(now).needs_check(now + Duration::from_secs(1)));
    assert!(Phase::Ready(now).needs_check(now + READY_TTL));
    assert!(!Phase::Failed(now).needs_check(now + Duration::from_secs(1)));
    assert!(Phase::Failed(now).needs_check(now + FAILED_TTL));
}

#[test]
fn only_ready_and_failed_settle_a_wait() {
    let now = Instant::now();
    assert!(Phase::Ready(now).settled());
    assert!(Phase::Failed(now).settled());
    assert!(!Phase::Checking.settled());
    assert!(!Phase::Restarting.settled());
    assert!(!Phase::Unknown.settled());
}

#[test]
fn gate_errors_carry_stable_codes() {
    let failed = failed_error();
    assert_eq!(failed.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(service_code(&failed), Some(SERVICE_FAILED_CODE));

    let restarting = waiting_error(true, GATE_WAIT);
    assert_eq!(service_code(&restarting), Some(SERVICE_RESTARTING_CODE));

    let waiting = waiting_error(false, Duration::from_secs(5));
    assert_eq!(service_code(&waiting), None);
    assert!(super::super::incus_run::is_timeout(&waiting));
    assert!(waiting.1.contains("5 s"));
}

#[test]
fn unrelated_errors_have_no_service_code() {
    let other = (StatusCode::SERVICE_UNAVAILABLE, "Incus is not available on this host".to_string());
    assert_eq!(service_code(&other), None);
}
