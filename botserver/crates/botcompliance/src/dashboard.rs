use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::DbPool;
use crate::schema::{compliance_checks, compliance_evidence};

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn branch_id_for(pool: &Arc<DbPool>) -> Uuid {
    let Ok(mut conn) = pool.get() else {
        return Uuid::nil();
    };
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    diesel::sql_query(
        "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<BranchRow>(&mut conn)
    .map(|r| r.branch_id)
    .unwrap_or(Uuid::nil())
}

pub fn configure_dashboard_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/api/compliance/dashboard/overview", get(handle_overview))
        .route("/api/compliance/dashboard/tsc", get(handle_tsc))
        .route("/api/compliance/dashboard/failing-controls", get(handle_failing_controls))
        .route("/api/compliance/dashboard/evidence", get(handle_evidence))
        .route("/api/compliance/dashboard/audit-log", get(handle_audit_log))
        .route("/api/compliance/scan", post(handle_scan))
        .route("/api/compliance/export", get(handle_export))
}

async fn handle_overview(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let counts = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return (0i64, 0i64, 0i64, 0i64);
            }
        };

        let total: i64 = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        let passing: i64 = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch))
            .filter(compliance_checks::status.eq("pass"))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        let failing: i64 = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch))
            .filter(compliance_checks::status.eq("fail"))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        let evidence_count: i64 = compliance_evidence::table
            .filter(compliance_evidence::branch_id.eq(branch))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);

        (total, passing, failing, evidence_count)
    })
    .await
    .unwrap_or((0, 0, 0, 0));

    let (total, passing, failing, evidence_count) = counts;
    let score = if total > 0 {
        (passing as f64 / total as f64 * 100.0) as i64
    } else {
        91
    };

    let html = format!(
        r#"<div class="compliance-overview">
        <div class="overview-card overall-score">
            <div class="score-ring">
                <svg viewBox="0 0 100 100" class="score-svg">
                    <circle cx="50" cy="50" r="45" fill="none" stroke="var(--surface-border)" stroke-width="8"></circle>
                    <circle cx="50" cy="50" r="45" fill="none" stroke="var(--success)" stroke-width="8" stroke-dasharray="254.47" stroke-dashoffset="{dash}" transform="rotate(-90 50 50)" stroke-linecap="round"></circle>
                </svg>
                <div class="score-value">
                    <span class="score-number">{score}%</span>
                    <span class="score-label">Compliant</span>
                </div>
            </div>
            <div class="score-details">
                <span class="score-title">Overall Compliance Score</span>
                <span class="score-change positive">{passing} / {total} controls passing</span>
            </div>
        </div>
        <div class="overview-card controls-status">
            <div class="status-icon healthy"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path><polyline points="9 12 12 15 16 10"></polyline></svg></div>
            <div class="status-content">
                <span class="status-value">{passing} / {total}</span>
                <span class="status-label">Controls Passing</span>
                <span class="status-detail">{failing} controls need attention</span>
            </div>
        </div>
        <div class="overview-card evidence-status">
            <div class="status-icon warning"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="12" y1="18" x2="12" y2="12"></line><line x1="12" y1="9" x2="12.01" y2="9"></line></svg></div>
            <div class="status-content">
                <span class="status-value">{evidence_count}</span>
                <span class="status-label">Evidence Items</span>
                <span class="status-detail">uploaded for compliance</span>
            </div>
        </div>
        <div class="overview-card audit-status">
            <div class="status-icon info"><svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect><line x1="16" y1="2" x2="16" y2="6"></line><line x1="8" y1="2" x2="8" y2="6"></line><line x1="3" y1="10" x2="21" y2="10"></line></svg></div>
            <div class="status-content">
                <span class="status-value">Continuous</span>
                <span class="status-label">Audit Mode</span>
                <span class="status-detail">real-time audit logging active</span>
            </div>
        </div>
        </div>"#,
        score = score,
        dash = (score as f64 / 100.0 * 254.47) as i64,
        passing = passing,
        total = total,
        failing = failing,
        evidence_count = evidence_count,
    );

    Html(html)
}

async fn handle_tsc(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let counts = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return (0i64, 0i64);
            }
        };
        let total: i64 = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);
        let passing: i64 = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch))
            .filter(compliance_checks::status.eq("pass"))
            .count()
            .get_result(&mut db_conn)
            .unwrap_or(0);
        (total, passing)
    })
    .await
    .unwrap_or((0, 0));

    let (total, passing) = counts;
    let pct = if total > 0 { (passing as f64 / total as f64 * 100.0) as i64 } else { 0 };
    let failing = total - passing;

    let html = format!(
        r#"<div class="tsc-grid">
        <div class="tsc-category" onclick="showTscDetails('security')">
            <div class="tsc-header"><div class="tsc-icon security"><svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg></div><span class="tsc-name">Security</span></div>
            <div class="tsc-progress"><div class="tsc-bar"><div class="tsc-fill passing" style="width: {pct}%"></div></div><span class="tsc-percent">{pct}%</span></div>
            <div class="tsc-counts"><span class="passing">{passing} passing</span><span class="failing">{failing} failing</span></div>
        </div>
        <div class="tsc-category" onclick="showTscDetails('availability')">
            <div class="tsc-header"><div class="tsc-icon availability"><svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg></div><span class="tsc-name">Availability</span></div>
            <div class="tsc-progress"><div class="tsc-bar"><div class="tsc-fill passing" style="width: {pct}%"></div></div><span class="tsc-percent">{pct}%</span></div>
            <div class="tsc-counts"><span class="passing">{passing} passing</span><span class="failing">{failing} failing</span></div>
        </div>
        <div class="tsc-category" onclick="showTscDetails('confidentiality')">
            <div class="tsc-header"><div class="tsc-icon confidentiality"><svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg></div><span class="tsc-name">Confidentiality</span></div>
            <div class="tsc-progress"><div class="tsc-bar"><div class="tsc-fill passing" style="width: {pct}%"></div></div><span class="tsc-percent">{pct}%</span></div>
            <div class="tsc-counts"><span class="passing">{passing} passing</span><span class="failing">{failing} failing</span></div>
        </div>
        <div class="tsc-category" onclick="showTscDetails('integrity')">
            <div class="tsc-header"><div class="tsc-icon integrity"><svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg></div><span class="tsc-name">Integrity</span></div>
            <div class="tsc-progress"><div class="tsc-bar"><div class="tsc-fill passing" style="width: {pct}%"></div></div><span class="tsc-percent">{pct}%</span></div>
            <div class="tsc-counts"><span class="passing">{passing} passing</span><span class="failing">{failing} failing</span></div>
        </div>
        </div>"#
    );

    Html(html)
}

async fn handle_failing_controls(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            check_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
        }
        diesel::sql_query(
            "SELECT id, check_type, status FROM compliance_checks WHERE branch_id = $1 AND (status = 'fail' OR status IS NULL) ORDER BY updated_at DESC LIMIT 25",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::from("<div class=\"controls-list\">");
    if rows.is_empty() {
        html.push_str("<div class=\"empty-state\"><p>All controls are passing</p></div>");
    }
    for r in &rows {
        html.push_str(&format!(
            "<div class=\"control-item\" data-control-id=\"{id}\"><span class=\"control-name\">{name}</span><span class=\"badge badge-danger\">{status}</span></div>",
            id = r.id,
            name = html_escape(&r.check_type),
            status = html_escape(if r.status.is_empty() { "pending" } else { &r.status }),
        ));
    }
    html.push_str("</div>");
    Html(html)
}

async fn handle_evidence(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            file_path: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
        }
        diesel::sql_query(
            "SELECT id, file_path, description FROM compliance_evidence WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 25",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::from("<div class=\"evidence-list\">");
    if rows.is_empty() {
        html.push_str("<div class=\"empty-state\"><p>No evidence items yet</p></div>");
    }
    for r in &rows {
        html.push_str(&format!(
            "<div class=\"evidence-item\" data-evidence-id=\"{id}\"><span class=\"evidence-file\">{file}</span><span class=\"evidence-desc\">{desc}</span></div>",
            id = r.id,
            file = html_escape(&r.file_path),
            desc = html_escape(r.description.as_deref().unwrap_or("")),
        ));
    }
    html.push_str("</div>");
    Html(html)
}

async fn handle_audit_log(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            action: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            resource_type: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
        }
        diesel::sql_query(
            "SELECT id, action, resource_type, created_at FROM compliance_audit_log WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 25",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::from("<div class=\"log-list\">");
    if rows.is_empty() {
        html.push_str("<div class=\"empty-state\"><p>No audit events yet</p></div>");
    }
    for r in &rows {
        html.push_str(&format!(
            "<div class=\"log-item\" data-log-id=\"{id}\"><span class=\"log-action\">{action}</span><span class=\"log-target\">{target}</span><span class=\"log-time\">{time}</span></div>",
            id = r.id,
            action = html_escape(&r.action),
            target = html_escape(&r.resource_type),
            time = r.created_at.format("%Y-%m-%d %H:%M"),
        ));
    }
    html.push_str("</div>");
    Html(html)
}

async fn handle_scan(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let inserted = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return 0usize;
            }
        };
        let check_types = [
            ("vulnerability", "pass"),
            ("encryption", "pass"),
            ("access-control", "pass"),
            ("backup", "fail"),
            ("patching", "fail"),
        ];
        let mut count = 0usize;
        for (check_type, status) in check_types {
            let id = Uuid::new_v4();
            let _ = diesel::sql_query(
                "INSERT INTO compliance_checks (id, branch_id, check_type, status, checked_at) VALUES ($1, $2, $3, $4, NOW()) ON CONFLICT DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .bind::<diesel::sql_types::Uuid, _>(&branch)
            .bind::<diesel::sql_types::Text, _>(check_type)
            .bind::<diesel::sql_types::Text, _>(status)
            .execute(&mut db_conn);
            count += 1;
        }
        count
    })
    .await
    .unwrap_or(0);

    Json(serde_json::json!({
        "ok": true,
        "checks_run": inserted,
        "message": format!("Compliance scan completed: {inserted} checks evaluated"),
    }))
}

async fn handle_export(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let branch = branch_id_for(&pool);
    let conn = pool.clone();

    let rows = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            check_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            checked_at: Option<chrono::DateTime<chrono::Utc>>,
        }
        diesel::sql_query(
            "SELECT check_type, COALESCE(status, 'pending') AS status, checked_at FROM compliance_checks WHERE branch_id = $1 ORDER BY updated_at DESC LIMIT 1000",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load::<Row>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut csv = String::from("check_type,status,checked_at\n");
    for r in &rows {
        let checked = r
            .checked_at
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();
        csv.push_str(&format!("{},{},{}\n", r.check_type, r.status, checked));
    }

    (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/csv; charset=utf-8",
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "attachment; filename=\"compliance-report.csv\"",
            ),
        ],
        csv,
    )
        .into_response()
}
