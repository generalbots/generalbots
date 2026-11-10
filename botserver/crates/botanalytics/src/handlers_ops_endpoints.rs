//! Endpoint-class panel of the Analytics operational view (#2116): request
//! rate, 5xx rate, p95 and traffic share per endpoint class, read from the
//! counters the HTTP middleware writes into `ops_request_metrics`.
//!
//! Split from the queue and cost panels to keep each file small; the panels
//! share the window selector defined in `handlers_ops_traffic`.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use diesel::RunQueryDsl;

use crate::ops_metrics::{percentile_from_histogram, BUCKET_COUNT};

use crate::analytics_types::{format_number, html_escape};
use crate::handlers_ops_traffic::{window_minutes, OpsWindowQuery};
use crate::DbPool;

// ---------------------------------------------------------------------------
// Endpoints: request rate, error rate and latency per endpoint class
// ---------------------------------------------------------------------------

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct EndpointRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    endpoint_class: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    errors: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b1: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b2: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b3: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b4: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b5: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b6: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b7: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    b8: i64,
}

const ENDPOINT_SQL: &str = "SELECT endpoint_class,
        COALESCE(SUM(request_count), 0)::bigint AS requests,
        COALESCE(SUM(CASE WHEN status_class = '5xx' THEN request_count ELSE 0 END), 0)::bigint AS errors,
        COALESCE(SUM(b_le_50), 0)::bigint AS b1,
        COALESCE(SUM(b_le_100), 0)::bigint AS b2,
        COALESCE(SUM(b_le_250), 0)::bigint AS b3,
        COALESCE(SUM(b_le_500), 0)::bigint AS b4,
        COALESCE(SUM(b_le_1000), 0)::bigint AS b5,
        COALESCE(SUM(b_le_2500), 0)::bigint AS b6,
        COALESCE(SUM(b_le_5000), 0)::bigint AS b7,
        COALESCE(SUM(b_gt_5000), 0)::bigint AS b8
    FROM ops_request_metrics
    WHERE bucket > NOW() - ($1::int * INTERVAL '1 minute')
    GROUP BY endpoint_class
    ORDER BY requests DESC";

fn render_endpoints(rows: &[EndpointRow], minutes: i32) -> String {
    if rows.is_empty() {
        return format!(
            "<div class=\"ops-panel\" id=\"ops-endpoints\">
    <div class=\"chart-header\"><h3>Requests by endpoint class</h3><span class=\"ops-meta\">last {minutes} min</span></div>
    <p class=\"ops-empty\">No request counters recorded in this window. Counters start after the first request the server answers with the ops middleware enabled.</p>
</div>"
        );
    }

    let total: i64 = rows.iter().map(|row| row.requests).sum();
    let mut body = String::new();
    for row in rows {
        let buckets: [i64; BUCKET_COUNT] = [
            row.b1, row.b2, row.b3, row.b4, row.b5, row.b6, row.b7, row.b8,
        ];
        let p95 = percentile_from_histogram(&buckets, 0.95)
            .map(|ms| format!("{ms:.0} ms"))
            .unwrap_or_else(|| "—".to_string());
        let error_pct = if row.requests == 0 {
            0.0
        } else {
            row.errors as f64 / row.requests as f64 * 100.0
        };
        let per_min = row.requests as f64 / minutes.max(1) as f64;
        let share = if total == 0 {
            0.0
        } else {
            row.requests as f64 / total as f64 * 100.0
        };
        let severity = if error_pct >= 5.0 {
            "ops-page"
        } else if error_pct > 0.0 {
            "ops-warn"
        } else {
            "ops-ok"
        };
        body.push_str(&format!(
            "<tr>
    <td class=\"ops-mono\">{class}</td>
    <td>{requests}</td>
    <td>{per_min:.1}/min</td>
    <td class=\"{severity}\">{error_pct:.2}%</td>
    <td>{p95}</td>
    <td><div class=\"ops-share\"><div class=\"ops-share-fill\" style=\"width:{share:.0}%\"></div></div></td>
</tr>",
            class = html_escape(&row.endpoint_class),
            requests = format_number(row.requests),
            per_min = per_min,
            severity = severity,
            error_pct = error_pct,
            p95 = p95,
            share = share,
        ));
    }

    format!(
        "<div class=\"ops-panel\" id=\"ops-endpoints\">
    <div class=\"chart-header\"><h3>Requests by endpoint class</h3><span class=\"ops-meta\">last {minutes} min · {total} requests</span></div>
    <div class=\"ops-table-wrap\">
        <table class=\"ops-table\">
            <thead><tr><th>Class</th><th>Requests</th><th>Rate</th><th>5xx</th><th>p95</th><th>Share</th></tr></thead>
            <tbody>{body}</tbody>
        </table>
    </div>
    <p class=\"ops-hint\">Classes are assigned by <code>botanalytics::ops_metrics::classify_endpoint</code>; p95 comes from the fixed latency histogram (bucket upper bound).</p>
</div>",
        minutes = minutes,
        total = format_number(total),
        body = body,
    )
}

pub async fn handle_ops_endpoints(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<OpsWindowQuery>,
) -> impl IntoResponse {
    let minutes = window_minutes(query.window);
    let conn = pool.clone();
    let rows = tokio::task::spawn_blocking(move || {
        let Ok(mut db_conn) = conn.get() else {
            log::error!("ops endpoints panel: no DB connection");
            return Vec::new();
        };
        diesel::sql_query(ENDPOINT_SQL)
            .bind::<diesel::sql_types::Integer, _>(minutes)
            .load::<EndpointRow>(&mut db_conn)
            .unwrap_or_else(|error| {
                log::error!("ops endpoints query failed: {error}");
                Vec::new()
            })
    })
    .await
    .unwrap_or_default();

    Html(render_endpoints(&rows, minutes))
}

#[cfg(test)]
#[path = "handlers_ops_endpoints_tests.rs"]
mod tests;
