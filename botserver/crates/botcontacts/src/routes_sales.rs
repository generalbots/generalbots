//! Deprecated `/api/sales/*` funnel, forecast and tracked-email surface (#2540).
//!
//! These routes used to duplicate the CRM surface served from
//! [`crate::routes`] and by `botmarketing`, with a second set of money maths.
//! The capabilities now have a single canonical home:
//!
//! | retired path | canonical path |
//! |--------------|----------------|
//! | `/api/sales/funnel` | `/api/crm/pipeline` |
//! | `/api/sales/funnel/move` | `/api/crm/deals/:id` |
//! | `/api/sales/funnel/conversion-rates` | `/api/crm/stats/conversion-rate` |
//! | `/api/sales/forecast` | `/api/crm/stats` |
//! | `/api/sales/forecast/ai-predict` | `/api/crm/ai/generate` |
//! | `/api/sales/email/send` | `/api/crm/email/send` |
//! | `/api/sales/email/webhook` | `/api/crm/email/track/open` |
//!
//! The routes stay mounted for one release so callers get an explicit `410
//! Gone` with a migration hint instead of a `404`. The former service modules
//! (`sales_funnel`, `forecast`, `email_integration`) remain library code with
//! their unit tests; only the duplicate HTTP surface is retired.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::CrateState;

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

gone_handler!(funnel, "/api/crm/pipeline");
gone_handler!(funnel_move, "/api/crm/deals/:id");
gone_handler!(conversion_rates, "/api/crm/stats/conversion-rate");
gone_handler!(forecast, "/api/crm/stats");
gone_handler!(forecast_ai_predict, "/api/crm/ai/generate");
gone_handler!(email_send, "/api/crm/email/send");
gone_handler!(email_webhook, "/api/crm/email/track/open");

pub fn configure_sales_routes() -> Router<Arc<CrateState>> {
    Router::new()
        .route("/api/sales/funnel", any(funnel))
        .route("/api/sales/funnel/move", any(funnel_move))
        .route("/api/sales/funnel/conversion-rates", any(conversion_rates))
        .route("/api/sales/forecast", any(forecast))
        .route("/api/sales/forecast/ai-predict", any(forecast_ai_predict))
        .route("/api/sales/email/send", any(email_send))
        .route("/api/sales/email/webhook", any(email_webhook))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_endpoint_answers_410_with_a_canonical_hint() {
        let response = gone_response("/api/crm/pipeline");
        assert_eq!(response.status(), StatusCode::GONE);
        let body = gone_body("/api/crm/pipeline");
        assert_eq!(body["status"], 410);
        assert_eq!(body["canonical"], "/api/crm/pipeline");
        assert!(body["message"]
            .as_str()
            .is_some_and(|message| message.contains("/api/crm/pipeline")));
    }

    #[test]
    fn sales_routes_remain_registered_as_tombstones() {
        let _router = configure_sales_routes();
    }
}
