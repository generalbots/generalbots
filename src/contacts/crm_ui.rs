use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::contacts::crm::{CrmAccount, CrmContact, CrmLead, CrmOpportunity};
use crate::core::shared::schema::{crm_accounts, crm_contacts, crm_leads, crm_opportunities};
use crate::core::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StageQuery {
    pub stage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

pub fn configure_crm_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/crm/count", get(handle_crm_count))
        .route("/api/ui/crm/pipeline", get(handle_crm_pipeline))
        .route("/api/ui/crm/leads", get(handle_crm_leads))
        .route("/api/ui/crm/opportunities", get(handle_crm_opportunities))
        .route("/api/ui/crm/contacts", get(handle_crm_contacts))
        .route("/api/ui/crm/accounts", get(handle_crm_accounts))
        .route("/api/ui/crm/search", get(handle_crm_search))
        .route("/api/ui/crm/stats/conversion-rate", get(handle_conversion_rate))
        .route("/api/ui/crm/stats/pipeline-value", get(handle_pipeline_value))
        .route("/api/ui/crm/stats/avg-deal", get(handle_avg_deal))
        .route("/api/ui/crm/stats/won-month", get(handle_won_month))
}

async fn handle_crm_count(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "all".to_string());

    let count: i64 = if stage == "all" {
        crm_leads::table
            .filter(crm_leads::org_id.eq(org_id))
            .filter(crm_leads::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    } else {
        crm_leads::table
            .filter(crm_leads::org_id.eq(org_id))
            .filter(crm_leads::bot_id.eq(bot_id))
            .filter(crm_leads::stage.eq(&stage))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    };

    Html(count.to_string())
}

async fn handle_crm_pipeline(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_pipeline("lead"));
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let stage = query.stage.unwrap_or_else(|| "new".to_string());

    let leads: Vec<CrmLead> = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .filter(crm_leads::stage.eq(&stage))
        .order(crm_leads::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if leads.is_empty() {
        return Html(render_empty_pipeline(&stage));
    }

    let mut html = String::new();
    for lead in leads {
        let value_str = lead
            .value
            .map(|v| format!("${}", v))
            .unwrap_or_else(|| "-".to_string());
        let contact_name = lead.contact_id.map(|_| "Contact").unwrap_or("-");

        html.push_str(&format!(
            "<div class=\"pipeline-card\" data-id=\"{}\">
                <div class=\"pipeline-card-header\">
                    <span class=\"lead-title\">{}</span>
                    <span class=\"lead-value\">{}</span>
                </div>
                <div class=\"pipeline-card-body\">
                    <span class=\"lead-contact\">{}</span>
                    <span class=\"lead-probability\">{}%</span>
                </div>
                <div class=\"pipeline-card-actions\">
                    <button class=\"btn-sm\" hx-put=\"/api/crm/leads/{}/stage?stage=qualified\" hx-swap=\"none\">Qualify</button>
                    <button class=\"btn-sm btn-secondary\" hx-get=\"/api/crm/leads/{}\" hx-target=\"#detail-panel\">View</button>
                </div>
            </div>",
            lead.id,
            html_escape(&lead.title),
            value_str,
            contact_name,
            lead.probability,
            lead.id,
            lead.id
        ));
    }

    Html(html)
}

async fn handle_crm_leads(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_table("leads", "📋", "No leads yet", "Create your first lead to get started"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let leads: Vec<CrmLead> = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .filter(crm_leads::closed_at.is_null())
        .order(crm_leads::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    if leads.is_empty() {
        return Html(render_empty_table("leads", "📋", "No leads yet", "Create your first lead to get started"));
    }

    let mut html = String::new();
    for lead in leads {
        let value_str = lead
            .value
            .map(|v| format!("${}", v))
            .unwrap_or_else(|| "-".to_string());
        let expected_close = lead
            .expected_close_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let source = lead.source.as_deref().unwrap_or("-");

        html.push_str(&format!(
            "<tr class=\"lead-row\" data-id=\"{}\">
                <td><input type=\"checkbox\" class=\"row-select\" value=\"{}\"></td>
                <td class=\"lead-title\">{}</td>
                <td>{}</td>
                <td><span class=\"stage-badge stage-{}\">{}</span></td>
                <td>{}%</td>
                <td>{}</td>
                <td>{}</td>
                <td class=\"actions\">
                    <button class=\"btn-icon\" hx-get=\"/api/crm/leads/{}\" hx-target=\"#detail-panel\" title=\"View\">👁</button>
                    <button class=\"btn-icon\" hx-delete=\"/api/crm/leads/{}\" hx-confirm=\"Delete this lead?\" hx-swap=\"none\" title=\"Delete\">🗑</button>
                </td>
            </tr>",
            lead.id,
            lead.id,
            html_escape(&lead.title),
            value_str,
            lead.stage,
            lead.stage,
            lead.probability,
            expected_close,
            source,
            lead.id,
            lead.id
        ));
    }

    Html(html)
}

async fn handle_crm_opportunities(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_table("opportunities", "💼", "No opportunities yet", "Qualify leads to create opportunities"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let opportunities: Vec<CrmOpportunity> = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.is_null())
        .order(crm_opportunities::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    if opportunities.is_empty() {
        return Html(render_empty_table("opportunities", "💼", "No opportunities yet", "Qualify leads to create opportunities"));
    }

    let mut html = String::new();
    for opp in opportunities {
        let value_str = opp
            .value
            .map(|v| format!("${}", v))
            .unwrap_or_else(|| "-".to_string());
        let expected_close = opp
            .expected_close_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());

        html.push_str(&format!(
            "<tr class=\"opportunity-row\" data-id=\"{}\">
                <td><input type=\"checkbox\" class=\"row-select\" value=\"{}\"></td>
                <td class=\"opp-name\">{}</td>
                <td>{}</td>
                <td><span class=\"stage-badge stage-{}\">{}</span></td>
                <td>{}%</td>
                <td>{}</td>
                <td class=\"actions\">
                    <button class=\"btn-icon btn-success\" hx-post=\"/api/crm/opportunities/{}/close\" hx-vals='{{\"won\":true}}' hx-swap=\"none\" title=\"Won\">✓</button>
                    <button class=\"btn-icon btn-danger\" hx-post=\"/api/crm/opportunities/{}/close\" hx-vals='{{\"won\":false}}' hx-swap=\"none\" title=\"Lost\">✗</button>
                    <button class=\"btn-icon\" hx-get=\"/api/crm/opportunities/{}\" hx-target=\"#detail-panel\" title=\"View\">👁</button>
                </td>
            </tr>",
            opp.id,
            opp.id,
            html_escape(&opp.name),
            value_str,
            opp.stage,
            opp.stage,
            opp.probability,
            expected_close,
            opp.id,
            opp.id,
            opp.id
        ));
    }

    Html(html)
}

async fn handle_crm_contacts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_table("contacts", "👥", "No contacts yet", "Add contacts to your CRM"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let contacts: Vec<CrmContact> = crm_contacts::table
        .filter(crm_contacts::org_id.eq(org_id))
        .filter(crm_contacts::bot_id.eq(bot_id))
        .order(crm_contacts::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    if contacts.is_empty() {
        return Html(render_empty_table("contacts", "👥", "No contacts yet", "Add contacts to your CRM"));
    }

    let mut html = String::new();
    for contact in contacts {
        let name = format!(
            "{} {}",
            contact.first_name.as_deref().unwrap_or(""),
            contact.last_name.as_deref().unwrap_or("")
        ).trim().to_string();
        let name = if name.is_empty() { "-".to_string() } else { name };
        let email = contact.email.as_deref().unwrap_or("-");
        let phone = contact.phone.as_deref().unwrap_or("-");
        let company = contact.company.as_deref().unwrap_or("-");

        html.push_str(&format!(
            "<tr class=\"contact-row\" data-id=\"{}\">
                <td><input type=\"checkbox\" class=\"row-select\" value=\"{}\"></td>
                <td class=\"contact-name\">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td><span class=\"status-badge status-{}\">{}</span></td>
                <td class=\"actions\">
                    <button class=\"btn-icon\" hx-get=\"/api/crm/contacts/{}\" hx-target=\"#detail-panel\" title=\"View\">👁</button>
                    <button class=\"btn-icon\" hx-delete=\"/api/crm/contacts/{}\" hx-confirm=\"Delete this contact?\" hx-swap=\"none\" title=\"Delete\">🗑</button>
                </td>
            </tr>",
            contact.id,
            contact.id,
            html_escape(&name),
            html_escape(email),
            html_escape(phone),
            html_escape(company),
            contact.status,
            contact.status,
            contact.id,
            contact.id
        ));
    }

    Html(html)
}

async fn handle_crm_accounts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_table("accounts", "🏢", "No accounts yet", "Add company accounts to your CRM"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let accounts: Vec<CrmAccount> = crm_accounts::table
        .filter(crm_accounts::org_id.eq(org_id))
        .filter(crm_accounts::bot_id.eq(bot_id))
        .order(crm_accounts::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    if accounts.is_empty() {
        return Html(render_empty_table("accounts", "🏢", "No accounts yet", "Add company accounts to your CRM"));
    }

    let mut html = String::new();
    for account in accounts {
        let website = account.website.as_deref().unwrap_or("-");
        let industry = account.industry.as_deref().unwrap_or("-");
        let employees = account
            .employees_count
            .map(|e| e.to_string())
            .unwrap_or_else(|| "-".to_string());
        let phone = account.phone.as_deref().unwrap_or("-");

        html.push_str(&format!(
            "<tr class=\"account-row\" data-id=\"{}\">
                <td><input type=\"checkbox\" class=\"row-select\" value=\"{}\"></td>
                <td class=\"account-name\">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class=\"actions\">
                    <button class=\"btn-icon\" hx-get=\"/api/crm/accounts/{}\" hx-target=\"#detail-panel\" title=\"View\">👁</button>
                    <button class=\"btn-icon\" hx-delete=\"/api/crm/accounts/{}\" hx-confirm=\"Delete this account?\" hx-swap=\"none\" title=\"Delete\">🗑</button>
                </td>
            </tr>",
            account.id,
            account.id,
            html_escape(&account.name),
            html_escape(website),
            html_escape(industry),
            employees,
            html_escape(phone),
            account.id,
            account.id
        ));
    }

    Html(html)
}

async fn handle_crm_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let Ok(mut conn) = state.conn.get() else {
        return Html("<div class=\"search-results-empty\"><p>Search unavailable</p></div>".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let pattern = format!("%{q}%");

    let contacts: Vec<CrmContact> = crm_contacts::table
        .filter(crm_contacts::org_id.eq(org_id))
        .filter(crm_contacts::bot_id.eq(bot_id))
        .filter(
            crm_contacts::first_name.ilike(&pattern)
                .or(crm_contacts::last_name.ilike(&pattern))
                .or(crm_contacts::email.ilike(&pattern))
                .or(crm_contacts::company.ilike(&pattern))
        )
        .limit(10)
        .load(&mut conn)
        .unwrap_or_default();

    let leads: Vec<CrmLead> = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .filter(crm_leads::title.ilike(&pattern))
        .limit(10)
        .load(&mut conn)
        .unwrap_or_default();

    if contacts.is_empty() && leads.is_empty() {
        return Html(format!(
            "<div class=\"search-results-empty\"><p>No results for \"{}\"</p></div>",
            html_escape(&q)
        ));
    }

    let mut html = String::from("<div class=\"search-results\">");

    if !contacts.is_empty() {
        html.push_str("<div class=\"search-section\"><h4>Contacts</h4>");
        for contact in contacts {
            let name = format!(
                "{} {}",
                contact.first_name.as_deref().unwrap_or(""),
                contact.last_name.as_deref().unwrap_or("")
            ).trim().to_string();
            let email = contact.email.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<div class=\"search-result-item\" hx-get=\"/api/crm/contacts/{}\" hx-target=\"#detail-panel\">
                    <span class=\"result-name\">{}</span>
                    <span class=\"result-detail\">{}</span>
                </div>",
                contact.id,
                html_escape(&name),
                html_escape(email)
            ));
        }
        html.push_str("</div>");
    }

    if !leads.is_empty() {
        html.push_str("<div class=\"search-section\"><h4>Leads</h4>");
        for lead in leads {
            let value = lead.value.map(|v| format!("${}", v)).unwrap_or_default();
            html.push_str(&format!(
                "<div class=\"search-result-item\" hx-get=\"/api/crm/leads/{}\" hx-target=\"#detail-panel\">
                    <span class=\"result-name\">{}</span>
                    <span class=\"result-detail\">{}</span>
                </div>",
                lead.id,
                html_escape(&lead.title),
                value
            ));
        }
        html.push_str("</div>");
    }

    html.push_str("</div>");
    Html(html)
}

async fn handle_conversion_rate(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0%".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let total_leads: i64 = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let won_opportunities: i64 = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.eq(Some(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let rate = if total_leads > 0 {
        (won_opportunities as f64 / total_leads as f64) * 100.0
    } else {
        0.0
    };

    Html(format!("{:.1}%", rate))
}

async fn handle_pipeline_value(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("$0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let opportunities: Vec<CrmOpportunity> = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.is_null())
        .load(&mut conn)
        .unwrap_or_default();

    let total: f64 = opportunities
        .iter()
        .filter_map(|o| o.value.as_ref())
        .filter_map(|v| v.to_string().parse::<f64>().ok())
        .sum();

    Html(format_currency(total))
}

async fn handle_avg_deal(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("$0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let won_opportunities: Vec<CrmOpportunity> = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.eq(Some(true)))
        .load(&mut conn)
        .unwrap_or_default();

    if won_opportunities.is_empty() {
        return Html("$0".to_string());
    }

    let total: f64 = won_opportunities
        .iter()
        .filter_map(|o| o.value.as_ref())
        .filter_map(|v| v.to_string().parse::<f64>().ok())
        .sum();

    let avg = total / won_opportunities.len() as f64;
    Html(format_currency(avg))
}

async fn handle_won_month(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.eq(Some(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

fn render_empty_pipeline(stage: &str) -> String {
    format!(
        "<div class=\"pipeline-empty\"><p>No {} items yet</p></div>",
        stage
    )
}

fn render_empty_table(_entity: &str, icon: &str, title: &str, hint: &str) -> String {
    format!(
        "<tr class=\"empty-row\">
        <td colspan=\"7\" class=\"empty-state\">
            <div class=\"empty-icon\">{}</div>
            <p>{}</p>
            <p class=\"empty-hint\">{}</p>
        </td>
    </tr>",
        icon, title, hint
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn format_currency(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.1}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}
