use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use super::sync_service::ExternalSyncService;
use super::sync_types::*;

pub fn external_sync_routes() -> Router<Arc<crate::CrateState>> {
    Router::new()
        .route("/sync/accounts", get(list_accounts_handler).post(connect_account_handler))
        .route("/sync/accounts/{account_id}", get(get_account_handler).delete(disconnect_account_handler))
        .route("/sync/accounts/{account_id}/sync", post(start_sync_handler))
        .route("/sync/accounts/{account_id}/history", get(get_sync_history_handler))
        .route("/sync/accounts/{account_id}/conflicts", get(get_conflicts_handler))
        .route("/sync/mappings/{mapping_id}/resolve", post(resolve_conflict_handler))
}

fn make_service(_state: &crate::CrateState) -> ExternalSyncService {
    ExternalSyncService::new(None, None)
}

async fn list_accounts_handler(
    State(state): State<Arc<crate::CrateState>>,
) -> impl IntoResponse {
    let service = make_service(&state);
    let (org_id, _) = state.get_bot_context();
    match service.list_accounts(org_id).await {
        Ok(accounts) => Json(accounts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_account_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = make_service(&state);
    match service.get_account(account_id).await {
        Ok(account) => Json(account).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn connect_account_handler(
    State(state): State<Arc<crate::CrateState>>,
    Json(request): Json<ConnectAccountRequest>,
) -> impl IntoResponse {
    let service = make_service(&state);
    let (org_id, user_id) = state.get_bot_context();
    match service.connect_account(org_id, user_id, &request).await {
        Ok(account) => Json(account).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn disconnect_account_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = make_service(&state);
    let (org_id, _) = state.get_bot_context();
    match service.disconnect_account(org_id, account_id).await {
        Ok(()) => Json(serde_json::json!({"success": true})).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn start_sync_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(account_id): Path<Uuid>,
    Json(request): Json<StartSyncRequest>,
) -> impl IntoResponse {
    let service = make_service(&state);
    let (org_id, _) = state.get_bot_context();
    match service.start_sync(org_id, account_id, &request, SyncTrigger::Manual).await {
        Ok(history) => Json(history).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_sync_history_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = make_service(&state);
    match service.get_sync_history(account_id).await {
        Ok(history) => Json(history).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_conflicts_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = make_service(&state);
    match service.get_conflicts(account_id).await {
        Ok(conflicts) => Json(conflicts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn resolve_conflict_handler(
    State(state): State<Arc<crate::CrateState>>,
    Path(mapping_id): Path<Uuid>,
    Json(request): Json<ResolveConflictRequest>,
) -> impl IntoResponse {
    let service = make_service(&state);
    let (org_id, _) = state.get_bot_context();
    match service.resolve_conflict(org_id, mapping_id, &request).await {
        Ok(mapping) => Json(mapping).into_response(),
        Err(e) => e.into_response(),
    }
}
