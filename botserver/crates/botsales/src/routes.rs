use axum::routing::{get, put};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/sales/deals", get(handlers::list_deals).post(handlers::create_deal))
        .route("/api/sales/deals/{id}", put(handlers::update_deal))
        .route("/api/sales/contacts", get(handlers::list_contacts))
        .route("/api/sales/activities", get(handlers::list_activities))
        .route("/api/sales/forecast", get(handlers::get_forecast))
}
