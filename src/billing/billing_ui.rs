use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{billing_invoices, billing_payments, billing_quotes};
use crate::shared::state::AppState;

fn bd_to_f64(bd: &BigDecimal) -> f64 {
    bd.to_f64().unwrap_or(0.0)
}

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_currency(amount: f64, currency: &str) -> String {
    match currency.to_uppercase().as_str() {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("€{:.2}", amount),
        "GBP" => format!("£{:.2}", amount),
        "BRL" => format!("R${:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = billing_invoices::table
            .filter(billing_invoices::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref status) = query.status {
            db_query = db_query.filter(billing_invoices::status.eq(status));
        }

        db_query = db_query.order(billing_invoices::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                billing_invoices::id,
                billing_invoices::invoice_number,
                billing_invoices::customer_name,
                billing_invoices::customer_email,
                billing_invoices::status,
                billing_invoices::issue_date,
                billing_invoices::due_date,
                billing_invoices::total,
                billing_invoices::amount_due,
                billing_invoices::currency,
            ))
            .load::<(Uuid, String, String, Option<String>, String, NaiveDate, NaiveDate, BigDecimal, BigDecimal, String)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(invoices) if !invoices.is_empty() => {
            let mut html = String::new();
            for (id, number, customer_name, customer_email, status, issue_date, due_date, total, amount_due, currency) in invoices {
                let name = customer_email.unwrap_or_else(|| customer_name.clone());
                let total_str = format_currency(bd_to_f64(&total), &currency);
                let due_str = format_currency(bd_to_f64(&amount_due), &currency);
                let issue_str = issue_date.format("%Y-%m-%d").to_string();
                let due_date_str = due_date.format("%Y-%m-%d").to_string();

                let status_class = match status.as_str() {
                    "paid" => "status-paid",
                    "sent" => "status-sent",
                    "overdue" => "status-overdue",
                    "void" => "status-void",
                    _ => "status-draft",
                };

                html.push_str(&format!(
                    r##"<tr class="invoice-row" data-id="{id}">
                        <td class="invoice-number">{}</td>
                        <td class="invoice-customer">{}</td>
                        <td class="invoice-date">{}</td>
                        <td class="invoice-due">{}</td>
                        <td class="invoice-total">{}</td>
                        <td class="invoice-balance">{}</td>
                        <td class="invoice-status"><span class="{}">{}</span></td>
                        <td class="invoice-actions">
                            <button class="btn-sm" hx-get="/api/billing/invoices/{id}" hx-target="#invoice-detail">View</button>
                        </td>
                    </tr>"##,
                    html_escape(&number),
                    html_escape(&name),
                    issue_str,
                    due_date_str,
                    total_str,
                    due_str,
                    status_class,
                    html_escape(&status)
                ));
            }
            Html(html)
        }
        _ => Html(
            r##"<tr class="empty-row">
                <td colspan="8" class="empty-state">
                    <div class="empty-icon">📄</div>
                    <p>No invoices yet</p>
                    <p class="empty-hint">Create your first invoice to get started</p>
                </td>
            </tr>"##.to_string(),
        ),
    }
}

async fn handle_payments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = billing_payments::table
            .filter(billing_payments::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref status) = query.status {
            db_query = db_query.filter(billing_payments::status.eq(status));
        }

        db_query = db_query.order(billing_payments::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                billing_payments::id,
                billing_payments::payment_number,
                billing_payments::amount,
                billing_payments::currency,
                billing_payments::payment_method,
                billing_payments::payer_name,
                billing_payments::payer_email,
                billing_payments::status,
                billing_payments::paid_at,
            ))
            .load::<(Uuid, String, BigDecimal, String, String, Option<String>, Option<String>, String, DateTime<Utc>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(payments) if !payments.is_empty() => {
            let mut html = String::new();
            for (id, number, amount, currency, method, payer_name, payer_email, status, paid_at) in payments {
                let amount_str = format_currency(bd_to_f64(&amount), &currency);
                let payer = payer_name.unwrap_or_else(|| payer_email.unwrap_or_else(|| "Unknown".to_string()));
                let date_str = paid_at.format("%Y-%m-%d %H:%M").to_string();

                let status_class = match status.as_str() {
                    "completed" => "status-completed",
                    "pending" => "status-pending",
                    "refunded" => "status-refunded",
                    "failed" => "status-failed",
                    _ => "status-default",
                };

                html.push_str(&format!(
                    r##"<tr class="payment-row" data-id="{id}">
                        <td class="payment-number">{}</td>
                        <td class="payment-payer">{}</td>
                        <td class="payment-amount">{}</td>
                        <td class="payment-method">{}</td>
                        <td class="payment-date">{}</td>
                        <td class="payment-status"><span class="{}">{}</span></td>
                        <td class="payment-actions">
                            <button class="btn-sm" hx-get="/api/billing/payments/{id}" hx-target="#payment-detail">View</button>
                        </td>
                    </tr>"##,
                    html_escape(&number),
                    html_escape(&payer),
                    amount_str,
                    html_escape(&method),
                    date_str,
                    status_class,
                    html_escape(&status)
                ));
            }
            Html(html)
        }
        _ => Html(
            r##"<tr class="empty-row">
                <td colspan="7" class="empty-state">
                    <div class="empty-icon">💳</div>
                    <p>No payments recorded</p>
                    <p class="empty-hint">Payments will appear here when invoices are paid</p>
                </td>
            </tr>"##.to_string(),
        ),
    }
}

async fn handle_quotes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = billing_quotes::table
            .filter(billing_quotes::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref status) = query.status {
            db_query = db_query.filter(billing_quotes::status.eq(status));
        }

        db_query = db_query.order(billing_quotes::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                billing_quotes::id,
                billing_quotes::quote_number,
                billing_quotes::customer_name,
                billing_quotes::customer_email,
                billing_quotes::status,
                billing_quotes::issue_date,
                billing_quotes::valid_until,
                billing_quotes::total,
                billing_quotes::currency,
            ))
            .load::<(Uuid, String, String, Option<String>, String, NaiveDate, NaiveDate, BigDecimal, String)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(quotes) if !quotes.is_empty() => {
            let mut html = String::new();
            for (id, number, customer_name, customer_email, status, issue_date, valid_until, total, currency) in quotes {
                let name = customer_email.unwrap_or_else(|| customer_name.clone());
                let total_str = format_currency(bd_to_f64(&total), &currency);
                let issue_str = issue_date.format("%Y-%m-%d").to_string();
                let valid_str = valid_until.format("%Y-%m-%d").to_string();

                let status_class = match status.as_str() {
                    "accepted" => "status-accepted",
                    "sent" => "status-sent",
                    "rejected" => "status-rejected",
                    "expired" => "status-expired",
                    "converted" => "status-converted",
                    _ => "status-draft",
                };

                html.push_str(&format!(
                    r##"<tr class="quote-row" data-id="{id}">
                        <td class="quote-number">{}</td>
                        <td class="quote-customer">{}</td>
                        <td class="quote-date">{}</td>
                        <td class="quote-valid">{}</td>
                        <td class="quote-total">{}</td>
                        <td class="quote-status"><span class="{}">{}</span></td>
                        <td class="quote-actions">
                            <button class="btn-sm" hx-get="/api/billing/quotes/{id}" hx-target="#quote-detail">View</button>
                        </td>
                    </tr>"##,
                    html_escape(&number),
                    html_escape(&name),
                    issue_str,
                    valid_str,
                    total_str,
                    status_class,
                    html_escape(&status)
                ));
            }
            Html(html)
        }
        _ => Html(
            r##"<tr class="empty-row">
                <td colspan="7" class="empty-state">
                    <div class="empty-icon">📝</div>
                    <p>No quotes yet</p>
                    <p class="empty-hint">Create quotes for your prospects</p>
                </td>
            </tr>"##.to_string(),
        ),
    }
}

async fn handle_stats_pending(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let totals: Vec<BigDecimal> = billing_invoices::table
            .filter(billing_invoices::bot_id.eq(bot_id))
            .filter(billing_invoices::status.eq_any(vec!["sent", "draft"]))
            .select(billing_invoices::amount_due)
            .load(&mut conn)
            .ok()?;

        let sum: f64 = totals.iter().map(bd_to_f64).sum();
        Some(sum)
    })
    .await
    .ok()
    .flatten();

    Html(format_currency(result.unwrap_or(0.0), "USD"))
}

async fn handle_revenue_month(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let now = Utc::now();
        let month_start = now.date_naive().with_day(1)?.and_hms_opt(0, 0, 0)?;

        let totals: Vec<BigDecimal> = billing_invoices::table
            .filter(billing_invoices::bot_id.eq(bot_id))
            .filter(billing_invoices::created_at.ge(month_start))
            .select(billing_invoices::total)
            .load(&mut conn)
            .ok()?;

        let sum: f64 = totals.iter().map(bd_to_f64).sum();
        Some(sum)
    })
    .await
    .ok()
    .flatten();

    Html(format_currency(result.unwrap_or(0.0), "USD"))
}

async fn handle_paid_month(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let now = Utc::now();
        let month_start = now.date_naive().with_day(1)?.and_hms_opt(0, 0, 0)?;

        let totals: Vec<BigDecimal> = billing_payments::table
            .filter(billing_payments::bot_id.eq(bot_id))
            .filter(billing_payments::status.eq("completed"))
            .filter(billing_payments::created_at.ge(month_start))
            .select(billing_payments::amount)
            .load(&mut conn)
            .ok()?;

        let sum: f64 = totals.iter().map(bd_to_f64).sum();
        Some(sum)
    })
    .await
    .ok()
    .flatten();

    Html(format_currency(result.unwrap_or(0.0), "USD"))
}

async fn handle_overdue(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let totals: Vec<BigDecimal> = billing_invoices::table
            .filter(billing_invoices::bot_id.eq(bot_id))
            .filter(billing_invoices::status.eq("overdue"))
            .select(billing_invoices::amount_due)
            .load(&mut conn)
            .ok()?;

        let sum: f64 = totals.iter().map(bd_to_f64).sum();
        Some(sum)
    })
    .await
    .ok()
    .flatten();

    Html(format_currency(result.unwrap_or(0.0), "USD"))
}

async fn handle_billing_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.clone().unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let pool = state.conn.clone();
    let search_term = format!("%{}%", q);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        billing_invoices::table
            .filter(billing_invoices::bot_id.eq(bot_id))
            .filter(
                billing_invoices::invoice_number.ilike(&search_term)
                    .or(billing_invoices::customer_name.ilike(&search_term))
                    .or(billing_invoices::customer_email.ilike(&search_term))
            )
            .order(billing_invoices::created_at.desc())
            .limit(20)
            .select((
                billing_invoices::id,
                billing_invoices::invoice_number,
                billing_invoices::customer_name,
                billing_invoices::status,
                billing_invoices::total,
                billing_invoices::currency,
            ))
            .load::<(Uuid, String, String, String, BigDecimal, String)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, number, customer, status, total, currency) in items {
                let total_str = format_currency(bd_to_f64(&total), &currency);

                html.push_str(&format!(
                    r##"<div class="search-result-item" hx-get="/api/billing/invoices/{id}" hx-target="#invoice-detail">
                        <span class="result-number">{}</span>
                        <span class="result-customer">{}</span>
                        <span class="result-status">{}</span>
                        <span class="result-total">{}</span>
                    </div>"##,
                    html_escape(&number),
                    html_escape(&customer),
                    html_escape(&status),
                    total_str
                ));
            }
            Html(format!(r##"<div class="search-results">{html}</div>"##))
        }
        _ => Html(format!(
            r##"<div class="search-results-empty">
                <p>No results for "{}"</p>
            </div>"##,
            html_escape(&q)
        )),
    }
}
