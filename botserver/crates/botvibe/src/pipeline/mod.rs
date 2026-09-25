//! Split from `pipeline.rs` per #1443 (AGENTS.md 450-line rule).
//! Run pipeline orchestration for the Vibe platform (Issue #805).
//!
//! A run is executed as a sequence of named stages (classify intent,
//! compile plan, execute plan). The pipeline engine drives the stages
//! through the tool registry, records per-stage telemetry and returns a
//! structured report. Stage outcomes mirror the registered tools, so a
//! not-yet-wired tool surfaces as an honest stage failure instead of a
//! silent success.

mod nodes;
mod runner;
#[cfg(test)]
mod tests;

use crate::telemetry::{ToolCallRecord, VibeTelemetry};
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeState, VibeToolCall, VibeUseCase};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use nodes::{PipelineStageKind};
pub use runner::{PipelineEngine, PipelineRunContext, PipelineRunReport, PipelineStage, PipelineStageReport, RunPipeline, StageStatus};
pub(crate) use runner::{stage, stage_approval, stage_continue, use_case_str};
