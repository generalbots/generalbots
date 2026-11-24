//! HTTP middleware that feeds the operational view of the Analytics app
//! (#2116).
//!
//! It records one counter per response into the process-local registry of
//! `botanalytics::ops_metrics`; a background flusher folds those counters into
//! `ops_request_metrics` every 30 s. The middleware never blocks the request:
//! recording is a mutex-protected hash-map update and the flush happens off
//! the request path.
//!
//! Compiled only when the `analytics` feature is enabled (see
//! `server_layers.rs`), so a lean build keeps its old middleware stack.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub async fn ops_metrics_middleware(request: Request, next: Next) -> Response {
    // #2459 — prefer the matched route TEMPLATE over the raw URI so the
    // per-route series stays bounded (no per-id cardinality explosion).
    let route = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|matched| matched.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let started = Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed();
    botanalytics::ops_metrics::record(&route, response.status().as_u16(), elapsed);
    // Feed the shared Prometheus collector too, so http_request_duration_ms
    // has real samples when /metrics is scraped. The hook sanitises labels.
    let collector = crate::sheet::state::metrics_collector();
    botcore::shared::analytics::track_route_latency(
        &collector,
        &route,
        response.status().as_u16(),
        elapsed.as_secs_f64() * 1000.0,
    )
    .await;
    response
}
