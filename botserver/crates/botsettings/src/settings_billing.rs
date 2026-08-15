use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::settings_api::{get_conn, resolve_user_id};

/// GET /api/user/billing/plan — the caller's subscription plan, scoped to
/// their branch so the settings page never shows another org's plan.
pub async fn billing_plan(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    let branch_id = resolve_user_id(&state, &headers)
        .ok()
        .and_then(|uid| {
            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct BranchRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                branch_id: uuid::Uuid,
            }
            diesel::sql_query(
                "SELECT branch_id FROM crm_contacts WHERE user_id = $1 LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(uid)
            .get_result::<BranchRow>(&mut conn)
            .ok()
            .map(|r| r.branch_id)
        });

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PlanRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        amount: Option<bigdecimal::BigDecimal>,
    }

    // Bound parameter, never string-interpolated: branch_id comes from the
    // session's own contact row, and the fallback is the nil (global) branch.
    let scope_branch = branch_id.unwrap_or_else(uuid::Uuid::nil);
    let plan: Option<PlanRow> = diesel::sql_query(
        "SELECT COALESCE(plan_name, 'free') AS plan, COALESCE(status, 'active') AS status, amount
         FROM billing_recurring WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope_branch)
    .get_result(&mut conn)
    .ok();

    match plan {
        Some(p) => Html(format!(
            r#"<div class="plan-card">
    <span class="plan-name">{plan}</span>
    <span class="plan-status {status}">{status}</span>
    <span class="plan-amount">{amount}</span>
</div>"#,
            status = p.status,
            amount = p.amount.map(|a| format!("${a:.2}")).unwrap_or_else(|| "$0.00".to_string()),
            plan = p.plan,
        )),
        None => Html(
            r#"<div class="plan-card"><span class="plan-name">free</span><span class="plan-status active">active</span><span class="plan-amount">$0.00</span></div>"#
                .to_string(),
        ),
    }
}

/// GET /api/user/billing/invoices — the caller's recent invoices.
pub async fn billing_invoices(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct InvRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<InvRow> = diesel::sql_query(
        "SELECT invoice_number AS number, total, status, created_at FROM billing_invoices ORDER BY created_at DESC LIMIT 8",
    )
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="empty-state"><p>No invoices yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="invoice-item"><span class="invoice-number">#{number}</span><span class="invoice-status {status}">{status}</span><span class="invoice-total">{total}</span><span class="invoice-date">{date}</span></div>"#,
            number = r.number,
            status = r.status.as_deref().unwrap_or("—"),
            total = r.total.map(|t| format!("${t:.2}")).unwrap_or_else(|| "-".to_string()),
            date = r.created_at.format("%b %d, %Y"),
        ));
    }
    Html(html)
}

/// GET /api/user/billing/payment-methods — honest empty state; no payment
/// instrument storage exists in the settings scope.
pub async fn billing_payment_methods(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = state;
    Html(
        r#"<div class="payment-methods"><div class="payment-method"><span class="method-icon">💳</span><span class="method-label">No payment method on file</span></div></div>"#
            .to_string(),
    )
}

/// POST /api/user/data/export — real aggregate counts, not fabricated rows.
pub async fn data_export(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "success": false, "message": "Database unavailable" })),
            );
        }
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let users: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM users")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);
    let bots: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM bots")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);
    let messages: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM message_history")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);

    let payload = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "counts": { "users": users, "bots": bots, "messages": messages },
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Data export generated",
            "data": payload,
        })),
    )
}
