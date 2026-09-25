//! Split from `teams.rs` per #1443 (AGENTS.md 450-line rule).

mod coordinator;
mod routes;
mod store;
#[cfg(test)]
mod tests;

use crate::agent_loop::AgentLoop;
use crate::prompt_manager::VibePromptManager;
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeRun, VibeRunConfig, VibeState, VibeUseCase};
use axum::{Extension, Json, Router};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use coordinator::{CreateTeamRequest, MemberRuntime, TeamCoordinator, TeamMember, TeamMemberRequest, VibeTeam};
pub(crate) use coordinator::{create_team};
pub use routes::{teams_router};
pub(crate) use routes::{TeamCreateResponse};
pub use store::{TeamRoutes, TeamStore};
