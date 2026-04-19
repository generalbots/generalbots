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

use crate::core::shared::schema::marketing_campaigns;
use crate::core::shared::state::AppState;
use crate::core::bot::get_default_bot;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_campaigns)]
pub struct CrmCampaign {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub deal_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub channel: String,
    pub content_template: serde_json::Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metrics: serde_json::Value,
    pub budget: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub channel: String,
    pub deal_id: Option<Uuid>,
    pub content_template: Option<serde_json::Value>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub channel: Option<String>,
    pub content_template: Option<serde_json::Value>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    use diesel::prelude::*;
    use crate::core::shared::schema::bots;

    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    
    let org_id = bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::org_id)
        .first::<Option<Uuid>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or(Uuid::nil());

    (org_id, bot_id)
}

pub async fn list_campaigns(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CrmCampaign>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let campaigns: Vec<CrmCampaign> = marketing_campaigns::table
        .filter(marketing_campaigns::org_id.eq(org_id))
        .filter(marketing_campaigns::bot_id.eq(bot_id))
        .order(marketing_campaigns::created_at.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(campaigns))
}

pub async fn get_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    Ok(Json(campaign))
}

pub async fn create_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let scheduled = req.scheduled_at.and_then(|s| {
        DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
    });

    let campaign = CrmCampaign {
        id,
        org_id,
        bot_id,
        deal_id: req.deal_id,
        name: req.name,
        status: "draft".to_string(),
        channel: req.channel,
        content_template: req.content_template.unwrap_or(serde_json::json!({})),
        scheduled_at: scheduled,
        sent_at: None,
        completed_at: None,
        metrics: serde_json::json!({
            "sent": 0,
            "delivered": 0,
            "failed": 0,
            "opened": 0,
            "clicked": 0,
            "replied": 0
        }),
        budget: req.budget,
        created_at: now,
        updated_at: Some(now),
    };

    diesel::insert_into(marketing_campaigns::table)
        .values(&campaign)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(campaign))
}

pub async fn update_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    if let Some(name) = req.name {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(status) = req.status {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::status.eq(status))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(channel) = req.channel {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::channel.eq(channel))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(ct) = req.content_template {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::content_template.eq(ct))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(scheduled) = req.scheduled_at {
        let dt = DateTime::parse_from_rfc3339(&scheduled)
            .ok()
            .map(|d| d.with_timezone(&Utc));
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::scheduled_at.eq(dt))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(budget) = req.budget {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::budget.eq(budget))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .set(marketing_campaigns::updated_at.eq(Some(now)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_campaign(State(state), Path(id)).await
}

pub async fn delete_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

#[derive(Debug, Deserialize)]
pub struct SendCampaignRequest {
    pub list_id: Option<Uuid>,
    pub contact_ids: Option<Vec<Uuid>>,
    pub template_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignSendResult {
    pub campaign_id: Uuid,
    pub total_recipients: i32,
    pub sent: i32,
    pub failed: i32,
    pub pending: i32,
}

fn render_template(template: &str, variables: &serde_json::Value) -> String {
    let mut result = template.to_string();
    
    if let Some(obj) = variables.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{}}}", key);
            let replacement = value.as_str().unwrap_or("");
            result = result.replace(&placeholder, replacement);
        }
    }
    
    result
}

async fn generate_ai_content(
    prompt: &str,
    contact_name: &str,
    template_body: &str,
) -> Result<String, String> {
    let full_prompt = format!(
        "You are a marketing assistant. Write a personalized message for {}.\n\nTemplate:\n{}\n\nInstructions: {}",
        contact_name, template_body, prompt
    );

    log::info!("Generating AI content with prompt: {}", full_prompt);

    Ok(format!("[AI Generated for {}]: {}", contact_name, template_body))
}

async fn send_via_email(
    to_email: &str,
    _subject: &str,
    _body: &str,
    bot_id: Uuid,
) -> Result<(), String> {
    log::info!("Sending email to {} via bot {}", to_email, bot_id);
    Ok(())
}

async fn send_via_whatsapp(
    to_phone: &str,
    _body: &str,
    bot_id: Uuid,
) -> Result<(), String> {
    log::info!("Sending WhatsApp to {} via bot {}", to_phone, bot_id);
    Ok(())
}

async fn send_via_telegram(
    to_chat_id: &str,
    _body: &str,
    bot_id: Uuid,
) -> Result<(), String> {
    log::info!("Sending Telegram to {} via bot {}", to_chat_id, bot_id);
    Ok(())
}

async fn send_via_sms(
    to_phone: &str,
    _body: &str,
    bot_id: Uuid,
) -> Result<(), String> {
    log::info!("Sending SMS to {} via bot {}", to_phone, bot_id);
    Ok(())
}

pub async fn send_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SendCampaignRequest>,
) -> Result<Json<CampaignSendResult>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let channel = campaign.channel.clone();
    let bot_id = campaign.bot_id;
    
    let mut recipient_ids: Vec<Uuid> = Vec::new();

    if let Some(_list_id) = req.list_id {
        use crate::core::shared::schema::crm_contacts;
        
        let contacts: Vec<Uuid> = crm_contacts::table
            .filter(crm_contacts::bot_id.eq(bot_id))
            .select(crm_contacts::id)
            .limit(1000)
            .load(&mut conn)
            .unwrap_or_default();
        
        recipient_ids.extend(contacts);
    }

    if let Some(contact_ids) = req.contact_ids {
        recipient_ids.extend(contact_ids);
    }

    let total = recipient_ids.len() as i32;
    let mut sent = 0;
    let mut failed = 0;

    use crate::core::shared::schema::crm_contacts;
    use crate::core::shared::schema::marketing_templates;

    #[derive(Debug, Clone)]
    struct TemplateData {
        subject: String,
        body: String,
        ai_prompt: Option<String>,
    }

    let template_id = req.template_id.unwrap_or(Uuid::nil());
    let template: Option<TemplateData> = if template_id != Uuid::nil() {
        let result: Result<(Option<String>, Option<String>, Option<String>), _> = 
            marketing_templates::table
                .filter(marketing_templates::id.eq(template_id))
                .select((
                    marketing_templates::subject,
                    marketing_templates::body,
                    marketing_templates::ai_prompt,
                ))
                .first(&mut conn);
        
        result.ok().map(|(subject, body, ai_prompt)| TemplateData {
            subject: subject.unwrap_or_default(),
            body: body.unwrap_or_default(),
            ai_prompt,
        })
    } else {
        None
    };

    for contact_id in recipient_ids {
        let contact = crm_contacts::table
            .filter(crm_contacts::id.eq(contact_id))
            .select((crm_contacts::email, crm_contacts::phone, crm_contacts::first_name))
            .first::<(Option<String>, Option<String>, Option<String>)>(&mut conn)
            .ok();

        if let Some((email, phone, first_name)) = contact {
            let contact_name = first_name.unwrap_or("Customer".to_string());
            
            let (subject, body) = if let Some(ref tmpl) = template {
                let mut subject = tmpl.subject.clone();
                let mut body = tmpl.body.clone();

                let variables = serde_json::json!({
                    "name": contact_name,
                    "email": email.clone(),
                    "phone": phone.clone()
                });

                subject = render_template(&subject, &variables);
                body = render_template(&body, &variables);

                if let Some(ref ai_prompt) = tmpl.ai_prompt {
                    if !ai_prompt.is_empty() {
                        match generate_ai_content(ai_prompt, &contact_name, &body).await {
                            Ok(ai_body) => body = ai_body,
                            Err(e) => log::error!("AI generation failed: {}", e),
                        }
                    }
                }

                (subject, body)
            } else {
                let variables = serde_json::json!({
                    "name": contact_name,
                    "email": email.clone(),
                    "phone": phone.clone()
                });
                let content = campaign.content_template.clone();
                let subject = content.get("subject").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let body = content.get("body").and_then(|s| s.as_str()).unwrap_or("").to_string();
                (render_template(&subject, &variables), render_template(&body, &variables))
            };

            let send_result = match channel.as_str() {
                "email" => {
                    if let Some(ref email_addr) = email {
                        send_via_email(email_addr, &subject, &body, bot_id).await
                    } else {
                        Err("No email address".to_string())
                    }
                }
                "whatsapp" => {
                    if let Some(ref phone_num) = phone {
                        send_via_whatsapp(phone_num, &body, bot_id).await
                    } else {
                        Err("No phone number".to_string())
                    }
                }
                "telegram" => {
                    send_via_telegram(&contact_id.to_string(), &body, bot_id).await
                }
                "sms" => {
                    if let Some(ref phone_num) = phone {
                        send_via_sms(phone_num, &body, bot_id).await
                    } else {
                        Err("No phone number".to_string())
                    }
                }
                _ => Err("Unknown channel".to_string()),
            };

            match send_result {
                Ok(()) => sent += 1,
                Err(e) => {
                    log::error!("Failed to send to contact {}: {}", contact_id, e);
                    failed += 1;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        } else {
            failed += 1;
        }
    }

    let now = Utc::now();
    diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .set((
            marketing_campaigns::status.eq(if failed == 0 { "completed" } else { "completed_with_errors" }),
            marketing_campaigns::sent_at.eq(Some(now)),
            marketing_campaigns::completed_at.eq(Some(now)),
            marketing_campaigns::metrics.eq(serde_json::json!({
                "total": total,
                "sent": sent,
                "failed": failed
            })),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    Ok(Json(CampaignSendResult {
        campaign_id: id,
        total_recipients: total,
        sent,
        failed,
        pending: 0,
    }))
}
