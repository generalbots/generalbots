//! Helpers shared by the campaign publish fan-out (#2536).
//!
//! Kept out of `publish.rs` so the route handler stays within the file-size
//! limit while the channel gate and typed-error logic remain in one place.

use diesel::prelude::*;
use diesel::OptionalExtension;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::compliance::suppression::hydrate_suppressions;
use crate::compliance::ChannelError;
use crate::schema::{marketing_campaign_events, marketing_campaigns, marketing_recipients};
use crate::state::AppState;

/// Minimal HTML tag stripper for WhatsApp/Instagram plain-text payloads.
pub(crate) fn plain_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            out.push(' ');
        } else if !in_tag {
            out.push(c);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) async fn publish_instagram(
    state: &Arc<AppState>,
    bot_id: &Uuid,
    image_url: &str,
    caption: &str,
) -> Result<String, String> {
    let inner = state.get_config.clone();
    let get_config: botinstagram::state::GetConfigFn = std::sync::Arc::new(move |bot_id_str, key, default| {
        inner(&Uuid::parse_str(bot_id_str).unwrap_or_else(|_| Uuid::nil()), key, default)
    });
    let adapter = botinstagram::adapter::InstagramAdapter::with_config(&get_config, &bot_id.to_string());
    adapter
        .post_to_instagram(image_url, caption)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) fn set_recipient_state(
    conn: &mut diesel::PgConnection,
    campaign_id: Uuid,
    contact_id: Uuid,
    status: &str,
    error_message: Option<&str>,
) -> diesel::QueryResult<usize> {
    let target = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq(campaign_id))
        .filter(marketing_recipients::contact_id.eq(contact_id));
    if status == "failed" {
        diesel::update(target)
            .set((
                marketing_recipients::status.eq(status),
                marketing_recipients::failed_at.eq(Some(chrono::Utc::now())),
                marketing_recipients::error_message.eq(error_message.map(String::from)),
            ))
            .execute(conn)
    } else if status == "sent" {
        diesel::update(target)
            .set((
                marketing_recipients::status.eq(status),
                marketing_recipients::sent_at.eq(Some(chrono::Utc::now())),
                marketing_recipients::error_message.eq(error_message.map(String::from)),
            ))
            .execute(conn)
    } else {
        // Compliance skip: record the status only, never a send timestamp.
        diesel::update(target)
            .set((
                marketing_recipients::status.eq(status),
                marketing_recipients::error_message.eq(error_message.map(String::from)),
            ))
            .execute(conn)
    }
}

pub(crate) fn merge_metrics(
    conn: &mut diesel::PgConnection,
    campaign_id: Uuid,
    totals: &Map<String, Value>,
) -> diesel::QueryResult<()> {
    let mut stored: Value = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .select(marketing_campaigns::metrics)
        .first::<Option<Value>>(conn)
        .optional()?
        .flatten()
        .unwrap_or_else(|| json!({}));

    let total_sent: i64 = totals.values().map(|v| v["sent"].as_i64().unwrap_or(0)).sum();
    let total_failed: i64 = totals.values().map(|v| v["failed"].as_i64().unwrap_or(0)).sum();

    if let Some(obj) = stored.as_object_mut() {
        let sent = obj.get("sent").and_then(|v| v.as_i64()).unwrap_or(0) + total_sent;
        let failed = obj.get("failed").and_then(|v| v.as_i64()).unwrap_or(0) + total_failed;
        obj.insert("sent".into(), json!(sent));
        obj.insert("failed".into(), json!(failed));
        obj.insert("channels".into(), json!(totals));
    }

    diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(campaign_id)))
        .set((
            marketing_campaigns::metrics.eq(Some(stored)),
            marketing_campaigns::status.eq(Some(if total_failed > 0 && total_sent == 0 {
                "failed"
            } else {
                "completed"
            })),
        ))
        .execute(conn)?;

    Ok(())
}

/// #2536 — hydrate the durable do-not-contact list before a publish fan-out.
pub(crate) fn hydrate_gate(conn: &mut diesel::PgConnection) {
    if let Err(error) = hydrate_suppressions(conn) {
        log::error!("publish could not hydrate suppressions: {error}");
    }
}

/// #2536 — uniform handling for a channel with no provider: a typed error for
/// the response plus an event row for the monitor. No send is attempted.
pub(crate) fn channel_failure(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    campaign_id: Uuid,
    channel: &str,
) -> Value {
    let error = ChannelError::UnsupportedChannel {
        channel: channel.to_string(),
    };
    record_skip_event(conn, branch_id, campaign_id, channel, None, error.code());
    error.to_value()
}

/// Logs a failed recipient-state write instead of silently discarding it.
/// Persisted status is what drives retry, dead-letter and bounce reporting.
pub(crate) fn log_state_result(result: diesel::QueryResult<usize>, context: &str) {
    if let Err(error) = result {
        log::error!("publish state not persisted ({context}): {error}");
    }
}

/// Records a compliance skip in the campaign event stream. Best effort: a
/// monitoring write must never fail a send, but a failure is always logged.
pub(crate) fn record_skip_event(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    campaign_id: Uuid,
    channel: &str,
    recipient_email: Option<&str>,
    status: &str,
) {
    if let Err(error) = diesel::insert_into(marketing_campaign_events::table)
        .values((
            marketing_campaign_events::branch_id.eq(branch_id),
            marketing_campaign_events::campaign_id.eq(campaign_id),
            marketing_campaign_events::channel.eq(Some(channel.to_string())),
            marketing_campaign_events::event_type.eq("skip".to_string()),
            marketing_campaign_events::recipient_email.eq(recipient_email.map(str::to_string)),
            marketing_campaign_events::status.eq(Some(status.to_string())),
        ))
        .execute(conn)
    {
        log::error!("publish skip event not recorded for campaign {campaign_id}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::plain_text;

    #[test]
    fn plain_text_strips_tags_and_collapses_whitespace() {
        assert_eq!(plain_text("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(plain_text(""), "");
    }
}
