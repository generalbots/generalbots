use std::sync::Arc;
use std::time::Duration;

use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::providers;
use crate::state::IntegrationState;

/// Supported schedule tokens and their run intervals in seconds.
pub(crate) fn schedules() -> Vec<(&'static str, i64)> {
    vec![
        ("@every_5m", 5 * 60),
        ("@every_15m", 15 * 60),
        ("@hourly", 3600),
        ("@daily", 86_400),
        ("@weekly", 7 * 86_400),
    ]
}

pub(crate) fn schedules_map() -> std::collections::BTreeMap<&'static str, i64> {
    schedules().into_iter().collect()
}

fn interval_seconds(schedule: &str) -> Option<i64> {
    schedules()
        .into_iter()
        .find(|(name, _)| *name == schedule)
        .map(|(_, seconds)| seconds)
}

#[derive(diesel::QueryableByName)]
struct AutomationRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    owner_user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    provider_slug: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    action_key: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    params: Value,
    #[diesel(sql_type = diesel::sql_types::Text)]
    schedule: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

const TICK_SECONDS: u64 = 60;

fn is_due(row: &AutomationRow, now: chrono::DateTime<chrono::Utc>) -> bool {
    let Some(seconds) = interval_seconds(&row.schedule) else {
        return false;
    };
    match row.last_run_at {
        None => true,
        Some(last) => now.signed_duration_since(last).num_seconds() >= seconds,
    }
}

async fn execute_batch(state: &IntegrationState, batch: Vec<AutomationRow>) {
    for rule in batch {
        let scope = crate::scope::ConnectionScope {
            user_id: rule.owner_user_id,
            org_id: rule.org_id,
            branch_id: rule.branch_id,
            bot_id: rule.bot_id,
        };
        let outcome =
            providers::invoke_registered(state, &scope, &rule.provider_slug, &rule.action_key, &rule.params)
                .await;
        let summary = match &outcome {
            Ok(result) => result.summary.clone(),
            Err(error) => format!("error: {error}"),
        };
        let status = if outcome.is_ok() { "success" } else { "failure" };
        let mut conn = match state.pool.get() {
            Ok(conn) => conn,
            Err(_) => return,
        };
        diesel::sql_query(
            "UPDATE integration_automations \
             SET last_run_at = NOW(), last_outcome = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(rule.id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(summary.as_str())
        .execute(&mut conn)
        .ok();
        log::info!(
            "automation {} {} {}",
            rule.provider_slug,
            rule.action_key,
            status
        );
    }
}

async fn run_due(state: &IntegrationState) {
    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("automation tick skipped, pool unavailable: {error}");
            return;
        }
    };
    let rows: Vec<AutomationRow> = diesel::sql_query(
        "SELECT a.id, a.org_id, a.branch_id, a.bot_id, a.owner_user_id, \
                a.provider_slug, a.action_key, a.params, a.schedule, a.last_run_at \
         FROM integration_automations a \
         INNER JOIN bots b ON b.id = a.bot_id AND b.is_active = TRUE \
         WHERE a.enabled = TRUE",
    )
    .load(&mut conn)
    .unwrap_or_else(|error| {
        log::warn!("automation due query failed: {error:?}");
        Vec::new()
    });
    drop(conn);
    let now = chrono::Utc::now();
    let due: Vec<AutomationRow> = rows.into_iter().filter(|row| is_due(row, now)).collect();
    if due.is_empty() {
        return;
    }
    execute_batch(state, due).await;
}

/// Spawns the background automation runner. Disabled by setting the
/// `INTEGRATION_AUTOMATIONS` environment variable to `off`.
pub fn spawn(state: Arc<IntegrationState>) {
    if std::env::var("INTEGRATION_AUTOMATIONS").as_deref() == Ok("off") {
        log::info!("integration automations disabled by configuration");
        return;
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(TICK_SECONDS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            run_due(&state).await;
        }
    });
    log::info!("integration automation runner started");
}
