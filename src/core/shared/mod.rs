//! Shared types and utilities
//!
//! This module re-exports common types from botlib and provides
//! botserver-specific shared functionality.

pub mod admin;
pub mod analytics;
pub mod models;
pub mod schema;
pub mod state;
#[cfg(test)]
pub mod test_utils;
pub mod utils;

// Re-export schema at module level for backward compatibility
pub use schema::*;

// Re-export from botlib for backward compatibility
pub use botlib::branding::{
    branding, copyright_text, footer_text, init_branding, is_white_label, log_prefix,
    platform_name, platform_short, BrandingConfig,
};
pub use botlib::error::{BotError, BotResult};
pub use botlib::message_types;
pub use botlib::message_types::MessageType;
pub use botlib::version::{
    get_botserver_version, init_version_registry, register_component, version_string,
    ComponentSource, ComponentStatus, ComponentVersion, VersionRegistry, BOTSERVER_NAME,
    BOTSERVER_VERSION,
};

// Re-export models from botlib
pub use botlib::models::{ApiResponse, Attachment, Suggestion};

// Re-export BotResponse and UserMessage with full path to avoid conflicts
pub use botlib::models::BotResponse;
pub use botlib::models::Session;
pub use botlib::models::UserMessage;

// Local re-exports - database models
pub use models::{
    Automation, Bot, BotConfiguration, BotMemory, Click, MessageHistory, NewTask, Organization,
    Task, TriggerKind, User, UserLoginToken, UserPreference, UserSession,
};

pub use utils::{create_conn, DbPool};

/// Prelude module for convenient imports
/// Usage: `use crate::shared::prelude::*;`
pub mod prelude {
    // Re-export everything commonly needed
    pub use super::schema::*;
    pub use super::{
        ApiResponse, Attachment, Automation, Bot, BotConfiguration, BotError, BotMemory,
        BotResponse, BotResult, Click, DbPool, MessageHistory, MessageType, NewTask, Organization,
        Session, Suggestion, Task, TriggerKind, User, UserLoginToken, UserMessage, UserPreference,
        UserSession,
    };

    // Diesel prelude for database operations
    pub use diesel::prelude::*;
    pub use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    // Common external types
    pub use chrono::{DateTime, Utc};
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
}
