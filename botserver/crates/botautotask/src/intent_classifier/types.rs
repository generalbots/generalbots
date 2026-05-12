use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    AppCreate, Todo, Monitor, Action, Schedule, Goal, Tool, Unknown,
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppCreate => write!(f, "APP_CREATE"),
            Self::Todo => write!(f, "TODO"),
            Self::Monitor => write!(f, "MONITOR"),
            Self::Action => write!(f, "ACTION"),
            Self::Schedule => write!(f, "SCHEDULE"),
            Self::Goal => write!(f, "GOAL"),
            Self::Tool => write!(f, "TOOL"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl From<&str> for IntentType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "APP_CREATE" | "APP" | "APPLICATION" | "CREATE_APP" => Self::AppCreate,
            "TODO" | "TASK" | "REMINDER" => Self::Todo,
            "MONITOR" | "WATCH" | "ALERT" | "ON_CHANGE" => Self::Monitor,
            "ACTION" | "EXECUTE" | "DO" | "RUN" => Self::Action,
            "SCHEDULE" | "SCHEDULED" | "DAILY" | "WEEKLY" | "MONTHLY" | "CRON" => Self::Schedule,
            "GOAL" | "OBJECTIVE" | "TARGET" | "ACHIEVE" => Self::Goal,
            "TOOL" | "COMMAND" | "TRIGGER" | "WHEN_I_SAY" => Self::Tool,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedIntent {
    pub id: String,
    pub original_text: String,
    pub intent_type: IntentType,
    pub confidence: f64,
    pub entities: ClassifiedEntities,
    pub suggested_name: Option<String>,
    pub requires_clarification: bool,
    pub clarification_question: Option<String>,
    pub alternative_types: Vec<AlternativeClassification>,
    pub classified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassifiedEntities {
    pub subject: Option<String>,
    pub action: Option<String>,
    pub domain: Option<String>,
    pub time_spec: Option<TimeSpec>,
    pub condition: Option<String>,
    pub recipient: Option<String>,
    pub features: Vec<String>,
    pub tables: Vec<String>,
    pub trigger_phrases: Vec<String>,
    pub target_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSpec {
    pub schedule_type: ScheduleType,
    pub time: Option<String>,
    pub day: Option<String>,
    pub interval: Option<String>,
    pub cron_expression: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleType { Once, Daily, Weekly, Monthly, Interval, Cron }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeClassification {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub success: bool,
    pub intent_type: IntentType,
    pub message: String,
    pub created_resources: Vec<CreatedResource>,
    pub app_url: Option<String>,
    pub task_id: Option<String>,
    pub schedule_id: Option<String>,
    pub tool_triggers: Vec<String>,
    pub next_steps: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedResource {
    pub resource_type: String,
    pub name: String,
    pub path: Option<String>,
}
