use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Html;
use axum::response::IntoResponse;
use chrono::Utc;
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::analytics_types::CountResult;
use crate::DbPool;

#[derive(Deserialize)]
pub struct MetricChartQuery {
    pub name: Option<String>,
}

pub async fn handle_dashboard_cards(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    let counts = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return (0i64, 0i64, 0i64, 0i64, 0.0f64);
            }
        };

        let messages = diesel::sql_query(
            "SELECT COUNT(*) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0);

        let active_sessions = diesel::sql_query(
            "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '30 minutes'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0);

        let conversations = diesel::sql_query(
            "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0);

        let users = diesel::sql_query(
            "SELECT COUNT(DISTINCT user_id) as count FROM user_sessions WHERE updated_at > NOW() - INTERVAL '7 days' AND user_id IS NOT NULL",
        )
        .get_result::<CountResult>(&mut db_conn)
        .map(|r| r.count)
        .unwrap_or(0);

        let avg_latency = diesel::sql_query(
            "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg FROM message_history WHERE role = 1 AND created_at > NOW() - INTERVAL '24 hours'",
        )
        .get_result::<crate::analytics_types::AvgResult>(&mut db_conn)
        .map(|r| r.avg.unwrap_or(0.0))
        .unwrap_or(0.0);

        (messages, active_sessions, conversations, users, avg_latency)
    })
    .await
    .unwrap_or((0, 0, 0, 0, 0.0));

    let (messages, active_sessions, conversations, users, avg_latency) = counts;
    let requests_per_sec = active_sessions as f64 / 1800.0;
    let latency_ms = avg_latency * 1000.0;
    let latency_display = if latency_ms < 1000.0 {
        format!("{latency_ms:.0} ms")
    } else {
        format!("{:.1} s", latency_ms / 1000.0)
    };

    let html = format!(
        r##"<div class="metric-card">
    <div class="metric-icon requests">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">{requests:.1}</span>
        <span class="metric-label">Requests/sec</span>
    </div>
    <div class="metric-trend up"><span>live</span></div>
</div>
<div class="metric-card">
    <div class="metric-icon latency">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">{latency_display}</span>
        <span class="metric-label">Avg Latency</span>
    </div>
    <div class="metric-trend down"><span>24h</span></div>
</div>
<div class="metric-card">
    <div class="metric-icon errors">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">—</span>
        <span class="metric-label">Error Rate</span>
    </div>
    <div class="metric-trend neutral"><span>24h</span></div>
</div>
<div class="metric-card">
    <div class="metric-icon users">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">{users}</span>
        <span class="metric-label">Active Users</span>
    </div>
    <div class="metric-trend up"><span>7d</span></div>
</div>
<div class="metric-card">
    <div class="metric-icon conversations">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">{conversations}</span>
        <span class="metric-label">Conversations</span>
    </div>
    <div class="metric-trend up"><span>24h</span></div>
</div>
<div class="metric-card">
    <div class="metric-icon uptime">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
    </div>
    <div class="metric-info">
        <span class="metric-value">{messages}</span>
        <span class="metric-label">Messages Today</span>
    </div>
    <div class="metric-trend neutral"><span>24h</span></div>
</div>"##,
        requests = requests_per_sec,
        latency_display = latency_display,
        users = users,
        conversations = conversations,
        messages = messages,
    );

    Html(html)
}

pub async fn handle_metric_chart(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<MetricChartQuery>,
) -> impl IntoResponse {
    let name = query.name.unwrap_or_else(|| "requests".to_string());
    let conn = pool.clone();

    let hourly = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return vec![0i64; 24];
            }
        };

        #[derive(Debug, diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct HourlyCount {
            #[diesel(sql_type = diesel::sql_types::Int4)]
            hour: i32,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let data = diesel::sql_query(
            "SELECT EXTRACT(HOUR FROM created_at)::int as hour, COUNT(*) as count
             FROM message_history
             WHERE created_at > NOW() - INTERVAL '24 hours'
             GROUP BY hour ORDER BY hour",
        )
        .load::<HourlyCount>(&mut db_conn)
        .unwrap_or_default();

        let mut counts = vec![0i64; 24];
        for row in data {
            if (0..24).contains(&row.hour) {
                counts[row.hour as usize] = row.count;
            }
        }
        counts
    })
    .await
    .unwrap_or_else(|_| vec![0i64; 24]);

    let max = hourly.iter().copied().max().unwrap_or(1).max(1) as f64;
    let color = match name.as_str() {
        "latency" => "#10b981",
        "errors" => "#ef4444",
        "throughput" => "#8b5cf6",
        _ => "#3b82f6",
    };

    let mut points = String::new();
    let mut area = String::new();
    for (i, count) in hourly.iter().enumerate() {
        let x = 50.0 + (i as f64 / 23.0) * 500.0;
        let y = 170.0 - (count.to_f64() / max * 130.0);
        points.push_str(&format!("{x:.1},{y:.1} "));
    }
    area.push_str("50,170 ");
    for (i, count) in hourly.iter().enumerate() {
        let x = 50.0 + (i as f64 / 23.0) * 500.0;
        let y = 170.0 - (count.to_f64() / max * 130.0);
        area.push_str(&format!("{x:.1},{y:.1} "));
    }
    area.push_str("550,170");

    let unit_label = match name.as_str() {
        "latency" => "ms",
        "errors" => "%",
        "throughput" => "MB",
        _ => "req",
    };

    let html = format!(
        r##"<svg viewBox="0 0 600 200" class="chart-svg">
    <defs>
        <linearGradient id="metric-gradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" style="stop-color:{color};stop-opacity:0.3"/>
            <stop offset="100%" style="stop-color:{color};stop-opacity:0"/>
        </linearGradient>
    </defs>
    <g class="chart-grid">
        <line x1="50" y1="20" x2="580" y2="20" stroke="var(--border)" stroke-dasharray="4"/>
        <line x1="50" y1="70" x2="580" y2="70" stroke="var(--border)" stroke-dasharray="4"/>
        <line x1="50" y1="120" x2="580" y2="120" stroke="var(--border)" stroke-dasharray="4"/>
        <line x1="50" y1="170" x2="580" y2="170" stroke="var(--border)" stroke-dasharray="4"/>
    </g>
    <g class="chart-axis-y">
        <text x="45" y="25" fill="var(--text-secondary)" font-size="10" text-anchor="end">{max:.0}</text>
        <text x="45" y="75" fill="var(--text-secondary)" font-size="10" text-anchor="end">{mid:.0}</text>
        <text x="45" y="170" fill="var(--text-secondary)" font-size="10" text-anchor="end">0</text>
    </g>
    <path class="chart-area" d="M{area} Z" fill="url(#metric-gradient)"/>
    <polyline class="chart-line" points="{points}" fill="none" stroke="{color}" stroke-width="2"/>
    <text x="575" y="15" fill="{color}" font-size="10" text-anchor="end">{unit}</text>
</svg>"##,
        color = color,
        max = max,
        mid = max / 2.0,
        area = area,
        points = points,
        unit = unit_label,
    );

    Html(html)
}

pub async fn handle_metrics_list(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    let stats = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let queries: Vec<(&str, &str)> = vec![
            ("users.total", "SELECT COUNT(*) FROM users"),
            (
                "users.active",
                "SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE updated_at > NOW() - INTERVAL '7 days'",
            ),
            ("sessions.total", "SELECT COUNT(*) FROM user_sessions"),
            (
                "messages.total",
                "SELECT COUNT(*) FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
            ),
            (
                "conversations.total",
                "SELECT COUNT(DISTINCT session_id) FROM message_history",
            ),
        ];

        let mut rows = Vec::new();
        for (name, sql) in queries {
            let value = diesel::sql_query(sql)
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);
            rows.push((name, "gauge", value));
        }
        rows
    })
    .await
    .unwrap_or_default();

    if stats.is_empty() {
        return Html(
            r##"<tbody>
    <tr><td colspan="6" class="loading-cell">No metrics available</td></tr>
</tbody>"##.to_string(),
        );
    }

    let mut rows_html = String::new();
    for (name, mtype, value) in stats {
        let category = if name.starts_with("users") || name.starts_with("sessions") {
            "system"
        } else {
            "application"
        };
        rows_html.push_str(&format!(
            r##"<tr>
    <td class="metric-name">{name}</td>
    <td>{mtype}</td>
    <td>{value}</td>
    <td><span class="category-badge">{category}</span></td>
    <td class="metric-desc">—</td>
    <td>{now}</td>
</tr>"##,
            now = Utc::now().format("%H:%M:%S UTC"),
        ));
    }

    Html(format!(r##"<tbody>{rows_html}</tbody>"##))
}

trait ToF64 {
    fn to_f64(&self) -> f64;
}

impl ToF64 for i64 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}
