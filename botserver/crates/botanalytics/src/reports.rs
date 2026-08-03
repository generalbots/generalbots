//! Business reports: CRM, billing and support report cards rendered as HTML
//! fragments for the analytics app. All queries are branch-scoped via the
//! configured bot context (global SaaS admin uses `Uuid::nil()`).

use std::sync::Arc;

use axum::{extract::State, response::Html};
use chrono::Utc;
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::{DbPool, GetDefaultBotFn};

pub struct ReportsState {
    pub pool: Arc<DbPool>,
    pub default_bot: Arc<GetDefaultBotFn>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    label: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ValueRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
    value: Option<bigdecimal::BigDecimal>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct AvgRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    avg: Option<f64>,
}

#[derive(Deserialize)]
pub struct ReportQuery {
    pub q: Option<String>,
}

fn count_rows(conn: &mut diesel::PgConnection, sql: &str) -> Vec<CountRow> {
    diesel::sql_query(sql)
        .load::<CountRow>(conn)
        .unwrap_or_default()
}

fn money(value: Option<&bigdecimal::BigDecimal>) -> String {
    value
        .map(|v| format!("${v:.2}"))
        .unwrap_or_else(|| "$0.00".to_string())
}

// ── CRM reports ────────────────────────────────────────────────────────────

pub async fn crm_pipeline(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let stages = ["new", "contacted", "qualified", "proposal", "negotiation", "won"];
    let colors = ["#4caf50", "#8bc34a", "#cddc39", "#ffc107", "#ff9800", "#2196f3"];
    let mut funnel = String::new();
    let mut max_count: i64 = 1;
    let mut counts = std::collections::HashMap::new();

    for stage in stages.iter() {
        let count: i64 = diesel::sql_query(
            "SELECT COUNT(*)::bigint AS count FROM crm_deals WHERE branch_id = $1 AND stage = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Text, _>(stage)
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);
        counts.insert(stage, count);
        max_count = max_count.max(count);
    }

    for (i, stage) in stages.iter().enumerate() {
        let count = counts.get(stage).copied().unwrap_or(0);
        let pct = if max_count > 0 {
            (count as f64 / max_count as f64) * 100.0
        } else {
            0.0
        };
        let label = capitalize(stage);
        funnel.push_str(&format!(
            r#"<div class="funnel-stage" style="--width: {pct:.0}%; --color: {color}"><span class="stage-label">{label}</span><span class="stage-value">{count}</span></div>"#,
            color = colors[i],
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Sales Pipeline</h4><span class="report-badge">Funnel</span></div>
<div class="report-card-body"><div class="chart-placeholder"><div class="funnel-chart">{funnel}</div></div></div>"##
    ))
}

pub async fn crm_conversion(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let total: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_deals WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let won: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_deals WHERE branch_id = $1 AND won = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let leads: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_leads WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let rate = if total > 0 { (won as f64 / total as f64) * 100.0 } else { 0.0 };

    Html(format!(
        r##"<div class="report-card-header"><h4>Lead Conversion Rate</h4><span class="report-badge">KPI</span></div>
<div class="report-card-body"><div class="kpi-display">
    <div class="kpi-main"><span class="kpi-value">{rate:.0}%</span></div>
    <div class="kpi-details">
        <div class="kpi-detail-item"><span class="detail-label">Leads</span><span class="detail-value">{leads}</span></div>
        <div class="kpi-detail-item"><span class="detail-label">Opportunities</span><span class="detail-value">{total}</span></div>
        <div class="kpi-detail-item"><span class="detail-label">Won</span><span class="detail-value">{won}</span></div>
    </div>
</div></div>"##
    ))
}

pub async fn crm_won_lost(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let won: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_deals WHERE branch_id = $1 AND won = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let lost: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM crm_deals WHERE branch_id = $1 AND won = false",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let total = won + lost;
    let won_pct = if total > 0 { (won as f64 / total as f64) * 100.0 } else { 0.0 };
    let lost_pct = if total > 0 { (lost as f64 / total as f64) * 100.0 } else { 0.0 };

    Html(format!(
        r##"<div class="report-card-header"><h4>Won/Lost Analysis</h4><span class="report-badge">Chart</span></div>
<div class="report-card-body"><div class="donut-chart-container">
    <div class="donut-chart">
        <svg viewBox="0 0 100 100" class="donut-svg">
            <circle cx="50" cy="50" r="40" fill="none" stroke="#4CAF50" stroke-width="15" stroke-dasharray="{won_pct:.0} 100" stroke-dashoffset="25" class="donut-segment won"/>
            <circle cx="50" cy="50" r="40" fill="none" stroke="#F44336" stroke-width="15" stroke-dasharray="{lost_pct:.0} 100" stroke-dashoffset="{won_pct_off:.0}" class="donut-segment lost"/>
        </svg>
        <div class="donut-center"><span class="donut-total">{total}</span><span class="donut-label">Total</span></div>
    </div>
    <div class="donut-legend">
        <div class="legend-item"><span class="legend-color" style="background:#4caf50"></span><span>Won</span><span class="legend-value">{won}</span></div>
        <div class="legend-item"><span class="legend-color" style="background:#f44336"></span><span>Lost</span><span class="legend-value">{lost}</span></div>
    </div>
</div></div>"##,
        won_pct_off = -won_pct - 25.0
    ))
}

pub async fn crm_forecast(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let this_month: Option<bigdecimal::BigDecimal> = diesel::sql_query(
        "SELECT COALESCE(SUM(value), 0) AS value FROM crm_deals
         WHERE branch_id = $1 AND won = false AND expected_close_date BETWEEN date_trunc('month', CURRENT_DATE) AND (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: ValueRow| r.value)
    .ok()
    .flatten();

    let next_month: Option<bigdecimal::BigDecimal> = diesel::sql_query(
        "SELECT COALESCE(SUM(value), 0) AS value FROM crm_deals
         WHERE branch_id = $1 AND won = false AND expected_close_date BETWEEN date_trunc('month', CURRENT_DATE) + INTERVAL '1 month' AND date_trunc('month', CURRENT_DATE) + INTERVAL '2 months'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: ValueRow| r.value)
    .ok()
    .flatten();

    Html(format!(
        r##"<div class="report-card-header"><h4>Sales Forecast</h4><span class="report-badge">Projection</span></div>
<div class="report-card-body"><div class="forecast-display">
    <div class="forecast-main"><span class="forecast-label">This Month</span><span class="forecast-value">{this}</span></div>
    <div class="forecast-main secondary"><span class="forecast-label">Next Month</span><span class="forecast-value">{next}</span></div>
</div></div>"##,
        this = money(this_month.as_ref()),
        next = money(next_month.as_ref()),
    ))
}

// ── Billing reports ─────────────────────────────────────────────────────────

pub async fn billing_aging(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let buckets = [
        ("Current", "due_date >= CURRENT_DATE"),
        ("1-30 days", "due_date BETWEEN CURRENT_DATE - INTERVAL '30 days' AND CURRENT_DATE - 1"),
        ("31-60 days", "due_date BETWEEN CURRENT_DATE - INTERVAL '60 days' AND CURRENT_DATE - INTERVAL '31 days'"),
        ("60+ days", "due_date < CURRENT_DATE - INTERVAL '60 days'"),
    ];

    let mut rows = String::new();
    for (label, cond) in buckets {
        let value: Option<bigdecimal::BigDecimal> = diesel::sql_query(&format!(
            "SELECT COALESCE(SUM(amount_due), 0) AS value FROM billing_invoices WHERE branch_id = $1 AND status <> 'paid' AND status <> 'void' AND {cond}"
        ))
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result(&mut conn)
        .map(|r: ValueRow| r.value)
        .ok()
        .flatten();
        rows.push_str(&format!(
            r#"<div class="aging-row"><span class="aging-label">{label}</span><span class="aging-value">{value}</span></div>"#,
            value = money(value.as_ref())
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Billing Aging</h4><span class="report-badge">AR</span></div>
<div class="report-card-body"><div class="aging-list">{rows}</div></div>"##
    ))
}

pub async fn billing_monthly(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let rows: Vec<CountRow> = count_rows(
        &mut conn,
        &format!(
            "SELECT to_char(paid_at, 'YYYY-MM') AS label, COUNT(*)::bigint AS count
             FROM billing_payments WHERE branch_id = '{}' AND paid_at IS NOT NULL AND status <> 'refunded'
             GROUP BY label ORDER BY label DESC LIMIT 6",
            branch
        ),
    );

    let mut bars = String::new();
    for row in rows.iter().rev() {
        let count = row.count;
        let label = &row.label;
        bars.push_str(&format!(
            r#"<div class="bar-column"><div class="bar-fill" style="height: {h}%"></div><span class="bar-label">{label}</span><span class="bar-value">{count}</span></div>"#,
            h = 20 + (count.min(100) * 70) / 100
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Monthly Payments</h4><span class="report-badge">Chart</span></div>
<div class="report-card-body"><div class="bar-chart">{bars}</div></div>"##
    ))
}

pub async fn billing_payments(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PaymentRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        payer: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        method: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        amount: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        paid_at: chrono::DateTime<Utc>,
    }

    let payments: Vec<PaymentRow> = diesel::sql_query(
        "SELECT COALESCE(payer_name, payer_email, '—') AS payer,
                COALESCE(payment_method, '—') AS method, amount, paid_at
         FROM billing_payments WHERE branch_id = $1 AND paid_at IS NOT NULL
         ORDER BY paid_at DESC LIMIT 6",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .unwrap_or_default();

    let mut rows = String::new();
    for p in payments {
        rows.push_str(&format!(
            r#"<tr><td>{payer}</td><td>{method}</td><td>{amount}</td><td>{date}</td></tr>"#,
            payer = p.payer,
            method = p.method,
            amount = money(p.amount.as_ref()),
            date = p.paid_at.format("%b %d, %Y"),
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Recent Payments</h4><span class="report-badge">Table</span></div>
<div class="report-card-body"><table class="report-table"><thead><tr><th>Payer</th><th>Method</th><th>Amount</th><th>Date</th></tr></thead><tbody>{rows}</tbody></table></div>"##
    ))
}

pub async fn billing_revenue(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let total: Option<bigdecimal::BigDecimal> = diesel::sql_query(
        "SELECT COALESCE(SUM(amount), 0) AS value FROM billing_payments
         WHERE branch_id = $1 AND paid_at IS NOT NULL AND status <> 'refunded'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: ValueRow| r.value)
    .ok()
    .flatten();

    let month: Option<bigdecimal::BigDecimal> = diesel::sql_query(
        "SELECT COALESCE(SUM(amount), 0) AS value FROM billing_payments
         WHERE branch_id = $1 AND paid_at IS NOT NULL AND paid_at >= date_trunc('month', CURRENT_DATE)",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: ValueRow| r.value)
    .ok()
    .flatten();

    let outstanding: Option<bigdecimal::BigDecimal> = diesel::sql_query(
        "SELECT COALESCE(SUM(amount_due), 0) AS value FROM billing_invoices
         WHERE branch_id = $1 AND status <> 'paid' AND status <> 'void'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: ValueRow| r.value)
    .ok()
    .flatten();

    Html(format!(
        r##"<div class="report-card-header"><h4>Revenue</h4><span class="report-badge">KPI</span></div>
<div class="report-card-body"><div class="kpi-display">
    <div class="kpi-main"><span class="kpi-value">{total}</span></div>
    <div class="kpi-details">
        <div class="kpi-detail-item"><span class="detail-label">This Month</span><span class="detail-value">{month}</span></div>
        <div class="kpi-detail-item"><span class="detail-label">Outstanding</span><span class="detail-value">{outstanding}</span></div>
    </div>
</div></div>"##,
        total = money(total.as_ref()),
        month = money(month.as_ref()),
        outstanding = money(outstanding.as_ref()),
    ))
}

// ── Support reports ─────────────────────────────────────────────────────────

pub async fn support_ai_rate(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let total: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM support_tickets WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let ai: i64 = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM support_tickets WHERE branch_id = $1 AND source = 'ai'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: CountRow| r.count)
    .unwrap_or(0);

    let rate = if total > 0 { (ai as f64 / total as f64) * 100.0 } else { 0.0 };

    Html(format!(
        r##"<div class="report-card-header"><h4>AI Resolution Rate</h4><span class="report-badge">KPI</span></div>
<div class="report-card-body"><div class="kpi-display">
    <div class="kpi-main"><span class="kpi-value">{rate:.0}%</span></div>
    <div class="kpi-details">
        <div class="kpi-detail-item"><span class="detail-label">AI Resolved</span><span class="detail-value">{ai}</span></div>
        <div class="kpi-detail-item"><span class="detail-label">Total</span><span class="detail-value">{total}</span></div>
    </div>
</div></div>"##
    ))
}

pub async fn support_by_category(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let rows: Vec<CountRow> = count_rows(
        &mut conn,
        &format!(
            "SELECT COALESCE(category, 'Uncategorized') AS label, COUNT(*)::bigint AS count
             FROM support_tickets WHERE branch_id = '{branch}' GROUP BY label ORDER BY count DESC LIMIT 6"
        ),
    );
    let max = rows.iter().map(|r| r.count).max().unwrap_or(1).max(1);

    let mut bars = String::new();
    for r in rows {
        let pct = (r.count as f64 / max as f64) * 100.0;
        bars.push_str(&format!(
            r#"<div class="bar-row"><span class="bar-label">{label}</span><div class="bar-track"><div class="bar-fill" style="width:{pct:.0}%"></div></div><span class="bar-value">{count}</span></div>"#,
            label = r.label,
            count = r.count,
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Tickets by Category</h4><span class="report-badge">Chart</span></div>
<div class="report-card-body"><div class="horizontal-bars">{bars}</div></div>"##
    ))
}

pub async fn support_by_priority(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let rows: Vec<CountRow> = count_rows(
        &mut conn,
        &format!(
            "SELECT COALESCE(priority, 'unknown') AS label, COUNT(*)::bigint AS count
             FROM support_tickets WHERE branch_id = '{branch}' GROUP BY label ORDER BY count DESC LIMIT 5"
        ),
    );

    let mut items = String::new();
    for r in rows {
        let color = match r.label.as_str() {
            "critical" | "urgent" => "#ef4444",
            "high" => "#f59e0b",
            "medium" | "normal" => "#3b82f6",
            _ => "#6b7280",
        };
        items.push_str(&format!(
            r#"<div class="priority-item"><span class="priority-dot" style="background:{color}"></span><span class="priority-label">{label}</span><span class="priority-count">{count}</span></div>"#,
            label = r.label,
            count = r.count,
        ));
    }

    Html(format!(
        r##"<div class="report-card-header"><h4>Tickets by Priority</h4><span class="report-badge">Chart</span></div>
<div class="report-card-body"><div class="priority-list">{items}</div></div>"##
    ))
}

pub async fn support_resolution_time(
    State(state): State<Arc<ReportsState>>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };
    let branch = (state.default_bot)(&mut conn);

    let avg: Option<f64> = diesel::sql_query(
        "SELECT AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 3600.0) AS avg
         FROM support_tickets WHERE branch_id = $1 AND resolved_at IS NOT NULL",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map(|r: AvgRow| r.avg)
    .ok()
    .flatten();

    let display = match avg {
        Some(h) if h < 24.0 => format!("{h:.1}h"),
        Some(h) => format!("{:.1}d", h / 24.0),
        None => "—".to_string(),
    };

    Html(format!(
        r##"<div class="report-card-header"><h4>Avg Resolution Time</h4><span class="report-badge">KPI</span></div>
<div class="report-card-body"><div class="kpi-display"><div class="kpi-main"><span class="kpi-value">{display}</span></div></div></div>"##
    ))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn configure_reports_routes() -> axum::Router<Arc<ReportsState>> {
    use axum::routing::get;
    axum::Router::new()
        .route("/api/reports/crm/pipeline", get(crm_pipeline))
        .route("/api/reports/crm/conversion", get(crm_conversion))
        .route("/api/reports/crm/won-lost", get(crm_won_lost))
        .route("/api/reports/crm/forecast", get(crm_forecast))
        .route("/api/reports/billing/aging", get(billing_aging))
        .route("/api/reports/billing/monthly", get(billing_monthly))
        .route("/api/reports/billing/payments", get(billing_payments))
        .route("/api/reports/billing/revenue", get(billing_revenue))
        .route("/api/reports/support/ai-rate", get(support_ai_rate))
        .route("/api/reports/support/by-category", get(support_by_category))
        .route("/api/reports/support/by-priority", get(support_by_priority))
        .route("/api/reports/support/resolution-time", get(support_resolution_time))
}
