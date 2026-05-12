use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub org_id: Uuid,
    pub org_slug: String,
    pub template: Option<String>,
    pub status: BotStatus,
    pub bucket: String,
    pub custom_ui: Option<String>,
    pub settings: BotSettings,
    pub access: BotAccess,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BotStatus {
    Active,
    Inactive,
    Maintenance,
    Creating,
    Error,
}

impl std::fmt::Display for BotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotStatus::Active => write!(f, "Active"),
            BotStatus::Inactive => write!(f, "Inactive"),
            BotStatus::Maintenance => write!(f, "Maintenance"),
            BotStatus::Creating => write!(f, "Creating"),
            BotStatus::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotSettings {
    pub llm_model: Option<String>,
    pub knowledge_bases: Vec<String>,
    pub channels: Vec<String>,
    pub webhooks: Vec<String>,
    pub schedules: Vec<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotAccess {
    pub admins: Vec<Uuid>,
    pub editors: Vec<Uuid>,
    pub viewers: Vec<Uuid>,
    pub is_public: bool,
    pub allowed_domains: Vec<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTemplate {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub files: Vec<TemplateFile>,
    pub preview_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub org_id: Uuid,
    pub template: Option<String>,
    pub created_by: Uuid,
    pub settings: Option<BotSettings>,
    pub custom_ui: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BotRoute {
    pub name: String,
    pub org_slug: String,
    pub bucket: String,
    pub custom_ui: Option<String>,
}

impl From<&BotConfig> for BotRoute {
    fn from(bot: &BotConfig) -> Self {
        BotRoute {
            name: bot.name.clone(),
            org_slug: bot.org_slug.clone(),
            bucket: bot.bucket.clone(),
            custom_ui: bot.custom_ui.clone(),
        }
    }
}
