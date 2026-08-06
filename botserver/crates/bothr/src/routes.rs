use axum::routing::{get, put};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/hr/employees", get(handlers::list_employees).post(handlers::create_employee))
        .route("/api/hr/employees/:id", put(handlers::update_employee))
        .route("/api/hr/recruitment", get(handlers::list_recruitment))
        .route("/api/hr/attendance", get(handlers::list_attendance))
        .route("/api/hr/performance", get(handlers::list_performance))
}
