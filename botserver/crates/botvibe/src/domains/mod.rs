//! Split from `domains.rs` per #1443 (AGENTS.md 450-line rule).
//! #756/#774/#770 — Project domain bindings.
//!
//! Binds a custom domain to a project environment (domain → project →
//! container/Caddy route). Each binding is persisted in `project_domains`
//! (scoped by branch_id like the rest of the SaaS surface), keeps a
//! verification state (DNS TXT token — #774) and a TLS state driven through
//! the Caddy admin API (ACME — #770).
//!
//! The driver talks to Caddy's admin API (`CADDY_API_URL`); when the proxy
//! is unreachable (dev/offline) the operations return structured errors
//! instead of panicking.

mod manage;
mod resolve;
mod resolve_2;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::ensure_schema_sql;
use crate::caddy::{self, CaddyResult};
use crate::types::DbPool;
use crate::vm_lifecycle::VmLifecycle;

pub use manage::{DNS_VERIFY_PREFIX, DomainResult, ProjectDomains};
pub(crate) use manage::{ProjectRow, project_last_deploy_target};
pub use resolve::{BindDomainRequest, DomainBind, PROJECT_DOMAINS_SCHEMA};
pub(crate) use resolve_2::{DomainRow};
