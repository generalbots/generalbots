//! Chat runs: durable event log and resume helpers (#1665, #1666).
//!
//! The summary row lives in `agent_runs`; this module owns only the ordered
//! `run_events` log around it.

pub mod bus;
pub mod observability;
pub mod quality;
pub mod store;
