//! Split from `projects.rs` per #1443 (AGENTS.md 450-line rule).
//! #743 — dynamic project registry for the Vibe agent.
//!
//! Replaces the hardcoded project/workspace modeling with a DB-backed
//! registry (`vibe_projects`): projects are created, listed, updated and
//! deleted through the REST API, so the underlying VM/tooling layers can
//! operate on whatever projects actually exist. Scoped by `branch_id`
//! (multi-tenant; nil UUID = global default-bot scope).

mod registry;
mod schema;
#[cfg(test)]
mod tests;

use crate::schema::ensure_schema_sql;
use crate::types::DbPool;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use registry::{ListProjectsQuery, ProjectRegistry, ProjectRegistryRef};
pub use schema::{CreateProjectRequest, Project, ProjectKind, UpdateProjectRequest, VIBE_PROJECTS_SCHEMA};
pub(crate) use schema::{ProjectRow};
