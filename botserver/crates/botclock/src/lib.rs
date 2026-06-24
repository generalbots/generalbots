//! Electronic Time Clock — Registro de Ponto Eletrônico (REP).
//!
//! `botclock` exposes an internationalized time-clock model. Brazilian
//! CLT-specific rules (hour bank, "intrajornada" rest, overtime 50%/100%)
//! live in `botbrazil::clt` and are gated behind a [`rules::Jurisdiction`]
//! enum, so the same engine serves other countries with their own rule
//! set.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod punch;
pub mod shift;
pub mod rules;
pub mod payroll;
pub mod timebank;

pub use punch::{Punch, PunchKind, PunchChannel, PunchError};
pub use shift::{Shift, ShiftBreak, ShiftAssignment, ShiftError};
pub use rules::{Jurisdiction, WorkdayRules, RoundingPolicy, RulesError};
pub use payroll::{PayrollPeriod, PayrollSummary, OvertimeBreakdown, PayrollError};
pub use timebank::{TimeBank, TimeBankEntry, TimeBankKind};
