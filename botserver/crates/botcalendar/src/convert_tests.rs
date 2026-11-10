//! iCalendar mapping: an exported calendar must import back to the same
//! events, and a record with malformed JSON columns must degrade to empty
//! values instead of failing the whole list (see #1338).
use super::*;

fn sample_event(calendar_id: Uuid) -> CalendarEvent {
    CalendarEvent {
        id: Uuid::parse_str("3f2504e0-4f89-11d3-9a0c-0305e82c3301").expect("fixture uuid"),
        calendar_id,
        title: "Quarterly review".to_string(),
        description: Some("Numbers & plans".to_string()),
        start_time: Utc
            .with_ymd_and_hms(2026, 9, 1, 9, 0, 0)
            .single()
            .expect("start"),
        end_time: Utc
            .with_ymd_and_hms(2026, 9, 1, 10, 30, 0)
            .single()
            .expect("end"),
        location: Some("Room 3".to_string()),
        attendees: vec!["ana@example.com".to_string(), "bo@example.com".to_string()],
        organizer: "owner@example.com".to_string(),
        reminder_minutes: Some(15),
        recurrence: Some("FREQ=WEEKLY;COUNT=4".to_string()),
        all_day: false,
        status: "confirmed".to_string(),
        color: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn sample_record(calendar_id: Uuid) -> CalendarEventRecord {
    CalendarEventRecord {
        id: Uuid::parse_str("3f2504e0-4f89-11d3-9a0c-0305e82c3302").expect("fixture uuid"),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        branch_id: Uuid::nil(),
        calendar_id,
        owner_id: Uuid::parse_str("7c9e6679-7425-40de-944b-e07fc1f90ae7").expect("owner"),
        title: "Standup".to_string(),
        description: None,
        location: None,
        start_time: Utc
            .with_ymd_and_hms(2026, 9, 2, 12, 0, 0)
            .single()
            .expect("start"),
        end_time: Utc
            .with_ymd_and_hms(2026, 9, 2, 12, 15, 0)
            .single()
            .expect("end"),
        all_day: false,
        recurrence_rule: None,
        recurrence_id: None,
        color: None,
        status: "confirmed".to_string(),
        visibility: "default".to_string(),
        busy_status: "busy".to_string(),
        reminders: serde_json::json!([{ "minutes_before": 15 }]),
        attendees: serde_json::json!(["ana@example.com"]),
        conference_data: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn export_writes_a_calendar_with_the_event_summary() {
    let calendar_id = Uuid::new_v4();
    let ical = export_to_ical(&[sample_event(calendar_id)], "Team");

    assert!(ical.contains("BEGIN:VCALENDAR"));
    assert!(ical.contains("END:VCALENDAR"));
    assert!(ical.contains("SUMMARY:Quarterly review"));
    assert!(ical.contains("mailto:owner@example.com"));
    assert!(ical.contains("mailto:ana@example.com"));
}

#[test]
fn export_then_import_round_trips_the_core_fields() {
    let calendar_id = Uuid::new_v4();
    let original = sample_event(calendar_id);

    let ical = export_to_ical(&[original.clone()], "Team");
    let imported = import_from_ical(&ical, "organizer@example.com", calendar_id);

    assert_eq!(imported.len(), 1, "one VEVENT must come back");
    let event = imported.first().expect("event");
    assert_eq!(event.id, original.id);
    assert_eq!(event.calendar_id, calendar_id);
    assert_eq!(event.title, original.title);
    assert_eq!(event.start_time, original.start_time);
    assert_eq!(event.end_time, original.end_time);
    // The caller-supplied organizer wins over the exported one.
    assert_eq!(event.organizer, "organizer@example.com");
}

#[test]
fn import_returns_empty_for_unparsable_input() {
    assert!(import_from_ical("not an icalendar document", "me@example.com", Uuid::nil()).is_empty());
}

#[test]
fn import_skips_components_that_are_not_events() {
    let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\n\
                BEGIN:VTODO\r\nUID:todo-1\r\nSUMMARY:Buy milk\r\nEND:VTODO\r\n\
                END:VCALENDAR";

    assert!(import_from_ical(ical, "me@example.com", Uuid::nil()).is_empty());
}

#[test]
fn import_drops_events_missing_a_uid_or_summary() {
    let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\n\
                BEGIN:VEVENT\r\nDTSTART:20260901T100000Z\r\nDTEND:20260901T110000Z\r\n\
                END:VEVENT\r\nEND:VCALENDAR";

    assert!(import_from_ical(ical, "me@example.com", Uuid::nil()).is_empty());
}

#[test]
fn record_to_event_maps_attendees_and_the_first_reminder() {
    let calendar_id = Uuid::new_v4();
    let event = record_to_event(sample_record(calendar_id));

    assert_eq!(event.calendar_id, calendar_id);
    assert_eq!(event.title, "Standup");
    assert_eq!(event.attendees, vec!["ana@example.com".to_string()]);
    assert_eq!(event.reminder_minutes, Some(15));
    assert_eq!(event.organizer, "7c9e6679-7425-40de-944b-e07fc1f90ae7");
}

#[test]
fn record_to_event_tolerates_malformed_json_columns() {
    let mut record = sample_record(Uuid::new_v4());
    record.attendees = serde_json::json!("not-an-array");
    record.reminders = serde_json::Value::Null;

    let event = record_to_event(record);

    assert!(event.attendees.is_empty());
    assert_eq!(event.reminder_minutes, None);
}

#[test]
fn record_to_event_ignores_a_reminder_without_minutes_before() {
    let mut record = sample_record(Uuid::new_v4());
    record.reminders = serde_json::json!([{ "channel": "email" }]);

    assert_eq!(record_to_event(record).reminder_minutes, None);
}

#[test]
fn a_date_only_value_maps_to_midnight_utc() {
    let date = NaiveDate::from_ymd_opt(2026, 9, 1).expect("date");
    let converted = date_perhaps_time_to_utc(DatePerhapsTime::Date(date)).expect("converted");

    assert_eq!(
        converted,
        Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0)
            .single()
            .expect("midnight")
    );
}
