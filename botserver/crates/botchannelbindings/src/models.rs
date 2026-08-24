use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{call_logs, channel_bindings};

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = channel_bindings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelBinding {
    pub bot_id: Uuid,
    pub phone_default: Option<String>,
    pub whatsapp_number: Option<String>,
    pub telegram_username: Option<String>,
    pub domains: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

impl ChannelBinding {
    pub fn empty(bot_id: Uuid) -> Self {
        Self {
            bot_id,
            phone_default: None,
            whatsapp_number: None,
            telegram_username: None,
            domains: serde_json::json!([]),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = call_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallLog {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub direction: String,
    pub from_number: Option<String>,
    pub to_number: Option<String>,
    pub status: String,
    pub duration_sec: Option<i32>,
    pub recording_ref: Option<String>,
    pub transcript: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BindingsBody {
    #[serde(default)]
    pub phone_default: Option<String>,
    #[serde(default)]
    pub whatsapp_number: Option<String>,
    #[serde(default)]
    pub telegram_username: Option<String>,
    #[serde(default)]
    pub domains: Vec<String>,
}
