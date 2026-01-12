use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub fn configure_billing_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/billing/invoices", get(handle_invoices))
        .route("/api/billing/payments", get(handle_payments))
        .route("/api/billing/quotes", get(handle_quotes))
        .route("/api/billing/stats/pending", get(handle_stats_pending))
        .route("/api/billing/stats/revenue-month", get(handle_revenue_month))
        .route("/api/billing/stats/paid-month", get(handle_paid_month))
        .route("/api/billing/stats/overdue", get(handle_overdue))
        .route("/api/billing/search", get(handle_billing_search))
}

async fn handle_invoices(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<StatusQuery>,
) -> impl IntoResponse {
    Html(
        r#"<tr class="empty-row">
        <td colspan="7" class="empty-state">
            <div class="empty-icon">📄</div>
            <p>No invoices yet</p>
            <p class="empty-hint">Create your first invoice to get started</p>
        </td>
    </tr>"#
            .to_string(),
    )
}

async fn handle_payments(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<StatusQuery>,
) -> impl IntoResponse {
    Html(
        r#"<tr class="empty-row">
        <td colspan="6" class="empty-state">
            <div class="empty-icon">💳</div>
            <p>No payments recorded</p>
            <p class="empty-hint">Payments will appear here when invoices are paid</p>
        </td>
    </tr>"#
            .to_string(),
    )
}

async fn handle_quotes(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<StatusQuery>,
) -> impl IntoResponse {
    Html(
        r#"<tr class="empty-row">
        <td colspan="6" class="empty-state">
            <div class="empty-icon">📝</div>
            <p>No quotes yet</p>
            <p class="empty-hint">Create quotes for your prospects</p>
        </td>
    </tr>"#
            .to_string(),
    )
}

async fn handle_stats_pending(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_revenue_month(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_paid_month(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_overdue(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("$0".to_string())
}

async fn handle_billing_search(
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
