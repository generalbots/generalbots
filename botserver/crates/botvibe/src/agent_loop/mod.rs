//! Split from `agent_loop.rs` per #1443 (AGENTS.md 450-line rule).

mod core;
mod llm;
mod llm_run;
mod llm_request;
mod llm_response;
mod llm_tool_calls;
mod llm_payload;
mod parsing;
mod run_loop;
mod verification;
#[cfg(test)]
mod tests;

use crate::permissions::{PermissionEngine, PermissionEngineRef};
use crate::prompt_manager::VibePromptManager;
use crate::skills::SkillStore;
use crate::telemetry::{ToolCallRecord, VibeTelemetry};
use crate::tool_executor::VibeToolExecutor;
use crate::types::{
    VibeProgressEvent, VibeRun, VibeRunState, VibeState, VibeToolCall, VibeUseCase,
};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

pub(crate) use core::{DEFAULT_TIMEOUT_SECS, MAX_TOOL_RESULT_CHARS, MAX_TOOL_RETRIES};
#[cfg(target_os = "windows")]
pub(crate) use core::{preferred_source_path};
pub use llm::{AgentLoop};
pub(crate) use llm::{LLM_REQUEST_TIMEOUT_SECS, LLM_RETRY_BACKOFF_SECS, LLM_USER_AGENT, MAX_LLM_RETRIES};
pub(crate) use llm_tool_calls::{looks_like_tool_intent};
pub use llm_payload::{LlmUsage};
pub(crate) use llm_payload::{estimate_llm_cost, native_tool_calls_from_value, parse_sse, restore_tool_name_from_llm, sanitize_tool_name_for_llm, usage_from_payload};
pub(crate) use parsing::{ExtractedToolCall, MAX_EMPTY_PARSE_RETRIES, canonical_tool_calls_json, extract_json_array, extract_json_object};
pub(crate) use run_loop::{DEFAULT_MAX_STEPS, truncate};
pub(crate) use verification::{MAX_VERIFY_FAILURES, ToolStep};
#[cfg(target_os = "windows")]
pub(crate) use verification::{path_requires_repair};
