use super::*;

/// Records an app-store install and returns the resolved app metadata.
pub async fn install_handler(Json(body): Json<InstallBody>) -> Json<serde_json::Value> {
    let app_id = body.app_id.unwrap_or(Default::default());
    let found = registry::all_apps().into_iter().find(|a| a.id == app_id);
    match found {
        Some(app) => {
            let stats = installs();
            let mut counts = match stats.counts.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let count = counts.entry(app.id.clone()).or_insert(0);
            *count += 1;
            Json(json!({
                "ok": true,
                "app_id": app.id,
                "title": app.title,
                "category": app.category,
                "url": app.url,
                "installs": *count,
            }))
        }
        None => Json(json!({ "ok": false, "error": "unknown_app", "app_id": app_id })),
    }
}
/// Returns the server-side install popularity ranking.
///
/// #2965 — this handler also serves `GET /api/apps/install` as a read-only
/// status/list probe. It must never mutate the counters: callers (including
/// anonymous-denied RBAC probes) may poll it any number of times.
pub async fn install_stats_handler() -> Json<serde_json::Value> {
    let stats = installs();
    let counts = match stats.counts.lock() {
        Ok(g) => g.clone(),
        Err(p) => p.into_inner().clone(),
    };
    let total: u64 = counts.values().sum();
    Json(json!({ "installs": counts, "total": total }))
}

#[cfg(test)]
mod install_status_tests {
    use super::install_stats_handler;

    /// #2965 — the GET status endpoint is a pure read. Two consecutive calls
    /// must return byte-for-byte identical payloads and a well-formed shape;
    /// a probe can never inflate popularity counts.
    #[tokio::test]
    async fn status_probe_is_read_only_and_well_formed() {
        let first = install_stats_handler().await.0;
        let second = install_stats_handler().await.0;
        assert_eq!(
            first, second,
            "GET /api/apps/install must not mutate install counters"
        );
        assert!(first.get("installs").is_some(), "missing installs map");
        assert!(first.get("total").is_some(), "missing total");
    }
}
