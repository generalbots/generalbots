use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::models::*;

#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub address_line1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub title: String,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub source: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadRequest {
    pub title: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
    pub lost_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOpportunityRequest {
    pub name: String,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpportunityRequest {
    pub name: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloseOpportunityRequest {
    pub won: bool,
    pub actual_close_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDealRequest {
    pub title: Option<String>,
    pub name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub source: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDealRequest {
    pub title: Option<String>,
    pub name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub source: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
    pub lost_reason: Option<String>,
    pub won: Option<bool>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub activity_type: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub stage: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub source: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PipelineStats {
    pub total_leads: i64,
    pub total_opportunities: i64,
    pub total_value: f64,
    pub won_value: f64,
    pub conversion_rate: f64,
    pub avg_deal_size: f64,
    pub stages: Vec<StageStats>,
}

#[derive(Debug, Serialize)]
pub struct StageStats {
    pub stage: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Deserialize)]
pub struct ImportPostgresRequest {
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub channel: String,
    pub deal_id: Option<Uuid>,
    pub content_template: Option<serde_json::Value>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub channel: Option<String>,
    pub content_template: Option<serde_json::Value>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CrmStats {
    pub total_contacts: i64,
    pub total_accounts: i64,
    pub total_leads: i64,
    pub total_opportunities: i64,
    pub total_campaigns: i64,
    pub pipeline_value: f64,
    pub won_this_month: i64,
    pub conversion_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct LeadStageQuery {
    pub stage: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadForm {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub value: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CountStageQuery {
    pub stage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContactsApiCreateRequest {
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
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub source: Option<ContactSource>,
    pub status: Option<ContactStatus>,
    pub group_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct ContactsApiUpdateRequest {
    pub first_name: Option<String>,
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
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub status: Option<ContactStatus>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListQuery {
    pub search: Option<String>,
    pub status: Option<ContactStatus>,
    pub group_id: Option<Uuid>,
    pub tag: Option<String>,
    pub is_favorite: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactListResponse {
    pub contacts: Vec<Contact>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub format: ImportFormat,
    pub data: String,
    pub field_mapping: Option<HashMap<String, String>>,
    pub group_id: Option<Uuid>,
    pub skip_duplicates: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Csv,
    Vcard,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub error_count: i32,
    pub errors: Vec<ImportError>,
    pub contact_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub line: i32,
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub contact_ids: Option<Vec<Uuid>>,
    pub group_id: Option<Uuid>,
    pub include_custom_fields: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Vcard,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub data: String,
    pub content_type: String,
    pub filename: String,
    pub contact_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionRequest {
    pub contact_ids: Vec<Uuid>,
    pub action: BulkAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BulkAction {
    Delete,
    Archive,
    AddToGroup { group_id: Uuid },
    RemoveFromGroup { group_id: Uuid },
    AddTag { tag: String },
    RemoveTag { tag: String },
    ChangeStatus { status: ContactStatus },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionResult {
    pub success: bool,
    pub affected_count: i32,
    pub errors: Vec<String>,
}
