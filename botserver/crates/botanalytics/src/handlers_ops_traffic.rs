//! Traffic panel of the Analytics operational view (#2116): turns per bucket,
//! sessions, active bots/tenants, WebSocket upgrades and deploy markers
//! overlaid on the same timeline.
//!
//! Deploy markers come from `ops_deploy_markers`, written at boot by
//! `botanalytics::ops_metrics::record_deploy_marker`. When the window has no
//! marker the panel says so instead of drawing a decorative line.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::analytics_types::{format_number, html_escape};
use crate::ops_metrics::build_version;
use crate::DbPool;

#[derive(Debug, Deserialize)]
pub struct OpsWindowQuery {
    pub window: Option<i32>,
}

/// Clamps the requested window to something the chart can draw.
pub fn window_minutes(requested: Option<i32>) -> i32 {
    requested.unwrap_or(120).clamp(5, 43_200)
}

fn granularity(minutes: i32) -> &'static str {
    if minutes <= 180 {
        "minute"
    } else if minutes <= 2_880 {
        "hour"
    } else {
        "day"
    }
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SeriesRow {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    ts: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    turns: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TotalsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    turns: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    sessions: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ws_upgrades: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct FleetRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    bots: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    tenants: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DeployRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    version: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    commit_hash: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    build_date: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    deployed_at: DateTime<Utc>,
}

const SERIES_SQL: &str = "SELECT date_trunc($2::text, created_at) AS ts, COUNT(*)::bigint AS turns
    FROM message_history
    WHERE role = 1 AND created_at > NOW() - ($1::int * INTERVAL '1 minute')
    GROUP BY 1 ORDER BY 1";

const TOTALS_SQL: &str = "SELECT
        COUNT(*)::bigint AS turns,
        COUNT(DISTINCT session_id)::bigint AS sessions,
        (SELECT COALESCE(SUM(request_count), 0)::bigint
           FROM ops_request_metrics
          WHERE endpoint_class = 'websocket'
            AND status_class = '1xx'
            AND bucket > NOW() - ($1::int * INTERVAL '1 minute')) AS ws_upgrades
    FROM message_history
    WHERE role = 1 AND created_at > NOW() - ($1::int * INTERVAL '1 minute')";

const FLEET_SQL: &str = "SELECT
        (SELECT COUNT(*) FROM bots WHERE is_active)::bigint AS bots,
        (SELECT COUNT(DISTINCT org_id) FROM bots WHERE org_id IS NOT NULL)::bigint AS tenants";

const DEPLOY_SQL: &str = "SELECT version, commit_hash, build_date, deployed_at
    FROM ops_deploy_markers
    WHERE deployed_at > NOW() - ($1::int * INTERVAL '1 minute')
    ORDER BY deployed_at ASC";

struct TrafficData {
    minutes: i32,
    series: Vec<(DateTime<Utc>, i64)>,
    totals: (i64, i64, i64),
    fleet: (i64, i64),
    deploys: Vec<DeployRow>,
}

fn load(pool: &DbPool, minutes: i32) -> Option<TrafficData> {
    let Ok(mut conn) = pool.get() else {
        log::error!("ops traffic panel: no DB connection");
        return None;
    };

    let series = diesel::sql_query(SERIES_SQL)
        .bind::<diesel::sql_types::Integer, _>(minutes)
        .bind::<diesel::sql_types::Text, _>(granularity(minutes))
        .load::<SeriesRow>(&mut conn)
        .map(|rows| rows.into_iter().map(|row| (row.ts, row.turns)).collect())
        .unwrap_or_else(|error| {
            log::error!("ops traffic series failed: {error}");
            Vec::new()
        });

    let totals = diesel::sql_query(TOTALS_SQL)
        .bind::<diesel::sql_types::Integer, _>(minutes)
        .get_result::<TotalsRow>(&mut conn)
        .map(|row| (row.turns, row.sessions, row.ws_upgrades))
        .unwrap_or_else(|error| {
            log::error!("ops traffic totals failed: {error}");
            (0, 0, 0)
        });

    let fleet = diesel::sql_query(FLEET_SQL)
        .get_result::<FleetRow>(&mut conn)
        .map(|row| (row.bots, row.tenants))
        .unwrap_or_else(|error| {
            log::error!("ops traffic fleet failed: {error}");
            (0, 0)
        });

    let deploys = diesel::sql_query(DEPLOY_SQL)
        .bind::<diesel::sql_types::Integer, _>(minutes)
        .load::<DeployRow>(&mut conn)
        .unwrap_or_else(|error| {
            log::error!("ops deploy markers failed: {error}");
            Vec::new()
        });

    Some(TrafficData {
        minutes,
        series,
        totals,
        fleet,
        deploys,
    })
}

fn chart(data: &TrafficData) -> String {
    if data.series.is_empty() {
        return "<div class=\"ops-empty\">No turns recorded in this window.</div>".to_string();
    }
    let max = data.series.iter().map(|(_, turns)| *turns).max().unwrap_or(1).max(1);
    let first = data.series.first().map(|(ts, _)| *ts);
    let last = data.series.last().map(|(ts, _)| *ts);
    let span_secs = match (first, last) {
        (Some(first), Some(last)) => (last - first).num_seconds().max(1) as f64,
        _ => 1.0,
    };
    let plot_left = 44.0;
    let plot_width = 528.0;
    let plot_bottom = 132.0;
    let plot_height = 100.0;
    let slot = plot_width / data.series.len() as f64;
    let bar_width = (slot * 0.7).max(1.0);

    let mut bars = String::new();
    for (index, (_, turns)) in data.series.iter().enumerate() {
        let height = (*turns as f64 / max as f64) * plot_height;
        let x = plot_left + index as f64 * slot;
        let y = plot_bottom - height;
        bars.push_str(&format!(
            "<rect class=\"ops-bar\" x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bar_width:.1}\" height=\"{height:.1}\" rx=\"1\" />"
        ));
    }

    let mut markers = String::new();
    if let Some(first) = first {
        for deploy in &data.deploys {
            let offset = (deploy.deployed_at - first).num_seconds() as f64 / span_secs;
            if !(0.0..=1.0).contains(&offset) {
                continue;
            }
            let x = plot_left + offset * plot_width;
            markers.push_str(&format!(
                "<line class=\"ops-deploy-line\" x1=\"{x:.1}\" y1=\"24\" x2=\"{x:.1}\" y2=\"{plot_bottom}\" />"
            ));
            markers.push_str(&format!(
                "<text class=\"ops-deploy-label\" x=\"{x:.1}\" y=\"18\" text-anchor=\"middle\">{}</text>",
                html_escape(&deploy.version)
            ));
        }
    }

    let start_label = first.map(|ts| ts.format("%H:%M").to_string()).unwrap_or_default();
    let end_label = last.map(|ts| ts.format("%H:%M").to_string()).unwrap_or_default();

    format!(
        "<svg class=\"ops-chart\" viewBox=\"0 0 600 160\" role=\"img\" aria-label=\"Turns over time with deploy markers\">
    <g class=\"ops-grid\">
        <line x1=\"44\" y1=\"32\" x2=\"572\" y2=\"32\" />
        <line x1=\"44\" y1=\"82\" x2=\"572\" y2=\"82\" />
        <line x1=\"44\" y1=\"132\" x2=\"572\" y2=\"132\" />
    </g>
    <text class=\"ops-axis\" x=\"38\" y=\"36\" text-anchor=\"end\">{max}</text>
    <text class=\"ops-axis\" x=\"38\" y=\"136\" text-anchor=\"end\">0</text>
    <text class=\"ops-axis\" x=\"44\" y=\"152\">{start}</text>
    <text class=\"ops-axis\" x=\"572\" y=\"152\" text-anchor=\"end\">{end}</text>
    {bars}
    {markers}
</svg>",
        max = format_number(max),
        start = start_label,
        end = end_label,
        bars = bars,
        markers = markers,
    )
}

fn deploy_list(data: &TrafficData) -> String {
    if data.deploys.is_empty() {
        return format!(
            "<p class=\"ops-hint\">No deploy markers in the last {} minutes. Running build: {version}.</p>",
            data.minutes,
            version = html_escape(&build_version())
        );
    }
    let mut rows = String::new();
    for deploy in &data.deploys {
        rows.push_str(&format!(
            "<li><span class=\"ops-deploy-version\">{version}</span> <span class=\"ops-mono\">{commit}</span> · built {built} · deployed {at}</li>",
            version = html_escape(&deploy.version),
            commit = html_escape(&deploy.commit_hash),
            built = html_escape(&deploy.build_date),
            at = deploy.deployed_at.format("%Y-%m-%d %H:%M UTC"),
        ));
    }
    format!("<ul class=\"ops-deploy-list\">{rows}</ul>")
}

fn render(data: &TrafficData) -> String {
    let (turns, sessions, ws_upgrades) = data.totals;
    let (bots, tenants) = data.fleet;
    format!(
        "<div class=\"ops-panel\" id=\"ops-traffic\">
    <div class=\"chart-header\">
        <h3>Traffic</h3>
        <span class=\"ops-meta\">last {minutes} min · {granularity} buckets</span>
    </div>
    <div class=\"ops-stats\">
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{turns}</span><span class=\"ops-stat-label\">Turns</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{sessions}</span><span class=\"ops-stat-label\">Sessions</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{ws}</span><span class=\"ops-stat-label\">WS upgrades</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{bots}</span><span class=\"ops-stat-label\">Active bots</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{tenants}</span><span class=\"ops-stat-label\">Tenants</span></div>
    </div>
    {chart}
    <h4 class=\"ops-subtitle\">Deploy markers</h4>
    {deploys}
</div>",
        minutes = data.minutes,
        granularity = granularity(data.minutes),
        turns = format_number(turns),
        sessions = format_number(sessions),
        ws = format_number(ws_upgrades),
        bots = format_number(bots),
        tenants = format_number(tenants),
        chart = chart(data),
        deploys = deploy_list(data),
    )
}

pub async fn handle_ops_traffic(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<OpsWindowQuery>,
) -> impl IntoResponse {
    let minutes = window_minutes(query.window);
    let conn = pool.clone();
    let loaded = tokio::task::spawn_blocking(move || load(&conn, minutes))
        .await
        .unwrap_or_else(|error| {
            log::error!("ops traffic panel task failed: {error}");
            None
        });

    match loaded {
        Some(data) => Html(render(&data)),
        None => Html(
            "<div class=\"ops-panel\" id=\"ops-traffic\"><p class=\"ops-empty\">Traffic data unavailable: the database pool is not reachable.</p></div>"
                .to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "handlers_ops_traffic_tests.rs"]
mod tests;
