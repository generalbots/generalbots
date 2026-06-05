//! Shift and shift-assignment types.

use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A scheduled break inside a shift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftBreak {
    /// Break name (e.g. "Lunch").
    pub name: String,
    /// Start time.
    pub start: NaiveTime,
    /// Duration in minutes.
    pub duration_min: u32,
    /// Whether the break is paid.
    pub paid: bool,
}

/// A recurring shift template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shift {
    /// Server-assigned shift ID.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Start time.
    pub start: NaiveTime,
    /// End time.
    pub end: NaiveTime,
    /// Optional list of breaks.
    pub breaks: Vec<ShiftBreak>,
    /// Total scheduled minutes excluding unpaid breaks.
    pub scheduled_minutes: u32,
}

impl Shift {
    /// Compute the net scheduled minutes after subtracting unpaid breaks.
    pub fn compute_scheduled_minutes(&self) -> u32 {
        let gross = minutes_between(self.start, self.end);
        let unpaid: u32 = self
            .breaks
            .iter()
            .filter(|b| !b.paid)
            .map(|b| b.duration_min)
            .sum();
        gross.saturating_sub(unpaid)
    }
}

fn minutes_between(start: NaiveTime, end: NaiveTime) -> u32 {
    let s = start.num_seconds_from_midnight();
    let e = end.num_seconds_from_midnight();
    if e >= s {
        (e - s) / 60
    } else {
        // Crosses midnight.
        ((86_400 - s) + e) / 60
    }
}

/// Assignment of an employee to a shift on a given date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftAssignment {
    /// Server-assigned assignment ID.
    pub id: Uuid,
    /// Employee user ID.
    pub employee_id: String,
    /// Shift ID.
    pub shift_id: Uuid,
    /// Calendar date.
    pub date: chrono::NaiveDate,
    /// Optional override notes.
    pub note: Option<String>,
    /// When the assignment was created.
    pub created_at: DateTime<Utc>,
}

/// Errors that may surface while managing shifts.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ShiftError {
    /// Start time is at or after end time on the same day.
    #[error("shift start must be before end (got {start} / {end})")]
    InvalidRange {
        /// Start.
        start: NaiveTime,
        /// End.
        end: NaiveTime,
    },
    /// Two unpaid breaks overlap.
    #[error("breaks overlap")]
    OverlappingBreaks,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn scheduled_minutes_excludes_unpaid_breaks() {
        let shift = Shift {
            id: Uuid::new_v4(),
            name: "Day 9-18".to_string(),
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            breaks: vec![ShiftBreak {
                name: "Lunch".to_string(),
                start: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                duration_min: 60,
                paid: false,
            }],
            scheduled_minutes: 0,
        };
        assert_eq!(shift.compute_scheduled_minutes(), 480);
    }

    #[test]
    fn shift_crossing_midnight() {
        let shift = Shift {
            id: Uuid::new_v4(),
            name: "Night".to_string(),
            start: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            breaks: vec![],
            scheduled_minutes: 0,
        };
        assert_eq!(shift.compute_scheduled_minutes(), 480);
        let _ = TimeDelta::hours(1);
    }
}
