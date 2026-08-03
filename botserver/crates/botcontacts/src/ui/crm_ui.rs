use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use diesel::dsl::sum;
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

fn get_bot_context(state: &CrateState) -> Uuid {
    state.get_bot_context()
}

pub async fn handle_crm_count(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("0".to_string());
    };

    let branch_id = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "all".to_string());

    let count: i64 = if stage == "all" || stage.is_empty() {
        crm_deals::table
            .filter(crm_deals::branch_id.eq(branch_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    } else {
        crm_deals::table
            .filter(crm_deals::branch_id.eq(branch_id))
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

    let branch_id = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "new".to_string());

    let leads: Vec<CrmDeal> = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
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
            lead.probability.unwrap_or(0),
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

    let branch_id = get_bot_context(&state);

    let contacts: Vec<crate::models::CrmContact> = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
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
            contact.first_name.as_str(),
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

    let branch_id = get_bot_context(&state);

    let accounts: Vec<crate::models::CrmAccount> = crm_accounts::table
        .filter(crm_accounts::branch_id.eq(branch_id))
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

    let branch_id = get_bot_context(&state);

    let deals: Vec<CrmDeal> = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .order(crm_deals::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if deals.is_empty() {
        return Html(r#"<div class="deals-empty"><p>No deals yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for deal in deals {
        let title = deal.title.as_deref().or(Some(deal.name.as_str())).unwrap_or("Untitled");
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

/// `/api/crm/search?q=` — global CRM search across deals, contacts and accounts.
pub async fn handle_crm_search(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="search-empty"><p>No results</p></div>"#.to_string());
    };

    let branch_id = get_bot_context(&state);
    let q = query.q.unwrap_or_default().trim().to_lowercase();
    if q.is_empty() {
        return Html(r#"<div class="search-empty"><p>Type to search deals, contacts and accounts</p></div>"#.to_string());
    }

    let pattern = format!("%{q}%");

    let deals: Vec<CrmDeal> = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .filter(
            crm_deals::title
                .like(&pattern)
                .or(crm_deals::name.like(&pattern)),
        )
        .order(crm_deals::updated_at.desc())
        .limit(8)
        .load(&mut conn)
        .unwrap_or_default();

    let contacts: Vec<crate::models::CrmContact> = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .filter(
            crm_contacts::first_name
                .like(&pattern)
                .or(crm_contacts::last_name.like(&pattern))
                .or(crm_contacts::email.like(&pattern)),
        )
        .order(crm_contacts::updated_at.desc())
        .limit(8)
        .load(&mut conn)
        .unwrap_or_default();

    let accounts: Vec<crate::models::CrmAccount> = crm_accounts::table
        .filter(crm_accounts::branch_id.eq(branch_id))
        .filter(crm_accounts::name.like(&pattern))
        .order(crm_accounts::updated_at.desc())
        .limit(8)
        .load(&mut conn)
        .unwrap_or_default();

    if deals.is_empty() && contacts.is_empty() && accounts.is_empty() {
        return Html(format!(
            r#"<div class="search-empty"><p>No results for "{q}"</p></div>"#
        ));
    }

    let mut html = String::new();
    if !deals.is_empty() {
        html.push_str(r#"<div class="search-group"><div class="search-group-title">Deals</div>"#);
        for deal in deals {
            let title = deal.title.as_deref().unwrap_or(&deal.name);
            html.push_str(&format!(
                r#"<a class="search-result" href="/crm#/deals/{}"><span class="search-result-name">{}</span><span class="search-result-meta">{}</span></a>"#,
                deal.id,
                html_escape(title),
                deal.value.map(|v| format!("${v}")).unwrap_or_default()
            ));
        }
        html.push_str("</div>");
    }
    if !contacts.is_empty() {
        html.push_str(r#"<div class="search-group"><div class="search-group-title">Contacts</div>"#);
        for contact in contacts {
            let name = format!(
                "{} {}",
                contact.first_name.as_str(),
                contact.last_name.as_deref().unwrap_or("")
            )
            .trim()
            .to_string();
            html.push_str(&format!(
                r#"<a class="search-result" href="/crm#/contacts/{}"><span class="search-result-name">{}</span><span class="search-result-meta">{}</span></a>"#,
                contact.id,
                html_escape(&name),
                html_escape(contact.email.as_deref().unwrap_or(""))
            ));
        }
        html.push_str("</div>");
    }
    if !accounts.is_empty() {
        html.push_str(r#"<div class="search-group"><div class="search-group-title">Accounts</div>"#);
        for account in accounts {
            html.push_str(&format!(
                r#"<a class="search-result" href="/crm#/accounts/{}"><span class="search-result-name">{}</span><span class="search-result-meta">{}</span></a>"#,
                account.id,
                html_escape(&account.name),
                html_escape(account.industry.as_deref().unwrap_or(""))
            ));
        }
        html.push_str("</div>");
    }

    Html(html)
}

/// `/api/crm/stats/pipeline-value` — sum of open deal values.
pub async fn handle_crm_stats_pipeline_value(State(state): State<Arc<CrateState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("$0".to_string());
    };
    let branch_id = get_bot_context(&state);

    let total: Option<f64> = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .filter(crm_deals::won.ne(true))
        .select(sum(crm_deals::value))
        .get_result(&mut conn)
        .unwrap_or(None);

    Html(match total {
        Some(v) => format!("${v:.2}"),
        None => "$0".to_string(),
    })
}

/// `/api/crm/stats/conversion-rate` — percentage of won deals.
pub async fn handle_crm_stats_conversion_rate(State(state): State<Arc<CrateState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("0%".to_string());
    };
    let branch_id = get_bot_context(&state);

    let total: i64 = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let won: i64 = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .filter(crm_deals::won.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let rate = if total > 0 { (won as f64 / total as f64) * 100.0 } else { 0.0 };
    Html(format!("{rate:.0}%"))
}

/// `/api/crm/stats/avg-deal` — average deal value.
pub async fn handle_crm_stats_avg_deal(State(state): State<Arc<CrateState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("$0".to_string());
    };
    let branch_id = get_bot_context(&state);

    let avg_value = {
        let total: Option<f64> = crm_deals::table
            .filter(crm_deals::branch_id.eq(branch_id))
            .select(sum(crm_deals::value))
            .get_result(&mut conn)
            .unwrap_or(None);
        let count: i64 = crm_deals::table
            .filter(crm_deals::branch_id.eq(branch_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        match total {
            Some(t) if count > 0 => t / count as f64,
            _ => 0.0,
        }
    };

    Html(format!("${avg_value:.2}"))
}

/// `/api/crm/stats/won-month` — value of deals won this month.
pub async fn handle_crm_stats_won_month(State(state): State<Arc<CrateState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html("$0".to_string());
    };
    let branch_id = get_bot_context(&state);

    let total: Option<f64> = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .filter(crm_deals::won.eq(true))
        .filter(crm_deals::closed_at.ge(chrono::Utc::now() - chrono::Duration::days(31)))
        .select(sum(crm_deals::value))
        .get_result(&mut conn)
        .unwrap_or(None);

    Html(match total {
        Some(v) => format!("${v:.2}"),
        None => "$0".to_string(),
    })
}

/// `/api/crm/accounts/search?q=` — account `<option>` list for form selects.
pub async fn handle_crm_accounts_search(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<option value="">No accounts</option>"#.to_string());
    };
    let branch_id = get_bot_context(&state);

    let accounts: Vec<crate::models::CrmAccount> = match query.q.as_deref() {
        Some(q) if !q.trim().is_empty() => {
            let pattern = format!("%{}%", q.trim());
            crm_accounts::table
                .filter(crm_accounts::branch_id.eq(branch_id))
                .filter(crm_accounts::name.like(&pattern))
                .order(crm_accounts::name)
                .limit(50)
                .load(&mut conn)
                .unwrap_or_default()
        }
        _ => crm_accounts::table
            .filter(crm_accounts::branch_id.eq(branch_id))
            .order(crm_accounts::name)
            .limit(50)
            .load(&mut conn)
            .unwrap_or_default(),
    };

    if accounts.is_empty() {
        return Html(r#"<option value="">No accounts available</option>"#.to_string());
    }

    let mut html = String::new();
    for account in accounts {
        html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            account.id,
            html_escape(&account.name)
        ));
    }
    Html(html)
}

/// `/api/crm/opportunities/search?q=` — opportunity `<option>` list for form selects.
pub async fn handle_crm_opportunities_search(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<option value="">No opportunities</option>"#.to_string());
    };
    let branch_id = get_bot_context(&state);

    let opportunities: Vec<CrmDeal> = match query.q.as_deref() {
        Some(q) if !q.trim().is_empty() => {
            let pattern = format!("%{}%", q.trim());
            crm_deals::table
                .filter(crm_deals::branch_id.eq(branch_id))
                .filter(crm_deals::won.ne(true))
                .filter(
                    crm_deals::title
                        .like(&pattern)
                        .or(crm_deals::name.like(&pattern)),
                )
                .order(crm_deals::name)
                .limit(50)
                .load(&mut conn)
                .unwrap_or_default()
        }
        _ => crm_deals::table
            .filter(crm_deals::branch_id.eq(branch_id))
            .filter(crm_deals::won.ne(true))
            .order(crm_deals::name)
            .limit(50)
            .load(&mut conn)
            .unwrap_or_default(),
    };

    if opportunities.is_empty() {
        return Html(r#"<option value="">No open opportunities</option>"#.to_string());
    }

    let mut html = String::new();
    for opp in opportunities {
        let title = opp.title.as_deref().unwrap_or(&opp.name);
        html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            opp.id,
            html_escape(title)
        ));
    }
    Html(html)
}
