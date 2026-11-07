//! Weekly notification digest job (#1813).
//!
//! Settings stores `email_digest`; this module is the sender the checkbox was
//! missing. It runs on a scheduler tick, builds a small activity summary for
//! every opted-in user, and delivers it through the platform notification
//! sender (never a user's Gmail/Outlook account). Delivery state — last sent,
//! last attempt and last error — is written back to `user_preferences` so the
//! Settings UI can show whether the digest works.

use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;
use botsettings::settings_digest::{
    build_digest_body, digest_due, last_sent_at, DigestSummary, DIGEST_STATUS_KEY,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text, Timestamptz, Uuid as SqlUuid};
use serde_json::{json, Value};

use super::routes::collab_routes::notification_email::NotificationSender;

/// How often the scheduler looks for due digests. The weekly cadence itself is
/// enforced by `digest_due`; this is only the polling interval.
const TICK_SECONDS: u64 = 900;

fn database_timestamp(value: Option<DateTime<Utc>>, fallback: DateTime<Utc>) -> DateTime<Utc> {
    value.unwrap_or(fallback)
}

/// Starts the background digest loop. The first pass waits a minute so boot
/// work (migrations, Drive sync) is not competing with mail delivery.
pub fn spawn(state: Arc<AppState>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        log::info!(
            "Weekly digest job started (poll every {TICK_SECONDS}s, retry after failed sends)"
        );
        loop {
            let state_for_tick = state.clone();
            match tokio::task::spawn_blocking(move || run_due(&state_for_tick)).await {
                Ok(Ok(sent)) if sent > 0 => log::info!("Weekly digest job sent {sent} digest(s)"),
                Ok(Ok(_)) => {}
                Ok(Err(error)) => log::warn!("Weekly digest job failed: {error}"),
                Err(error) => log::warn!("Weekly digest worker task failed: {error}"),
            }
            tokio::time::sleep(Duration::from_secs(TICK_SECONDS)).await;
        }
    });
}

#[derive(diesel::QueryableByName)]
struct DigestUser {
    #[diesel(sql_type = SqlUuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    username: String,
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

fn count(
    conn: &mut diesel::PgConnection,
    sql: &str,
    user_id: uuid::Uuid,
    since: DateTime<Utc>,
) -> i64 {
    diesel::sql_query(sql)
        .bind::<SqlUuid, _>(user_id)
        .bind::<Timestamptz, _>(since)
        .get_result::<CountRow>(conn)
        .map(|row| row.count)
        .unwrap_or_else(|error| {
            log::warn!("digest summary query failed: {error}");
            0
        })
}

fn collect_summary(
    conn: &mut diesel::PgConnection,
    user_id: uuid::Uuid,
    username: &str,
    email: &str,
    since: DateTime<Utc>,
) -> DigestSummary {
    let messages = count(
        conn,
        "SELECT COUNT(*)::bigint AS count FROM message_history \
         WHERE user_id = $1 AND created_at >= $2",
        user_id,
        since,
    );
    let conversations = count(
        conn,
        "SELECT COUNT(DISTINCT session_id)::bigint AS count FROM message_history \
         WHERE user_id = $1 AND created_at >= $2",
        user_id,
        since,
    );
    let drive_files = count(
        conn,
        "SELECT COUNT(*)::bigint AS count FROM drive_files \
         WHERE user_id = $1 AND updated_at >= $2",
        user_id,
        since,
    );
    let active_tasks = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM conversation_autotasks \
         WHERE user_id = $1 AND status IN ('pending', 'scheduled', 'running')",
    )
    .bind::<SqlUuid, _>(user_id)
    .get_result::<CountRow>(conn)
    .map(|row| row.count)
    .unwrap_or(0);
    let task_runs = count(
        conn,
        "SELECT COUNT(*)::bigint AS count FROM conversation_autotasks \
         WHERE user_id = $1 AND last_run_at >= $2",
        user_id,
        since,
    );
    let mentions = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM collab_comments c \
         WHERE c.deleted = FALSE AND c.created_at >= $1 \
           AND EXISTS ( \
             SELECT 1 FROM jsonb_array_elements_text(c.mentions::jsonb) AS mention \
             WHERE LOWER(mention) IN (LOWER($2), LOWER($3)) \
           )",
    )
    .bind::<Timestamptz, _>(since)
    .bind::<Text, _>(username)
    .bind::<Text, _>(email)
    .get_result::<CountRow>(conn)
    .map(|row| row.count)
    .unwrap_or(0);

    DigestSummary {
        messages,
        conversations,
        drive_files,
        active_tasks,
        task_runs,
        mentions,
    }
}

fn updated_status(previous: &Value, now: DateTime<Utc>, error: Option<&str>) -> Value {
    let last_sent = if error.is_none() {
        Value::String(now.to_rfc3339())
    } else {
        previous.get("last_sent_at").cloned().unwrap_or(Value::Null)
    };
    json!({
        "last_sent_at": last_sent,
        "last_attempt_at": now.to_rfc3339(),
        "last_error": error.map(str::to_string),
    })
}

fn run_due(state: &Arc<AppState>) -> Result<usize, String> {
    // Recipients are loaded once, then the pooled connection is released
    // before each SMTP send: delivery can take seconds and must never pin a
    // pooled connection for the whole batch.
    let users = {
        let mut conn = state
            .conn
            .get()
            .map_err(|error| format!("database pool unavailable: {error}"))?;
        diesel::sql_query(
            "SELECT u.id, u.email::text AS email, u.username::text AS username \
             FROM users u \
             JOIN user_preferences p ON p.user_id = u.id \
               AND p.preference_key = 'notifications' \
             WHERE COALESCE(p.preference_value->>'email_digest', 'false') IN ('true', 'on', '1') \
               AND u.email <> ''",
        )
        .load::<DigestUser>(&mut conn)
        .map_err(|error| format!("cannot load digest recipients: {error}"))?
    };

    let sender = NotificationSender::configured();
    let now = Utc::now();
    let mut sent = 0usize;

    for user in users {
        let (previous_status, body) = {
            let mut conn = state
                .conn
                .get()
                .map_err(|error| format!("database pool unavailable: {error}"))?;
            let previous_status = botsettings::read_pref(&mut conn, user.id, DIGEST_STATUS_KEY);
            let timezone = botsettings::read_pref(&mut conn, user.id, "timezone")
                .as_str()
                .unwrap_or("UTC")
                .to_string();
            if !digest_due(&previous_status, now, &timezone) {
                continue;
            }
            let since = database_timestamp(
                last_sent_at(&previous_status),
                now - ChronoDuration::days(botsettings::settings_digest::DIGEST_INTERVAL_DAYS),
            );
            let summary = collect_summary(&mut conn, user.id, &user.username, &user.email, since);
            (
                previous_status,
                build_digest_body(&user.username, since, now, &timezone, summary),
            )
        };

        let delivery = match sender.as_ref() {
            Some(sender) => sender.send(&user.email, "Your General Bots weekly digest", &body),
            None => Err(
                "No platform notification sender is configured (set MAIL_* or the \
                 workspace mailbox); the digest cannot be delivered"
                    .to_string(),
            ),
        };

        let new_status = match delivery {
            Ok(()) => {
                sent += 1;
                botlog::safe_info!("Weekly digest sent to user {} ({})", user.id, user.email);
                updated_status(&previous_status, now, None)
            }
            Err(message) => {
                log::warn!(
                    "Weekly digest delivery failed for {}: {message}",
                    user.email
                );
                updated_status(&previous_status, now, Some(&message))
            }
        };
        let mut conn = state
            .conn
            .get()
            .map_err(|error| format!("database pool unavailable: {error}"))?;
        botsettings::write_pref(&mut conn, user.id, DIGEST_STATUS_KEY, &new_status);
    }

    Ok(sent)
}
