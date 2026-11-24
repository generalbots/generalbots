use super::*;

// Canonical office metric family names (no PII, stable for Prometheus scraping).
pub const OFFICE_RECALC_DURATION_MS: &str = "office_recalc_duration_ms";
pub const OFFICE_WORKBOOK_MEMORY_BYTES: &str = "office_workbook_memory_bytes";
pub const OFFICE_EXPORT_DURATION_MS: &str = "office_export_duration_ms";
pub const HTTP_REQUEST_DURATION_MS: &str = "http_request_duration_ms";

/// Reduce an arbitrary label to a bounded `[a-z0-9_]` token. Used for app,
/// format and route labels; never pass file names, user or tenant ids here.
fn sanitize_label(raw: &str) -> String {
    let normalized: String = raw
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            'A'..='Z' => c.to_ascii_lowercase(),
            _ => '_',
        })
        .collect();
    let trimmed = normalized.trim_matches('_');
    let token: String = trimmed.chars().take(32).collect();
    if token.is_empty() {
        "unknown".to_string()
    } else {
        token
    }
}

/// One-way opaque token for an identifier (workbook/branch). The raw id is
/// hashed so no file name, user id or tenant id can ever reach the exporter.
fn opaque_token(prefix: &str, raw: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw.hash(&mut hasher);
    format!("{prefix}{:016x}", hasher.finish())
}

/// Emit a count + sum + max ("histogram-ish") distribution for a duration.
/// `scope` is the shared, sanitized label set; it must contain no raw ids.
async fn record_duration_distribution(
    collector: &MetricsCollector,
    family: &str,
    scope: HashMap<String, String>,
    duration_ms: f64,
) {
    if !duration_ms.is_finite() || duration_ms < 0.0 {
        return;
    }
    collector
        .increment(format!("{family}_count"), scope.clone())
        .await;
    collector
        .record(format!("{family}_sum"), duration_ms, scope.clone())
        .await;
    collector
        .observe_max(format!("{family}_max"), duration_ms, scope)
        .await;
}

pub async fn collect_system_metrics(collector: &MetricsCollector, state: &AppState) {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get database connection for metrics: {}", e);
            return;
        }
    };

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_users: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let active_cutoff = Utc::now() - Duration::days(7);
    let active_users: i64 = diesel::sql_query(
        "SELECT COUNT(DISTINCT user_id) as count FROM user_sessions WHERE updated_at > $1",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(active_cutoff)
    .get_result::<CountResult>(&mut conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let total_sessions: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM user_sessions")
        .get_result::<CountResult>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    #[derive(QueryableByName)]
    struct SizeResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_size: i64,
    }

    let storage_bytes: i64 =
        diesel::sql_query("SELECT COALESCE(SUM(file_size), 0) as total_size FROM kb_documents")
            .get_result::<SizeResult>(&mut conn)
            .map(|r| r.total_size)
            .unwrap_or(0);

    let storage_gb = storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let mut labels = HashMap::new();
    labels.insert("source".to_string(), "system".to_string());

    collector
        .gauge(
            "users.total".to_string(),
            total_users as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge(
            "users.active".to_string(),
            active_users as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge(
            "sessions.total".to_string(),
            total_sessions as f64,
            labels.clone(),
        )
        .await;
    collector
        .gauge("storage.gb".to_string(), storage_gb, labels.clone())
        .await;
}

pub async fn track_api_call(
    collector: &MetricsCollector,
    endpoint: String,
    duration_ms: f64,
    status: u16,
) {
    let mut labels = HashMap::new();
    labels.insert("endpoint".to_string(), endpoint);
    labels.insert("status".to_string(), status.to_string());

    collector
        .increment("api.calls".to_string(), labels.clone())
        .await;
    collector
        .record("api.duration_ms".to_string(), duration_ms, labels.clone())
        .await;

    if status >= 500 {
        collector.increment("api.errors".to_string(), labels).await;
    }
}

pub async fn track_message(collector: &MetricsCollector, channel: String, user_id: String) {
    let mut labels = HashMap::new();
    // #2459 — a raw user id in a label is both PII and unbounded cardinality.
    // Keep the bounded channel token and hash the user into an opaque id.
    labels.insert("channel".to_string(), sanitize_label(&channel));
    labels.insert("user".to_string(), opaque_token("u", &user_id));

    collector
        .increment("messages.total".to_string(), labels)
        .await;
}

pub async fn track_file_operation(
    collector: &MetricsCollector,
    operation: String,
    size_bytes: i64,
    success: bool,
) {
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), operation);
    labels.insert("success".to_string(), success.to_string());

    collector
        .increment("files.operations".to_string(), labels.clone())
        .await;

    if success {
        collector
            .record("files.bytes".to_string(), size_bytes as f64, labels)
            .await;
    }
}

/// #2459 — recalculation time per sheet/app and branch scope.
///
/// `app` is a sheet/app token (e.g. "sheet"); `branch` is hashed into an
/// opaque label so no tenant/branch id is exposed. File names and user ids
/// are never accepted. Exposes `_count`, `_sum` and `_max` milliseconds.
pub async fn track_office_recalc(
    collector: &MetricsCollector,
    app: &str,
    branch: Option<&str>,
    duration_ms: f64,
) {
    let mut scope = HashMap::new();
    scope.insert("app".to_string(), sanitize_label(app));
    if let Some(branch) = branch {
        scope.insert("branch".to_string(), opaque_token("br", branch));
    }
    record_duration_distribution(collector, OFFICE_RECALC_DURATION_MS, scope, duration_ms).await;
}

/// #2459 — measured resident memory for a single workbook, in bytes.
///
/// `workbook_id` is hashed to an opaque `wb*` label; the raw id (which could
/// be a file name or user id) never reaches the exporter.
pub async fn track_office_workbook_memory(
    collector: &MetricsCollector,
    workbook_id: &str,
    bytes: u64,
) {
    let mut labels = HashMap::new();
    labels.insert(
        "workbook".to_string(),
        opaque_token("wb", workbook_id),
    );
    collector
        .gauge(
            OFFICE_WORKBOOK_MEMORY_BYTES.to_string(),
            bytes as f64,
            labels,
        )
        .await;
}

/// #2459 — export duration per (sanitized) output format, e.g. "pdf"/"docx".
/// Exposes `_count`, `_sum` and `_max` milliseconds.
pub async fn track_office_export(
    collector: &MetricsCollector,
    format: &str,
    duration_ms: f64,
) {
    let mut scope = HashMap::new();
    scope.insert("format".to_string(), sanitize_label(format));
    record_duration_distribution(collector, OFFICE_EXPORT_DURATION_MS, scope, duration_ms).await;
}

/// #2459 — request latency per route template (and status), replacing the
/// per-class-only histogram. `route` is sanitized to a bounded token.
pub async fn track_route_latency(
    collector: &MetricsCollector,
    route: &str,
    status: u16,
    duration_ms: f64,
) {
    let mut scope = HashMap::new();
    scope.insert("route".to_string(), sanitize_label(route));
    scope.insert("status".to_string(), status.to_string());
    record_duration_distribution(collector, HTTP_REQUEST_DURATION_MS, scope, duration_ms).await;
}
