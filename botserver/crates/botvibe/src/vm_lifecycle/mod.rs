//! Split from `vm_lifecycle.rs` per #1443 (AGENTS.md 450-line rule).
//! #744 — Per-project VM lifecycle.
//!
//! Models the lifecycle of project VMs: a dev VM exists from project start,
//! a prod VM is raised on publish, a CI runner can live on the dev VM, and
//! each env is tiered (small/medium/large). VM records are persisted in
//! `vm_instances` (scoped by branch_id like the rest of the SaaS surface);
//! the driver layer talks to the host Incus so `incus list` reflects the
//! real containers (dev `...-dev[-runner]` / prod `...-prod` naming).
//!
//! ALM mapping: ALM org = branch short id, ALM repo = project name
//! (mirrors g.tmp #744: ALM org=branch, repo=project).

mod lifecycle;
mod reap;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::ensure_schema_sql;
use crate::types::DbPool;

pub use lifecycle::{CreateVmRequest, VALID_ENVS, VALID_TIERS, VM_INSTANCES_SCHEMA, VmInstance, VmLifecycle, VmResult};
pub(crate) use lifecycle::{VmRow};
