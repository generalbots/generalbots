//! Split from `telemetry.rs` per #1443 (AGENTS.md 450-line rule).
//! Telemetry module for Vibe run instrumentation.
//!
//! Records events, aggregates metrics per run and globally, and provides
//! query endpoints for dashboards. Uses in-memory storage with a cap of
//! 50 000 events.

mod core;
mod emit;
mod query;
#[cfg(test)]
mod tests;

use crate::types::{VibeRun, VibeTelemetryEvent, VibeTelemetryEventType, VibeUseCase};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

pub(crate) use core::{MAX_EVENTS};
pub use emit::{ToolCallRecord};
pub use query::{UseCaseMetrics, VibeGlobalMetrics, VibeRunMetricsSummary, VibeTelemetry};
