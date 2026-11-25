use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::campaigns::CrmCampaign;
use crate::compliance::{evaluate_send_gate, hydrate_suppressions, MarketingChannel};
use crate::email::direct_gate_inputs;
use crate::schema::{marketing_campaigns, marketing_recipients};
use crate::state::AppState;

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
pub struct WhatsAppMetrics {
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub total_read: i64,
    pub delivery_rate: f64,
    pub read_rate: f64,
}

pub async fn send_whatsapp_message(
    state: &Arc<AppState>,
    bot_id: Uuid,
    payload: WhatsAppCampaignPayload,
) -> Result<WhatsAppSendResult, String> {
    let media_type = payload.media_url.as_deref().map(|url| {
        if url.ends_with(".mp4") {
            "video"
        } else if url.ends_with(".png") || url.ends_with(".jpg") || url.ends_with(".jpeg") {
            "image"
        } else if url.ends_with(".pdf") {
            "document"
        } else {
            "image"
        }
    });

    let result = if payload.template_name.is_some() {
        (state.send_whatsapp)(
            bot_id,
            &payload.to,
            &payload.body,
            payload.template_name.as_deref(),
            None,
        )
    } else if payload.media_url.is_some() {
        (state.send_whatsapp)(
            bot_id,
            &payload.to,
            &payload.body,
            None,
            payload.media_url.as_deref(),
        )
    } else {
        (state.send_whatsapp)(bot_id, &payload.to, &payload.body, None, None)
    };

    match result {
        Ok(message_id) => {
            if let Some(recipient_id) = payload.recipient_id {
                let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
                if let Err(e) = diesel::update(
                    marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("sent"),
                    marketing_recipients::sent_at.eq(Some(Utc::now())),
                    marketing_recipients::response.eq(Some(serde_json::json!({ "message_id": message_id }))),
                ))
                .execute(&mut conn)
                {
                    log::warn!("Failed to mark WhatsApp recipient as sent: {e}");
                }
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
                if let Err(e) = diesel::update(
                    marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("failed"),
                    marketing_recipients::failed_at.eq(Some(Utc::now())),
                    marketing_recipients::error_message.eq(Some(send_err.clone())),
                ))
                .execute(&mut conn)
                {
                    log::warn!("Failed to mark WhatsApp recipient as failed: {e}");
                }
            }

            let _ = media_type;
            Ok(WhatsAppSendResult {
                success: false,
                message_id: None,
                error: Some(send_err),
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

    let (_, _, bot_id) = state.get_scope();

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .first(&mut *state.conn.get().map_err(|e| format!("DB error: {}", e))?)
        .map_err(|_| "Campaign not found")?;

    let default_metrics = serde_json::json!({});
    let template_data = campaign.metrics.as_ref().unwrap_or(&default_metrics);
    let body = template_data
        .get("body")
        .and_then(|b| b.as_str())
        .unwrap_or("")
        .to_string();

    for (contact_id, phone, name) in contacts {
        let personalized_body = crate::email_bulk::render_plain(&body, &name, "");

        let payload = WhatsAppCampaignPayload {
            to: phone,
            body: personalized_body,
            media_url: template_data.get("media_url").and_then(|m| m.as_str()).map(String::from),
            campaign_id: Some(campaign_id),
            recipient_id: Some(contact_id),
            template_name: None,
        };

        match send_whatsapp_message(state, bot_id, payload).await {
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

    let recipients: Vec<(Option<String>, Option<serde_json::Value>)> = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq(campaign_id))
        .filter(marketing_recipients::channel.eq("whatsapp"))
        .select((marketing_recipients::status, marketing_recipients::response))
        .load(&mut conn)
        .map_err(|e| format!("Query error: {}", e))?;

    let parse_response = |r: &Option<serde_json::Value>| -> Option<serde_json::Value> { r.clone() };

    let total = recipients.len() as i64;
    let sent = recipients.iter().filter(|(s, _)| s.as_deref() == Some("sent")).count() as i64;
    let delivered = recipients
        .iter()
        .filter(|(_, r)| {
            parse_response(r)
                .and_then(|v| v.get("status").cloned())
                .and_then(|s| s.as_str().map(|st| st == "delivered"))
                .unwrap_or(false)
        })
        .count() as i64;
    let failed = recipients.iter().filter(|(s, _)| s.as_deref() == Some("failed")).count() as i64;
    let read = recipients
        .iter()
        .filter(|(_, r)| {
            parse_response(r)
                .and_then(|v| v.get("status").cloned())
                .and_then(|s| s.as_str().map(|st| st == "read"))
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

                if let Err(e) = diesel::update(
                    marketing_recipients::table.filter(
                        marketing_recipients::response
                            .eq(Some(serde_json::json!({ "message_id": message_id }))),
                    ),
                )
                .set((
                    marketing_recipients::status.eq(status_str),
                    marketing_recipients::delivered_at.eq(delivered_at),
                ))
                .execute(&mut conn)
                {
                    log::warn!("Failed to persist WhatsApp delivery status: {e}");
                }
            }
        }
    }

    Ok(())
}

/// Resolves the caller's tenant from verified JWT claims (#2890, #2891). The
/// default bot scope is only a fallback when a claim is absent; the bot id is
/// never nil because provider configuration is keyed by a real bot row.
pub(crate) fn direct_send_scope(state: &AppState, headers: &HeaderMap) -> (Uuid, Uuid, Uuid) {
    let (default_org, default_branch, bot_id) = state.get_scope();
    let caller_branch = crate::scope::branch_from_jwt_pool(headers, &state.conn);
    let caller_org = botsecurity_core::verified_jwt::bearer_token(headers)
        .and_then(|token| botsecurity_core::verified_jwt::verify_saas_jwt(token).ok())
        .and_then(|claims| claims.org_id);
    (
        caller_org.filter(|id| !id.is_nil()).unwrap_or(default_org),
        caller_branch.filter(|id| !id.is_nil()).unwrap_or(default_branch),
        bot_id,
    )
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
    headers: HeaderMap,
    Json(req): Json<SendWhatsAppRequest>,
) -> Result<Json<WhatsAppSendResult>, (StatusCode, String)> {
    let (_, branch_id, bot_id) = direct_send_scope(&state, &headers);
    if branch_id.is_nil() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "no tenant scope".to_string()));
    }

    // #2891 — the one-off API must pass the same consent/suppression gate as
    // bulk/publish/worker before the provider sees the number.
    let mut conn = state.conn.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
    if let Err(error) = hydrate_suppressions(&mut conn) {
        log::error!("direct whatsapp could not hydrate suppressions: {error}");
    }
    let (status, subscribed, metadata) =
        direct_gate_inputs(&mut conn, branch_id, MarketingChannel::Whatsapp, &req.to)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let decision = evaluate_send_gate(
        branch_id,
        MarketingChannel::Whatsapp,
        &req.to,
        status.as_deref(),
        subscribed,
        metadata.as_ref(),
    );
    if !decision.is_allowed() {
        log::info!("direct whatsapp refused for {}: {}", req.to, decision.status());
        return Err((StatusCode::FORBIDDEN, format!("recipient refused: {}", decision.status())));
    }
    drop(conn);

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

#[cfg(test)]
mod direct_send_gate_tests {
    use super::*;
    use crate::compliance::{SendGateDecision, SuppressionKey, SuppressionStore};

    /// The shared gate is pure with respect to the HTTP layer: a suppressed
    /// address must be refused even when its consent record says opted-in.
    #[test]
    fn suppressed_address_is_refused_even_with_opt_in() {
        let branch = Uuid::new_v4();
        let address = format!("optedin-{}@example.invalid", Uuid::new_v4());
        SuppressionStore::global()
            .add(SuppressionKey::new(branch, MarketingChannel::Email, &address))
            .expect("test suppression store available");

        let decision = evaluate_send_gate(
            branch,
            MarketingChannel::Email,
            &address,
            None,
            Some(true),
            None,
        );
        assert_eq!(decision, SendGateDecision::Suppressed);
        assert!(!decision.is_allowed());
    }

    #[test]
    fn unsubscribed_status_and_opted_out_contact_are_refused() {
        let branch = Uuid::new_v4();
        let address = format!("unsubscribed-{}@example.invalid", Uuid::new_v4());
        let status = evaluate_send_gate(
            branch,
            MarketingChannel::Email,
            &address,
            Some("unsubscribed"),
            Some(true),
            None,
        );
        assert_eq!(status, SendGateDecision::Suppressed);

        let opted_out = evaluate_send_gate(
            branch,
            MarketingChannel::Email,
            &format!("optout-{address}"),
            None,
            Some(false),
            None,
        );
        assert_eq!(opted_out, SendGateDecision::NoConsent);
    }

    #[test]
    fn consented_unsuppressed_address_is_allowed() {
        let branch = Uuid::new_v4();
        let address = format!("allowed-{}@example.invalid", Uuid::new_v4());
        let decision = evaluate_send_gate(
            branch,
            MarketingChannel::Whatsapp,
            &address,
            Some("sent"),
            Some(true),
            None,
        );
        assert_eq!(decision, SendGateDecision::Allow);
        assert!(decision.is_allowed());
    }
}
