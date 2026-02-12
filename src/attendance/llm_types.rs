// LLM assist types extracted from llm_assist.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct LlmAssistConfig {
    pub tips_enabled: bool,
    pub polish_enabled: bool,
    pub smart_replies_enabled: bool,
    pub auto_summary_enabled: bool,
    pub sentiment_enabled: bool,
    pub bot_system_prompt: Option<String>,
    pub bot_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TipRequest {
    pub session_id: Uuid,
    pub customer_message: String,
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}

#[derive(Debug, Deserialize)]
pub struct PolishRequest {
    pub session_id: Uuid,
    pub message: String,
    #[serde(default = "default_tone")]
    pub tone: String,
}

fn default_tone() -> String {
    "professional".to_string()
}

#[derive(Debug, Deserialize)]
pub struct SmartRepliesRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}

#[derive(Debug, Deserialize)]
pub struct SummaryRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SentimentRequest {
    pub session_id: Uuid,
    pub message: String,
    #[serde(default)]
    pub history: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TipResponse {
    pub success: bool,
    pub tips: Vec<AttendantTip>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttendantTip {
    pub tip_type: TipType,
    pub content: String,
    pub confidence: f32,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipType {
    Intent,
    Action,
    Warning,
    Knowledge,
    History,
    General,
}

#[derive(Debug, Serialize)]
pub struct PolishResponse {
    pub success: bool,
    pub original: String,
    pub polished: String,
    pub changes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SmartRepliesResponse {
    pub success: bool,
    pub replies: Vec<SmartReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SmartReply {
    pub text: String,
    pub tone: String,
    pub confidence: f32,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub success: bool,
    pub summary: ConversationSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ConversationSummary {
    pub brief: String,
    pub key_points: Vec<String>,
    pub customer_needs: Vec<String>,
    pub unresolved_issues: Vec<String>,
    pub sentiment_trend: String,
    pub recommended_action: String,
    pub message_count: i32,
    pub duration_minutes: i32,
}

#[derive(Debug, Serialize)]
pub struct SentimentResponse {
    pub success: bool,
    pub sentiment: SentimentAnalysis,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SentimentAnalysis {
    pub overall: String,
    pub score: f32,
    pub emotions: Vec<Emotion>,
    pub escalation_risk: String,
    pub urgency: String,
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Emotion {
    pub name: String,
    pub intensity: f32,
}
