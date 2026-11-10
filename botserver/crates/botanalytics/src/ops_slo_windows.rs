//! Window queries behind the SLO burn panel (#2116).
//!
//! Two tables feed the burn rates: `ops_request_metrics` (the HTTP middleware
//! counters) and `llm_metrics` (the LLM observability layer). Each window is
//! one aggregate row per table, so a panel refresh costs a handful of indexed
//! range scans instead of one query per objective.
//!
//! Split from `handlers_ops.rs` to keep both files readable; the rendering and
//! the burn math live there.

use diesel::RunQueryDsl;

use crate::ops_metrics::BUCKET_COUNT;
use crate::ops_slo::{MetricKind, Objective};

/// Everything the SLO table needs from one time window.
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

/// Static assets are excluded: an unavailable `.js` must not burn the API SLO.
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
        percentile_cont(0.95) WITHIN GROUP (ORDER BY ttft_ms::float8)
            FILTER (WHERE ttft_ms IS NOT NULL) AS ttft_p95,
        percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms::float8) AS answer_p95
    FROM llm_metrics
    WHERE timestamp > NOW() - ($1::int * INTERVAL '1 minute')";

/// Collects one window. A failed query leaves that signal empty, so the panel
/// renders "no data" for it instead of failing the whole fragment.
pub fn collect_window(
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

/// Threshold declared for a metric, or the documented fallback.
pub fn threshold_of(objectives: &[Objective], metric: MetricKind, fallback: i64) -> i64 {
    objectives
        .iter()
        .find(|objective| objective.metric == metric)
        .and_then(|objective| objective.threshold_ms)
        .map(|value| value as i64)
        .unwrap_or(fallback)
}
