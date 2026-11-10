//! Per-endpoint request counters for the operational view (#2116).
//!
//! The HTTP middleware in botserver (`main_module/server/ops_metrics_layer.rs`)
//! calls [`record`] for every request. Counters live in a process-local
//! registry and are folded into `ops_request_metrics` every 30 s by
//! [`spawn_flusher`], so the Analytics app can show request rate, error rate
//! and latency per endpoint class without a metrics vendor or a per-request
//! database write.
//!
//! No `unwrap`/`expect`/`panic!` here: a poisoned registry lock or a failed
//! flush is logged and the server keeps serving.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use diesel::RunQueryDsl;

use crate::DbPool;

/// Upper bounds (ms) of the seven closed histogram buckets; the eighth bucket
/// is open-ended (>= 5000 ms).
pub const BUCKET_UPPER_BOUNDS_MS: [u64; 7] = [50, 100, 250, 500, 1000, 2500, 5000];

/// Number of histogram buckets (closed bounds + the open one).
pub const BUCKET_COUNT: usize = 8;

/// How often the registry is folded into `ops_request_metrics`.
const FLUSH_INTERVAL_SECS: u64 = 30;

/// Rows older than this are pruned so the table stays bounded.
const RETENTION_DAYS: i32 = 30;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    minute: i64,
    endpoint_class: String,
    status_class: String,
}

#[derive(Debug, Clone, Copy)]
struct Aggregate {
    count: i64,
    sum_ms: f64,
    max_ms: f64,
    buckets: [i64; BUCKET_COUNT],
}

impl Default for Aggregate {
    fn default() -> Self {
        Self {
            count: 0,
            sum_ms: 0.0,
            max_ms: 0.0,
            buckets: [0; BUCKET_COUNT],
        }
    }
}

impl Aggregate {
    fn observe(&mut self, latency: Duration) {
        let ms = latency.as_secs_f64() * 1000.0;
        self.count += 1;
        self.sum_ms += ms;
        if ms > self.max_ms {
            self.max_ms = ms;
        }
        self.buckets[bucket_index(ms)] += 1;
    }
}

/// One drained registry entry, ready to be upserted.
#[derive(Debug, Clone)]
pub struct BucketRow {
    pub minute_epoch: i64,
    pub endpoint_class: String,
    pub status_class: String,
    pub count: i64,
    pub sum_ms: f64,
    pub max_ms: f64,
    pub buckets: [i64; BUCKET_COUNT],
}

static REGISTRY: LazyLock<Mutex<HashMap<Key, Aggregate>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Maps a request path to a coarse endpoint class. The classes are the
/// dimensions the operations dashboard groups by; they must stay stable
/// because `ops_request_metrics.endpoint_class` stores them as text.
pub fn classify_endpoint(path: &str) -> &'static str {
    if path == "/ws" || path.starts_with("/ws?") || path.starts_with("/api/ws") {
        "websocket"
    } else if path.starts_with("/api/analytics") || path.starts_with("/api/ui/analytics") {
        "analytics"
    } else if path.starts_with("/api/ui") {
        "ui"
    } else if path.starts_with("/api/cloud") {
        "cloud"
    } else if path.starts_with("/api/auth") {
        "auth"
    } else if path.starts_with("/api/monitoring") || path.starts_with("/api/ops") {
        "ops"
    } else if path.starts_with("/api") {
        "api"
    } else if path.starts_with("/suite")
        || path.starts_with("/cloud")
        || path.starts_with("/login")
        || path.starts_with("/static")
        || path.starts_with("/themes")
        || path.starts_with("/auth")
    {
        "static"
    } else {
        "other"
    }
}

/// Maps an HTTP status to its class. A `101 Switching Protocols` is a
/// successful WebSocket upgrade, so it lands in `1xx`, not in an error class.
pub fn classify_status(status: u16) -> &'static str {
    match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    }
}

/// Histogram bucket index for a latency in milliseconds.
pub fn bucket_index(latency_ms: f64) -> usize {
    BUCKET_UPPER_BOUNDS_MS
        .iter()
        .position(|bound| latency_ms <= *bound as f64)
        .unwrap_or(BUCKET_COUNT - 1)
}

/// Upper bound of a bucket, used to report an approximate percentile.
pub fn bucket_bound_ms(index: usize) -> f64 {
    match BUCKET_UPPER_BOUNDS_MS.get(index) {
        Some(bound) => *bound as f64,
        None => *BUCKET_UPPER_BOUNDS_MS
            .last()
            .unwrap_or(&5_000) as f64,
    }
}

/// Approximate percentile from a merged histogram. `None` when no request was
/// observed, so callers render "no data" instead of a fabricated zero.
pub fn percentile_from_histogram(buckets: &[i64; BUCKET_COUNT], percentile: f64) -> Option<f64> {
    let total: i64 = buckets.iter().sum();
    if total == 0 {
        return None;
    }
    let target = (total as f64 * percentile).ceil() as i64;
    let mut cumulative = 0i64;
    for (index, count) in buckets.iter().enumerate() {
        cumulative += count;
        if cumulative >= target {
            return Some(bucket_bound_ms(index));
        }
    }
    Some(bucket_bound_ms(BUCKET_COUNT - 1))
}

/// Share of observations above `threshold_ms`, derived from the histogram.
/// The result is exact when the threshold sits on a bucket boundary, which is
/// why `ops/slo.yaml` uses boundaries for the latency objectives.
pub fn ratio_above_histogram(buckets: &[i64; BUCKET_COUNT], threshold_ms: f64) -> Option<f64> {
    let total: i64 = buckets.iter().sum();
    if total == 0 {
        return None;
    }
    let bad: i64 = buckets
        .iter()
        .enumerate()
        .filter(|(index, _)| bucket_bound_ms(*index) > threshold_ms)
        .map(|(_, count)| *count)
        .sum();
    Some(bad as f64 / total as f64)
}

/// Records one request. Called from the HTTP middleware on every response.
pub fn record(path: &str, status: u16, latency: Duration) {
    let now = chrono::Utc::now().timestamp();
    let key = Key {
        minute: now - now.rem_euclid(60),
        endpoint_class: classify_endpoint(path).to_string(),
        status_class: classify_status(status).to_string(),
    };
    match REGISTRY.lock() {
        Ok(mut registry) => {
            registry.entry(key).or_default().observe(latency);
        }
        Err(error) => log::error!("ops metrics registry lock poisoned: {error}"),
    }
}

/// Takes every pending counter out of the registry.
pub fn drain() -> Vec<BucketRow> {
    let drained = match REGISTRY.lock() {
        Ok(mut registry) => std::mem::take(&mut *registry),
        Err(error) => {
            log::error!("ops metrics registry lock poisoned while draining: {error}");
            return Vec::new();
        }
    };
    drained
        .into_iter()
        .map(|(key, aggregate)| BucketRow {
            minute_epoch: key.minute,
            endpoint_class: key.endpoint_class,
            status_class: key.status_class,
            count: aggregate.count,
            sum_ms: aggregate.sum_ms,
            max_ms: aggregate.max_ms,
            buckets: aggregate.buckets,
        })
        .collect()
}

/// Folds the drained counters into `ops_request_metrics`.
pub fn flush(pool: &DbPool, rows: &[BucketRow]) {
    if rows.is_empty() {
        return;
    }
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::error!("ops metrics flush skipped, no DB connection: {error}");
            return;
        }
    };
    for row in rows {
        let bucket = match chrono::DateTime::from_timestamp(row.minute_epoch, 0) {
            Some(ts) => ts,
            None => {
                let minute = row.minute_epoch;
                log::error!("ops metrics flush skipped, bad bucket {minute}");
                continue;
            }
        };
        let result = diesel::sql_query(
            "INSERT INTO ops_request_metrics
                 (bucket, endpoint_class, status_class, request_count, latency_sum_ms,
                  latency_max_ms, b_le_50, b_le_100, b_le_250, b_le_500, b_le_1000,
                  b_le_2500, b_le_5000, b_gt_5000)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
             ON CONFLICT (bucket, endpoint_class, status_class) DO UPDATE SET
                 request_count = ops_request_metrics.request_count + EXCLUDED.request_count,
                 latency_sum_ms = ops_request_metrics.latency_sum_ms + EXCLUDED.latency_sum_ms,
                 latency_max_ms = GREATEST(ops_request_metrics.latency_max_ms, EXCLUDED.latency_max_ms),
                 b_le_50 = ops_request_metrics.b_le_50 + EXCLUDED.b_le_50,
                 b_le_100 = ops_request_metrics.b_le_100 + EXCLUDED.b_le_100,
                 b_le_250 = ops_request_metrics.b_le_250 + EXCLUDED.b_le_250,
                 b_le_500 = ops_request_metrics.b_le_500 + EXCLUDED.b_le_500,
                 b_le_1000 = ops_request_metrics.b_le_1000 + EXCLUDED.b_le_1000,
                 b_le_2500 = ops_request_metrics.b_le_2500 + EXCLUDED.b_le_2500,
                 b_le_5000 = ops_request_metrics.b_le_5000 + EXCLUDED.b_le_5000,
                 b_gt_5000 = ops_request_metrics.b_gt_5000 + EXCLUDED.b_gt_5000,
                 updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Timestamptz, _>(bucket)
        .bind::<diesel::sql_types::Text, _>(row.endpoint_class.as_str())
        .bind::<diesel::sql_types::Text, _>(row.status_class.as_str())
        .bind::<diesel::sql_types::BigInt, _>(row.count)
        .bind::<diesel::sql_types::Double, _>(row.sum_ms)
        .bind::<diesel::sql_types::Double, _>(row.max_ms)
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[0])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[1])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[2])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[3])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[4])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[5])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[6])
        .bind::<diesel::sql_types::BigInt, _>(row.buckets[7])
        .execute(&mut conn);
        if let Err(error) = result {
            let class = row.endpoint_class.as_str();
            log::error!("ops metrics flush failed for {class}: {error}");
        }
    }
}

/// Drops buckets older than the retention window.
pub fn prune(pool: &DbPool) {
    let Ok(mut conn) = pool.get() else {
        log::error!("ops metrics prune skipped, no DB connection");
        return;
    };
    if let Err(error) = diesel::sql_query(
        "DELETE FROM ops_request_metrics WHERE bucket < NOW() - make_interval(days => $1)",
    )
    .bind::<diesel::sql_types::Integer, _>(RETENTION_DAYS)
    .execute(&mut conn)
    {
        log::error!("ops metrics prune failed: {error}");
    }
}

/// Background loop: flush every 30 s and prune once an hour.
pub fn spawn_flusher(pool: std::sync::Arc<DbPool>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(FLUSH_INTERVAL_SECS));
        let mut ticks: u64 = 0;
        loop {
            ticker.tick().await;
            let rows = drain();
            let pool_for_flush = pool.clone();
            let flush_result = tokio::task::spawn_blocking(move || {
                flush(&pool_for_flush, &rows);
            })
            .await;
            if let Err(error) = flush_result {
                log::error!("ops metrics flush task failed: {error}");
            }
            ticks += 1;
            if ticks % 120 == 0 {
                let pool_for_prune = pool.clone();
                let prune_result = tokio::task::spawn_blocking(move || prune(&pool_for_prune)).await;
                if let Err(error) = prune_result {
                    log::error!("ops metrics prune task failed: {error}");
                }
            }
        }
    });
}

/// Build metadata of the running binary. Mirrors `botcore::product::build_info`
/// (same workspace build, same environment); botanalytics does not depend on
/// botcore, so the values are read here instead of imported.
pub fn build_version() -> String {
    let raw = option_env!("BOTSERVER_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    raw.trim().trim_start_matches('v').to_string()
}

pub fn build_commit() -> String {
    build_text(option_env!("BOTSERVER_COMMIT"))
}

pub fn build_date() -> String {
    build_text(option_env!("BOTSERVER_BUILD_DATE"))
}

fn build_text(value: Option<&'static str>) -> String {
    match value.map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => "unknown".to_string(),
    }
}

/// Records a deploy marker when the running build differs from the last one.
pub fn record_deploy_marker(pool: std::sync::Arc<DbPool>) {
    tokio::task::spawn_blocking(move || {
        let Ok(mut conn) = pool.get() else {
            log::error!("deploy marker skipped, no DB connection");
            return;
        };
        let version = build_version();
        let commit = build_commit();
        let date = build_date();
        let result = diesel::sql_query(
            "INSERT INTO ops_deploy_markers (version, commit_hash, build_date)
             SELECT $1, $2, $3
             WHERE NOT EXISTS (
                 SELECT 1 FROM ops_deploy_markers WHERE version = $1 AND commit_hash = $2
             )",
        )
        .bind::<diesel::sql_types::Text, _>(version.as_str())
        .bind::<diesel::sql_types::Text, _>(commit.as_str())
        .bind::<diesel::sql_types::Text, _>(date.as_str())
        .execute(&mut conn);
        match result {
            Ok(inserted) => log::info!(
                "deploy marker check: version={version} commit={commit} inserted={inserted}"
            ),
            Err(error) => log::error!("deploy marker insert failed: {error}"),
        }
    });
}

#[cfg(test)]
#[path = "ops_metrics_tests.rs"]
mod tests;
