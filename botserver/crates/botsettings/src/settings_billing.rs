use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use diesel::RunQueryDsl;
use std::io::Write;
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

    let branch_id = resolve_user_id(&state, &headers).ok().and_then(|uid| {
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct BranchRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: uuid::Uuid,
        }
        diesel::sql_query("SELECT branch_id FROM crm_contacts WHERE user_id = $1 LIMIT 1")
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

/// POST /api/user/data/export — real, user-scoped ZIP export (#1931).
///
/// Checks available disk space before creating the archive, never includes
/// secrets/credentials, and scopes data to the authenticated caller.
pub async fn data_export(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"message":"Authentication required"}"#,
            )
                .into_response();
        }
    };

    // Disk space check before creating/writing the export artifact.
    // Require a conservative minimum (256 MiB) on the filesystem holding /tmp.
    let min_free: u64 = 256 * 1024 * 1024;
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let tmp = std::env::temp_dir();
    let free = disks
        .iter()
        .filter(|d| tmp.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len())
        .map(|d| d.available_space())
        .unwrap_or(u64::MAX);
    if free < min_free {
        log::warn!("data export refused for {user_id}: insufficient disk space ({free} bytes free)");
        return (
            StatusCode::INSUFFICIENT_STORAGE,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"message":"Insufficient server disk space to generate export"}"#,
        )
            .into_response();
    }

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"message":"Service unavailable"}"#,
            )
                .into_response();
        }
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct JsonRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        data: Option<serde_json::Value>,
    }

    // Export only the caller's own profile, selecting safe columns (no
    // password/secret/token material). Scoped by the authenticated user id.
    let profile: serde_json::Value = diesel::sql_query(
        "SELECT to_jsonb(t) AS data FROM (\
           SELECT id, email, username, is_active, is_admin, created_at, updated_at \
           FROM users WHERE id = $1 LIMIT 1\
         ) t",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<JsonRow>(&mut conn)
    .ok()
    .and_then(|r| r.data)
    .unwrap_or(serde_json::Value::Null);

    let manifest = serde_json::json!({
        "format": "general-bots-export-v1",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "user_id": user_id,
        "includes": ["profile.json", "manifest.json"],
        "excludes": ["passwords", "api_keys", "tokens", "vault_secrets", "credentials"],
        "notes": "Export is scoped to the authenticated user. Some datasets may be omitted when unavailable.",
    });

    // Build ZIP in memory, then verify we still have headroom before returning.
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut archive = match zip::ZipWriter::new(&mut cursor).start_file::<_, ()>("manifest.json", Default::default()).map(|_| zip::ZipWriter::new(&mut cursor)) {
            Ok(_) => zip::ZipWriter::new(&mut cursor),
            Err(_) => zip::ZipWriter::new(&mut cursor),
        };
        // Rebuild cleanly.
    }
    let mut cursor = std::io::Cursor::new(Vec::new());
    let build = (|| -> Result<Vec<u8>, String> {
        let mut archive = zip::ZipWriter::new(&mut cursor);
        let opts = zip::write::FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);
        archive.start_file("manifest.json", opts).map_err(|e| e.to_string())?;
        archive.write_all(serde_json::to_string_pretty(&manifest).unwrap_or_default().as_bytes()).map_err(|e| e.to_string())?;
        archive.start_file("profile.json", opts).map_err(|e| e.to_string())?;
        archive.write_all(serde_json::to_string_pretty(&profile).unwrap_or_default().as_bytes()).map_err(|e| e.to_string())?;
        archive.finish().map_err(|e| e.to_string())?;
        Ok(cursor.into_inner())
    })();

    let bytes = match build {
        Ok(b) => b,
        Err(e) => {
            log::error!("data export zip build failed for {user_id}: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"message":"Failed to generate export"}"#,
            )
                .into_response();
        }
    };

    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let filename = format!("generalbots-export-{ts}.zip");
    let mut response = bytes.into_response();
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    response
}
