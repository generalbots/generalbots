use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudRule {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub condition_json: serde_json::Value,
    pub action: String,
    pub severity: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudEvent {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub risk_score: i32,
    pub risk_level: String,
    pub triggered_rules: Vec<serde_json::Value>,
    pub ml_score: Option<rust_decimal::Decimal>,
    pub action_taken: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudBlocklistEntry {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub block_type: String,
    pub block_value: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAssessmentRequest {
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAssessmentResult {
    pub risk_score: i32,
    pub risk_level: String,
    pub action_taken: String,
    pub triggered_rules: Vec<String>,
    pub ml_score: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub condition: serde_json::Value,
    pub action: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocklistRequest {
    pub block_type: String,
    pub block_value: String,
    pub reason: Option<String>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FraudStats {
    pub total_events: i64,
    pub blocked_count: i64,
    pub flagged_count: i64,
    pub reviewed_count: i64,
    pub high_risk_count: i64,
    pub rules_active: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityCheck {
    pub identifier: String,
    pub identifier_type: String,
    pub event_type: String,
    pub threshold: i32,
    pub window_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityResult {
    pub current_count: i32,
    pub threshold: i32,
    pub exceeded: bool,
}
