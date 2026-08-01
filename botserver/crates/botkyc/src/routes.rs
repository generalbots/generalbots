use axum::routing::{get, post, put};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/kyc/verifications", get(handlers::list_verifications))
        .route("/api/kyc/verifications/:id", put(handlers::update_verification))
        .route("/api/kyc/signatures", get(handlers::list_signatures))
        .route("/api/kyc/signatures/:id/sign", post(handlers::sign_document))
        .route("/api/kyc/certificates", get(handlers::list_certificates))
}
