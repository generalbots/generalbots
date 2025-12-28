pub mod app_generator;
pub mod app_logs;
pub mod ask_later;
pub mod autotask_api;
pub mod designer_ai;
pub mod intent_classifier;
pub mod intent_compiler;
pub mod safety_layer;
pub mod task_types;

pub use app_generator::{
    AppGenerator, AppStructure, FileType, GeneratedApp, GeneratedFile, GeneratedPage, PageType,
    SyncResult,
};
pub use app_logs::{
    generate_client_logger_js, get_designer_error_context, log_generator_error, log_generator_info,
    log_runtime_error, log_validation_error, start_log_cleanup_scheduler, AppLogEntry, AppLogStore,
    ClientLogRequest, LogLevel, LogQueryParams, LogSource, LogStats, APP_LOGS,
};
pub use ask_later::{ask_later_keyword, PendingInfoItem};
pub use autotask_api::{
    apply_recommendation_handler, cancel_task_handler, classify_intent_handler,
    compile_intent_handler, create_and_execute_handler, execute_plan_handler, execute_task_handler,
    get_approvals_handler, get_decisions_handler, get_pending_items_handler, get_stats_handler,
    get_task_logs_handler, list_tasks_handler, pause_task_handler, resume_task_handler,
    simulate_plan_handler, simulate_task_handler, submit_approval_handler, submit_decision_handler,
    submit_pending_item_handler,
};
pub use designer_ai::DesignerAI;
pub use task_types::{AutoTask, AutoTaskStatus, ExecutionMode, TaskPriority};
pub use intent_classifier::{ClassifiedIntent, IntentClassifier, IntentType};
pub use intent_compiler::{CompiledIntent, IntentCompiler};
pub use safety_layer::{AuditEntry, ConstraintCheckResult, SafetyLayer, SimulationResult};

use crate::core::urls::ApiUrls;

pub fn configure_autotask_routes() -> axum::Router<std::sync::Arc<crate::shared::state::AppState>> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route(ApiUrls::AUTOTASK_CREATE, post(create_and_execute_handler))
        .route(ApiUrls::AUTOTASK_CLASSIFY, post(classify_intent_handler))
        .route(ApiUrls::AUTOTASK_COMPILE, post(compile_intent_handler))
        .route(ApiUrls::AUTOTASK_EXECUTE, post(execute_plan_handler))
        .route(
            &ApiUrls::AUTOTASK_SIMULATE.replace(":plan_id", "{plan_id}"),
            post(simulate_plan_handler),
        )
        .route(ApiUrls::AUTOTASK_LIST, get(list_tasks_handler))
        .route(ApiUrls::AUTOTASK_STATS, get(get_stats_handler))
        .route(
            &ApiUrls::AUTOTASK_PAUSE.replace(":task_id", "{task_id}"),
            post(pause_task_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_RESUME.replace(":task_id", "{task_id}"),
            post(resume_task_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_CANCEL.replace(":task_id", "{task_id}"),
            post(cancel_task_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_TASK_SIMULATE.replace(":task_id", "{task_id}"),
            post(simulate_task_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_DECISIONS.replace(":task_id", "{task_id}"),
            get(get_decisions_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_DECIDE.replace(":task_id", "{task_id}"),
            post(submit_decision_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_APPROVALS.replace(":task_id", "{task_id}"),
            get(get_approvals_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_APPROVE.replace(":task_id", "{task_id}"),
            post(submit_approval_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_TASK_EXECUTE.replace(":task_id", "{task_id}"),
            post(execute_task_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_LOGS.replace(":task_id", "{task_id}"),
            get(get_task_logs_handler),
        )
        .route(
            &ApiUrls::AUTOTASK_RECOMMENDATIONS_APPLY.replace(":rec_id", "{rec_id}"),
            post(apply_recommendation_handler),
        )
        .route(ApiUrls::AUTOTASK_PENDING, get(get_pending_items_handler))
        .route(
            &ApiUrls::AUTOTASK_PENDING_ITEM.replace(":item_id", "{item_id}"),
            post(submit_pending_item_handler),
        )
        .route("/api/app-logs/client", post(handle_client_logs))
        .route("/api/app-logs/list", get(handle_list_logs))
        .route("/api/app-logs/stats", get(handle_log_stats))
        .route("/api/app-logs/clear/{app_name}", post(handle_clear_logs))
        .route("/api/app-logs/logger.js", get(handle_logger_js))
}

async fn handle_client_logs(
    axum::Json(payload): axum::Json<ClientLogsPayload>,
) -> impl axum::response::IntoResponse {
    for log in payload.logs {
        APP_LOGS.log_client(log, None, None);
    }
    axum::Json(serde_json::json!({"success": true}))
}

#[derive(serde::Deserialize)]
struct ClientLogsPayload {
    logs: Vec<ClientLogRequest>,
}

async fn handle_list_logs(
    axum::extract::Query(params): axum::extract::Query<LogQueryParams>,
) -> impl axum::response::IntoResponse {
    let logs = APP_LOGS.get_logs(&params);
    axum::Json(logs)
}

async fn handle_log_stats() -> impl axum::response::IntoResponse {
    let stats = APP_LOGS.get_stats();
    axum::Json(stats)
}

async fn handle_clear_logs(
    axum::extract::Path(app_name): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    APP_LOGS.clear_app_logs(&app_name);
    axum::Json(
        serde_json::json!({"success": true, "message": format!("Logs cleared for {}", app_name)}),
    )
}

async fn handle_logger_js() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        generate_client_logger_js(),
    )
}
