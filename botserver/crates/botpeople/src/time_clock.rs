use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockType {
    In,
    Out,
    BreakStart,
    BreakEnd,
    LunchStart,
    LunchEnd,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeClockEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub tenant_id: Uuid,
    pub clock_type: ClockType,
    pub timestamp: DateTime<Utc>,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub location_name: Option<String>,
    pub location_verified: bool,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub notes: Option<String>,
    pub approved: bool,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSchedule {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub tenant_id: Uuid,
    pub weekday: u8,
    pub start_time: NaiveTimeWrapper,
    pub end_time: NaiveTimeWrapper,
    pub break_minutes: i32,
    pub tolerance_minutes: i32,
    pub valid_from: NaiveDate,
    pub valid_until: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NaiveTimeWrapper(pub chrono::NaiveTime);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkShift {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub date: NaiveDate,
    pub entries: Vec<TimeClockEntry>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub total_worked_minutes: i32,
    pub total_break_minutes: i32,
    pub overtime_minutes: i32,
    pub late_minutes: i32,
    pub early_leave_minutes: i32,
    pub status: ShiftStatus,
    pub approval_required: bool,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShiftStatus {
    Open,
    InProgress,
    Completed,
    PendingApproval,
    Approved,
    Rejected,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkLocation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: i32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBankEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub date: NaiveDate,
    pub minutes: i32,
    pub kind: TimeBankKind,
    pub reason: String,
    pub approved: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeBankKind {
    Overtime,
    Undertime,
    Compensatory,
    Holiday,
    Vacation,
    Sick,
}

pub struct TimeClockService;

impl TimeClockService {
    pub fn new() -> Self {
        Self
    }

    pub fn verify_location<'a>(
        &self,
        geo: &GeoLocation,
        locations: &'a [WorkLocation],
    ) -> Option<&'a WorkLocation> {
        locations
            .iter()
            .filter(|l| l.active)
            .find(|l| {
                let distance = haversine_meters(geo.latitude, geo.longitude, l.latitude, l.longitude);
                distance <= l.radius_meters as f64
            })
    }

    pub fn compute_shift(
        &self,
        entries: Vec<TimeClockEntry>,
        schedule: Option<&WorkSchedule>,
        date: NaiveDate,
    ) -> WorkShift {
        let actual_start = entries
            .iter()
            .filter(|e| e.clock_type == ClockType::In)
            .min_by_key(|e| e.timestamp)
            .map(|e| e.timestamp);
        let actual_end = entries
            .iter()
            .filter(|e| e.clock_type == ClockType::Out)
            .max_by_key(|e| e.timestamp)
            .map(|e| e.timestamp);
        let total_break = compute_break_minutes(&entries);
        let total_worked = if let (Some(s), Some(e)) = (actual_start, actual_end) {
            ((e - s).num_minutes() - total_break as i64).max(0) as i32
        } else {
            0
        };
        let (late, early, overtime) = match schedule {
            Some(sched) => {
                // `saturating_sub` only clamps at `i32::MIN`, so on signed
                // integers an early arrival would make lateness negative
                // (10 - 5 is fine, but 3 - 5 must be 0, not -2). The trailing
                // `.max(0)` is the clamp that matters here.
                let late = actual_start
                    .and_then(|clock_in| {
                        combine(date, sched.start_time.0)
                            .map(|sched_start| (clock_in - sched_start).num_minutes() as i32)
                    })
                    .unwrap_or(0)
                    .saturating_sub(sched.tolerance_minutes)
                    .max(0);
                let early = actual_end
                    .and_then(|clock_out| {
                        combine(date, sched.end_time.0)
                            .map(|sched_end| (sched_end - clock_out).num_minutes() as i32)
                    })
                    .unwrap_or(0)
                    .saturating_sub(sched.tolerance_minutes)
                    .max(0);
                let scheduled_minutes = (sched.end_time.0 - sched.start_time.0).num_minutes() as i32;
                let overtime = (total_worked - scheduled_minutes).max(0);
                (late, early, overtime)
            }
            None => (0, 0, 0),
        };
        let status = if actual_start.is_none() && actual_end.is_none() {
            ShiftStatus::Absent
        } else if actual_end.is_none() {
            ShiftStatus::InProgress
        } else if late > 0 || early > 0 {
            ShiftStatus::PendingApproval
        } else {
            ShiftStatus::Completed
        };
        let approval_required = late > 10 || early > 10 || overtime > 60;
        WorkShift {
            id: Uuid::new_v4(),
            employee_id: entries.first().map(|e| e.employee_id).unwrap_or_else(Uuid::nil),
            date,
            entries,
            scheduled_start: schedule.and_then(|s| combine(date, s.start_time.0)),
            scheduled_end: schedule.and_then(|s| combine(date, s.end_time.0)),
            actual_start,
            actual_end,
            total_worked_minutes: total_worked,
            total_break_minutes: total_break,
            overtime_minutes: overtime,
            late_minutes: late,
            early_leave_minutes: early,
            status,
            approval_required,
            approved: false,
        }
    }
}

fn combine(date: NaiveDate, time: chrono::NaiveTime) -> Option<DateTime<Utc>> {
    let dt = date.and_time(time);
    Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}

fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_371_000.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

fn compute_break_minutes(entries: &[TimeClockEntry]) -> i32 {
    let mut break_starts: Vec<DateTime<Utc>> = entries
        .iter()
        .filter(|e| e.clock_type == ClockType::BreakStart || e.clock_type == ClockType::LunchStart)
        .map(|e| e.timestamp)
        .collect();
    let mut break_ends: Vec<DateTime<Utc>> = entries
        .iter()
        .filter(|e| e.clock_type == ClockType::BreakEnd || e.clock_type == ClockType::LunchEnd)
        .map(|e| e.timestamp)
        .collect();
    break_starts.sort();
    break_ends.sort();
    let mut total = 0_i64;
    for (s, e) in break_starts.iter().zip(break_ends.iter()) {
        total += (*e - *s).num_minutes();
    }
    total.max(0) as i32
}

impl Default for TimeClockService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "time_clock_schedule_tests.rs"]
mod schedule_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(emp: Uuid, kind: ClockType, minutes_offset: i64) -> TimeClockEntry {
        TimeClockEntry {
            id: Uuid::new_v4(),
            employee_id: emp,
            tenant_id: Uuid::nil(),
            clock_type: kind,
            timestamp: Utc::now() + chrono::Duration::minutes(minutes_offset),
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
            created_at: Utc::now(),
        }
    }

    #[test]
    fn compute_shift_full_day() {
        let svc = TimeClockService::new();
        let emp = Uuid::new_v4();
        let entries = vec![
            entry(emp, ClockType::In, 0),
            entry(emp, ClockType::LunchStart, 240),
            entry(emp, ClockType::LunchEnd, 300),
            entry(emp, ClockType::Out, 540),
        ];
        let shift = svc.compute_shift(entries, None, Utc::now().date_naive());
        assert_eq!(shift.total_break_minutes, 60);
        assert_eq!(shift.total_worked_minutes, 480);
    }

    #[test]
    fn location_within_radius_passes() {
        let svc = TimeClockService::new();
        let geo = GeoLocation {
            latitude: -23.55,
            longitude: -46.63,
            accuracy_meters: Some(10.0),
            captured_at: Utc::now(),
        };
        let locs = vec![WorkLocation {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            name: "Office".into(),
            latitude: -23.55,
            longitude: -46.63,
            radius_meters: 100,
            active: true,
        }];
        assert!(svc.verify_location(&geo, &locs).is_some());
    }

    fn office() -> WorkLocation {
        WorkLocation {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            name: "Office".into(),
            latitude: -23.55,
            longitude: -46.63,
            radius_meters: 100,
            active: true,
        }
    }

    fn geo(lat: f64, lon: f64) -> GeoLocation {
        GeoLocation {
            latitude: lat,
            longitude: lon,
            accuracy_meters: Some(10.0),
            captured_at: Utc::now(),
        }
    }

    #[test]
    fn a_location_outside_the_radius_fails() {
        let svc = TimeClockService::new();
        let locs = vec![office()];
        // ~1.1 km north of the office.
        assert!(svc.verify_location(&geo(-23.54, -46.63), &locs).is_none());
        assert!(svc.verify_location(&geo(-23.55, -46.63), &locs).is_some());
    }

    #[test]
    fn inactive_locations_never_verify() {
        let svc = TimeClockService::new();
        let mut loc = office();
        loc.active = false;
        assert!(svc.verify_location(&geo(-23.55, -46.63), &[loc]).is_none());
        assert!(svc
            .verify_location(&geo(-23.55, -46.63), &[])
            .is_none());
    }

    #[test]
    fn no_entries_means_absent() {
        let svc = TimeClockService::new();
        let shift = svc.compute_shift(Vec::new(), None, Utc::now().date_naive());
        assert_eq!(shift.status, ShiftStatus::Absent);
        assert_eq!(shift.total_worked_minutes, 0);
        assert!(!shift.approval_required);
    }

    #[test]
    fn an_open_shift_is_in_progress() {
        let svc = TimeClockService::new();
        let emp = Uuid::new_v4();
        let entries = vec![entry(emp, ClockType::In, 0)];
        let shift = svc.compute_shift(entries, None, Utc::now().date_naive());
        assert_eq!(shift.status, ShiftStatus::InProgress);
        assert_eq!(shift.total_worked_minutes, 0);
    }

    #[test]
    fn break_minutes_are_subtracted_and_never_negative() {
        let svc = TimeClockService::new();
        let emp = Uuid::new_v4();
        let entries = vec![
            entry(emp, ClockType::In, 0),
            entry(emp, ClockType::BreakStart, 60),
            entry(emp, ClockType::BreakEnd, 75),
            entry(emp, ClockType::Out, 120),
        ];
        let shift = svc.compute_shift(entries, None, Utc::now().date_naive());
        assert_eq!(shift.total_break_minutes, 15);
        assert_eq!(shift.total_worked_minutes, 105);
    }

    #[test]
    fn an_unmatched_break_end_does_not_go_negative() {
        let svc = TimeClockService::new();
        let emp = Uuid::new_v4();
        let entries = vec![
            entry(emp, ClockType::In, 0),
            entry(emp, ClockType::BreakEnd, 30),
            entry(emp, ClockType::Out, 60),
        ];
        let shift = svc.compute_shift(entries, None, Utc::now().date_naive());
        assert_eq!(shift.total_break_minutes, 0);
        assert_eq!(shift.total_worked_minutes, 60);
    }

    #[test]
    fn a_long_shift_requires_approval_for_overtime() {
        let svc = TimeClockService::new();
        let emp = Uuid::new_v4();
        // 10 hours worked, no schedule: overtime is only computed with one, so
        // the shift completes without approval.
        let entries = vec![entry(emp, ClockType::In, 0), entry(emp, ClockType::Out, 600)];
        let shift = svc.compute_shift(entries, None, Utc::now().date_naive());
        assert_eq!(shift.status, ShiftStatus::Completed);
        assert_eq!(shift.total_worked_minutes, 600);
    }
}
