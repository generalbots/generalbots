use axum::routing::{get, post};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/timeclock/clock", post(handlers::clock_in_out))
        .route("/api/timeclock/records", get(handlers::list_records))
        .route("/api/timeclock/overtime", get(handlers::list_overtime))
        .route("/api/timeclock/overtime/:id/approve", post(handlers::approve_overtime))
        .route("/api/timeclock/reports", get(handlers::get_reports))
}
