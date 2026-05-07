use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{CrmDeal, html_escape};
use crate::schema::{crm_deals, crm_contacts, crm_accounts};
use crate::CrateState;

#[derive(Debug, Deserialize)]
pub struct StageQuery {
    pub stage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn get_bot_context(state: &CrateState) -> (Uuid, Uuid) {
    state.get_bot_context()
}

pub async fn handle_crm_count(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "all".to_string());

    let count: i64 = if stage == "all" || stage.is_empty() {
        crm_deals::table
            .filter(crm_deals::org_id.eq(org_id))
            .filter(crm_deals::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    } else {
        crm_deals::table
            .filter(crm_deals::org_id.eq(org_id))
            .filter(crm_deals::bot_id.eq(bot_id))
            .filter(crm_deals::stage.eq(&stage))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    };

    Html(count.to_string())
}

pub async fn handle_crm_pipeline(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="pipeline-empty"><p>No items yet</p></div>"#.to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "new".to_string());

    let leads: Vec<CrmDeal> = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .filter(crm_deals::stage.eq(&stage))
        .order(crm_deals::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if leads.is_empty() {
        return Html(format!(r#"<div class="pipeline-empty"><p>No {stage} items yet</p></div>"#));
    }

    let mut html = String::new();
    for lead in leads {
        let value_str = lead
            .value
            .map(|v| format!("${v}"))
            .unwrap_or_else(|| "-".to_string());
        let contact_name = lead.contact_id.map(|_| "Contact").unwrap_or("-");

        let card_html = format!(
            r##"<div class="pipeline-card" data-id="{}">
<div class="pipeline-card-header">
<span class="lead-title">{}</span>
<span class="lead-value">{}</span>
</div>
<div class="pipeline-card-body">
<span class="lead-contact">{}</span>
<span class="lead-probability">{}%</span>
</div>
<div class="pipeline-card-actions">
<button class="btn-sm" hx-put="/api/crm/leads/{}/stage?stage=qualified" hx-swap="none">Qualify</button>
<button class="btn-sm btn-accent" hx-post="/api/crm/leads/{}/convert" hx-swap="none">Convert</button>
<button class="btn-sm btn-secondary" hx-get="/api/ui/crm/leads/{}" hx-target="#detail-panel">View</button>
</div>
</div>"##,
            lead.id,
            html_escape(lead.title.as_deref().unwrap_or("")),
            value_str,
            contact_name,
            lead.probability,
            lead.id,
            lead.id,
            lead.id
        );
        html.push_str(&card_html);
    }

    Html(html)
}

pub async fn handle_crm_contacts(
    State(state): State<Arc<CrateState>>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="contacts-empty"><p>No contacts yet</p></div>"#.to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let contacts: Vec<crate::models::CrmContact> = crm_contacts::table
        .filter(crm_contacts::org_id.eq(org_id))
        .filter(crm_contacts::bot_id.eq(bot_id))
        .order(crm_contacts::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if contacts.is_empty() {
        return Html(r#"<div class="contacts-empty"><p>No contacts yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for contact in contacts {
        let name = format!(
            "{} {}",
            contact.first_name.as_deref().unwrap_or(""),
            contact.last_name.as_deref().unwrap_or("")
        ).trim().to_string();
        let email = contact.email.as_deref().unwrap_or("-");
        html.push_str(&format!(
            r#"<div class="contact-item" data-id="{}"><span class="contact-name">{}</span><span class="contact-email">{}</span></div>"#,
            contact.id,
            html_escape(&name),
            html_escape(email)
        ));
    }

    Html(html)
}

pub async fn handle_crm_accounts(
    State(state): State<Arc<CrateState>>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="accounts-empty"><p>No accounts yet</p></div>"#.to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let accounts: Vec<crate::models::CrmAccount> = crm_accounts::table
        .filter(crm_accounts::org_id.eq(org_id))
        .filter(crm_accounts::bot_id.eq(bot_id))
        .order(crm_accounts::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if accounts.is_empty() {
        return Html(r#"<div class="accounts-empty"><p>No accounts yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for account in accounts {
        html.push_str(&format!(
            r#"<div class="account-item" data-id="{}"><span class="account-name">{}</span><span class="account-industry">{}</span></div>"#,
            account.id,
            html_escape(&account.name),
            html_escape(account.industry.as_deref().unwrap_or("-"))
        ));
    }

    Html(html)
}

pub async fn handle_crm_deals(
    State(state): State<Arc<CrateState>>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="deals-empty"><p>No deals yet</p></div>"#.to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let deals: Vec<CrmDeal> = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .order(crm_deals::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if deals.is_empty() {
        return Html(r#"<div class="deals-empty"><p>No deals yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for deal in deals {
        let title = deal.title.as_deref().or(deal.name.as_deref()).unwrap_or("Untitled");
        let value_str = deal.value.map(|v| format!("${v}")).unwrap_or_else(|| "-".to_string());
        let stage = deal.stage.as_deref().unwrap_or("-");
        html.push_str(&format!(
            r#"<div class="deal-item" data-id="{}"><span class="deal-title">{}</span><span class="deal-value">{}</span><span class="deal-stage">{}</span></div>"#,
            deal.id,
            html_escape(title),
            value_str,
            html_escape(stage)
        ));
    }

    Html(html)
}
