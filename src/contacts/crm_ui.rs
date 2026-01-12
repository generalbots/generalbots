use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StageQuery {
    pub stage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CountResponse {
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub value: String,
}

pub fn configure_crm_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/crm/count", get(handle_crm_count))
        .route("/api/crm/pipeline", get(handle_crm_pipeline))
        .route("/api/crm/leads", get(handle_crm_leads))
        .route("/api/crm/opportunities", get(handle_crm_opportunities))
        .route("/api/crm/contacts", get(handle_crm_contacts))
        .route("/api/crm/accounts", get(handle_crm_accounts))
        .route("/api/crm/search", get(handle_crm_search))
        .route("/api/crm/stats/conversion-rate", get(handle_conversion_rate))
        .route("/api/crm/stats/pipeline-value", get(handle_pipeline_value))
        .route("/api/crm/stats/avg-deal", get(handle_avg_deal))
        .route("/api/crm/stats/won-month", get(handle_won_month))
}

async fn handle_crm_count(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let _stage = query.stage.unwrap_or_else(|| "all".to_string());
    Html("0".to_string())
}

async fn handle_crm_pipeline(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<StageQuery>,
) -> impl IntoResponse {
    let stage = query.stage.unwrap_or_else(|| "lead".to_string());
    Html(format!(
        r#"<div class="pipeline-empty">
            <p>No {} items yet</p>
        </div>"#,
        stage
    ))
}

async fn handle_crm_leads(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html(r#"<tr class="empty-row">
        <td colspan="7" class="empty-state">
            <div class="empty-icon">📋</div>
            <p>No leads yet</p>
            <p class="empty-hint">Create your first lead to get started</p>
        </td>
    </tr>"#.to_string())
}

async fn handle_crm_opportunities(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html(r#"<tr class="empty-row">
        <td colspan="7" class="empty-state">
            <div class="empty-icon">💼</div>
            <p>No opportunities yet</p>
            <p class="empty-hint">Qualify leads to create opportunities</p>
        </td>
    </tr>"#.to_string())
}

async fn handle_crm_contacts(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html(r#"<tr class="empty-row">
        <td colspan="6" class="empty-state">
            <div class="empty-icon">👥</div>
            <p>No contacts yet</p>
            <p class="empty-hint">Add contacts to your CRM</p>
        </td>
    </tr>"#.to_string())
}

async fn handle_crm_accounts(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html(r#"<tr class="empty-row">
        <td colspan="6" class="empty-state">
            <div class="empty-icon">🏢</div>
            <p>No accounts yet</p>
            <p class="empty-hint">Add company accounts to your CRM</p>
        </td>
    </tr>"#.to_string())
}

async fn handle_crm_search(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }
    Html(format!(
        r#"<div class="search-results-empty">
            <p>No results for "{}"</p>
        </div>"#,
        q
    ))
}

async fn handle_conversion_rate(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html("0%".to_string())
}

async fn handle_pipeline_value(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_avg_deal(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_won_month(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Html("0".to_string())
}
