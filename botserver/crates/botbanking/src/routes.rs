use axum::routing::{get, post, put};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/banking/transactions", get(handlers::list_transactions).post(handlers::create_transaction))
        .route("/api/banking/platforms", get(handlers::list_platforms))
        .route("/api/banking/reconcile", post(handlers::reconcile))
        .route("/api/banking/reports", get(handlers::get_report))
        .route("/api/banking/reconcile/pairs", get(handlers::list_reconcile_pairs))
        .route("/api/banking/reconcile/match", post(handlers::manual_match))
        .route("/api/banking/platforms/{id}/sync", put(handlers::sync_platform))
}
