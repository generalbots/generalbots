use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub crm_enabled: bool,
    pub campaigns_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCrmLink {
    pub email_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCampaignLink {
    pub email_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub list_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadExtractionRequest {
    pub from: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadExtractionResponse {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReplyRequest {
    pub email_id: Uuid,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReplyResponse {
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCategoryResponse {
    pub category: String,
    pub confidence: f32,
}
