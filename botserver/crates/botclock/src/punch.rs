//! Punch (clock-in/-out) event types.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of punch event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PunchKind {
    /// Start of work (entrada).
    In,
    /// Start of break (saída para intervalo).
    BreakStart,
    /// End of break (retorno do intervalo).
    BreakEnd,
    /// End of work (saída).
    Out,
}

impl PunchKind {
    /// Returns the opposite kind when possible.
    pub fn toggle(self) -> Option<Self> {
        match self {
            Self::In => Some(Self::Out),
            Self::Out => Some(Self::In),
            Self::BreakStart => Some(Self::BreakEnd),
            Self::BreakEnd => Some(Self::BreakStart),
        }
    }
}

/// Channel through which the punch was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PunchChannel {
    /// Web form (browser).
    Web,
    /// Mobile app.
    Mobile,
    /// Mobile app with GPS verification.
    MobileGps,
    /// Biometric terminal (face / fingerprint).
    Biometric,
    /// Manual entry by HR.
    Manual,
}

/// A single recorded punch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Punch {
    pub id: Uuid,
    /// Employee user ID.
    pub employee_id: String,
    /// Kind of punch.
    pub kind: PunchKind,
    /// Channel.
    pub channel: PunchChannel,
    /// When the punch was recorded (UTC).
    pub at: DateTime<Utc>,
    /// Optional GPS latitude (`MobileGps` only).
    pub lat: Option<f64>,
    /// Optional GPS longitude (`MobileGps` only).
    pub lng: Option<f64>,
    /// Optional geofence radius in meters used to validate the location.
    pub geofence_radius_m: Option<u32>,
    /// Free-form note (e.g. "forgot to clock in yesterday").
    pub note: Option<String>,
}

impl Punch {
    /// Calendar date in UTC for grouping into workdays.
    pub fn date(&self) -> NaiveDate {
        self.at.date_naive()
    }
}

/// Errors that may surface during punch validation.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PunchError {
    /// Two punches of the same kind in the same minute.
    #[error("duplicate punch {kind:?} for employee {employee} at {at}")]
    Duplicate {
        /// Kind.
        kind: PunchKind,
        /// Employee.
        employee: String,
        /// Timestamp.
        at: DateTime<Utc>,
    },
    /// Punch sequence is invalid (e.g. Out without a preceding In).
    #[error("invalid punch sequence")]
    InvalidSequence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_pairs() {
        assert_eq!(PunchKind::In.toggle(), Some(PunchKind::Out));
        assert_eq!(PunchKind::BreakStart.toggle(), Some(PunchKind::BreakEnd));
    }

    #[test]
    fn date_truncates_time() {
        let punch = Punch {
            id: Uuid::new_v4(),
            employee_id: "u1".to_string(),
            kind: PunchKind::In,
            channel: PunchChannel::Web,
            at: Utc::now(),
            lat: None,
            lng: None,
            geofence_radius_m: None,
            note: None,
        };
        let _ = punch.date();
    }
}
