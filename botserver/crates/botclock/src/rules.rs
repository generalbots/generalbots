//! Jurisdiction-aware workday rules.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Which country's labor rules apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Jurisdiction {
    /// Brazilian CLT (Consolidação das Leis do Trabalho).
    BrazilCLT,
    /// US Fair Labor Standards Act.
    UsFLSA,
    /// EU Working Time Directive.
    EuWTD,
    /// Generic / internationalized default.
    Generic,
}

/// Rounding policy applied to punches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundingPolicy {
    /// No rounding.
    None,
    /// Round to nearest 5 minutes.
    Nearest5,
    /// Round to nearest 15 minutes (common in US).
    Nearest15,
    /// Round to nearest 1 minute.
    Nearest1,
}

impl RoundingPolicy {
    /// Apply the rounding policy to a timestamp.
    pub fn apply(&self, ts: DateTime<Utc>) -> DateTime<Utc> {
        let secs = match self {
            Self::None => return ts,
            Self::Nearest1 => 60,
            Self::Nearest5 => 300,
            Self::Nearest15 => 900,
        };
        let epoch = ts.timestamp();
        let rounded = (epoch + secs / 2) / secs * secs;
        DateTime::<Utc>::from_timestamp(rounded, 0).unwrap_or(ts)
    }
}

/// Workday rules parameterized by jurisdiction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkdayRules {
    /// Jurisdiction.
    pub jurisdiction: Jurisdiction,
    /// Maximum legal daily work (minutes, before overtime).
    pub max_daily_min: u32,
    /// Maximum legal weekly work (minutes, before overtime).
    pub max_weekly_min: u32,
    /// Minimum intra-day rest in minutes (e.g. 60 for CLT art. 71).
    pub min_intra_rest_min: u32,
    /// Rounding policy applied to all punches.
    pub rounding: RoundingPolicy,
    /// Multiplier for the first 2 daily overtime hours.
    pub overtime_first_rate_bps: u32,
    /// Multiplier for overtime beyond the first 2 hours.
    pub overtime_rest_rate_bps: u32,
}

impl WorkdayRules {
    /// Brazilian CLT defaults (art. 58, 71, 192).
    pub fn brazil_clt() -> Self {
        Self {
            jurisdiction: Jurisdiction::BrazilCLT,
            max_daily_min: 480,
            max_weekly_min: 2_640,
            min_intra_rest_min: 60,
            rounding: RoundingPolicy::Nearest1,
            overtime_first_rate_bps: 15_000,
            overtime_rest_rate_bps: 20_000,
        }
    }

    /// Generic / safe default for international use.
    pub fn generic() -> Self {
        Self {
            jurisdiction: Jurisdiction::Generic,
            max_daily_min: 480,
            max_weekly_min: 2_400,
            min_intra_rest_min: 30,
            rounding: RoundingPolicy::None,
            overtime_first_rate_bps: 12_500,
            overtime_rest_rate_bps: 15_000,
        }
    }

    /// Returns the overtime minutes for a given total of minutes worked.
    pub fn overtime_minutes(&self, total_min: u32) -> u32 {
        total_min.saturating_sub(self.max_daily_min)
    }
}

/// Errors raised by the rules engine.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RulesError {
    /// Punches cannot span more than 24h.
    #[error("workday exceeds 24h")]
    WorkdayTooLong,
    /// Intra-day rest below legal minimum.
    #[error("intra-day rest {actual} < required {required}")]
    InsufficientRest {
        /// Actual rest minutes.
        actual: u32,
        /// Required rest minutes.
        required: u32,
    },
}

impl WorkdayRules {
    /// Check that the gap between two punch events is at least the legal
    /// minimum rest, and return [`RulesError::InsufficientRest`] otherwise.
    pub fn ensure_min_rest(&self, gap: Duration) -> Result<(), RulesError> {
        let minutes = gap.num_minutes().max(0) as u32;
        if minutes < self.min_intra_rest_min {
            return Err(RulesError::InsufficientRest {
                actual: minutes,
                required: self.min_intra_rest_min,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brazil_clt_defaults() {
        let r = WorkdayRules::brazil_clt();
        assert_eq!(r.jurisdiction, Jurisdiction::BrazilCLT);
        assert_eq!(r.max_daily_min, 480);
        assert_eq!(r.min_intra_rest_min, 60);
    }

    #[test]
    fn overtime_above_daily_limit() {
        let r = WorkdayRules::brazil_clt();
        assert_eq!(r.overtime_minutes(500), 20);
        assert_eq!(r.overtime_minutes(480), 0);
    }

    #[test]
    fn rounding_15() {
        let ts = DateTime::<Utc>::from_timestamp(1_700_000_007, 0).unwrap();
        let rounded = RoundingPolicy::Nearest15.apply(ts);
        assert_eq!(rounded.timestamp() % 900, 0);
    }
}
