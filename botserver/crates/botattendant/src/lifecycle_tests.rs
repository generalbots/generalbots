//! Table-driven tests for the contact-center lifecycle rules.

use super::*;
use chrono::Duration;

#[test]
fn session_numbers_are_zero_padded_and_never_negative() {
    let cases = [
        (0i64, "SES-000001"),
        (1, "SES-000002"),
        (123, "SES-000124"),
        (999_998, "SES-999999"),
        // Beyond the padding the number simply grows.
        (1_000_000, "SES-1000001"),
        // A broken aggregate must not produce "SES-000000".
        (-5, "SES-000001"),
        (i64::MAX, "SES-9223372036854775807"),
    ];
    for (count, expected) in cases {
        assert_eq!(format_session_number(count), expected, "count {count}");
    }
}

#[test]
fn the_wait_timer_counts_seconds_since_the_session_started() {
    let now = Utc::now();
    assert_eq!(wait_seconds(now, now), 0);
    assert_eq!(wait_seconds(now - Duration::seconds(90), now), 90);
    assert_eq!(wait_seconds(now - Duration::hours(2), now), 7200);
}

#[test]
fn a_backwards_clock_never_produces_a_negative_wait() {
    let now = Utc::now();
    assert_eq!(wait_seconds(now + Duration::seconds(30), now), 0);
}

#[test]
fn the_handle_timer_is_only_defined_after_assignment() {
    let now = Utc::now();
    assert_eq!(handle_seconds(None, now), None);
    assert_eq!(handle_seconds(Some(now - Duration::seconds(45)), now), Some(45));
    assert_eq!(handle_seconds(Some(now), now), Some(0));
    assert_eq!(handle_seconds(Some(now + Duration::seconds(45)), now), Some(0));
}

#[test]
fn session_statuses_round_trip_through_the_wire_format() {
    for (wire, status) in [
        ("waiting", SessionStatus::Waiting),
        ("active", SessionStatus::Active),
        ("ended", SessionStatus::Ended),
    ] {
        assert_eq!(SessionStatus::parse(wire), Some(status));
        assert_eq!(status.as_str(), wire);
    }
    for unknown in ["", "closed", "Waiting", "paused"] {
        assert_eq!(SessionStatus::parse(unknown), None, "status {unknown:?}");
    }
}

#[test]
fn only_the_ended_state_is_terminal() {
    use SessionStatus::{Active, Ended, Waiting};
    let allowed = [
        (Waiting, Waiting),
        (Waiting, Active),
        (Waiting, Ended),
        (Active, Waiting),
        (Active, Active),
        (Active, Ended),
        (Ended, Ended),
    ];
    for (from, to) in allowed {
        assert!(from.can_transition_to(to), "{from:?} -> {to:?} must be allowed");
    }
    assert!(!Ended.can_transition_to(Waiting));
    assert!(!Ended.can_transition_to(Active));
}

#[test]
fn work_is_refused_on_a_closed_session_only() {
    assert!(accepts_work("waiting"));
    assert!(accepts_work("active"));
    assert!(!accepts_work("ended"));
    // A label this crate does not own stays editable.
    assert!(accepts_work("escalated"));
}

#[test]
fn going_on_a_break_stamps_the_break_time_only() {
    let now = Utc::now();
    let stamps = activity_stamps(AGENT_BREAK, now);
    assert_eq!(stamps.break_started_at, Some(now));
    assert_eq!(stamps.available_since, None);
}

#[test]
fn becoming_available_stamps_the_availability_time_only() {
    let now = Utc::now();
    let stamps = activity_stamps(AGENT_ONLINE, now);
    assert_eq!(stamps.break_started_at, None);
    assert_eq!(stamps.available_since, Some(now));
}

#[test]
fn other_agent_statuses_clear_both_stamps() {
    let now = Utc::now();
    for status in [AGENT_BUSY, AGENT_OFFLINE, "", "lunch"] {
        let stamps = activity_stamps(status, now);
        assert_eq!(stamps.break_started_at, None, "status {status:?}");
        assert_eq!(stamps.available_since, None, "status {status:?}");
    }
}

#[test]
fn only_the_first_customer_visible_message_stamps_the_response_time() {
    let now = Utc::now();
    assert!(stamps_first_response(None, false));
    assert!(!stamps_first_response(None, true), "internal notes are not responses");
    assert!(!stamps_first_response(Some(now), false), "already stamped");
    assert!(!stamps_first_response(Some(now), true));
}

#[test]
fn a_transfer_prefers_the_agent_over_the_queue() {
    let agent = Uuid::new_v4();
    let queue = Uuid::new_v4();
    assert_eq!(transfer_target(Some(agent), Some(queue)), TransferTarget::Agent(agent));
    assert_eq!(transfer_target(Some(agent), None), TransferTarget::Agent(agent));
    assert_eq!(transfer_target(None, Some(queue)), TransferTarget::Queue(queue));
    assert_eq!(transfer_target(None, None), TransferTarget::Unrouted);
}

#[test]
fn status_badges_map_to_the_published_css_classes() {
    let sessions = [
        ("waiting", "status-waiting"),
        ("active", "status-active"),
        ("ended", "status-ended"),
        ("unknown", "status-default"),
        ("", "status-default"),
    ];
    for (status, class) in sessions {
        assert_eq!(session_status_class(status), class);
    }

    let agents = [
        (AGENT_ONLINE, "status-online"),
        (AGENT_BUSY, "status-busy"),
        (AGENT_BREAK, "status-break"),
        (AGENT_OFFLINE, "status-offline"),
        ("unknown", "status-offline"),
    ];
    for (status, class) in agents {
        assert_eq!(agent_status_class(status), class);
    }
}
