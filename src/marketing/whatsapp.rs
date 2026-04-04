use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::channels::whatsapp::WhatsAppAdapter;
use crate::core::bot::channels::ChannelAdapter;
use crate::core::shared::schema::{
    marketing_campaigns, marketing_recipients,
};
use crate::core::shared::state::AppState;
use crate::marketing::campaigns::CrmCampaign;
use crate::core::shared::models::BotResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppCampaignPayload {
    pub to: String,
    pub body: String,
    pub media_url: Option<String>,
    pub campaign_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
    pub template_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplate {
    pub name: String,
    pub language: String,
    pub components: Vec<WhatsAppTemplateComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplateComponent {
    pub component_type: String,
    pub parameters: Vec<WhatsAppTemplateParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplateParameter {
    pub parameter_type: String,
    pub text: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppBusinessConfig {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub phone_number_id: Option<String>,
    pub business_account_id: Option<String>,
    pub access_token: Option<String>,
    pub webhooks_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMetrics {
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub total_read: i64,
    pub delivery_rate: f64,
    pub read_rate: f64,
}

fn get_whatsapp_config(
    state: &AppState,
    bot_id: Uuid,
) -> Result<WhatsAppBusinessConfig, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    #[derive(QueryableByName)]
    struct WhatsAppConfigRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        phone_number_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        business_account_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        access_token: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
        webhooks_verified: Option<bool>,
    }

    let config = diesel::sql_query("SELECT id, bot_id, phone_number_id, business_account_id, access_token, webhooks_verified FROM whatsapp_business WHERE bot_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<WhatsAppConfigRow>(&mut conn)
        .map_err(|e| format!("WhatsApp config not found: {}", e))?;

    Ok(WhatsAppBusinessConfig {
        id: config.id,
        bot_id: config.bot_id,
        phone_number_id: config.phone_number_id,
        business_account_id: config.business_account_id,
        access_token: config.access_token,
        webhooks_verified: config.webhooks_verified.unwrap_or(false),
    })
}

pub async fn send_whatsapp_message(
    state: &Arc<AppState>,
    bot_id: Uuid,
    payload: WhatsAppCampaignPayload,
) -> Result<WhatsAppSendResult, String> {
    let config = get_whatsapp_config(state, bot_id)?;

    if config.phone_number_id.is_none() || config.access_token.is_none() {
        return Err("WhatsApp not configured for this bot".to_string());
    }

    let adapter = WhatsAppAdapter::new(&state, bot_id);

    let result: Result<String, Box<dyn std::error::Error + Send + Sync>> = if let Some(template_name) = payload.template_name {
        adapter
            .send_template_message(
                &payload.to,
                &template_name,
                "pt_BR",
                vec![],
            )
            .await
    } else if let Some(media_url) = &payload.media_url {
        let media_type = if media_url.ends_with(".mp4") {
            "video"
        } else if media_url.ends_with(".png") || media_url.ends_with(".jpg") || media_url.ends_with(".jpeg") {
            "image"
        } else if media_url.ends_with(".pdf") {
            "document"
        } else {
            "image"
        };
        adapter
            .send_media_message(&payload.to, media_url, media_type, Some(&payload.body))
            .await
            .map(|_| "sent".to_string())
    } else {
        let response = BotResponse::new(
            bot_id.to_string(),
            "marketing".to_string(),
            payload.to.clone(),
            payload.body.clone(),
            "whatsapp".to_string(),
        );
        adapter.send_message(response).await.map(|_| "sent".to_string())
    };

    match result {
        Ok(message_id) => {
            if let Some(recipient_id) = payload.recipient_id {
                let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
                diesel::update(
                    marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("sent"),
                    marketing_recipients::sent_at.eq(Some(Utc::now())),
                    marketing_recipients::response.eq(serde_json::json!({ "message_id": message_id })),
                ))
                .execute(&mut conn)
                .ok();
            }

            Ok(WhatsAppSendResult {
                success: true,
                message_id: Some(message_id),
                error: None,
            })
        }
        Err(send_err) => {
            if let Some(recipient_id) = payload.recipient_id {
                let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
                diesel::update(
                    marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("failed"),
                    marketing_recipients::failed_at.eq(Some(Utc::now())),
                    marketing_recipients::error_message.eq(Some(send_err.to_string())),
                ))
                .execute(&mut conn)
                .ok();
            }

            Ok(WhatsAppSendResult {
                success: false,
                message_id: None,
                error: Some(send_err.to_string()),
            })
        }
    }
}

pub async fn send_bulk_whatsapp_messages(
    state: &Arc<AppState>,
    campaign_id: Uuid,
    contacts: Vec<(Uuid, String, String)>,
) -> Result<(i32, i32), String> {
    let mut sent = 0;
    let mut failed = 0;

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .first(&mut *state.conn.get().map_err(|e| format!("DB error: {}", e))?)
        .map_err(|_| "Campaign not found")?;

    let body = campaign
        .content_template
        .get("body")
        .and_then(|b| b.as_str())
        .unwrap_or("")
        .to_string();

    for (contact_id, phone, name) in contacts {
        let personalized_body = body.replace("{{name}}", &name);

        let payload = WhatsAppCampaignPayload {
            to: phone,
            body: personalized_body,
            media_url: campaign.content_template.get("media_url").and_then(|m| m.as_str()).map(String::from),
            campaign_id: Some(campaign_id),
            recipient_id: Some(contact_id),
            template_name: None,
        };

        match send_whatsapp_message(state, campaign.bot_id, payload).await {
            Ok(result) => {
                if result.success {
                    sent += 1;
                } else {
                    failed += 1;
                    log::error!("WhatsApp send failed: {:?}", result.error);
                }
            }
            Err(e) => {
                failed += 1;
                log::error!("WhatsApp error: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok((sent, failed))
}

pub async fn get_whatsapp_metrics(
    state: &Arc<AppState>,
    campaign_id: Uuid,
) -> Result<WhatsAppMetrics, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let recipients: Vec<(String, Option<serde_json::Value>)> = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq(campaign_id))
        .filter(marketing_recipients::channel.eq("whatsapp"))
        .select((marketing_recipients::status, marketing_recipients::response))
        .load(&mut conn)
        .map_err(|e| format!("Query error: {}", e))?;

    let total = recipients.len() as i64;
    let sent = recipients.iter().filter(|(s, _)| s == "sent").count() as i64;
    let delivered = recipients
        .iter()
        .filter(|(_, r)| {
            r.as_ref()
                .and_then(|v| v.get("status"))
                .and_then(|s| s.as_str())
                .map(|s| s == "delivered")
                .unwrap_or(false)
        })
        .count() as i64;
    let failed = recipients.iter().filter(|(s, _)| s == "failed").count() as i64;
    let read = recipients
        .iter()
        .filter(|(_, r)| {
            r.as_ref()
                .and_then(|v| v.get("status"))
                .and_then(|s| s.as_str())
                .map(|s| s == "read")
                .unwrap_or(false)
        })
        .count() as i64;

    Ok(WhatsAppMetrics {
        total_sent: total,
        total_delivered: delivered,
        total_failed: failed,
        total_read: read,
        delivery_rate: if sent > 0 { (delivered as f64 / sent as f64) * 100.0 } else { 0.0 },
        read_rate: if delivered > 0 { (read as f64 / delivered as f64) * 100.0 } else { 0.0 },
    })
}

pub async fn handle_webhook_event(
    state: &Arc<AppState>,
    payload: serde_json::Value,
) -> Result<(), String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    if let Some(statuses) = payload.get("entry").and_then(|e| e.as_array())
        .and_then(|e| e.first())
        .and_then(|e| e.get("changes"))
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("value"))
        .and_then(|v| v.get("statuses"))
        .and_then(|s| s.as_array())
    {
        for status in statuses {
            if let (Some(message_id), Some(status_str)) = (
                status.get("id").and_then(|m| m.as_str()),
                status.get("status").and_then(|s| s.as_str()),
            ) {
                let delivered_at = if status_str == "delivered" {
                    Some(Utc::now())
                } else {
                    None
                };

                diesel::update(marketing_recipients::table.filter(
                    marketing_recipients::response
                        .eq(serde_json::json!({ "message_id": message_id })),
                ))
                .set((
                    marketing_recipients::status.eq(status_str),
                    marketing_recipients::delivered_at.eq(delivered_at),
                ))
                .execute(&mut conn)
                .ok();
            }
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct SendWhatsAppRequest {
    pub to: String,
    pub body: String,
    pub media_url: Option<String>,
    pub template_name: Option<String>,
}

pub async fn send_whatsapp_api(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SendWhatsAppRequest>,
) -> Result<Json<WhatsAppSendResult>, (StatusCode, String)> {
    let bot_id = Uuid::nil();

    let payload = WhatsAppCampaignPayload {
        to: req.to,
        body: req.body,
        media_url: req.media_url,
        campaign_id: None,
        recipient_id: None,
        template_name: req.template_name,
    };

    match send_whatsapp_message(&state, bot_id, payload).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
