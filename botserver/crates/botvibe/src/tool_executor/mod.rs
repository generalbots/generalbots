//! Split from `tool_executor.rs` per #1443 (AGENTS.md 450-line rule).

mod executor;
mod registry;

use crate::types::{VibeState, VibeToolCall, VibeToolResult, VibeUseCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use executor::{ToolHandler, VibeToolExecutor};
pub(crate) use executor::{RegisteredTool};
pub use registry::{ToolCategory, ToolDescriptor, ToolFuture, ToolRegistry, ToolSchema, ToolSchemaExt};
