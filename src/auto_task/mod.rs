pub mod app_generator;
pub mod ask_later;
pub mod auto_task;
pub mod autotask_api;
pub mod designer_ai;
pub mod intent_classifier;
pub mod intent_compiler;
pub mod safety_layer;

pub use app_generator::{
    AppGenerator, AppStructure, GeneratedApp, GeneratedPage, GeneratedScript, PageType, ScriptType,
    SyncResult,
};
pub use ask_later::{ask_later_keyword, PendingInfoItem};
pub use auto_task::{AutoTask, AutoTaskStatus, ExecutionMode, TaskPriority};
pub use autotask_api::{
    apply_recommendation_handler, cancel_task_handler, classify_intent_handler,
    compile_intent_handler, create_and_execute_handler, execute_plan_handler, execute_task_handler,
    get_approvals_handler, get_decisions_handler, get_pending_items_handler, get_stats_handler,
    get_task_logs_handler, list_tasks_handler, pause_task_handler, resume_task_handler,
    simulate_plan_handler, simulate_task_handler, submit_approval_handler, submit_decision_handler,
    submit_pending_item_handler,
};
pub use designer_ai::DesignerAI;
pub use intent_classifier::{ClassifiedIntent, IntentClassifier, IntentType};
pub use intent_compiler::{CompiledIntent, IntentCompiler};
pub use safety_layer::{AuditEntry, ConstraintCheckResult, SafetyLayer, SimulationResult};

pub fn configure_autotask_routes() -> axum::Router<std::sync::Arc<crate::shared::state::AppState>> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/autotask/create", post(create_and_execute_handler))
        .route("/api/autotask/classify", post(classify_intent_handler))
        .route("/api/autotask/compile", post(compile_intent_handler))
        .route("/api/autotask/execute", post(execute_plan_handler))
        .route(
            "/api/autotask/simulate/:plan_id",
            post(simulate_plan_handler),
        )
        .route("/api/autotask/list", get(list_tasks_handler))
        .route("/api/autotask/stats", get(get_stats_handler))
        .route("/api/autotask/:task_id/pause", post(pause_task_handler))
        .route("/api/autotask/:task_id/resume", post(resume_task_handler))
        .route("/api/autotask/:task_id/cancel", post(cancel_task_handler))
        .route(
            "/api/autotask/:task_id/simulate",
            post(simulate_task_handler),
        )
        .route(
            "/api/autotask/:task_id/decisions",
            get(get_decisions_handler),
        )
        .route(
            "/api/autotask/:task_id/decide",
            post(submit_decision_handler),
        )
        .route(
            "/api/autotask/:task_id/approvals",
            get(get_approvals_handler),
        )
        .route(
            "/api/autotask/:task_id/approve",
            post(submit_approval_handler),
        )
        .route("/api/autotask/:task_id/execute", post(execute_task_handler))
        .route("/api/autotask/:task_id/logs", get(get_task_logs_handler))
        .route(
            "/api/autotask/recommendations/:rec_id/apply",
            post(apply_recommendation_handler),
        )
        .route("/api/autotask/pending", get(get_pending_items_handler))
        .route(
            "/api/autotask/pending/:item_id",
            post(submit_pending_item_handler),
        )
}
