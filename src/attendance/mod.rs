//! Attendance Module
//!
//! Provides attendance tracking and human handoff queue functionality.
//!
//! ## Features
//!
//! - **Queue System**: Human handoff for conversations that need human attention
//! - **Keyword Services**: Check-in/out, break/resume tracking via keywords
//! - **Drive Integration**: S3 storage for attendance records
//!
//! ## Usage
//!
//! Enable with the `attendance` feature flag in Cargo.toml:
//! ```toml
//! [features]
//! default = ["attendance"]
//! ```

pub mod drive;
pub mod keyword_services;
pub mod queue;

// Re-export main types for convenience
pub use drive::{AttendanceDriveConfig, AttendanceDriveService, RecordMetadata, SyncResult};
pub use keyword_services::{
    AttendanceCommand, AttendanceRecord, AttendanceResponse, AttendanceService, KeywordConfig,
    KeywordParser, ParsedCommand,
};
pub use queue::{
    AssignRequest, AttendantStats, AttendantStatus, QueueFilters, QueueItem, QueueStatus,
    TransferRequest,
};

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Configure attendance routes
pub fn configure_attendance_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Queue management endpoints
        .route("/api/attendance/queue", get(queue::list_queue))
        .route("/api/attendance/attendants", get(queue::list_attendants))
        .route("/api/attendance/assign", post(queue::assign_conversation))
        .route(
            "/api/attendance/transfer",
            post(queue::transfer_conversation),
        )
        .route(
            "/api/attendance/resolve/:session_id",
            post(queue::resolve_conversation),
        )
        .route("/api/attendance/insights", get(queue::get_insights))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that types are properly exported
        let _config = KeywordConfig::default();
        let _parser = KeywordParser::new();
    }
}
