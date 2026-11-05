//! Chat runs: durable event log and resume helpers (#1665, #1666).
//!
//! The summary row lives in `agent_runs`; this module owns only the ordered
//! `run_events` log around it.

pub mod bus;
pub mod caps;
pub mod caps_resolve;
pub mod goal_runner;
pub mod group_budget;
pub mod observability;
pub mod quality;
pub mod steering;
pub mod subagents;
pub mod store;
pub mod ttft;
pub mod ttft_stats;
pub mod tools;
