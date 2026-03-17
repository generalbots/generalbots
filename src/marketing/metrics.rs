use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::schema::{
    email_tracking, marketing_campaigns, marketing_recipients,
};
use crate::core::shared::state::AppState;
use crate::marketing::campaigns::CrmCampaign;

type RecipientTimestamps = (Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<DateTime<Utc>>);
type EmailTimestamps = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetrics {
    pub campaign_id: Uuid,
    pub channel: String,
    pub total_recipients: i64,
    pub sent: i64,
    pub delivered: i64,
    pub failed: i64,
    pub opened: i64,
    pub clicked: i64,
    pub replied: i64,
    pub open_rate: f64,
    pub click_rate: f64,
    pub conversion_rate: f64,
    pub cost_per_result: f64,
    pub total_spend: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBreakdown {
    pub channel: String,
    pub recipients: i64,
    pub sent: i64,
    pub delivered: i64,
    pub opened: i64,
    pub clicked: i64,
    pub open_rate: f64,
    pub click_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesMetric {
    pub timestamp: DateTime<Utc>,
    pub sent: i64,
    pub delivered: i64,
    pub opened: i64,
    pub clicked: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub total_recipients: i64,
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_opened: i64,
    pub total_clicked: i64,
    pub avg_open_rate: f64,
    pub avg_click_rate: f64,
    pub channel_breakdown: Vec<ChannelBreakdown>,
}

fn calculate_open_rate(delivered: i64, opened: i64) -> f64 {
    if delivered > 0 {
        (opened as f64 / delivered as f64) * 100.0
    } else {
        0.0
    }
}

fn calculate_click_rate(delivered: i64, clicked: i64) -> f64 {
    if delivered > 0 {
        (clicked as f64 / delivered as f64) * 100.0
    } else {
        0.0
    }
}

pub async fn get_campaign_metrics(
    state: &Arc<AppState>,
    campaign_id: Uuid,
) -> Result<CampaignMetrics, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .first(&mut conn)
        .map_err(|_| "Campaign not found")?;

    let recipients: Vec<(String, Option<serde_json::Value>)> = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq(campaign_id))
        .select((marketing_recipients::status, marketing_recipients::response))
        .load(&mut conn)
        .map_err(|e| format!("Query error: {}", e))?;

    let total = recipients.len() as i64;
    let sent = recipients.iter().filter(|(s, _)| s == "sent").count() as i64;
    let delivered = recipients
        .iter()
        .filter(|(s, r)| {
            let is_delivered = s == "delivered" || s == "sent";
            let has_delivery_status = r
                .as_ref()
                .and_then(|v| v.get("status"))
                .and_then(|s| s.as_str())
                .map(|st| st == "delivered" || st == "read")
                .unwrap_or(false);
            is_delivered || has_delivery_status
        })
        .count() as i64;
    let failed = recipients.iter().filter(|(s, _)| s == "failed").count() as i64;
    let replied = recipients
        .iter()
        .filter(|(_, r)| {
            r.as_ref()
                .and_then(|v| v.get("type"))
                .and_then(|t| t.as_str())
                .map(|t| t == "reply")
                .unwrap_or(false)
        })
        .count() as i64;

    let email_opens = if campaign.channel == "email" || campaign.channel == "multi" {
        email_tracking::table
            .filter(email_tracking::campaign_id.eq(campaign_id))
            .filter(email_tracking::opened.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .unwrap_or(0)
    } else {
        0
    };

    let email_clicks = if campaign.channel == "email" || campaign.channel == "multi" {
        email_tracking::table
            .filter(email_tracking::campaign_id.eq(campaign_id))
            .filter(email_tracking::clicked.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .unwrap_or(0)
    } else {
        0
    };

    let open_rate = calculate_open_rate(delivered, email_opens);
    let click_rate = calculate_click_rate(delivered, email_clicks);
    let conversion_rate = if delivered > 0 {
        (replied as f64 / delivered as f64) * 100.0
    } else {
        0.0
    };

    let budget = campaign.budget.unwrap_or(0.0);
    let cost_per_result = if sent > 0 {
        budget / sent as f64
    } else {
        0.0
    };

    Ok(CampaignMetrics {
        campaign_id,
        channel: campaign.channel,
        total_recipients: total,
        sent,
        delivered,
        failed,
        opened: email_opens,
        clicked: email_clicks,
        replied,
        open_rate,
        click_rate,
        conversion_rate,
        cost_per_result,
        total_spend: budget,
    })
}

pub async fn get_campaign_metrics_by_channel(
    state: &Arc<AppState>,
    campaign_id: Uuid,
) -> Result<Vec<ChannelBreakdown>, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let channels = vec!["email", "whatsapp", "instagram", "facebook", "telegram", "sms"];

    let mut breakdown = Vec::new();

    for channel in channels {
        let recipients: Vec<String> = marketing_recipients::table
            .filter(marketing_recipients::campaign_id.eq(campaign_id))
            .filter(marketing_recipients::channel.eq(channel))
            .select(marketing_recipients::status)
            .load(&mut conn)
            .unwrap_or_default();

        if recipients.is_empty() {
            continue;
        }

        let total = recipients.len() as i64;
        let sent = recipients.iter().filter(|s| *s == "sent").count() as i64;
        let delivered = recipients.iter().filter(|s| *s == "delivered" || *s == "read").count() as i64;
        let opened = if channel == "email" {
            email_tracking::table
                .filter(email_tracking::campaign_id.eq(campaign_id))
                .filter(email_tracking::opened.eq(true))
                .count()
                .get_result::<i64>(&mut conn)
                .unwrap_or(0)
        } else {
            0
        };
        let clicked = if channel == "email" {
            email_tracking::table
                .filter(email_tracking::campaign_id.eq(campaign_id))
                .filter(email_tracking::clicked.eq(true))
                .count()
                .get_result::<i64>(&mut conn)
                .unwrap_or(0)
        } else {
            0
        };

        breakdown.push(ChannelBreakdown {
            channel: channel.to_string(),
            recipients: total,
            sent,
            delivered,
            opened,
            clicked,
            open_rate: calculate_open_rate(delivered, opened),
            click_rate: calculate_click_rate(delivered, clicked),
        });
    }

    Ok(breakdown)
}

pub async fn get_time_series_metrics(
    state: &Arc<AppState>,
    campaign_id: Uuid,
    interval_hours: i32,
) -> Result<Vec<TimeSeriesMetric>, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let recipients: Vec<RecipientTimestamps> =
        marketing_recipients::table
            .filter(marketing_recipients::campaign_id.eq(campaign_id))
            .select((
                marketing_recipients::sent_at,
                marketing_recipients::delivered_at,
                marketing_recipients::failed_at,
            ))
            .load(&mut conn)
            .map_err(|e| format!("Query error: {}", e))?;

    let mut sent_by_hour: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut delivered_by_hour: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut opened_by_hour: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut clicked_by_hour: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();

    for (sent, delivered, failed) in recipients {
        if sent.is_some() || failed.is_some() {
            let ts = sent
                .or(failed)
                .map(|t| t.timestamp() / (interval_hours as i64 * 3600))
                .unwrap_or(0);
            *sent_by_hour.entry(ts).or_insert(0) += 1;
        }
        if let Some(d) = delivered {
            let ts = d.timestamp() / (interval_hours as i64 * 3600);
            *delivered_by_hour.entry(ts).or_insert(0) += 1;
        }
    }

    let email_events: Vec<EmailTimestamps> =
        email_tracking::table
            .filter(email_tracking::campaign_id.eq(campaign_id))
            .select((email_tracking::opened_at, email_tracking::clicked_at))
            .load(&mut conn)
            .unwrap_or_default();

    for (opened, clicked) in email_events {
        if let Some(ts) = opened {
            let key = ts.timestamp() / (interval_hours as i64 * 3600);
            *opened_by_hour.entry(key).or_insert(0) += 1;
        }
        if let Some(ts) = clicked {
            let key = ts.timestamp() / (interval_hours as i64 * 3600);
            *clicked_by_hour.entry(key).or_insert(0) += 1;
        }
    }

    let mut metrics: Vec<TimeSeriesMetric> = sent_by_hour
        .keys()
        .map(|&ts| TimeSeriesMetric {
            timestamp: DateTime::from_timestamp(ts * interval_hours as i64 * 3600, 0)
                .unwrap_or_else(Utc::now),
            sent: *sent_by_hour.get(&ts).unwrap_or(&0),
            delivered: *delivered_by_hour.get(&ts).unwrap_or(&0),
            opened: *opened_by_hour.get(&ts).unwrap_or(&0),
            clicked: *clicked_by_hour.get(&ts).unwrap_or(&0),
        })
        .collect();

    metrics.sort_by_key(|m| m.timestamp);
    Ok(metrics)
}

pub async fn get_aggregate_metrics(
    state: &Arc<AppState>,
    org_id: Uuid,
    bot_id: Uuid,
) -> Result<AggregateMetrics, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let total_campaigns: i64 = marketing_campaigns::table
        .filter(marketing_campaigns::org_id.eq(org_id))
        .filter(marketing_campaigns::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_campaigns: i64 = marketing_campaigns::table
        .filter(marketing_campaigns::org_id.eq(org_id))
        .filter(marketing_campaigns::bot_id.eq(bot_id))
        .filter(marketing_campaigns::status.eq("running"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let campaigns: Vec<CrmCampaign> = marketing_campaigns::table
        .filter(marketing_campaigns::org_id.eq(org_id))
        .filter(marketing_campaigns::bot_id.eq(bot_id))
        .select(marketing_campaigns::all_columns)
        .load(&mut conn)
        .unwrap_or_default();

    let campaign_ids: Vec<Uuid> = campaigns.iter().map(|c| c.id).collect();

    let recipients: Vec<(String, String)> = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq_any(campaign_ids.clone()))
        .select((marketing_recipients::channel, marketing_recipients::status))
        .load(&mut conn)
        .unwrap_or_default();

    let total_recipients = recipients.len() as i64;
    let total_sent = recipients.iter().filter(|(_, s)| s == "sent").count() as i64;
    let total_delivered = recipients.iter().filter(|(_, s)| s == "delivered" || s == "read").count() as i64;
    let _total_failed = recipients.iter().filter(|(_, s)| s == "failed").count() as i64;

    let total_opened: i64 = email_tracking::table
        .filter(email_tracking::campaign_id.eq_any(campaign_ids.clone()))
        .filter(email_tracking::opened.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_clicked: i64 = email_tracking::table
        .filter(email_tracking::campaign_id.eq_any(campaign_ids.clone()))
        .filter(email_tracking::clicked.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let avg_open_rate = if total_delivered > 0 {
        (total_opened as f64 / total_delivered as f64) * 100.0
    } else {
        0.0
    };

    let avg_click_rate = if total_delivered > 0 {
        (total_clicked as f64 / total_delivered as f64) * 100.0
    } else {
        0.0
    };

    let channel_breakdown = get_channel_breakdown(&mut conn, &campaign_ids.clone()).await?;

    Ok(AggregateMetrics {
        total_campaigns,
        active_campaigns,
        total_recipients,
        total_sent,
        total_delivered,
        total_opened,
        total_clicked,
        avg_open_rate,
        avg_click_rate,
        channel_breakdown,
    })
}

async fn get_channel_breakdown(
    conn: &mut diesel::PgConnection,
    campaign_ids: &[Uuid],
) -> Result<Vec<ChannelBreakdown>, String> {
    let channels = vec!["email", "whatsapp", "instagram", "facebook", "telegram", "sms"];
    let mut breakdown = Vec::new();

    for channel in channels {
        let recipients: Vec<String> = marketing_recipients::table
            .filter(marketing_recipients::campaign_id.eq_any(campaign_ids))
            .filter(marketing_recipients::channel.eq(channel))
            .select(marketing_recipients::status)
            .load(conn)
            .unwrap_or_default();

        if recipients.is_empty() {
            continue;
        }

        let total = recipients.len() as i64;
        let sent = recipients.iter().filter(|s| *s == "sent").count() as i64;
        let delivered = recipients.iter().filter(|s| *s == "delivered" || *s == "read").count() as i64;

        let opened = if channel == "email" {
            email_tracking::table
                .filter(email_tracking::campaign_id.eq_any(campaign_ids))
                .filter(email_tracking::opened.eq(true))
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        } else {
            0
        };

        let clicked = if channel == "email" {
            email_tracking::table
                .filter(email_tracking::campaign_id.eq_any(campaign_ids))
                .filter(email_tracking::clicked.eq(true))
                .count()
                .get_result::<i64>(conn)
                .unwrap_or(0)
        } else {
            0
        };

        breakdown.push(ChannelBreakdown {
            channel: channel.to_string(),
            recipients: total,
            sent,
            delivered,
            opened,
            clicked,
            open_rate: calculate_open_rate(delivered, opened),
            click_rate: calculate_click_rate(delivered, clicked),
        });
    }

    Ok(breakdown)
}

pub async fn get_campaign_metrics_api(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<Uuid>,
) -> Result<Json<CampaignMetrics>, (StatusCode, String)> {
    match get_campaign_metrics(&state, campaign_id).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_campaign_channel_breakdown_api(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<Uuid>,
) -> Result<Json<Vec<ChannelBreakdown>>, (StatusCode, String)> {
    match get_campaign_metrics_by_channel(&state, campaign_id).await {
        Ok(breakdown) => Ok(Json(breakdown)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_campaign_timeseries_api(
    State(state): State<Arc<AppState>>,
    Path((campaign_id, interval)): Path<(Uuid, i32)>,
) -> Result<Json<Vec<TimeSeriesMetric>>, (StatusCode, String)> {
    match get_time_series_metrics(&state, campaign_id, interval).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_aggregate_metrics_api(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AggregateMetrics>, (StatusCode, String)> {
    let (org_id, bot_id) = get_default_context(&state);

    match get_aggregate_metrics(&state, org_id, bot_id).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

fn get_default_context(state: &Arc<AppState>) -> (Uuid, Uuid) {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return (Uuid::nil(), Uuid::nil()),
    };

    #[derive(QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        org_id: Option<Uuid>,
    }

    let bot = diesel::sql_query("SELECT id, org_id FROM bots LIMIT 1")
        .get_result::<BotRow>(&mut conn)
        .ok();

    match bot {
        Some(b) => (b.org_id.unwrap_or(Uuid::nil()), b.id),
        None => (Uuid::nil(), Uuid::nil()),
    }
}
