//! Time-bank (banco de horas) state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of time-bank entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeBankKind {
    /// Credit from overtime worked.
    Credit,
    /// Debit from time-off taken.
    Debit,
    /// Compensatory leave granted.
    Compensatory,
    /// Expiration of an old entry.
    Expiration,
}

/// A single time-bank ledger entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeBankEntry {
    /// Server-assigned entry ID.
    pub id: Uuid,
    /// Employee user ID.
    pub employee_id: String,
    /// Kind.
    pub kind: TimeBankKind,
    /// Minutes added (positive) or removed (negative).
    pub minutes: i32,
    /// Free-form reason.
    pub reason: String,
    /// When the entry was created.
    pub at: DateTime<Utc>,
}

/// Employee's time-bank account.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeBank {
    /// Employee user ID.
    pub employee_id: String,
    /// All entries.
    pub entries: Vec<TimeBankEntry>,
}

impl TimeBank {
    /// Net balance in minutes (sum of all entries).
    pub fn balance_min(&self) -> i32 {
        self.entries.iter().map(|e| e.minutes).sum()
    }

    /// Apply a new entry, mutating the bank in place.
    pub fn apply(&mut self, entry: TimeBankEntry) {
        self.entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_sums_entries() {
        let mut bank = TimeBank {
            employee_id: "emp-1".to_string(),
            entries: Vec::new(),
        };
        bank.apply(TimeBankEntry {
            id: Uuid::new_v4(),
            employee_id: "emp-1".to_string(),
            kind: TimeBankKind::Credit,
            minutes: 120,
            reason: "OT Tuesday".to_string(),
            at: Utc::now(),
        });
        bank.apply(TimeBankEntry {
            id: Uuid::new_v4(),
            employee_id: "emp-1".to_string(),
            kind: TimeBankKind::Debit,
            minutes: -60,
            reason: "Doctor appt".to_string(),
            at: Utc::now(),
        });
        assert_eq!(bank.balance_min(), 60);
    }
}
