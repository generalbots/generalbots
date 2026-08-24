//! botautomation — NL scheduled adaptive agents and the
//! planner-executor-verifier engine (issues #1170 and #1171, backend).
//!
//! The crate owns four tables (`agent_schedules`, `agent_runs`,
//! `agent_spans`, `compute_usage_hourly`), a self-contained cron parser,
//! the run engine with forks/merges/repair loops, notification delivery
//! and the `/api/automations/*` dashboard API. LLM, tool and channel
//! capabilities are injected by the integrator through [`state::AutomationService`].

pub mod api;
pub mod cron;
pub mod delivery;
pub mod engine;
pub mod merge;
pub mod metering;
pub mod models;
pub mod scheduler;
pub mod schema;
pub mod state;

pub use api::configure_routes;
pub use models::{AgentRun, AgentSchedule, AgentSpan, DeliveryPrefs, ScheduleCreateBody};
pub use state::{AutomationService, DbPool, DeliveryFn, LlmFn, ToolFn};
