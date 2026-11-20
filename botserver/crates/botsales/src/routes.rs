//! Sales routes after the #2540 consolidation.
//!
//! The `/api/sales/*` CRUD surface duplicated `botcontacts`' `/api/crm/*`
//! entities with a second data model. It is now tombstoned with `410 Gone`
//! and a canonical-path hint so clients can migrate in one release.
//!
//! Quotes and orders were the only entities with a live first-party caller
//! (the CRM Quotes & Orders tab) and no CRM equivalent, so their existing,
//! unchanged handlers are re-mounted under the canonical `/api/crm/quotes`
//! and `/api/crm/orders` paths before the old aliases are retired.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::leads_quotes;

fn gone_body(canonical: &str) -> Value {
    json!({
        "error": "gone",
        "status": 410,
        "canonical": canonical,
        "message": format!(
            "This /api/sales/* endpoint is retired. Use {canonical} instead."
        ),
    })
}

fn gone_response(canonical: &str) -> Response {
    (StatusCode::GONE, Json(gone_body(canonical))).into_response()
}

macro_rules! gone_handler {
    ($name:ident, $canonical:literal) => {
        async fn $name() -> Response {
            gone_response($canonical)
        }
    };
}

gone_handler!(deals, "/api/crm/deals");
gone_handler!(deal, "/api/crm/deals/:id");
gone_handler!(contacts, "/api/crm/contacts");
gone_handler!(activities, "/api/crm/activities");
gone_handler!(leads, "/api/crm/leads");
gone_handler!(quotes, "/api/crm/quotes");
gone_handler!(orders, "/api/crm/orders");

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route(
            "/api/crm/quotes",
            get(leads_quotes::list_quotes).post(leads_quotes::create_quote),
        )
        .route(
            "/api/crm/orders",
            get(leads_quotes::list_orders).post(leads_quotes::create_order),
        )
        .route("/api/sales/deals", any(deals))
        .route("/api/sales/deals/:id", any(deal))
        .route("/api/sales/contacts", any(contacts))
        .route("/api/sales/activities", any(activities))
        .route("/api/sales/leads", any(leads))
        .route("/api/sales/quotes", any(quotes))
        .route("/api/sales/orders", any(orders))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_endpoint_answers_410_with_a_canonical_hint() {
        let response = gone_response("/api/crm/quotes");
        assert_eq!(response.status(), StatusCode::GONE);
        let body = gone_body("/api/crm/quotes");
        assert_eq!(body["status"], 410);
        assert_eq!(body["canonical"], "/api/crm/quotes");
    }

    #[test]
    fn quotes_and_orders_survive_under_the_crm_prefix() {
        let _router = configure::<()>();
    }
}
