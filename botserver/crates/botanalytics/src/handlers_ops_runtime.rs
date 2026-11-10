//! Endpoint, queue and cost panels of the Analytics operational view (#2116).
//!
//! Every number comes from a table the server already writes:
//! `ops_request_metrics` (HTTP middleware), `agent_runs` / `subagent_runs` /
//! `handoff_queue` / `email_offline_queue` (queues) and `llm_metrics` /
//! `llm_budget` (tokens, cost, quota). A source that is unavailable or empty
//! renders an explicit "no data" row.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use diesel::RunQueryDsl;

use crate::analytics_types::{format_number, html_escape};
use crate::handlers_ops_traffic::{window_minutes, OpsWindowQuery};
use crate::ops_metrics::{percentile_from_histogram, BUCKET_COUNT};
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

// ---------------------------------------------------------------------------
// Queue: depth, wait p95, retries and dead letters
// ---------------------------------------------------------------------------

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct AgentRunQueueRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    queued: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    running: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    failed: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    wait_p95_seconds: Option<f64>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct HandoffQueueRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    queued: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    wait_p95_seconds: Option<f64>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    queued: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SubagentQueueRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    queued: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    failed: i64,
}

struct QueueRow {
    name: &'static str,
    queued: i64,
    running: Option<i64>,
    failed: Option<i64>,
    wait_p95: Option<f64>,
}

fn load_queue(pool: &DbPool, minutes: i32) -> Vec<QueueRow> {
    let Ok(mut conn) = pool.get() else {
        log::error!("ops queue panel: no DB connection");
        return Vec::new();
    };
    let mut rows = Vec::new();

    match diesel::sql_query(
        "SELECT
            COUNT(*) FILTER (WHERE status = 'queued')::bigint AS queued,
            COUNT(*) FILTER (WHERE status = 'running')::bigint AS running,
            COUNT(*) FILTER (WHERE status = 'failed'
                              AND created_at > NOW() - ($1::int * INTERVAL '1 minute'))::bigint AS failed,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (started_at - created_at)))
                FILTER (WHERE started_at IS NOT NULL
                          AND created_at > NOW() - ($1::int * INTERVAL '1 minute')) AS wait_p95_seconds
         FROM agent_runs",
    )
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .get_result::<AgentRunQueueRow>(&mut conn)
    {
        Ok(row) => rows.push(QueueRow {
            name: "agent runs",
            queued: row.queued,
            running: Some(row.running),
            failed: Some(row.failed),
            wait_p95: row.wait_p95_seconds,
        }),
        Err(error) => log::error!("ops queue agent_runs query failed: {error}"),
    }

    match diesel::sql_query(
        "SELECT
            COUNT(*) FILTER (WHERE status = 'queued')::bigint AS queued,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (NOW() - created_at)))
                FILTER (WHERE status = 'queued') AS wait_p95_seconds
         FROM handoff_queue",
    )
    .get_result::<HandoffQueueRow>(&mut conn)
    {
        Ok(row) => rows.push(QueueRow {
            name: "handoff",
            queued: row.queued,
            running: None,
            failed: None,
            wait_p95: row.wait_p95_seconds,
        }),
        Err(error) => log::error!("ops queue handoff_queue query failed: {error}"),
    }

    match diesel::sql_query("SELECT COUNT(*)::bigint AS queued FROM email_offline_queue")
        .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => rows.push(QueueRow {
            name: "email offline",
            queued: row.queued,
            running: None,
            failed: None,
            wait_p95: None,
        }),
        Err(error) => log::error!("ops queue email_offline_queue query failed: {error}"),
    }

    match diesel::sql_query(
        "SELECT
            COUNT(*) FILTER (WHERE status = 'pending')::bigint AS queued,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failed
         FROM subagent_runs",
    )
    .get_result::<SubagentQueueRow>(&mut conn)
    {
        Ok(row) => rows.push(QueueRow {
            name: "subagents",
            queued: row.queued,
            running: None,
            failed: Some(row.failed),
            wait_p95: None,
        }),
        Err(error) => log::error!("ops queue subagent_runs query failed: {error}"),
    }

    rows
}

fn render_queue(rows: &[QueueRow], minutes: i32) -> String {
    if rows.is_empty() {
        return format!(
            "<div class=\"ops-panel\" id=\"ops-queue\">
    <div class=\"chart-header\"><h3>Queue depth</h3><span class=\"ops-meta\">last {minutes} min</span></div>
    <p class=\"ops-empty\">No queue table answered the query. Check the database connection.</p>
</div>"
        );
    }

    let mut body = String::new();
    for row in rows {
        let wait = match row.wait_p95 {
            Some(seconds) if seconds >= 60.0 => format!("{:.1} min", seconds / 60.0),
            Some(seconds) => format!("{seconds:.0} s"),
            None => "—".to_string(),
        };
        let depth_class = if row.queued > 100 {
            "ops-page"
        } else if row.queued > 10 {
            "ops-warn"
        } else {
            "ops-ok"
        };
        body.push_str(&format!(
            "<tr>
    <td>{name}</td>
    <td class=\"{depth_class}\">{queued}</td>
    <td>{running}</td>
    <td>{failed}</td>
    <td>{wait}</td>
</tr>",
            name = html_escape(row.name),
            depth_class = depth_class,
            queued = row.queued,
            running = row.running.map(format_number).unwrap_or_else(|| "—".to_string()),
            failed = row.failed.map(format_number).unwrap_or_else(|| "—".to_string()),
            wait = wait,
        ));
    }

    format!(
        "<div class=\"ops-panel\" id=\"ops-queue\">
    <div class=\"chart-header\"><h3>Queue depth</h3><span class=\"ops-meta\">wait p95 and failures over the last {minutes} min</span></div>
    <div class=\"ops-table-wrap\">
        <table class=\"ops-table\">
            <thead><tr><th>Queue</th><th>Depth</th><th>Running</th><th>Dead letters</th><th>Wait p95</th></tr></thead>
            <tbody>{body}</tbody>
        </table>
    </div>
    <p class=\"ops-hint\">Dead letters are runs that finished in <code>failed</code> inside the window. Mitigation: <a href=\"https://github.com/generalbots/generalbots/blob/main/botbook/src/10-configuration-deployment/operations/runbook-disk-and-queue.md\" target=\"_blank\" rel=\"noopener\">stuck queue runbook</a>.</p>
</div>",
        minutes = minutes,
        body = body,
    )
}

pub async fn handle_ops_queue(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<OpsWindowQuery>,
) -> impl IntoResponse {
    let minutes = window_minutes(query.window);
    let conn = pool.clone();
    let rows = tokio::task::spawn_blocking(move || load_queue(&conn, minutes))
        .await
        .unwrap_or_else(|error| {
            log::error!("ops queue panel task failed: {error}");
            Vec::new()
        });
    Html(render_queue(&rows, minutes))
}

// ---------------------------------------------------------------------------
// Cost: tokens, spend per conversation and per tenant, quota burn
// ---------------------------------------------------------------------------

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CostTotalsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    input_tokens: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    output_tokens: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    conversations: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    failures: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    cost: f64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TenantCostRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    tenant: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    tokens: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    cost: f64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct QuotaRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot: String,
    #[diesel(sql_type = diesel::sql_types::Double)]
    daily_spend: f64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    daily_limit: f64,
}

struct CostData {
    totals: CostTotalsRow,
    tenants: Vec<TenantCostRow>,
    quotas: Vec<QuotaRow>,
}

fn load_cost(pool: &DbPool, minutes: i32) -> Option<CostData> {
    let Ok(mut conn) = pool.get() else {
        log::error!("ops cost panel: no DB connection");
        return None;
    };

    let totals = match diesel::sql_query(
        "SELECT
            COUNT(*)::bigint AS requests,
            COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens,
            COUNT(DISTINCT session_id)::bigint AS conversations,
            COUNT(*) FILTER (WHERE NOT success)::bigint AS failures,
            COALESCE(SUM(estimated_cost), 0)::float8 AS cost
         FROM llm_metrics
         WHERE timestamp > NOW() - ($1::int * INTERVAL '1 minute')",
    )
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .get_result::<CostTotalsRow>(&mut conn)
    {
        Ok(row) => row,
        Err(error) => {
            log::error!("ops cost totals failed: {error}");
            return None;
        }
    };

    let tenants = diesel::sql_query(
        "SELECT COALESCE(o.name::text, 'unassigned') AS tenant,
                COALESCE(SUM(m.total_tokens), 0)::bigint AS tokens,
                COALESCE(SUM(m.estimated_cost), 0)::float8 AS cost
         FROM llm_metrics m
         LEFT JOIN bots b ON b.id = m.bot_id
         LEFT JOIN organizations o ON o.org_id = b.org_id
         WHERE m.timestamp > NOW() - ($1::int * INTERVAL '1 minute')
         GROUP BY 1
         ORDER BY cost DESC
         LIMIT 5",
    )
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .load::<TenantCostRow>(&mut conn)
    .unwrap_or_else(|error| {
        log::error!("ops cost tenants failed: {error}");
        Vec::new()
    });

    let quotas = diesel::sql_query(
        "SELECT b.name::text AS bot,
                lb.daily_spend::float8 AS daily_spend,
                lb.daily_limit::float8 AS daily_limit
         FROM llm_budget lb
         JOIN bots b ON b.id = lb.bot_id
         WHERE lb.daily_limit > 0
         ORDER BY lb.daily_spend / NULLIF(lb.daily_limit, 0) DESC
         LIMIT 5",
    )
    .load::<QuotaRow>(&mut conn)
    .unwrap_or_else(|error| {
        log::error!("ops cost quota query failed: {error}");
        Vec::new()
    });

    Some(CostData {
        totals,
        tenants,
        quotas,
    })
}

fn render_cost(data: &CostData, minutes: i32) -> String {
    let totals = &data.totals;
    let per_conversation = if totals.conversations == 0 {
        None
    } else {
        Some(totals.cost / totals.conversations as f64)
    };
    let per_conversation_label = match per_conversation {
        Some(value) => format!("${value:.4}"),
        None => "—".to_string(),
    };

    let mut tenant_rows = String::new();
    for tenant in &data.tenants {
        tenant_rows.push_str(&format!(
            "<tr><td>{name}</td><td>{tokens}</td><td>${cost:.4}</td></tr>",
            name = html_escape(&tenant.tenant),
            tokens = format_number(tenant.tokens),
            cost = tenant.cost,
        ));
    }
    if tenant_rows.is_empty() {
        tenant_rows.push_str("<tr><td colspan=\"3\" class=\"ops-empty-cell\">No tenant spend recorded in this window.</td></tr>");
    }

    let mut quota_rows = String::new();
    for quota in &data.quotas {
        let burn = if quota.daily_limit > 0.0 {
            quota.daily_spend / quota.daily_limit
        } else {
            0.0
        };
        let class = if burn >= 1.0 {
            "ops-page"
        } else if burn >= 0.8 {
            "ops-warn"
        } else {
            "ops-ok"
        };
        quota_rows.push_str(&format!(
            "<tr><td>{bot}</td><td>${spend:.2} / ${limit:.2}</td><td class=\"{class}\">{pct:.0}%</td></tr>",
            bot = html_escape(&quota.bot),
            spend = quota.daily_spend,
            limit = quota.daily_limit,
            class = class,
            pct = burn * 100.0,
        ));
    }
    if quota_rows.is_empty() {
        quota_rows.push_str("<tr><td colspan=\"3\" class=\"ops-empty-cell\">No LLM budget rows configured.</td></tr>");
    }

    format!(
        "<div class=\"ops-panel\" id=\"ops-cost\">
    <div class=\"chart-header\"><h3>LLM cost &amp; tokens</h3><span class=\"ops-meta\">last {minutes} min · source: llm_metrics</span></div>
    <div class=\"ops-stats\">
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{requests}</span><span class=\"ops-stat-label\">LLM calls</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{input}</span><span class=\"ops-stat-label\">Input tokens</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{output}</span><span class=\"ops-stat-label\">Output tokens</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">${cost:.4}</span><span class=\"ops-stat-label\">Cost</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{per_conversation}</span><span class=\"ops-stat-label\">Cost / conversation</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{failures}</span><span class=\"ops-stat-label\">Failed calls</span></div>
    </div>
    <div class=\"ops-columns\">
        <div>
            <h4 class=\"ops-subtitle\">Cost per tenant</h4>
            <table class=\"ops-table\"><thead><tr><th>Tenant</th><th>Tokens</th><th>Cost</th></tr></thead><tbody>{tenant_rows}</tbody></table>
        </div>
        <div>
            <h4 class=\"ops-subtitle\">Daily quota burn</h4>
            <table class=\"ops-table\"><thead><tr><th>Bot</th><th>Spend</th><th>Burn</th></tr></thead><tbody>{quota_rows}</tbody></table>
        </div>
    </div>
</div>",
        minutes = minutes,
        requests = format_number(totals.requests),
        input = format_number(totals.input_tokens),
        output = format_number(totals.output_tokens),
        cost = totals.cost,
        per_conversation = per_conversation_label,
        failures = format_number(totals.failures),
        tenant_rows = tenant_rows,
        quota_rows = quota_rows,
    )
}

pub async fn handle_ops_cost(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<OpsWindowQuery>,
) -> impl IntoResponse {
    let minutes = window_minutes(query.window);
    let conn = pool.clone();
    let loaded = tokio::task::spawn_blocking(move || load_cost(&conn, minutes))
        .await
        .unwrap_or_else(|error| {
            log::error!("ops cost panel task failed: {error}");
            None
        });

    match loaded {
        Some(data) => Html(render_cost(&data, minutes)),
        None => Html(
            "<div class=\"ops-panel\" id=\"ops-cost\"><p class=\"ops-empty\">Cost data unavailable: the database pool is not reachable.</p></div>"
                .to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "handlers_ops_runtime_tests.rs"]
mod tests;
