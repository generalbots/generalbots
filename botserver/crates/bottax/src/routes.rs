use axum::routing::{get, post};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/tax/nfe", get(handlers::list_nfe).post(handlers::create_nfe))
        .route("/api/tax/nfe/:id/authorize", post(handlers::authorize_nfe))
        .route("/api/tax/nfse", get(handlers::list_nfse).post(handlers::create_nfse))
        .route("/api/tax/cte", get(handlers::list_cte).post(handlers::create_cte))
        .route("/api/tax/sped", get(handlers::list_sped))
}
