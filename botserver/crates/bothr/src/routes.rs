use axum::routing::{get, post, put};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/hr/employees", get(handlers::list_employees).post(handlers::create_employee))
        .route("/api/hr/employees/:id", put(handlers::update_employee))
        .route("/api/hr/recruitment", get(handlers::list_recruitment).post(handlers::create_recruitment))
        .route("/api/hr/attendance", get(handlers::list_attendance))
        .route("/api/hr/performance", get(handlers::list_performance))
        .route("/api/hr/payroll", get(handlers::list_payroll))
        .route("/api/hr/payroll/run", post(handlers::run_payroll))
        .route("/api/hr/benefits", get(handlers::list_benefits))
        .route("/api/hr/training", get(handlers::list_training).post(handlers::add_course))
        .route("/api/hr/review-cycles", post(handlers::start_review_cycle))
        .route("/api/hr/reports", get(handlers::list_reports))
}
