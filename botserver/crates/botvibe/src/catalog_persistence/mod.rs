//! Split from `catalog_persistence.rs` per #1443 (AGENTS.md 450-line rule).
//! Write-through persistence for the Vibe catalog entities (#816).
//!
//! `CanvasStore`, `IssueStore`, `SessionStore` and `TeamStore` keep their
//! in-memory `RwLock<Vec<_>>` as the live authority (same pattern as
//! `run_store`), but every mutation is also upserted into the corresponding
//! `vibe_*` table so data survives a restart. When a pool is provided the
//! store hydrates itself from the database on construction.

mod load;
mod save;

use crate::canvases::VibeCanvas;
use crate::issues::{IssueState, VibeIssue};
use crate::sessions::VibeSession;
use crate::teams::{TeamMember, VibeTeam};
use crate::types::{
    DbPool, VibeRun, VibeTelemetryEvent, VibeTelemetryEventType, VibeUseCase,
};
use diesel::prelude::*;
use uuid::Uuid;

pub use load::{load_canvases, load_issues, load_sessions, load_teams, load_telemetry_events, save_telemetry_event};
pub use save::{delete_canvas, save_canvas, save_issue, save_session, save_team};
