use botschema::conversation_autotasks as table;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{STATUS_RUNNING, STATUS_SCHEDULED};

#[derive(Queryable, Serialize)]
pub struct AutotaskRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub kind: String,
    pub schedule: String,
    pub schedule_human: Option<String>,
    pub command: String,
    pub unit_name: Option<String>,
    pub status: String,
    pub output_path: Option<String>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = table)]
pub(super) struct NewAutotask<'a> {
    pub(super) conversation_id: Uuid,
    pub(super) bot_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) title: &'a str,
    pub(super) kind: &'a str,
    pub(super) schedule: &'a str,
    pub(super) schedule_human: Option<&'a str>,
    pub(super) command: &'a str,
    pub(super) unit_name: Option<&'a str>,
    pub(super) status: &'a str,
    pub(super) next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutotaskRequest {
    pub title: String,
    pub schedule: String,
    pub command: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub schedule_human: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
}

pub(super) fn next_run_for(schedule: &str) -> Option<DateTime<Utc>> {
    #[cfg(feature = "automation")]
    {
        use std::str::FromStr;
        // Users and crontab write five fields (`0 9 * * 1`); the cron crate
        // parses six or seven (seconds first). Normalize instead of rejecting
        // every standard crontab entry.
        let fields = schedule.split_whitespace().count();
        let normalized = if fields == 5 {
            format!("0 {schedule}")
        } else {
            schedule.to_string()
        };
        let parsed = cron::Schedule::from_str(&normalized).ok()?;
        return parsed.after(&Utc::now()).next();
    }
    #[cfg(not(feature = "automation"))]
    {
        let _ = schedule;
        None
    }
}

pub(super) fn respond(rows: &[AutotaskRow]) -> Value {
    let active = rows
        .iter()
        .filter(|row| row.status == STATUS_RUNNING || row.status == STATUS_SCHEDULED)
        .count();
    let persistent = rows
        .iter()
        .filter(|row| row.kind == "systemd" && row.status == STATUS_RUNNING)
        .count();
    json!({
        "autotasks": rows,
        "active": active,
        "persistent": persistent,
        "has_work": active > 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{next_run_for, respond, AutotaskRow};
    use crate::main_module::routes::conversation_autotasks::{
        STATUS_ORPHANED, STATUS_RUNNING, STATUS_SCHEDULED, STATUS_STOPPED,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn row(id: Uuid, kind: &str, status: &str) -> AutotaskRow {
        AutotaskRow {
            id,
            conversation_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            user_id: Uuid::nil(),
            title: "Reminder".to_string(),
            kind: kind.to_string(),
            schedule: "0 9 * * 1".to_string(),
            schedule_human: Some("Every Monday at 09:00".to_string()),
            command: "echo hi".to_string(),
            unit_name: Some(
                crate::main_module::routes::conversation_autotasks::workspace::unit_name_for(id),
            ),
            status: status.to_string(),
            output_path: None,
            last_run_at: None,
            next_run_at: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// The chat indicator reads exactly these three fields.
    #[test]
    fn respond_reports_only_live_jobs_as_active() {
        let rows = vec![
            row(Uuid::new_v4(), "cron", STATUS_SCHEDULED),
            row(Uuid::new_v4(), "systemd", STATUS_RUNNING),
            row(Uuid::new_v4(), "cron", STATUS_STOPPED),
            row(Uuid::new_v4(), "systemd", STATUS_ORPHANED),
        ];
        let view = respond(&rows);
        assert_eq!(view["active"], 2);
        assert_eq!(
            view["persistent"], 1,
            "only a running systemd unit is persistent"
        );
        assert_eq!(view["has_work"], true);
        assert_eq!(view["autotasks"].as_array().map(|a| a.len()), Some(4));
    }

    #[test]
    fn stopping_every_job_clears_the_indicator() {
        let rows = vec![
            row(Uuid::new_v4(), "cron", STATUS_STOPPED),
            row(Uuid::new_v4(), "systemd", STATUS_ORPHANED),
        ];
        let view = respond(&rows);
        assert_eq!(view["active"], 0);
        assert_eq!(view["has_work"], false);
        assert_eq!(view["persistent"], 0);
    }

    #[test]
    fn unit_names_are_stable_and_namespaced() {
        let id = Uuid::nil();
        assert_eq!(
            crate::main_module::routes::conversation_autotasks::workspace::unit_name_for(id),
            "gb-autotask-00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            crate::main_module::routes::conversation_autotasks::workspace::conversation_root(id),
            "/opt/vibe/conversations/00000000-0000-0000-0000-000000000000"
        );
    }

    /// Reconciliation depends on a parseable schedule; an invalid one must not
    /// be reported as scheduled.
    #[test]
    fn next_run_is_computed_only_for_valid_cron() {
        assert!(next_run_for("not a cron expression").is_none());
        #[cfg(feature = "automation")]
        {
            let next =
                next_run_for("0 9 * * 1").expect("a 5-field crontab entry must yield a next run");
            assert!(next > Utc::now());
            let six = next_run_for("0 0 9 * * 1").expect("a 6-field expression is accepted");
            assert!(six > Utc::now());
        }
    }
}
