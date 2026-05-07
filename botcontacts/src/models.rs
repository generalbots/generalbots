use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_contacts)]
pub struct CrmContact {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_accounts)]
pub struct CrmAccount {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub annual_revenue: Option<f64>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_pipeline_stages)]
pub struct CrmPipelineStage {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub stage_order: i32,
    pub probability: i32,
    pub is_won: bool,
    pub is_lost: bool,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_deals)]
pub struct CrmDeal {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub am_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub title: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage_id: Option<Uuid>,
    pub stage: Option<String>,
    pub probability: i32,
    pub source: Option<String>,
    pub segment_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub actual_close_date: Option<chrono::NaiveDate>,
    pub period: Option<i32>,
    pub deal_date: Option<chrono::NaiveDate>,
    pub closed_at: Option<DateTime<Utc>>,
    pub lost_reason: Option<String>,
    pub won: Option<bool>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub custom_fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_opportunities)]
pub struct CrmOpportunity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage_id: Option<Uuid>,
    pub stage: String,
    pub probability: i32,
    pub source: Option<String>,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub actual_close_date: Option<chrono::NaiveDate>,
    pub won: Option<bool>,
    pub owner_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_activities)]
pub struct CrmActivity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub activity_type: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_notes)]
pub struct CrmNote {
    pub id: Uuid,
    pub org_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub content: String,
    pub author_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_campaigns)]
pub struct CrmCampaign {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub deal_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub channel: String,
    pub content_template: serde_json::Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metrics: serde_json::Value,
    pub budget: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, String>,
    pub source: Option<ContactSource>,
    pub status: ContactStatus,
    pub is_favorite: bool,
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContactStatus {
    #[default]
    Active,
    Inactive,
    Lead,
    Customer,
    Prospect,
    Archived,
}

impl std::fmt::Display for ContactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Lead => write!(f, "lead"),
            Self::Customer => write!(f, "customer"),
            Self::Prospect => write!(f, "prospect"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactSource {
    Manual,
    Import,
    WebForm,
    Api,
    Email,
    Meeting,
    Referral,
    Social,
}

impl std::fmt::Display for ContactSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Import => write!(f, "import"),
            Self::WebForm => write!(f, "web_form"),
            Self::Api => write!(f, "api"),
            Self::Email => write!(f, "email"),
            Self::Meeting => write!(f, "meeting"),
            Self::Referral => write!(f, "referral"),
            Self::Social => write!(f, "social"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Email,
    Call,
    Meeting,
    Task,
    Note,
    StatusChange,
    Created,
    Updated,
    Imported,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Call => write!(f, "call"),
            Self::Meeting => write!(f, "meeting"),
            Self::Task => write!(f, "task"),
            Self::Note => write!(f, "note"),
            Self::StatusChange => write!(f, "status_change"),
            Self::Created => write!(f, "created"),
            Self::Updated => write!(f, "updated"),
            Self::Imported => write!(f, "imported"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactActivity {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: Option<String>,
    pub related_id: Option<Uuid>,
    pub related_type: Option<String>,
    pub performed_by: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub fn stage_probability(stage: &str) -> i32 {
    match stage {
        "new" => 10,
        "qualified" => 25,
        "proposal" => 50,
        "negotiation" => 75,
        "won" | "converted" => 100,
        "lost" => 0,
        "qualification" => 25,
        _ => 25,
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn format_currency(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}
