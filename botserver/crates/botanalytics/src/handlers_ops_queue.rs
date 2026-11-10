//! Queue-depth panel of the Analytics operational view (#2116): depth,
//! running, dead letters and wait p95 for every queue the server drains.
//!
//! Sources: `agent_runs`, `subagent_runs`, `handoff_queue` and
//! `email_offline_queue`. A queue whose table is missing renders no row, and
//! the panel says so instead of showing a zero depth.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use diesel::RunQueryDsl;

use crate::analytics_types::{format_number, html_escape};
use crate::handlers_ops_traffic::{window_minutes, OpsWindowQuery};
use crate::DbPool;

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
            percentile_cont(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (started_at - created_at))::float8)
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
            percentile_cont(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (NOW() - created_at))::float8)
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
            running = row
                .running
                .map(format_number)
                .unwrap_or_else(|| "—".to_string()),
            failed = row
                .failed
                .map(format_number)
                .unwrap_or_else(|| "—".to_string()),
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

#[cfg(test)]
#[path = "handlers_ops_queue_tests.rs"]
mod tests;
