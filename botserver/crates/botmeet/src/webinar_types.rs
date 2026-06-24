// Re-export all types from webinar_api::types as the single source of truth.
// WebinarStatsResponse is the only type unique to this module.

pub use crate::webinar_api::types::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarStatsResponse {
    pub active_webinars: i32,
    pub total_participants: i32,
    pub total_minutes: i64,
    pub storage_used_bytes: i64,
}
