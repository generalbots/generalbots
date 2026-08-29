use axum::routing::{get, put};
use axum::Router;
use crate::{handlers, leads_quotes};

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/sales/deals", get(handlers::list_deals).post(handlers::create_deal))
        .route("/api/sales/deals/:id", put(handlers::update_deal))
        .route("/api/sales/contacts", get(handlers::list_contacts))
        .route("/api/sales/activities", get(handlers::list_activities))
        .route("/api/sales/leads", get(leads_quotes::list_leads).post(leads_quotes::create_lead))
        .route("/api/sales/quotes", get(leads_quotes::list_quotes).post(leads_quotes::create_quote))
        .route("/api/sales/orders", get(leads_quotes::list_orders).post(leads_quotes::create_order))
}
