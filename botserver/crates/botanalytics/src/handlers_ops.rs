//! SLO burn panel of the Analytics operational view (#2116).
//!
//! Reads `ops_request_metrics` (written by the server middleware) and
//! `llm_metrics` (written by the LLM observability layer), then applies the
//! targets and burn-rate thresholds from `ops/slo.yaml` through
//! [`crate::ops_slo`]. Every cell is either a measured number or an explicit
//! "no data" — nothing is synthesised.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use diesel::RunQueryDsl;

use crate::analytics_types::html_escape;
use crate::ops_metrics::{percentile_from_histogram, ratio_above_histogram, BUCKET_COUNT};
use crate::ops_slo::{
    burn_rate, runbook_base_url, severity, BurnSeverity, MetricKind, Objective, SloConfig,
    BURN_WINDOWS,
};
use crate::DbPool;

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowStats {
    pub requests: i64,
    pub errors: i64,
    pub ws_requests: i64,
    pub ws_failures: i64,
    pub buckets: [i64; BUCKET_COUNT],
    pub llm_requests: i64,
    pub llm_ttft_samples: i64,
    pub llm_ttft_slow: i64,
    pub llm_answer_slow: i64,
    pub llm_ttft_p95: Option<f64>,
    pub llm_answer_p95: Option<f64>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RequestWindowRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    errors: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ws_requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ws_failures: i64,
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

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LlmWindowRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ttft_samples: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ttft_slow: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    answer_slow: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    ttft_p95: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    answer_p95: Option<f64>,
}

const REQUEST_WINDOW_SQL: &str = "SELECT
        COALESCE(SUM(request_count), 0)::bigint AS requests,
        COALESCE(SUM(CASE WHEN status_class = '5xx' THEN request_count ELSE 0 END), 0)::bigint AS errors,
        COALESCE(SUM(CASE WHEN endpoint_class = 'websocket' THEN request_count ELSE 0 END), 0)::bigint AS ws_requests,
        COALESCE(SUM(CASE WHEN endpoint_class = 'websocket' AND status_class IN ('4xx','5xx')
                          THEN request_count ELSE 0 END), 0)::bigint AS ws_failures,
        COALESCE(SUM(b_le_50), 0)::bigint AS b1,
        COALESCE(SUM(b_le_100), 0)::bigint AS b2,
        COALESCE(SUM(b_le_250), 0)::bigint AS b3,
        COALESCE(SUM(b_le_500), 0)::bigint AS b4,
        COALESCE(SUM(b_le_1000), 0)::bigint AS b5,
        COALESCE(SUM(b_le_2500), 0)::bigint AS b6,
        COALESCE(SUM(b_le_5000), 0)::bigint AS b7,
        COALESCE(SUM(b_gt_5000), 0)::bigint AS b8
    FROM ops_request_metrics
    WHERE endpoint_class <> 'static'
      AND bucket > NOW() - ($1::int * INTERVAL '1 minute')";

const LLM_WINDOW_SQL: &str = "SELECT
        COUNT(*)::bigint AS requests,
        COUNT(*) FILTER (WHERE ttft_ms IS NOT NULL)::bigint AS ttft_samples,
        COUNT(*) FILTER (WHERE ttft_ms IS NOT NULL AND ttft_ms > $2)::bigint AS ttft_slow,
        COUNT(*) FILTER (WHERE latency_ms > $3)::bigint AS answer_slow,
        percentile_cont(0.95) WITHIN GROUP (ORDER BY ttft_ms)
            FILTER (WHERE ttft_ms IS NOT NULL) AS ttft_p95,
        percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms) AS answer_p95
    FROM llm_metrics
    WHERE timestamp > NOW() - ($1::int * INTERVAL '1 minute')";

/// Collects one window. Returns zeroed stats when a query fails, so the panel
/// shows "no data" for that window instead of failing the whole fragment.
fn collect_window(
    conn: &mut diesel::PgConnection,
    minutes: i32,
    ttft_threshold: i64,
    answer_threshold: i64,
) -> WindowStats {
    let mut stats = WindowStats::default();

    match diesel::sql_query(REQUEST_WINDOW_SQL)
        .bind::<diesel::sql_types::Integer, _>(minutes)
        .get_result::<RequestWindowRow>(conn)
    {
        Ok(row) => {
            stats.requests = row.requests;
            stats.errors = row.errors;
            stats.ws_requests = row.ws_requests;
            stats.ws_failures = row.ws_failures;
            stats.buckets = [
                row.b1, row.b2, row.b3, row.b4, row.b5, row.b6, row.b7, row.b8,
            ];
        }
        Err(error) => log::error!("ops SLO request window ({minutes} min) failed: {error}"),
    }

    match diesel::sql_query(LLM_WINDOW_SQL)
        .bind::<diesel::sql_types::Integer, _>(minutes)
        .bind::<diesel::sql_types::BigInt, _>(ttft_threshold)
        .bind::<diesel::sql_types::BigInt, _>(answer_threshold)
        .get_result::<LlmWindowRow>(conn)
    {
        Ok(row) => {
            stats.llm_requests = row.requests;
            stats.llm_ttft_samples = row.ttft_samples;
            stats.llm_ttft_slow = row.ttft_slow;
            stats.llm_answer_slow = row.answer_slow;
            stats.llm_ttft_p95 = row.ttft_p95;
            stats.llm_answer_p95 = row.answer_p95;
        }
        Err(error) => log::error!("ops SLO LLM window ({minutes} min) failed: {error}"),
    }

    stats
}

fn threshold_of(objectives: &[Objective], metric: MetricKind, fallback: i64) -> i64 {
    objectives
        .iter()
        .find(|objective| objective.metric == metric)
        .and_then(|objective| objective.threshold_ms)
        .map(|value| value as i64)
        .unwrap_or(fallback)
}

/// Observed bad ratio for one objective in one window. `None` means the window
/// has no observations for that signal.
pub fn bad_ratio(objective: &Objective, stats: &WindowStats) -> Option<f64> {
    match objective.metric {
        MetricKind::LatencyHistogram => {
            let threshold = objective.threshold_ms.unwrap_or(500.0);
            ratio_above_histogram(&stats.buckets, threshold).map(|ratio| 1.0 - ratio)
        }
        MetricKind::ErrorRate => {
            if stats.requests == 0 {
                None
            } else {
                Some(stats.errors as f64 / stats.requests as f64)
            }
        }
        MetricKind::WsHandshake => {
            if stats.ws_requests == 0 {
                None
            } else {
                Some(stats.ws_failures as f64 / stats.ws_requests as f64)
            }
        }
        MetricKind::LlmFirstToken => {
            if stats.llm_ttft_samples == 0 {
                None
            } else {
                Some(stats.llm_ttft_slow as f64 / stats.llm_ttft_samples as f64)
            }
        }
        MetricKind::LlmAnswer => {
            if stats.llm_requests == 0 {
                None
            } else {
                Some(stats.llm_answer_slow as f64 / stats.llm_requests as f64)
            }
        }
    }
}

/// Human value of the signal (what was measured), used next to the burn rate.
fn observed_label(objective: &Objective, stats: &WindowStats) -> String {
    match objective.metric {
        MetricKind::LatencyHistogram => {
            match percentile_from_histogram(&stats.buckets, 0.95) {
                Some(ms) => format!("p95 {ms:.0} ms"),
                None => "no data".to_string(),
            }
        }
        MetricKind::ErrorRate => {
            if stats.requests == 0 {
                "no data".to_string()
            } else {
                format!(
                    "{:.2}% 5xx",
                    stats.errors as f64 / stats.requests as f64 * 100.0
                )
            }
        }
        MetricKind::WsHandshake => {
            if stats.ws_requests == 0 {
                "no data".to_string()
            } else {
                format!(
                    "{:.2}% failed",
                    stats.ws_failures as f64 / stats.ws_requests as f64 * 100.0
                )
            }
        }
        MetricKind::LlmFirstToken => match stats.llm_ttft_p95 {
            Some(ms) => format!("p95 {ms:.0} ms"),
            None => "no data".to_string(),
        },
        MetricKind::LlmAnswer => match stats.llm_answer_p95 {
            Some(ms) => format!("p95 {ms:.0} ms"),
            None => "no data".to_string(),
        },
    }
}

fn render_cell(burn: Option<f64>, alert: crate::ops_slo::AlertPolicy) -> String {
    let severity = severity(burn, alert);
    let text = match burn {
        Some(value) => format!("{value:.1}x"),
        None => "—".to_string(),
    };
    format!(
        "<td class=\"ops-burn {}\" title=\"{}\">{}</td>",
        severity.css_class(),
        severity.label(),
        text
    )
}

fn render(config: &SloConfig, windows: &[(String, WindowStats)], budget: &WindowStats) -> String {
    let base = runbook_base_url();
    let first_window = windows.first().map(|(_, stats)| *stats).unwrap_or_default();
    let mut rows = String::new();

    for objective in &config.objectives {
        let mut worst = BurnSeverity::Ok;
        let mut cells = String::new();
        for (_, stats) in windows {
            let burn = burn_rate(bad_ratio(objective, stats), objective);
            let severity = severity(burn, config.alert);
            if severity_rank(severity) > severity_rank(worst) {
                worst = severity;
            }
            cells.push_str(&render_cell(burn, config.alert));
        }
        let budget_used = match bad_ratio(objective, budget) {
            Some(bad) => format!("{:.0}%", bad / objective.budget_ratio() * 100.0),
            None => "—".to_string(),
        };
        let runbook = if objective.runbook.is_empty() {
            String::new()
        } else {
            let url = format!("{base}/{}", objective.runbook);
            format!(
                "<a class=\"ops-runbook\" href=\"{}\" target=\"_blank\" rel=\"noopener\">runbook</a>",
                html_escape(&url)
            )
        };
        let panel = if objective.panel.is_empty() {
            String::new()
        } else {
            format!(
                "<a class=\"ops-panel-link\" href=\"#{}\" onclick=\"showOpsPanel('{}');return false;\">panel</a>",
                html_escape(&objective.panel),
                html_escape(&objective.panel)
            )
        };
        rows.push_str(&format!(
            "<tr>
    <td class=\"ops-objective\"><strong>{id}</strong><span class=\"ops-objective-kind\">{kind}</span></td>
    <td>{target:.1}%</td>
    <td class=\"ops-observed\">{observed}</td>
    {cells}
    <td>{budget_used}</td>
    <td><span class=\"ops-status {cls}\">{label}</span></td>
    <td class=\"ops-links\">{runbook}{panel}</td>
</tr>",
            id = html_escape(&objective.id),
            kind = objective.metric.label(),
            target = objective.target * 100.0,
            observed = observed_label(objective, &first_window),
            cells = cells,
            budget_used = budget_used,
            cls = worst.css_class(),
            label = worst.label(),
            runbook = runbook,
            panel = panel,
        ));
    }

    let headers: String = windows
        .iter()
        .map(|(label, _)| format!("<th>{label} burn</th>"))
        .collect();

    format!(
        "<div class=\"ops-panel\" id=\"ops-slo\">
    <div class=\"chart-header\">
        <h3>SLO burn &amp; error budget</h3>
        <span class=\"ops-meta\">source: {source} · owner: {owner} · window: {days}d · alert at {fast}x / {medium}x / {slow}x burn</span>
    </div>
    <div class=\"ops-table-wrap\">
        <table class=\"ops-table\">
            <thead><tr><th>Objective</th><th>Target</th><th>Observed</th>{headers}<th>Budget used</th><th>Status</th><th>Links</th></tr></thead>
            <tbody>{rows}</tbody>
        </table>
    </div>
    <p class=\"ops-hint\">Burn rate = observed bad ratio ÷ error budget. Budget used is measured over the full SLO window. API latency comes from the request histogram (thresholds on bucket boundaries); LLM percentiles come from <code>llm_metrics</code>.</p>
</div>",
        source = html_escape(&config.source),
        owner = html_escape(&config.owner),
        days = config.window_days,
        fast = config.alert.fast_burn,
        medium = config.alert.medium_burn,
        slow = config.alert.slow_burn,
        headers = headers,
        rows = rows,
    )
}

fn severity_rank(severity: BurnSeverity) -> u8 {
    match severity {
        BurnSeverity::Ok => 0,
        BurnSeverity::NoData => 1,
        BurnSeverity::Slow => 2,
        BurnSeverity::Medium => 3,
        BurnSeverity::Fast => 4,
    }
}

pub async fn handle_ops_slo(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let config = SloConfig::load();
    let conn = pool.clone();
    let config_for_task = config.clone();

    let windows = tokio::task::spawn_blocking(move || {
        let Ok(mut db_conn) = conn.get() else {
            log::error!("ops SLO panel: no DB connection");
            return (Vec::new(), WindowStats::default());
        };
        let ttft = threshold_of(&config_for_task.objectives, MetricKind::LlmFirstToken, 2_000);
        let answer = threshold_of(&config_for_task.objectives, MetricKind::LlmAnswer, 15_000);
        let mut collected = Vec::new();
        for (label, minutes) in BURN_WINDOWS {
            collected.push((
                label.to_string(),
                collect_window(&mut db_conn, minutes, ttft, answer),
            ));
        }
        let budget_minutes = (config_for_task.window_days as i32).saturating_mul(1_440);
        let budget = collect_window(&mut db_conn, budget_minutes.max(60), ttft, answer);
        (collected, budget)
    })
    .await
    .unwrap_or_else(|error| {
        log::error!("ops SLO panel task failed: {error}");
        (Vec::new(), WindowStats::default())
    });

    let (windows, budget) = windows;
    if windows.is_empty() {
        return Html(
            "<div class=\"ops-panel\" id=\"ops-slo\"><p class=\"ops-empty\">SLO data unavailable: the database pool is not reachable.</p></div>"
                .to_string(),
        );
    }
    Html(render(&config, &windows, &budget))
}

#[cfg(test)]
#[path = "handlers_ops_tests.rs"]
mod tests;
