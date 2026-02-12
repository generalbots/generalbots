use super::service::ContactsService;
use super::types::*;
use super::error::ContactsError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

pub fn contacts_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_contacts_handler))
        .route("/", post(create_contact_handler))
        .route("/:id", get(get_contact_handler))
        .route("/:id", put(update_contact_handler))
        .route("/:id", delete(delete_contact_handler))
        .route("/import", post(import_contacts_handler))
        .route("/export", post(export_contacts_handler))
        .with_state(state)
}

pub async fn list_contacts_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ContactListQuery>,
) -> Result<Json<ContactListResponse>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let response = service.list_contacts(organization_id, query).await?;
    Ok(Json(response))
}

pub async fn create_contact_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateContactRequest>,
) -> Result<Json<Contact>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let contact = service.create_contact(organization_id, None, request).await?;
    Ok(Json(contact))
}

pub async fn get_contact_handler(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<Contact>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let contact = service.get_contact(organization_id, contact_id).await?;
    Ok(Json(contact))
}

pub async fn update_contact_handler(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    Json(request): Json<UpdateContactRequest>,
) -> Result<Json<Contact>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let contact = service.update_contact(organization_id, contact_id, request, None).await?;
    Ok(Json(contact))
}

pub async fn delete_contact_handler(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
) -> Result<StatusCode, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    service.delete_contact(organization_id, contact_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn import_contacts_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImportRequest>,
) -> Result<Json<ImportResult>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let result = service.import_contacts(organization_id, None, request).await?;
    Ok(Json(result))
}

pub async fn export_contacts_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ExportResult>, ContactsError> {
    let organization_id = Uuid::nil();
    let service = ContactsService::new(Arc::new(state.conn.clone()));
    let result = service.export_contacts(organization_id, request).await?;
    Ok(Json(result))
}
