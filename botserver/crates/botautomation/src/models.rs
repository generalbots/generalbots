//! Row and request models for the automation crate.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::agent_schedules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentSchedule {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub title: String,
    pub goal: String,
    pub cron_expr: String,
    pub timezone: String,
    pub owner_user_id: Uuid,
    pub delivery: serde_json::Value,
    pub enabled: bool,
    pub max_runtime_secs: i32,
    pub tool_allowlist: Option<serde_json::Value>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentSchedule {
    /// Parses the `delivery` JSONB column into typed preferences; a malformed
    /// value degrades to the documented defaults instead of failing the read.
    pub fn delivery_prefs(&self) -> DeliveryPrefs {
        serde_json::from_value(self.delivery.clone()).unwrap_or_default()
    }

    /// Returns the tool allowlist as string names; `None` means unrestricted.
    pub fn allowlisted_tools(&self) -> Option<Vec<String>> {
        self.tool_allowlist.as_ref().and_then(|v| {
            serde_json::from_value::<Vec<String>>(v.clone()).ok()
        })
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::agent_schedules)]
pub struct NewAgentSchedule {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub title: String,
    pub goal: String,
    pub cron_expr: String,
    pub timezone: String,
    pub owner_user_id: Uuid,
    pub delivery: serde_json::Value,
    pub enabled: bool,
    pub max_runtime_secs: i32,
    pub tool_allowlist: Option<serde_json::Value>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::agent_runs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentRun {
    pub id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub trigger_kind: String,
    pub status: String,
    pub plan: Option<serde_json::Value>,
    pub steps: Option<serde_json::Value>,
    pub result_summary: Option<String>,
    pub artifacts: Option<serde_json::Value>,
    pub verdict: Option<serde_json::Value>,
    pub delivery_status: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::agent_runs)]
pub struct NewAgentRun {
    pub id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub trigger_kind: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::agent_spans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentSpan {
    pub id: Uuid,
    pub run_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub kind: String,
    pub name: String,
    pub input_ref: Option<String>,
    pub output_ref: Option<String>,
    pub tokens_in: Option<i32>,
    pub tokens_out: Option<i32>,
    pub vm_seconds: Option<i32>,
    pub status: String,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::agent_spans)]
pub struct NewAgentSpan {
    pub id: Uuid,
    pub run_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub kind: String,
    pub name: String,
    pub input_ref: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
}

/// Delivery preferences stored in the `delivery` JSONB column. Defaults mirror
/// the migration default `'{"email":true,"sms":false,"channels":[]}'`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryPrefs {
    #[serde(default = "default_email")]
    pub email: bool,
    #[serde(default)]
    pub sms: bool,
    #[serde(default)]
    pub channels: Vec<String>,
}

impl Default for DeliveryPrefs {
    fn default() -> Self {
        Self {
            email: true,
            sms: false,
            channels: Vec::new(),
        }
    }
}

fn default_email() -> bool {
    true
}

/// Body of `POST /api/automations/schedules`.
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleCreateBody {
    pub title: String,
    pub goal: String,
    pub cron_expr: String,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub delivery: Option<DeliveryPrefs>,
    #[serde(default)]
    pub max_runtime_secs: Option<i32>,
    #[serde(default)]
    pub tool_allowlist: Option<Vec<String>>,
}

/// Body of `PUT /api/automations/schedules/:id`; every field is optional.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScheduleUpdateBody {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub cron_expr: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub delivery: Option<DeliveryPrefs>,
    #[serde(default)]
    pub max_runtime_secs: Option<i32>,
    #[serde(default)]
    pub tool_allowlist: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_prefs_defaults_match_migration() {
        let prefs = DeliveryPrefs::default();
        assert!(prefs.email);
        assert!(!prefs.sms);
        assert!(prefs.channels.is_empty());
    }

    #[test]
    fn delivery_prefs_parse_empty_object_with_defaults() {
        let prefs: DeliveryPrefs = serde_json::from_str("{}").expect("parse empty object");
        assert_eq!(prefs, DeliveryPrefs::default());
    }

    #[test]
    fn delivery_prefs_roundtrip_full_object() {
        let raw = r#"{"email":false,"sms":true,"channels":["whatsapp","telegram"]}"#;
        let prefs: DeliveryPrefs = serde_json::from_str(raw).expect("parse full object");
        assert!(!prefs.email);
        assert!(prefs.sms);
        assert_eq!(prefs.channels.len(), 2);
    }

    #[test]
    fn schedule_create_body_all_optional_fields_default() {
        let body: ScheduleCreateBody = serde_json::from_str(
            r#"{"title":"t","goal":"g","cron_expr":"* * * * *"}"#,
        )
        .expect("parse minimal body");
        assert!(body.timezone.is_none());
        assert!(body.delivery.is_none());
        assert!(body.max_runtime_secs.is_none());
        assert!(body.tool_allowlist.is_none());
    }
}
