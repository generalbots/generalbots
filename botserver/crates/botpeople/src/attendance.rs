use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::Attendance;

type Storage = Arc<Mutex<HashMap<Uuid, Attendance>>>;

#[derive(Clone)]
pub struct AttendanceService {
    storage: Storage,
}

impl AttendanceService {
    pub fn new() -> Self {
        AttendanceService {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn clock_in(&self, employee_id: Uuid) -> Result<Attendance, String> {
        let today = Utc::now().date_naive();
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let already = store.values().any(|a| a.employee_id == employee_id && a.date == today && a.clock_out.is_none());
        if already {
            return Err("Employee already clocked in today".to_string());
        }
        drop(store);
        let id = Uuid::new_v4();
        let now = Utc::now();
        let attendance = Attendance {
            id,
            employee_id,
            date: today,
            clock_in: Some(now),
            clock_out: None,
            hours_worked: 0.0,
            overtime: 0.0,
        };
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.insert(id, attendance.clone());
        Ok(attendance)
    }

    pub fn clock_out(&self, employee_id: Uuid) -> Result<Attendance, String> {
        let today = Utc::now().date_naive();
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let record = store
            .values_mut()
            .find(|a| a.employee_id == employee_id && a.date == today && a.clock_out.is_none())
            .ok_or_else(|| "No active clock-in found for today".to_string())?;
        let now = Utc::now();
        record.clock_out = Some(now);
        if let Some(clock_in) = record.clock_in {
            let duration = now.signed_duration_since(clock_in);
            let hours = duration.num_minutes() as f64 / 60.0;
            record.hours_worked = hours.min(24.0);
            record.overtime = if hours > 8.0 { hours - 8.0 } else { 0.0 };
        }
        Ok(record.clone())
    }

    pub fn get_daily_summary(&self, employee_id: Uuid, date: NaiveDate) -> Result<Attendance, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store
            .values()
            .find(|a| a.employee_id == employee_id && a.date == date)
            .cloned()
            .ok_or_else(|| format!("No attendance record for {employee_id} on {date}"))
    }

    pub fn get_range(&self, employee_id: Uuid, from: NaiveDate, to: NaiveDate) -> Result<Vec<Attendance>, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut records: Vec<Attendance> = store
            .values()
            .filter(|a| a.employee_id == employee_id && a.date >= from && a.date <= to)
            .cloned()
            .collect();
        records.sort_by_key(|a| a.date);
        Ok(records)
    }

    pub fn calculate_hours_in_range(&self, employee_id: Uuid, from: NaiveDate, to: NaiveDate) -> Result<f64, String> {
        let records = self.get_range(employee_id, from, to)?;
        let total: f64 = records.iter().map(|a| a.hours_worked).sum();
        Ok(total)
    }

    pub fn calculate_overtime_in_range(&self, employee_id: Uuid, from: NaiveDate, to: NaiveDate) -> Result<f64, String> {
        let records = self.get_range(employee_id, from, to)?;
        let total: f64 = records.iter().map(|a| a.overtime).sum();
        Ok(total)
    }
}

impl Default for AttendanceService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod attendance_tests {
    use super::*;

    fn employee() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn clock_in_records_an_open_entry_for_today() {
        let service = AttendanceService::new();
        let id = employee();
        let record = service.clock_in(id).expect("clock in");
        assert_eq!(record.employee_id, id);
        assert!(record.clock_in.is_some());
        assert!(record.clock_out.is_none());
        assert_eq!(record.hours_worked, 0.0);
        assert_eq!(record.overtime, 0.0);
    }

    #[test]
    fn a_second_clock_in_on_the_same_day_is_refused() {
        let service = AttendanceService::new();
        let id = employee();
        service.clock_in(id).expect("first clock in");
        assert!(service.clock_in(id).is_err());
    }

    #[test]
    fn clock_out_without_clock_in_is_refused() {
        let service = AttendanceService::new();
        assert!(service.clock_out(employee()).is_err());
    }

    #[test]
    fn clock_out_closes_the_open_entry() {
        let service = AttendanceService::new();
        let id = employee();
        service.clock_in(id).expect("clock in");
        let closed = service.clock_out(id).expect("clock out");
        assert!(closed.clock_out.is_some());
        assert!(closed.hours_worked >= 0.0);
        assert!(closed.hours_worked <= 24.0);
        // A second clock-out finds nothing open.
        assert!(service.clock_out(id).is_err());
    }

    #[test]
    fn daily_summary_and_range_report_the_recorded_day() {
        let service = AttendanceService::new();
        let id = employee();
        service.clock_in(id).expect("clock in");
        service.clock_out(id).expect("clock out");
        let today = Utc::now().date_naive();

        let summary = service.get_daily_summary(id, today).expect("summary");
        assert_eq!(summary.employee_id, id);

        let range = service.get_range(id, today, today).expect("range");
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].date, today);
    }

    #[test]
    fn a_missing_day_is_reported_not_invented() {
        let service = AttendanceService::new();
        let id = employee();
        let past = NaiveDate::from_ymd_opt(2020, 1, 1).expect("date");
        assert!(service.get_daily_summary(id, past).is_err());
        assert!(service.get_range(id, past, past).expect("range").is_empty());
    }

    #[test]
    fn hours_and_overtime_aggregate_over_a_range() {
        let service = AttendanceService::new();
        let id = employee();
        let today = Utc::now().date_naive();
        service.clock_in(id).expect("clock in");
        service.clock_out(id).expect("clock out");

        let hours = service.calculate_hours_in_range(id, today, today).expect("hours");
        let overtime = service
            .calculate_overtime_in_range(id, today, today)
            .expect("overtime");
        assert!(hours >= 0.0);
        assert!(overtime >= 0.0);
        assert!(overtime <= hours);
    }

    #[test]
    fn records_are_scoped_per_employee() {
        let service = AttendanceService::new();
        let first = employee();
        let second = employee();
        service.clock_in(first).expect("first");
        service.clock_in(second).expect("second");
        let today = Utc::now().date_naive();
        assert_eq!(service.get_range(first, today, today).expect("range").len(), 1);
        assert_eq!(service.get_range(second, today, today).expect("range").len(), 1);
    }
}
