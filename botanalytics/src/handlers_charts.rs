use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use axum::{extract::State, response::Html, response::IntoResponse};
use diesel::RunQueryDsl;

use crate::analytics_types::*;
use crate::DbPool;

pub async fn handle_timeseries_messages(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    let hourly_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT EXTRACT(HOUR FROM created_at) as hour, COUNT(*) as count
             FROM message_history
             WHERE created_at > NOW() - INTERVAL '24 hours'
             GROUP BY EXTRACT(HOUR FROM created_at)
             ORDER BY hour",
        )
        .load::<HourlyCount>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let hours: Vec<i32> = (0..24).collect();
    let mut counts: Vec<i64> = vec![0; 24];

    for data in hourly_data {
        let hour_idx = data.hour as usize;
        if hour_idx < 24 {
            counts[hour_idx] = data.count;
        }
    }

    let labels: Vec<String> = hours.iter().map(|h| format!("{}:00", h)).collect();
    let max_count = counts.iter().max().copied().unwrap_or(1).max(1);

    let mut html = String::new();
    html.push_str("<div class=\"chart-container\">");
    html.push_str("<div class=\"chart-bars\">");

    for (i, count) in counts.iter().enumerate() {
        let height_pct = (*count as f64 / max_count as f64) * 100.0;
        let _ = write!(
            html,
            "<div class=\"chart-bar\" style=\"height: {}%\" title=\"{}: {} messages\"></div>",
            height_pct, labels[i], count
        );
    }

    html.push_str("</div>");
    html.push_str("<div class=\"chart-labels\">");
    for (i, label) in labels.iter().enumerate() {
        if i % 4 == 0 {
            let _ = write!(html, "<span>{label}</span>");
        }
    }
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_timeseries_response(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HourlyAvg {
        #[diesel(sql_type = diesel::sql_types::Double)]
        hour: f64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_time: Option<f64>,
    }

    let hourly_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT EXTRACT(HOUR FROM created_at) as hour,
             AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_time
             FROM message_history
             WHERE role = 1 AND created_at > NOW() - INTERVAL '24 hours'
             GROUP BY EXTRACT(HOUR FROM created_at)
             ORDER BY hour",
        )
        .load::<HourlyAvg>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut avgs: Vec<f64> = vec![0.0; 24];

    for data in hourly_data {
        let hour_idx = data.hour as usize;
        if hour_idx < 24 {
            avgs[hour_idx] = data.avg_time.unwrap_or(0.0);
        }
    }

    let labels: Vec<String> = (0..24).map(|h| format!("{}:00", h)).collect();
    let max_avg = avgs.iter().copied().fold(0.0f64, f64::max).max(0.1);

    let mut html = String::new();
    html.push_str("<div class=\"chart-container line-chart\">");
    html.push_str("<svg viewBox=\"0 0 480 200\" preserveAspectRatio=\"none\">");
    html.push_str("<polyline fill=\"none\" stroke=\"var(--primary)\" stroke-width=\"2\" points=\"");

    for (i, avg) in avgs.iter().enumerate() {
        let x = (i as f64 / 23.0) * 480.0;
        let y = 180.0f64.mul_add(-(*avg / max_avg), 200.0);
        let _ = write!(html, "{x},{y} ");
    }

    html.push_str("\"/></svg>");
    html.push_str("<div class=\"chart-labels\">");
    for (i, label) in labels.iter().enumerate() {
        if i % 4 == 0 {
            let _ = write!(html, "<span>{label}</span>");
        }
    }
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_channels_distribution(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ChannelCount {
        #[diesel(sql_type = diesel::sql_types::Text)]
        channel: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let channel_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT COALESCE(channel, 'web') as channel, COUNT(*) as count
             FROM sessions
             WHERE created_at > NOW() - INTERVAL '7 days'
             GROUP BY channel
             ORDER BY count DESC
             LIMIT 5",
        )
        .load::<ChannelCount>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let total: i64 = channel_data.iter().map(|c| c.count).sum();
    let colors = ["#4f46e5", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6"];

    let mut html = String::new();
    html.push_str("<div class=\"pie-chart-container\">");
    html.push_str("<div class=\"pie-chart\">");

    let mut offset = 0.0f64;
    for (i, data) in channel_data.iter().enumerate() {
        let pct = if total > 0 {
            (data.count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let color = colors[i % colors.len()];
        let _ = write!(
            html,
            "<div class=\"pie-segment\" style=\"--offset: {}; --value: {}; --color: {};\"></div>",
            offset, pct, color
        );
        offset += pct;
    }

    html.push_str("</div>");
    html.push_str("<div class=\"pie-legend\">");

    for (i, data) in channel_data.iter().enumerate() {
        let pct = if total > 0 {
            (data.count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let color = colors[i % colors.len()];
        let _ = write!(
            html,
            "<div class=\"legend-item\"><span class=\"legend-color\" style=\"background: {};\"></span>{} ({:.0}%)</div>",
            color, html_escape(&data.channel), pct
        );
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

pub async fn handle_bots_performance(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let conn = pool.clone();

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotStats {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let bot_data = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT b.name, COUNT(mh.id) as count
             FROM bots b
             LEFT JOIN sessions s ON s.bot_id = b.id
             LEFT JOIN message_history mh ON mh.session_id = s.id
             WHERE mh.created_at > NOW() - INTERVAL '24 hours' OR mh.created_at IS NULL
             GROUP BY b.id, b.name
             ORDER BY count DESC
             LIMIT 5",
        )
        .load::<BotStats>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let max_count = bot_data.iter().map(|b| b.count).max().unwrap_or(1).max(1);

    let mut html = String::new();
    html.push_str("<div class=\"horizontal-bars\">");

    for data in bot_data.iter() {
        let pct = (data.count as f64 / max_count as f64) * 100.0;
        html.push_str("<div class=\"bar-row\">");
        let _ = write!(
            html,
            "<span class=\"bar-label\">{}</span>",
            html_escape(&data.name)
        );
        let _ = write!(
            html,
            "<div class=\"bar-track\"><div class=\"bar-fill\" style=\"width: {pct}%;\"></div></div>"
        );
        let _ = write!(html, "<span class=\"bar-value\">{}</span>", data.count);
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}
