//! Row and request/response types for the memory service.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Visibility scope: visible to the owner only.
pub const SCOPE_PRIVATE: &str = "private";
/// Visibility scope: additionally visible to members of the same branch.
pub const SCOPE_BRANCH: &str = "branch";

const KIND_MAX_CHARS: usize = 32;
const CONTENT_MAX_CHARS: usize = 4000;
const SOURCE_MAX_CHARS: usize = 120;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::user_memories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserMemory {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub owner_user_id: Uuid,
    pub scope: String,
    pub kind: String,
    pub content: String,
    pub source: String,
    pub confidence: f32,
    pub pinned: bool,
    pub superseded_by: Option<Uuid>,
    pub embedding_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_confidence() -> f32 {
    0.8
}

/// Body of `POST /api/memory/items`.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryBody {
    #[serde(default = "default_kind")]
    pub kind: String,
    pub content: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default)]
    pub pinned: bool,
}

fn default_kind() -> String {
    "fact".to_string()
}

fn default_scope() -> String {
    SCOPE_PRIVATE.to_string()
}

/// Body of `PUT /api/memory/items/{id}`; only present fields are applied.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateMemoryBody {
    pub kind: Option<String>,
    pub content: Option<String>,
    pub scope: Option<String>,
    pub pinned: Option<bool>,
}

/// One entry inside an import payload.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryItem {
    #[serde(default = "default_kind")]
    pub kind: String,
    pub content: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub source: Option<String>,
}

/// Body of `POST /api/memory/import`.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportBody {
    pub items: Vec<MemoryItem>,
    #[serde(default)]
    pub dry_run: bool,
}

/// Outcome counters for an import run.
///
/// Live runs report performed work in `created`/`superseded`/`skipped` and
/// leave `would_create` at zero. Dry runs write nothing: `created` stays at
/// zero while `would_create`, `superseded` and `skipped` carry projections.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ImportReport {
    pub created: usize,
    pub superseded: usize,
    pub skipped: usize,
    pub would_create: usize,
}

/// Validates a scope value; returns its normalized form.
pub fn ensure_scope(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_lowercase();
    match normalized.as_str() {
        SCOPE_PRIVATE | SCOPE_BRANCH => Ok(normalized),
        _ => Err(format!("invalid scope '{value}': use private or branch")),
    }
}

/// Validates and normalizes a kind label (defaults to `fact` when blank).
pub fn ensure_kind(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_lowercase();
    if normalized.chars().count() > KIND_MAX_CHARS {
        return Err(format!("kind exceeds {KIND_MAX_CHARS} characters"));
    }
    if normalized.is_empty() {
        return Ok(default_kind());
    }
    Ok(normalized)
}

/// Validates memory content: non-empty after trim, bounded length.
pub fn ensure_content(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("content must not be empty".to_string());
    }
    if trimmed.chars().count() > CONTENT_MAX_CHARS {
        return Err(format!("content exceeds {CONTENT_MAX_CHARS} characters"));
    }
    Ok(trimmed.to_string())
}

/// Validates a provenance label; blanks normalize to `manual`.
pub fn ensure_source(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok("manual".to_string());
    }
    if trimmed.chars().count() > SOURCE_MAX_CHARS {
        return Err(format!("source exceeds {SOURCE_MAX_CHARS} characters"));
    }
    Ok(trimmed.to_string())
}
