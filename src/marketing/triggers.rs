use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::models::TriggerKind;
use crate::core::shared::schema::email_tracking;
use crate::core::shared::state::AppState;

pub fn trigger_deal_stage_change(
    conn: &mut PgConnection,
    deal_id: Uuid,
    _old_stage: &str,
    new_stage: &str,
    _bot_id: Uuid,
) {
    use crate::core::shared::schema::system_automations::dsl::*;

    let automations: Vec<crate::core::shared::models::Automation> = system_automations
        .filter(
            crate::core::shared::models::system_automations::dsl::kind
                .eq(TriggerKind::DealStageChange as i32),
        )
        .filter(is_active.eq(true))
        .filter(bot_id.eq(bot_id))
        .load(conn)
        .unwrap_or_default();

    for automation in automations {
        let target_stage = automation.target.as_deref().unwrap_or("");
        if target_stage.is_empty() || target_stage == new_stage {
            if let Err(e) = execute_campaign_for_deal(conn, &automation.param, deal_id) {
                log::error!("Failed to trigger campaign for deal stage change: {}", e);
            }
        }
    }
}

pub fn trigger_contact_change(
    conn: &mut PgConnection,
    contact_id: Uuid,
    change_type: &str,
    _bot_id: Uuid,
) {
    use crate::core::shared::schema::system_automations::dsl::*;

    let automations: Vec<crate::core::shared::models::Automation> = system_automations
        .filter(
            crate::core::shared::models::system_automations::dsl::kind
                .eq(TriggerKind::ContactChange as i32),
        )
        .filter(is_active.eq(true))
        .filter(bot_id.eq(bot_id))
        .load(conn)
        .unwrap_or_default();

    for automation in automations {
        let target_value = automation.target.as_deref().unwrap_or("");
        if target_value.is_empty() || target_value == change_type {
            if let Err(e) = execute_campaign_for_contact(conn, &automation.param, contact_id) {
                log::error!("Failed to trigger campaign for contact change: {}", e);
            }
        }
    }
}

pub fn trigger_email_opened(
    conn: &mut PgConnection,
    campaign_id: Uuid,
    contact_id: Uuid,
    _bot_id: Uuid,
) {
    use crate::core::shared::schema::system_automations::dsl::*;

    let automations: Vec<crate::core::shared::models::Automation> = system_automations
        .filter(
            crate::core::shared::models::system_automations::dsl::kind
                .eq(TriggerKind::EmailOpened as i32),
        )
        .filter(is_active.eq(true))
        .filter(bot_id.eq(bot_id))
        .load(conn)
        .unwrap_or_default();

    for automation in automations {
        let target_campaign = automation.target.as_deref().unwrap_or("");
        if target_campaign.is_empty() || target_campaign == campaign_id.to_string() {
            if let Err(e) = execute_campaign_for_contact(conn, &automation.param, contact_id) {
                log::error!("Failed to trigger campaign for email opened: {}", e);
            }
        }
    }
}

fn execute_campaign_for_deal(
    conn: &mut PgConnection,
    campaign_id: &str,
    deal_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::core::shared::schema::marketing_campaigns::dsl::marketing_campaigns;
    use crate::core::shared::schema::marketing_campaigns::id;
    use crate::core::shared::schema::marketing_campaigns::deal_id as campaign_deal_id;
    use crate::core::shared::schema::marketing_campaigns::status;
    use crate::core::shared::schema::marketing_campaigns::sent_at;

    if let Ok(cid) = Uuid::parse_str(campaign_id) {
        diesel::update(marketing_campaigns.filter(id.eq(cid)))
            .set((
                campaign_deal_id.eq(Some(deal_id)),
                status.eq("triggered"),
                sent_at.eq(Some(chrono::Utc::now())),
            ))
            .execute(conn)?;
        log::info!("Campaign {} triggered for deal {}", campaign_id, deal_id);
    }
    Ok(())
}

fn execute_campaign_for_contact(
    conn: &mut PgConnection,
    campaign_id: &str,
    contact_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::core::shared::schema::marketing_campaigns as mc_table;
    use crate::core::shared::schema::marketing_recipients as mr_table;

    if let Ok(cid) = Uuid::parse_str(campaign_id) {
        diesel::update(mc_table::table.filter(mc_table::id.eq(cid)))
            .set((
                mc_table::status.eq("triggered"),
                mc_table::sent_at.eq(Some(chrono::Utc::now())),
            ))
            .execute(conn)?;

        diesel::insert_into(mr_table::table)
            .values((
                mr_table::id.eq(Uuid::new_v4()),
                mr_table::campaign_id.eq(Some(cid)),
                mr_table::contact_id.eq(Some(contact_id)),
                mr_table::channel.eq("automation"),
                mr_table::status.eq("pending"),
                mr_table::created_at.eq(chrono::Utc::now()),
            ))
            .execute(conn)?;

        log::info!(
            "Campaign {} triggered for contact {}",
            campaign_id,
            contact_id
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct EmailOpenRequest {
    pub message_id: Option<String>,
    pub token: Option<String>,
}

pub async fn track_email_open(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmailOpenRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    if let Some(token_str) = req.token {
        if let Ok(token) = Uuid::parse_str(&token_str) {
            let record: Option<(Uuid, Option<Uuid>)> = email_tracking::table
                .filter(email_tracking::open_token.eq(token))
                .select((email_tracking::id, email_tracking::recipient_id))
                .first(&mut conn)
                .ok();

            if let Some((id, recipient_id)) = record {
                diesel::update(email_tracking::table.filter(email_tracking::id.eq(id)))
                    .set((
                        email_tracking::opened.eq(true),
                        email_tracking::opened_at.eq(Some(now)),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

                log::info!("Email opened: tracking_id={}", id);

                if let Some(contact_id) = recipient_id {
                    let (bot_id, _) = crate::core::bot::get_default_bot(&mut conn);
                    trigger_email_opened(&mut conn, Uuid::nil(), contact_id, bot_id);
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "status": "tracked" })))
}
