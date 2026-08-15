use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::BillingApiState;
use crate::schema::{billing_invoices, billing_payments, billing_quotes, billing_recurring};

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

fn bd_to_f64(bd: &BigDecimal) -> f64 {
    bd.to_f64().unwrap_or(0.0)
}

const DEFAULT_QUOTA_LIMITS: &[(&str, i64)] = &[
    ("invoices", 100),
    ("team", 50),
    ("bots", 10),
];

fn parse_quota_limit(value: &serde_json::Value) -> Option<i64> {
    match value {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
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
        "EUR" => format!("\u{20ac}{:.2}", amount),
        "GBP" => format!("\u{00a3}{:.2}", amount),
        "BRL" => format!("R${:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
}

pub fn configure_billing_routes() -> Router<Arc<BillingApiState>> {
    Router::new()
        .route("/api/ui/billing/invoices", get(handle_invoices))
        .route("/api/ui/billing/payments", get(handle_payments))
        .route("/api/ui/billing/quotes", get(handle_quotes))
        .route("/api/ui/billing/stats/pending", get(handle_stats_pending))
        .route("/api/ui/billing/stats/revenue-month", get(handle_revenue_month))
        .route("/api/ui/billing/stats/paid-month", get(handle_paid_month))
        .route("/api/ui/billing/stats/overdue", get(handle_overdue))
        .route("/api/ui/billing/search", get(handle_billing_search))
        .route("/api/ui/billing/dashboard/metrics", get(handle_dashboard_metrics))
        .route("/api/ui/billing/dashboard/spending-chart", get(handle_spending_chart))
        .route("/api/ui/billing/dashboard/cost-breakdown", get(handle_cost_breakdown))
        .route("/api/ui/billing/dashboard/quotas", get(handle_dashboard_quotas))
        .route("/api/ui/billing/invoices/export", get(handle_invoices_export))
        .route("/api/ui/billing/subscription/upgrade", post(handle_subscription_upgrade))
        .route("/api/ui/billing/subscription/cancel", post(handle_subscription_cancel))
        .route("/api/admin/billing/quotas", put(handle_admin_billing_quotas))
        .route("/api/admin/billing/alerts", put(handle_admin_billing_alerts))
}

async fn handle_dashboard_metrics(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("Error connecting to DB".to_string());
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    // 1. Current Period Spending (Sum of invoices total for branch_id)
    let total_spent = billing_invoices::table
        .filter(billing_invoices::branch_id.eq(branch_id))
        .select(diesel::dsl::sum(billing_invoices::total))
        .first::<Option<BigDecimal>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or_else(|| BigDecimal::from(0));

    let total_spent_f = bd_to_f64(&total_spent);

    // 2. Projected (Total spent + sum of monthly recurring subscriptions amount)
    let total_recurring = billing_recurring::table
        .filter(billing_recurring::branch_id.eq(branch_id))
        .select(diesel::dsl::sum(billing_recurring::amount))
        .first::<Option<BigDecimal>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or_else(|| BigDecimal::from(0));

    let total_recurring_f = bd_to_f64(&total_recurring);
    let projected_f = total_spent_f + total_recurring_f;

    // 3. Budget (Default Limit: $4000.00)
    let budget_limit = 4000.0;
    let budget_used_pct = if budget_limit <= 0.0 { 0.0 } else { (total_spent_f / budget_limit) * 100.0 };
    let budget_remaining = (budget_limit - total_spent_f).max(0.0);

    // 4. Savings (Sum of discount_amount for branch_id)
    let total_savings = billing_invoices::table
        .filter(billing_invoices::branch_id.eq(branch_id))
        .select(diesel::dsl::sum(billing_invoices::discount_amount))
        .first::<Option<BigDecimal>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or_else(|| BigDecimal::from(0));

    let total_savings_f = bd_to_f64(&total_savings);

    let html = format!(
        r#"<div class="metric-card spending">
            <div class="metric-icon">
                <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><line x1="12" y1="1" x2="12" y2="23"></line><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path></svg>
            </div>
            <div class="metric-content">
                <span class="metric-value">${:.2}</span>
                <span class="metric-label">Current Period</span>
            </div>
            <span class="metric-trend positive">-12% vs last period</span>
        </div>
        <div class="metric-card forecast">
            <div class="metric-icon">
                <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline></svg>
            </div>
            <div class="metric-content">
                <span class="metric-value">${:.2}</span>
                <span class="metric-label">Projected</span>
            </div>
            <span class="metric-trend">End of period</span>
        </div>
        <div class="metric-card budget">
            <div class="metric-icon">
                <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect><line x1="1" y1="10" x2="23" y2="10"></line></svg>
            </div>
            <div class="metric-content">
                <span class="metric-value">{:.1}%</span>
                <span class="metric-label">Budget Used</span>
            </div>
            <span class="metric-trend">${:.2} remaining</span>
        </div>
        <div class="metric-card savings">
            <div class="metric-icon">
                <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M19 5c-1.5 0-2.8 1.4-3 2-3.5-1.5-11-.3-11 5 0 1.8 0 3 2 4.5V20h4v-2h3v2h4v-4c1-.5 1.7-1 2-2h2v-4h-2c0-1-.5-1.5-1-2V5z"></path><path d="M2 9v1c0 1.1.9 2 2 2h1"></path></svg>
            </div>
            <div class="metric-content">
                <span class="metric-value">${:.2}</span>
                <span class="metric-label">Savings</span>
            </div>
            <span class="metric-trend positive">This month</span>
        </div>"#,
        total_spent_f, projected_f, budget_used_pct, budget_remaining, total_savings_f
    );
    Html(html)
}

async fn handle_spending_chart(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    #[derive(diesel::QueryableByName)]
    struct BarRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        label: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)]
        total: BigDecimal,
    }
    let rows: Vec<BarRow> = diesel::sql_query(
        "SELECT to_char(issue_date, 'Mon') AS label, COALESCE(SUM(total), 0) AS total \
         FROM billing_invoices WHERE branch_id = $1 \
         AND issue_date >= date_trunc('month', NOW()) - interval '6 months' \
         GROUP BY to_char(issue_date, 'Mon') ORDER BY MIN(issue_date)",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="chart-empty">No invoice data yet</div>"#.to_string());
    }

    let max = rows.iter().map(|r| bd_to_f64(&r.total)).fold(0.0_f64, f64::max).max(1.0);
    let mut html = String::from(r#"<div class="chart-bars">"#);
    for r in &rows {
        let v = bd_to_f64(&r.total);
        let pct = (v / max * 100.0).max(4.0) as u32;
        html.push_str(&format!(
            r#"<div class="chart-bar" style="height: {pct}%"><span class="bar-label">{}</span><span class="bar-value">${:.0}</span></div>"#,
            html_escape(&r.label), v
        ));
    }
    html.push_str("</div>");
    Html(html)
}

async fn handle_cost_breakdown(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    #[derive(diesel::QueryableByName)]
    struct StatusRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)]
        total: BigDecimal,
    }
    let rows: Vec<StatusRow> = diesel::sql_query(
        "SELECT COALESCE(status, 'draft') AS status, COALESCE(SUM(total), 0) AS total \
         FROM billing_invoices WHERE branch_id = $1 GROUP BY COALESCE(status, 'draft')",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="breakdown-empty">No invoice data yet</div>"#.to_string());
    }

    let grand: f64 = rows.iter().map(|r| bd_to_f64(&r.total)).sum();
    let colors = ["#3b82f6", "#10b981", "#f59e0b", "#8b5cf6", "#ef4444", "#06b6d4", "#84cc16"];
    let mut html = String::new();
    for (i, r) in rows.iter().enumerate() {
        let v = bd_to_f64(&r.total);
        let pct = if grand > 0.0 { v / grand * 100.0 } else { 0.0 };
        let color = colors[i % colors.len()];
        html.push_str(&format!(
            r#"<div class="breakdown-item"><div class="breakdown-color" style="background: {color}"></div><span class="breakdown-label">{}</span><span class="breakdown-value">${:.2}</span><span class="breakdown-percent">{:.0}%</span></div>"#,
            html_escape(&r.status), v, pct
        ));
    }
    Html(html)
}

async fn handle_dashboard_quotas(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("Error connecting to DB".to_string());
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    // Dynamic counts
    let bot_count = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM bots WHERE branch_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let contact_count = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_contacts WHERE org_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let invoice_count = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM billing_invoices WHERE branch_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<CountRow>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    #[derive(diesel::QueryableByName)]
    struct QuotaRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        quota_key: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        quota_limit: i64,
    }
    let quota_rows: Vec<QuotaRow> = diesel::sql_query(
        "SELECT quota_key, quota_limit FROM billing_quota_settings WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .load(&mut conn)
    .unwrap_or_default();

    let mut quota_limits: HashMap<String, i64> = HashMap::new();
    for &(key, default) in DEFAULT_QUOTA_LIMITS {
        quota_limits.insert(key.to_string(), default);
    }
    for row in quota_rows {
        quota_limits.insert(row.quota_key, row.quota_limit);
    }

    let invoice_limit = quota_limits.get("invoices").copied().unwrap_or(100);
    let team_limit = quota_limits.get("team").copied().unwrap_or(50);
    let bot_limit = quota_limits.get("bots").copied().unwrap_or(10);

    let bot_pct = (bot_count as f64 / bot_limit as f64 * 100.0).min(100.0);
    let team_pct = (contact_count as f64 / team_limit as f64 * 100.0).min(100.0);
    let invoice_pct = (invoice_count as f64 / invoice_limit as f64 * 100.0).min(100.0);

    let html = format!(
        r#"<div class="quota-item"><div class="quota-header"><span class="quota-name">Invoices</span><span class="quota-usage">{invoice_count} / {invoice_limit}</span></div><div class="quota-bar"><div class="quota-fill" style="width: {invoice_pct:.1}%"></div></div></div>
        <div class="quota-item"><div class="quota-header"><span class="quota-name">Team Members</span><span class="quota-usage">{} / {}</span></div><div class="quota-bar"><div class="quota-fill" style="width: {:.1}%"></div></div></div>
        <div class="quota-item"><div class="quota-header"><span class="quota-name">Bots</span><span class="quota-usage">{} / {}</span></div><div class="quota-bar"><div class="quota-fill" style="width: {:.1}%"></div></div></div>"#,
        contact_count, team_limit, team_pct, bot_count, bot_limit, bot_pct
    );
    Html(html)
}

async fn handle_invoices_export(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let rows = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        #[derive(diesel::QueryableByName)]
        struct CsvRow {
            #[diesel(sql_type = diesel::sql_types::Varchar)]
            invoice_number: String,
            #[diesel(sql_type = diesel::sql_types::Date)]
            issue_date: chrono::NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            total: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
            status: Option<String>,
        }
        diesel::sql_query(
            "SELECT invoice_number, issue_date, total, status FROM billing_invoices \
             WHERE branch_id = $1 ORDER BY issue_date DESC LIMIT 1000",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<CsvRow>(&mut conn)
        .ok()
    })
    .await
    .ok()
    .flatten();

    let mut csv_content = String::from("Invoice ID,Date,Amount,Status\n");
    if let Some(rows) = rows {
        for r in rows {
            csv_content.push_str(&format!(
                "{},{},${:.2},{}\n",
                r.invoice_number,
                r.issue_date,
                bd_to_f64(&r.total),
                r.status.as_deref().unwrap_or("draft")
            ));
        }
    }
    (
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            ("Content-Disposition", "attachment; filename=\"invoices.csv\""),
        ],
        csv_content,
    )
}

#[derive(Deserialize)]
struct UpgradeRequest {
    plan_id: String,
}

async fn handle_subscription_upgrade(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Json(req): Json<UpgradeRequest>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Json(serde_json::json!({
            "success": false,
            "error": "Database unavailable"
        }));
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    match diesel::sql_query(
        "UPDATE billing_recurring \
         SET plan_name = $1, status = 'active', updated_at = NOW() \
         WHERE branch_id = $2",
    )
    .bind::<diesel::sql_types::Text, _>(&req.plan_id)
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .execute(&mut conn)
    {
        Ok(rows) if rows > 0 => Json(serde_json::json!({
            "success": true,
            "plan_id": req.plan_id,
            "message": "Subscription upgraded successfully"
        })),
        Ok(_) => Json(serde_json::json!({
            "success": false,
            "error": "No active subscription found for this branch"
        })),
        Err(e) => {
            tracing::error!("Failed to upgrade subscription: {e}");
            Json(serde_json::json!({
                "success": false,
                "error": "Failed to upgrade subscription"
            }))
        }
    }
}

#[derive(Deserialize)]
struct CancelRequest {
    reason: Option<String>,
}

async fn handle_subscription_cancel(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Json(req): Json<CancelRequest>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Json(serde_json::json!({
            "success": false,
            "error": "Database unavailable"
        }));
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    match diesel::sql_query(
        "UPDATE billing_recurring \
         SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW() \
         WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .execute(&mut conn)
    {
        Ok(rows) if rows > 0 => Json(serde_json::json!({
            "success": true,
            "message": "Subscription cancelled",
            "reason": req.reason
        })),
        Ok(_) => Json(serde_json::json!({
            "success": false,
            "error": "No active subscription found for this branch"
        })),
        Err(e) => {
            tracing::error!("Failed to cancel subscription: {e}");
            Json(serde_json::json!({
                "success": false,
                "error": "Failed to cancel subscription"
            }))
        }
    }
}

async fn handle_admin_billing_quotas(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Json(quotas): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Json(serde_json::json!({
            "success": false,
            "error": "Database unavailable"
        }));
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    let now = chrono::Utc::now();
    let mut saved: usize = 0;

    if let Some(obj) = quotas.as_object() {
        for (key, value) in obj {
            let Some(limit) = parse_quota_limit(value) else {
                continue;
            };
            let key = key.trim().to_lowercase();
            if key.is_empty() || key.len() > 64 {
                continue;
            }
            let result = diesel::sql_query(
                "INSERT INTO billing_quota_settings \
                 (id, branch_id, quota_key, quota_limit, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $5) \
                 ON CONFLICT (branch_id, quota_key) \
                 DO UPDATE SET quota_limit = EXCLUDED.quota_limit, updated_at = $5",
            )
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Text, _>(&key)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn);

            if result.is_ok() {
                saved += 1;
            }
        }
    }

    Json(serde_json::json!({
        "success": saved > 0,
        "saved": saved,
        "message": format!("Quotas updated ({saved} values saved)")
    }))
}

async fn handle_admin_billing_alerts(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Json(settings): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.pool.get() else {
        return Json(serde_json::json!({
            "success": false,
            "error": "Database unavailable"
        }));
    };

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| crate::get_bot_context(&state.pool, &state.get_default_bot));

    let settings_json = serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
    let now = chrono::Utc::now();

    let result = diesel::sql_query(
        "INSERT INTO billing_alert_settings (id, branch_id, settings, created_at, updated_at) \
         VALUES ($1, $2, $3::jsonb, $4, $4) \
         ON CONFLICT (branch_id) DO UPDATE SET settings = EXCLUDED.settings, updated_at = $4",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(&settings_json)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn);

    match result {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Alert settings updated successfully",
            "settings": settings
        })),
        Err(e) => {
            tracing::error!("Failed to persist billing alert settings: {e}");
            Json(serde_json::json!({
                "success": false,
                "error": "Failed to save alert settings"
            }))
        }
    }
}

async fn handle_invoices(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let mut db_query = billing_invoices::table
            .filter(billing_invoices::branch_id.eq(branch_id))
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
            .load::<(Uuid, String, Option<String>, Option<String>, Option<String>, NaiveDate, Option<NaiveDate>, Option<BigDecimal>, BigDecimal, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(invoices) if !invoices.is_empty() => {
            let mut html = String::new();
            for (id, number, customer_name, customer_email, status, issue_date, due_date, total, amount_due, currency) in invoices {
                let name = customer_email.or(customer_name).unwrap_or_else(|| "Unknown".to_string());
                let total_str = format_currency(bd_to_f64(&total.unwrap_or_default()), &currency.as_deref().unwrap_or("USD"));
                let due_str = format_currency(bd_to_f64(&amount_due), &currency.as_deref().unwrap_or("USD"));
                let issue_str = issue_date.format("%Y-%m-%d").to_string();
                let due_date_str = due_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "-".to_string());

                let status_class = match status.as_deref().unwrap_or("draft") {
                    "paid" => "status-paid",
                    "sent" => "status-sent",
                    "overdue" => "status-overdue",
                    "void" => "status-void",
                    _ => "status-draft",
                };

                html.push_str(&format!(
                    r##"<tr class="invoice-row" data-id="{id}"><td class="invoice-number">{}</td><td class="invoice-customer">{}</td><td class="invoice-date">{}</td><td class="invoice-due">{}</td><td class="invoice-total">{}</td><td class="invoice-balance">{}</td><td class="invoice-status"><span class="{}">{}</span></td><td class="invoice-actions"><button class="btn-sm" hx-get="/api/billing/invoices/{id}" hx-target="#invoice-detail">View</button></td></tr>"##,
                    html_escape(&number),
                    html_escape(&name),
                    issue_str,
                    due_date_str,
                    total_str,
                    due_str,
                    status_class,
                    html_escape(status.as_deref().unwrap_or(""))
                ));
            }
            Html(html)
        }
        _ => Html(r#"<tr class="empty-row"><td colspan="8" class="empty-state"><div class="empty-icon">Document</div><p>No invoices yet</p><p class="empty-hint">Create your first invoice to get started</p></td></tr>"#.to_string()),
    }
}

async fn handle_payments(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let mut db_query = billing_payments::table
            .filter(billing_payments::branch_id.eq(branch_id))
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
            .load::<(Uuid, String, BigDecimal, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<DateTime<Utc>>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(payments) if !payments.is_empty() => {
            let mut html = String::new();
            for (id, number, amount, currency, method, payer_name, payer_email, status, paid_at) in payments {
                let amount_str = format_currency(bd_to_f64(&amount), currency.as_deref().unwrap_or("USD"));
                let payer = payer_name.or(payer_email).unwrap_or_else(|| "Unknown".to_string());
                let date_str = paid_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".to_string());

                let status_class = match status.as_deref().unwrap_or("draft") {
                    "completed" => "status-completed",
                    "pending" => "status-pending",
                    "refunded" => "status-refunded",
                    "failed" => "status-failed",
                    _ => "status-default",
                };

                html.push_str(&format!(
                    r##"<tr class="payment-row" data-id="{id}"><td class="payment-number">{}</td><td class="payment-payer">{}</td><td class="payment-amount">{}</td><td class="payment-method">{}</td><td class="payment-date">{}</td><td class="payment-status"><span class="{}">{}</span></td><td class="payment-actions"><button class="btn-sm" hx-get="/api/billing/payments/{id}" hx-target="#payment-detail">View</button></td></tr>"##,
                    html_escape(&number),
                    html_escape(&payer),
                    amount_str,
                    html_escape(method.as_deref().unwrap_or("")),
                    date_str,
                    status_class,
                    html_escape(status.as_deref().unwrap_or(""))
                ));
            }
            Html(html)
        }
        _ => Html(r#"<tr class="empty-row"><td colspan="7" class="empty-state"><div class="empty-icon">Payment</div><p>No payments recorded</p><p class="empty-hint">Payments will appear here when invoices are paid</p></td></tr>"#.to_string()),
    }
}

async fn handle_quotes(
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let mut db_query = billing_quotes::table
            .filter(billing_quotes::branch_id.eq(branch_id))
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
            .load::<(Uuid, String, Option<String>, Option<String>, Option<String>, NaiveDate, Option<NaiveDate>, Option<BigDecimal>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(quotes) if !quotes.is_empty() => {
            let mut html = String::new();
            for (id, number, customer_name, customer_email, status, issue_date, valid_until, total, currency) in quotes {
                let name = customer_email.or(customer_name).unwrap_or_else(|| "Unknown".to_string());
                let total_str = format_currency(bd_to_f64(&total.unwrap_or_default()), &currency.as_deref().unwrap_or("USD"));
                let issue_str = issue_date.format("%Y-%m-%d").to_string();
                let valid_str = valid_until.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "-".to_string());

                let status_class = match status.as_deref().unwrap_or("draft") {
                    "accepted" => "status-accepted",
                    "sent" => "status-sent",
                    "rejected" => "status-rejected",
                    "expired" => "status-expired",
                    "converted" => "status-converted",
                    _ => "status-draft",
                };

                html.push_str(&format!(
                    r##"<tr class="quote-row" data-id="{id}"><td class="quote-number">{}</td><td class="quote-customer">{}</td><td class="quote-date">{}</td><td class="quote-valid">{}</td><td class="quote-total">{}</td><td class="quote-status"><span class="{}">{}</span></td><td class="quote-actions"><button class="btn-sm" hx-get="/api/billing/quotes/{id}" hx-target="#quote-detail">View</button></td></tr>"##,
                    html_escape(&number),
                    html_escape(&name),
                    issue_str,
                    valid_str,
                    total_str,
                    status_class,
                    html_escape(status.as_deref().unwrap_or(""))
                ));
            }
            Html(html)
        }
        _ => Html(r#"<tr class="empty-row"><td colspan="7" class="empty-state"><div class="empty-icon">Quote</div><p>No quotes yet</p><p class="empty-hint">Create quotes for your prospects</p></td></tr>"#.to_string()),
    }
}

async fn handle_stats_pending(State(state): State<Arc<BillingApiState>>, headers: HeaderMap) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let totals: Vec<BigDecimal> = billing_invoices::table
            .filter(billing_invoices::branch_id.eq(branch_id))
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

async fn handle_revenue_month(State(state): State<Arc<BillingApiState>>, headers: HeaderMap) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let now = Utc::now();
        let month_start = now.date_naive().with_day(1)?.and_hms_opt(0, 0, 0)?;

        let totals: Vec<Option<BigDecimal>> = billing_invoices::table
            .filter(billing_invoices::branch_id.eq(branch_id))
            .filter(billing_invoices::created_at.ge(month_start))
            .select(billing_invoices::total)
            .load(&mut conn)
            .ok()?;

        let sum: f64 = totals.iter().map(|t| t.as_ref().map_or(0.0, |v| bd_to_f64(v))).sum();
        Some(sum)
    })
    .await
    .ok()
    .flatten();

    Html(format_currency(result.unwrap_or(0.0), "USD"))
}

async fn handle_paid_month(State(state): State<Arc<BillingApiState>>, headers: HeaderMap) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let now = Utc::now();
        let month_start = now.date_naive().with_day(1)?.and_hms_opt(0, 0, 0)?;

        let totals: Vec<BigDecimal> = billing_payments::table
            .filter(billing_payments::branch_id.eq(branch_id))
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

async fn handle_overdue(State(state): State<Arc<BillingApiState>>, headers: HeaderMap) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        let totals: Vec<BigDecimal> = billing_invoices::table
            .filter(billing_invoices::branch_id.eq(branch_id))
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
    State(state): State<Arc<BillingApiState>>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.clone().unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;
    let search_term = format!("%{}%", q);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
            .unwrap_or_else(|| crate::get_bot_context(&pool, &get_default_bot));
        if branch_id == Uuid::nil() { return None; }

        billing_invoices::table
            .filter(billing_invoices::branch_id.eq(branch_id))
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
            .load::<(Uuid, String, Option<String>, Option<String>, Option<BigDecimal>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, number, customer, status, total, currency) in items {
                let total_str = format_currency(bd_to_f64(&total.unwrap_or_default()), &currency.as_deref().unwrap_or("USD"));

                html.push_str(&format!(
                    r##"<div class="search-result-item" hx-get="/api/billing/invoices/{id}" hx-target="#invoice-detail"><span class="result-number">{}</span><span class="result-customer">{}</span><span class="result-status">{}</span><span class="result-total">{}</span></div>"##,
                    html_escape(&number),
                    html_escape(customer.as_deref().unwrap_or("")),
                    html_escape(status.as_deref().unwrap_or("")),
                    total_str
                ));
            }
            Html(format!(r#"<div class="search-results">{html}</div>"#))
        }
        _ => Html(format!(
            r#"<div class="search-results-empty"><p>No results for "{}"</p></div>"#,
            html_escape(&q)
        )),
    }
}
