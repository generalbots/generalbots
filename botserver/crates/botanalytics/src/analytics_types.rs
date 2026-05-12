use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, diesel::Queryable)]
pub struct AnalyticsStats {
    pub message_count: i64,
    pub session_count: i64,
    pub active_sessions: i64,
    pub avg_response_time: f64,
}

impl Default for AnalyticsStats {
    fn default() -> Self {
        Self {
            message_count: 0,
            session_count: 0,
            active_sessions: 0,
            avg_response_time: 0.0,
        }
    }
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AvgResult {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub avg: Option<f64>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HourlyCount {
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub hour: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub query: Option<String>,
    #[serde(rename = "timeRange")]
    pub time_range: Option<String>,
}

pub(crate) fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
