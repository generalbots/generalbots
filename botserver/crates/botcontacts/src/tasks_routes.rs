use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use super::tasks_service::TasksIntegrationService;
use super::tasks_types::*;

pub fn tasks_integration_routes() -> Router<Arc<crate::CrateState>> {
    Router::new()
        .route("/tasks/{task_id}/contacts", get(get_task_contacts_handler).post(assign_contact_handler))
        .route("/tasks/{task_id}/contacts/bulk", post(bulk_assign_contacts_handler))
        .route("/tasks/{task_id}/contacts/{contact_id}", delete(unassign_contact_handler).post(update_task_contact_handler))
        .route("/tasks/{task_id}/contacts/suggestions", get(get_task_suggestions_handler))
        .route("/contacts/{contact_id}/tasks", get(get_contact_tasks_handler))
        .route("/contacts/{contact_id}/tasks/stats", get(get_contact_task_stats_handler))
        .route("/contacts/{contact_id}/tasks/workload", get(get_contact_workload_handler))
        .route("/contacts/{contact_id}/tasks/create", post(create_task_for_contact_handler))
}

async fn get_task_contacts_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(task_id): Path<Uuid>,
    Query(query): Query<TaskContactsQuery>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.get_task_contacts(org_id, task_id, &query).await {
        Ok(contacts) => Json(contacts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn assign_contact_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<AssignContactRequest>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, user_id) = state.get_bot_context();
    match service.assign_contact_to_task(org_id, task_id, &request, user_id).await {
        Ok(tc) => Json(tc).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn bulk_assign_contacts_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<BulkAssignContactsRequest>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, user_id) = state.get_bot_context();
    match service.bulk_assign_contacts(org_id, task_id, &request, user_id).await {
        Ok(results) => Json(results).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn unassign_contact_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path((task_id, contact_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.unassign_contact_from_task(org_id, task_id, contact_id).await {
        Ok(()) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn update_task_contact_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path((task_id, contact_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateTaskContactRequest>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.update_task_contact(org_id, task_id, contact_id, &request).await {
        Ok(tc) => Json(tc).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_task_suggestions_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(task_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.get_suggested_contacts(org_id, task_id, None).await {
        Ok(suggestions) => Json(suggestions).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_contact_tasks_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(contact_id): Path<Uuid>,
    Query(query): Query<ContactTasksQuery>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.get_contact_tasks(org_id, contact_id, &query).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_contact_task_stats_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(contact_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.get_contact_task_stats(org_id, contact_id).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_contact_workload_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(contact_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, _) = state.get_bot_context();
    match service.get_contact_workload(org_id, contact_id).await {
        Ok(workload) => Json(workload).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn create_task_for_contact_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(contact_id): Path<Uuid>,
    Json(request): Json<CreateTaskForContactRequest>,
) -> impl IntoResponse {
    let service = TasksIntegrationService::new(state.db_pool.clone());
    let (org_id, user_id) = state.get_bot_context();
    match service.create_task_for_contact(org_id, contact_id, &request, user_id).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}
