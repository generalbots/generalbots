pub mod admin;
pub mod analytics;
pub mod enums;
pub mod memory_monitor;
pub mod models;
pub mod schema;
pub mod state;
pub mod test_utils;
pub mod utils;
pub mod prelude {




#[cfg(test)]


pub use enums::*;
pub use schema::*;


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


pub use botlib::models::{ApiResponse, Attachment, Suggestion};


pub use botlib::models::BotResponse;
pub use botlib::models::Session;
pub use botlib::models::UserMessage;


pub use models::{
    Automation, Bot, BotConfiguration, BotMemory, Click, MessageHistory, Organization,
    TriggerKind, User, UserLoginToken, UserPreference, UserSession,
};

#[cfg(feature = "tasks")]
pub use models::{NewTask, Task};

pub use utils::{
    create_conn, format_timestamp_plain, format_timestamp_srt, format_timestamp_vtt,
    get_content_type, parse_hex_color, sanitize_path_component, sanitize_path_for_filename,
    sanitize_sql_value, DbPool,
};

pub use crate::security::sql_guard::sanitize_identifier;




    pub use super::schema::*;
    pub use super::{
        ApiResponse, Attachment, Automation, Bot, BotConfiguration, BotError, BotMemory,
        BotResponse, BotResult, Click, DbPool, MessageHistory, MessageType, Organization,
        Session, Suggestion, TriggerKind, User, UserLoginToken, UserMessage, UserPreference,
        UserSession,
    };

    #[cfg(feature = "tasks")]
    pub use super::{NewTask, Task};


    pub use diesel::prelude::*;
    pub use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};


    pub use chrono::{DateTime, Utc};
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;
}
