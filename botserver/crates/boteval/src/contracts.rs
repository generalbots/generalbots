//! Contracts and validators. A contract describes what a good response looks
//! like, and the validator produces a structured pass/fail report.
//!
//! Split into the checks/validators (`checks`), the text-normalisation and
//! language helpers (`text`) and their unit tests; every public item keeps its
//! `contracts::` path through the re-exports below.

mod checks;
mod text;

#[cfg(test)]
mod tests;

pub use checks::{validate_response, validate_tool_floor, CheckOutcome, CheckResult};
