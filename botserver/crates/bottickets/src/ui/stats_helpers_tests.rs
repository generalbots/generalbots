use super::*;
use chrono::{DateTime, Duration, TimeZone, Utc};

fn ticket(created_at: DateTime<Utc>, resolved_at: Option<DateTime<Utc>>) -> SupportTicket {
    SupportTicket {
        id: Uuid::new_v4(),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        ticket_number: "T-1".to_string(),
        subject: "subject".to_string(),
        description: None,
        status: if resolved_at.is_some() { "resolved" } else { "open" }.to_string(),
        priority: "medium".to_string(),
        category: None,
        source: "web".to_string(),
        requester_id: None,
        requester_email: None,
        requester_name: None,
        assignee_id: None,
        team_id: None,
        due_date: None,
        first_response_at: None,
        resolved_at,
        closed_at: None,
        satisfaction_rating: None,
        tags: Vec::new(),
        custom_fields: serde_json::Value::Null,
        created_at,
        updated_at: created_at,
        branch_id: None,
        record_type: "ticket".to_string(),
    }
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 10, hour, minute, 0)
        .single()
        .expect("valid timestamp")
}

#[test]
fn resolution_spans_report_hours_below_a_day_and_days_above() {
    let cases = [
        (5.0, "5.0h"),
        (23.9, "23.9h"),
        (0.5, "0.5h"),
        (24.0, "1.0d"),
        (30.0, "1.2d"),
        (0.0, "0.0h"),
    ];
    for (hours, expected) in cases {
        assert_eq!(format_resolution_span(hours), expected, "hours={hours}");
    }
}

#[test]
fn the_average_uses_minutes_so_short_resolutions_are_not_zero() {
    let tickets = vec![
        ticket(at(9, 0), Some(at(9, 30))),
        ticket(at(10, 0), Some(at(11, 0))),
    ];
    let average = average_resolution_hours(&tickets).expect("average");
    // 0.5h and 1.0h -> 0.75h
    assert!((average - 0.75).abs() < 1e-9);
    assert_eq!(format_resolution_span(average), "0.8h");
}

#[test]
fn unresolved_tickets_do_not_dilute_the_average() {
    let tickets = vec![
        ticket(at(9, 0), Some(at(13, 0))),
        ticket(at(9, 0), None),
    ];
    assert_eq!(average_resolution_hours(&tickets), Some(4.0));
}

#[test]
fn no_resolved_ticket_reports_none() {
    assert_eq!(average_resolution_hours(&[]), None);
    assert_eq!(
        average_resolution_hours(&[ticket(at(9, 0), None)]),
        None
    );
}

#[test]
fn a_full_day_boundary_flips_to_days() {
    let created = at(0, 0);
    let resolved = created + Duration::hours(24);
    let average = average_resolution_hours(&[ticket(created, Some(resolved))]).expect("average");
    assert_eq!(format_resolution_span(average), "1.0d");
}
