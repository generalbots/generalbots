use super::*;
use chrono::TimeZone;

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 10).expect("valid date")
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 10, hour, minute, 0)
        .single()
        .expect("valid timestamp")
}

fn entry(kind: ClockType, timestamp: DateTime<Utc>) -> TimeClockEntry {
    TimeClockEntry {
        id: Uuid::new_v4(),
        employee_id: Uuid::nil(),
        tenant_id: Uuid::nil(),
        clock_type: kind,
        timestamp,
        scheduled_time: None,
        latitude: None,
        longitude: None,
        location_name: None,
        location_verified: false,
        device_id: None,
        ip_address: None,
        notes: None,
        approved: false,
        approved_by: None,
        created_at: timestamp,
    }
}

fn schedule() -> WorkSchedule {
    WorkSchedule {
        id: Uuid::new_v4(),
        employee_id: Uuid::nil(),
        tenant_id: Uuid::nil(),
        weekday: 2,
        start_time: NaiveTimeWrapper(chrono::NaiveTime::from_hms_opt(9, 0, 0).expect("start")),
        end_time: NaiveTimeWrapper(chrono::NaiveTime::from_hms_opt(17, 0, 0).expect("end")),
        break_minutes: 60,
        tolerance_minutes: 5,
        valid_from: day(),
        valid_until: None,
    }
}

#[test]
fn lateness_beyond_the_tolerance_is_recorded_and_needs_no_approval() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(9, 10)),
        entry(ClockType::Out, at(17, 0)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    assert_eq!(shift.late_minutes, 5);
    assert_eq!(shift.early_leave_minutes, 0);
    assert_eq!(shift.overtime_minutes, 0);
    assert_eq!(shift.total_worked_minutes, 470);
    assert_eq!(shift.status, ShiftStatus::PendingApproval);
    assert!(!shift.approval_required);
    assert_eq!(shift.scheduled_start, Some(at(9, 0)));
    assert_eq!(shift.scheduled_end, Some(at(17, 0)));
}

#[test]
fn a_tolerance_absorbs_small_deviations() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(9, 3)),
        entry(ClockType::Out, at(16, 58)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    assert_eq!(shift.late_minutes, 0);
    assert_eq!(shift.early_leave_minutes, 0);
    assert_eq!(shift.status, ShiftStatus::Completed);
    assert!(!shift.approval_required);
}

#[test]
fn leaving_early_beyond_the_tolerance_is_flagged() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(9, 0)),
        entry(ClockType::Out, at(16, 50)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    assert_eq!(shift.early_leave_minutes, 5);
    assert_eq!(shift.late_minutes, 0);
    assert_eq!(shift.status, ShiftStatus::PendingApproval);
}

#[test]
fn overtime_past_the_schedule_requires_approval_only_above_an_hour() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(8, 0)),
        entry(ClockType::Out, at(17, 30)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    assert_eq!(shift.total_worked_minutes, 570);
    assert_eq!(shift.overtime_minutes, 90);
    assert_eq!(shift.late_minutes, 0);
    assert_eq!(shift.status, ShiftStatus::Completed);
    assert!(shift.approval_required);
}

#[test]
fn more_than_ten_minutes_late_requires_approval() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(9, 30)),
        entry(ClockType::Out, at(17, 0)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    assert_eq!(shift.late_minutes, 25);
    assert!(shift.approval_required);
    assert_eq!(shift.status, ShiftStatus::PendingApproval);
}

#[test]
fn breaks_are_subtracted_before_overtime_is_computed() {
    let svc = TimeClockService::new();
    let entries = vec![
        entry(ClockType::In, at(9, 0)),
        entry(ClockType::LunchStart, at(12, 0)),
        entry(ClockType::LunchEnd, at(13, 0)),
        entry(ClockType::Out, at(17, 30)),
    ];

    let shift = svc.compute_shift(entries, Some(&schedule()), day());

    // 8.5h span minus the 1h lunch leaves 7.5h worked, below the 8h schedule:
    // no overtime and no approval requirement.
    assert_eq!(shift.total_break_minutes, 60);
    assert_eq!(shift.total_worked_minutes, 450);
    assert_eq!(shift.overtime_minutes, 0);
    assert_eq!(shift.status, ShiftStatus::Completed);
}

