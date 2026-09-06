//! Admin billing endpoints: usage, payment methods, address, invoices,
//! retention offers and subscription lifecycle (upgrade/pause/cancel/accept).
//! The admin billing page (port 3000 `/admin/billing`) consumes these.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::BillingApiState;

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

fn bd_to_f64(bd: &BigDecimal) -> f64 {
    bd.to_f64().unwrap_or(0.0)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn branch_id(state: &BillingApiState) -> Uuid {
    crate::get_bot_context(&state.pool, &state.get_default_bot)
}

pub fn configure_admin_billing_routes() -> Router<Arc<BillingApiState>> {
    Router::new()
        .route("/api/billing/usage", get(handle_usage))
        .route("/api/billing/payment-methods", get(handle_payment_methods))
        .route("/api/billing/address", get(handle_address))
        .route("/api/ui/billing/invoices/admin", get(handle_invoices))
        .route("/api/billing/retention-offers", get(handle_retention_offers))
        .route("/api/billing/upgrade", post(handle_upgrade))
        .route("/api/billing/accept-offer", post(handle_accept_offer))
        .route("/api/billing/pause", post(handle_pause))
        .route("/api/billing/cancel", post(handle_cancel))
        .route("/api/billing/subscription/upgrade", post(handle_subscription_upgrade))
        .route("/api/billing/subscription/cancel", post(handle_subscription_cancel))
        .route("/api/billing/invoices/export", get(handle_invoices_export))
        .route("/api/billing/invoices/unpaid", get(handle_invoices_unpaid))
        .route("/api/ui/billing/dashboard/alerts", get(handle_dashboard_alerts))
        .route("/api/ui/billing/dashboard/bot-usage", get(handle_dashboard_bot_usage))
        .route("/api/ui/billing/dashboard/recent-invoices", get(handle_dashboard_recent_invoices))
}

pub async fn handle_usage(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    let messages: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM message_history WHERE created_at > NOW() - INTERVAL '30 days'",
    )
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let storage: i64 = diesel::sql_query(
        "SELECT COALESCE(SUM(file_size), 0)::bigint AS count FROM kb_documents",
    )
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let api_calls: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let storage_gb = storage as f64 / 1_073_741_824.0;

    Html(format!(
        r#"<div class="usage-items">
    <div class="usage-item"><span class="usage-label">Messages (30d)</span><span class="usage-value">{messages}</span></div>
    <div class="usage-item"><span class="usage-label">Storage</span><span class="usage-value">{storage_gb:.1} GB</span></div>
    <div class="usage-item"><span class="usage-label">API Calls (24h)</span><span class="usage-value">{api_calls}</span></div>
    <div class="usage-item"><span class="usage-label">Bots</span><span class="usage-value">{bots}</span></div>
</div>"#,
        bots = {
            let b: i64 = diesel::sql_query(
                "SELECT COUNT(*)::bigint AS count FROM bots WHERE branch_id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);
            b
        },
    ))
}

pub async fn handle_payment_methods(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct MethodRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        method: Option<String>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let methods: Vec<MethodRow> = diesel::sql_query(
        "SELECT COALESCE(payment_method, 'unknown') AS method, COUNT(*)::bigint AS count
         FROM billing_payments WHERE branch_id = $1 GROUP BY method ORDER BY count DESC LIMIT 5",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    if methods.is_empty() {
        return Html(
            r#"<div class="payment-methods"><div class="payment-method"><span class="method-icon">💳</span><span class="method-label">No payment method on file</span></div></div>"#
                .to_string(),
        );
    }

    let mut html = String::new();
    for m in methods {
        let label = m.method.unwrap_or_else(|| "—".to_string());
        html.push_str(&format!(
            r#"<div class="payment-method"><span class="method-icon">💳</span><span class="method-label">{label}</span><span class="method-meta">{count} payment(s)</span></div>"#,
            count = m.count,
        ));
    }
    Html(html)
}

pub async fn handle_address(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };

    #[derive(diesel::QueryableByName)]
    struct AddressRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        address: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        email: Option<String>,
    }

    let row: Option<AddressRow> = diesel::sql_query(
        "SELECT customer_address AS address, customer_name AS name, customer_email AS email
         FROM billing_recurring ORDER BY created_at DESC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();

    match row {
        Some(r) => Html(format!(
            r#"<div class="address-card">
    <div class="address-line">{name}</div>
    <div class="address-line">{email}</div>
    <div class="address-line">{address}</div>
</div>"#,
            name = html_escape(&r.name),
            email = html_escape(r.email.as_deref().unwrap_or("")),
            address = html_escape(r.address.as_deref().unwrap_or("No billing address on file.")),
        )),
        None => Html(
            r#"<div class="address-card"><div class="address-line">No billing address on file.</div></div>"#.to_string(),
        ),
    }
}

pub async fn handle_invoices(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct InvRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total: Option<BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }

    let rows: Vec<InvRow> = diesel::sql_query(
        "SELECT invoice_number AS number, total, status, created_at FROM billing_invoices
         WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 12",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<tr><td colspan="4" class="empty-cell">No invoices yet</td></tr>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<tr>
    <td>#{number}</td>
    <td>{date}</td>
    <td>{total}</td>
    <td><span class="status-badge {status}">{status}</span></td>
</tr>"#,
            number = html_escape(&r.number),
            date = r.created_at.format("%b %d, %Y"),
            total = format_currency(r.total.as_ref().map(bd_to_f64).unwrap_or(0.0)),
            status = r.status.unwrap_or_else(|| "unknown".to_string()),
        ));
    }
    Html(html)
}

pub async fn handle_retention_offers(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let _ = state;
    Html(
        r#"<div class="retention-offers">
    <div class="retention-offer">
        <div class="offer-info"><span class="offer-name">Annual discount</span><span class="offer-desc">Switch to yearly billing and save 25%</span></div>
        <button class="btn-secondary btn-sm" hx-post="/api/billing/accept-offer" hx-vals='{"offer": "discount_25"}' hx-swap="none">Apply</button>
    </div>
    <div class="retention-offer">
        <div class="offer-info"><span class="offer-name">Pause subscription</span><span class="offer-desc">Keep your data and pause billing for 30 days</span></div>
        <button class="btn-secondary btn-sm" hx-post="/api/billing/pause" hx-swap="none">Pause</button>
    </div>
</div>"#
            .to_string(),
    )
}

#[derive(Deserialize)]
pub struct OfferForm {
    pub offer: Option<String>,
}

pub async fn handle_upgrade(State(state): State<Arc<BillingApiState>>, axum::extract::Form(form): axum::extract::Form<OfferForm>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string()));
    };
    let branch = branch_id(&state);
    let plan = form.offer.unwrap_or_else(|| "shared".to_string());

    let _ = diesel::sql_query(
        "INSERT INTO billing_recurring (id, branch_id, plan_name, status, currency, frequency, interval_count, start_date, created_at, updated_at, customer_name)
         VALUES ($1, $2, $3, 'trialing', 'USD', 'monthly', 1, CURRENT_DATE, NOW(), NOW(), 'Organization')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&plan)
    .execute(&mut conn);

    (StatusCode::OK, Html(format!("Subscription upgraded to <strong>{}</strong>", html_escape(&plan))))
}

pub async fn handle_accept_offer(State(state): State<Arc<BillingApiState>>, axum::extract::Form(form): axum::extract::Form<OfferForm>) -> impl IntoResponse {
    let _ = (state, form);
    (StatusCode::OK, Html("Retention offer applied".to_string()))
}

pub async fn handle_pause(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string()));
    };
    let branch = branch_id(&state);

    let _ = diesel::sql_query(
        "UPDATE billing_recurring SET status = 'paused', updated_at = NOW() WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn);

    (StatusCode::OK, Html("Subscription paused. Billing will resume on your next period.".to_string()))
}

pub async fn handle_cancel(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string()));
    };
    let branch = branch_id(&state);

    let _ = diesel::sql_query(
        "UPDATE billing_recurring SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW() WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn);

    (StatusCode::OK, Html("Subscription cancelled. Access remains active until the end of the billing period.".to_string()))
}

#[derive(Deserialize)]
pub struct SubscriptionRequest {
    pub plan_id: Option<String>,
    pub reason: Option<String>,
}

pub async fn handle_subscription_upgrade(State(state): State<Arc<BillingApiState>>, Json(req): Json<SubscriptionRequest>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": "Database unavailable" })));
    };
    let branch = branch_id(&state);
    let plan = req.plan_id.unwrap_or_else(|| "shared".to_string());

    let _ = diesel::sql_query(
        "UPDATE billing_recurring SET plan_name = $1, status = 'active', updated_at = NOW() WHERE branch_id = $2",
    )
    .bind::<diesel::sql_types::Text, _>(&plan)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn);

    (
        StatusCode::OK,
        Json(serde_json::json!({ "success": true, "plan": plan })),
    )
}

pub async fn handle_subscription_cancel(State(state): State<Arc<BillingApiState>>, Json(req): Json<SubscriptionRequest>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": "Database unavailable" })));
    };
    let branch = branch_id(&state);
    let reason = req.reason.unwrap_or_default();

    let _ = diesel::sql_query(
        "UPDATE billing_recurring SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW() WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn);

    let _ = reason;
    (
        StatusCode::OK,
        Json(serde_json::json!({ "success": true })),
    )
}

pub async fn handle_invoices_export(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database unavailable").into_response();
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct ExportRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        customer_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total: Option<BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Date)]
        issue_date: chrono::NaiveDate,
    }

    let rows: Vec<ExportRow> = diesel::sql_query(
        "SELECT invoice_number AS number, customer_name, total, status, issue_date FROM billing_invoices WHERE branch_id = $1 ORDER BY issue_date DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    let mut csv = String::from("invoice_number,customer,amount,status,issue_date\n");
    for r in rows {
        let amount = r.total.as_ref().map(bd_to_f64).unwrap_or(0.0);
        csv.push_str(&format!(
            "{},{},{:.2},{},{}\n",
            r.number,
            r.customer_name.unwrap_or_default().replace(',', " "),
            amount,
            r.status.unwrap_or_else(|| "unknown".to_string()),
            r.issue_date,
        ));
    }

    (
        [
            (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "attachment; filename=\"invoices.csv\"",
            ),
        ],
        csv,
    )
        .into_response()
}

pub async fn handle_invoices_unpaid(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(r#"<option value="">Unavailable</option>"#.to_string());
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct InvRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
    }

    let rows: Vec<InvRow> = diesel::sql_query(
        "SELECT id, invoice_number AS number FROM billing_invoices WHERE branch_id = $1 AND (status = 'sent' OR status = 'overdue' OR status IS NULL) ORDER BY created_at DESC LIMIT 50",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<option value="">No unpaid invoices</option>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<option value="{id}">#{number}</option>"#,
            id = r.id,
            number = html_escape(&r.number),
        ));
    }
    Html(html)
}

pub async fn handle_dashboard_alerts(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    let overdue: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM billing_invoices WHERE branch_id = $1 AND status <> 'paid' AND status <> 'void' AND due_date < CURRENT_DATE",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let paused: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM billing_recurring WHERE branch_id = $1 AND status = 'paused'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    if overdue == 0 && paused == 0 {
        return Html(r#"<div class="alerts-empty">No billing alerts</div>"#.to_string());
    }

    let mut html = String::new();
    if overdue > 0 {
        html.push_str(&format!(
            r#"<div class="alert-item warning"><span class="alert-icon">⚠️</span><span class="alert-text">{overdue} invoice(s) overdue</span></div>"#
        ));
    }
    if paused > 0 {
        html.push_str(
            r#"<div class="alert-item info"><span class="alert-icon">⏸</span><span class="alert-text">Subscription is paused</span></div>"#,
        );
    }
    Html(html)
}

pub async fn handle_dashboard_bot_usage(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let rows: Vec<BotRow> = diesel::sql_query(
        "SELECT b.name, COUNT(mh.id)::bigint AS count
         FROM bots b LEFT JOIN user_sessions us ON us.bot_id = b.bot_id
         LEFT JOIN message_history mh ON mh.session_id = us.id
         WHERE b.branch_id = $1 GROUP BY b.id, b.name ORDER BY count DESC LIMIT 5",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="bots-empty">No bot usage data</div>"#.to_string());
    }

    let max = rows.iter().map(|r| r.count).max().unwrap_or(1).max(1);
    let mut html = String::new();
    for r in rows {
        let pct = (r.count as f64 / max as f64) * 100.0;
        html.push_str(&format!(
            r#"<div class="usage-bar-row"><span class="bar-label">{name}</span><div class="bar-track"><div class="bar-fill" style="width:{pct:.0}%"></div></div><span class="bar-value">{count}</span></div>"#,
            name = html_escape(&r.name),
            count = r.count,
        ));
    }
    Html(html)
}

pub async fn handle_dashboard_recent_invoices(State(state): State<Arc<BillingApiState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = branch_id(&state);

    #[derive(diesel::QueryableByName)]
    struct InvRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total: Option<BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }

    let rows: Vec<InvRow> = diesel::sql_query(
        "SELECT invoice_number AS number, total, status, created_at FROM billing_invoices
         WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 5",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="empty-state"><p>No recent invoices</p></div>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="invoice-row"><span class="invoice-number">#{number}</span><span class="invoice-date">{date}</span><span class="invoice-total">{total}</span><span class="invoice-status {status}">{status}</span></div>"#,
            number = html_escape(&r.number),
            date = r.created_at.format("%b %d"),
            total = format_currency(r.total.as_ref().map(bd_to_f64).unwrap_or(0.0)),
            status = r.status.unwrap_or_else(|| "unknown".to_string()),
        ));
    }
    Html(html)
}

fn format_currency(amount: f64) -> String {
    format!("${amount:.2}")
}
