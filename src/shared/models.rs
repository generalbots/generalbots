use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerKind {
    Scheduled = 0,
    TableUpdate = 1,
    TableInsert = 2,
    TableDelete = 3,
}

impl TriggerKind {
    pub fn _from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Scheduled),
            1 => Some(Self::TableUpdate),
            2 => Some(Self::TableInsert),
            3 => Some(Self::TableDelete),
            _ => None,
        }
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = system_automations)]
pub struct Automation {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub kind: i32,
    pub target: Option<String>,
    pub schedule: Option<String>,
    pub param: String,
    pub is_active: bool,
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub context_data: serde_json::Value,
    pub current_tool: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub bot_id: String,
    pub user_id: String,
    pub session_id: String,
    pub channel: String,
    pub content: String,
    pub message_type: i32,
    pub media_url: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub context_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,  // The button text that will be sent as message
    pub context: String,  // The context name to set when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResponse {
    pub bot_id: String,
    pub user_id: String,
    pub session_id: String,
    pub channel: String,
    pub content: String,
    pub message_type: i32,
    pub stream_token: Option<String>,
    pub is_complete: bool,
    pub suggestions: Vec<Suggestion>,
    pub context_name: Option<String>,
    pub context_length: usize,
    pub context_max_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = bot_memories)]
pub struct BotMemory {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub mod schema {
    diesel::table! {
        organizations (org_id) {
            org_id -> Uuid,
            name -> Text,
            slug -> Text,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        bots (id) {
            id -> Uuid,
            name -> Varchar,
            description -> Nullable<Text>,
            llm_provider -> Varchar,
            llm_config -> Jsonb,
            context_provider -> Varchar,
            context_config -> Jsonb,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_active -> Nullable<Bool>,
            tenant_id -> Nullable<Uuid>,
        }
    }

    diesel::table! {
        system_automations (id) {
            id -> Uuid,
            bot_id -> Uuid,
            kind -> Int4,
            target -> Nullable<Text>,
            schedule -> Nullable<Text>,
            param -> Text,
            is_active -> Bool,
            last_triggered -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        user_sessions (id) {
            id -> Uuid,
            user_id -> Uuid,
            bot_id -> Uuid,
            title -> Text,
            context_data -> Jsonb,
            current_tool -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        message_history (id) {
            id -> Uuid,
            session_id -> Uuid,
            user_id -> Uuid,
            role -> Int4,
            content_encrypted -> Text,
            message_type -> Int4,
            message_index -> Int8,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        users (id) {
            id -> Uuid,
            username -> Text,
            email -> Text,
            password_hash -> Text,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        clicks (id) {
            id -> Uuid,
            campaign_id -> Text,
            email -> Text,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        bot_memories (id) {
            id -> Uuid,
            bot_id -> Uuid,
            key -> Text,
            value -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        kb_documents (id) {
            id -> Text,
            bot_id -> Text,
            user_id -> Text,
            collection_name -> Text,
            file_path -> Text,
            file_size -> Integer,
            file_hash -> Text,
            first_published_at -> Text,
            last_modified_at -> Text,
            indexed_at -> Nullable<Text>,
            metadata -> Text,
            created_at -> Text,
            updated_at -> Text,
        }
    }

    diesel::table! {
        basic_tools (id) {
            id -> Text,
            bot_id -> Text,
            tool_name -> Text,
            file_path -> Text,
            ast_path -> Text,
            file_hash -> Text,
            mcp_json -> Nullable<Text>,
            tool_json -> Nullable<Text>,
            compiled_at -> Text,
            is_active -> Integer,
            created_at -> Text,
            updated_at -> Text,
        }
    }

    diesel::table! {
        kb_collections (id) {
            id -> Text,
            bot_id -> Text,
            user_id -> Text,
            name -> Text,
            folder_path -> Text,
            qdrant_collection -> Text,
            document_count -> Integer,
            is_active -> Integer,
            created_at -> Text,
            updated_at -> Text,
        }
    }

    diesel::table! {
        user_kb_associations (id) {
            id -> Text,
            user_id -> Text,
            bot_id -> Text,
            kb_name -> Text,
            is_website -> Integer,
            website_url -> Nullable<Text>,
            created_at -> Text,
            updated_at -> Text,
        }
    }

    diesel::table! {
        session_tool_associations (id) {
            id -> Text,
            session_id -> Text,
            tool_name -> Text,
            added_at -> Text,
        }
    }

    diesel::table! {
        bot_configuration (id) {
            id -> Uuid,
            bot_id -> Uuid,
            config_key -> Text,
            config_value -> Text,
 
            is_encrypted -> Bool,

            config_type -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }
}

pub use schema::*;
