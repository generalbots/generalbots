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
    let path = request.uri().path().to_string();
    let started = Instant::now();
    let response = next.run(request).await;
    botanalytics::ops_metrics::record(&path, response.status().as_u16(), started.elapsed());
    response
}
