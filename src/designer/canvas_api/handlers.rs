use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;
use crate::designer::canvas_api::service::CanvasService;
use crate::designer::canvas_api::types::*;
use crate::designer::canvas_api::error::CanvasError;

pub fn canvas_routes(state: Arc<AppState>) -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/", post(create_canvas_handler))
        .route("/:id", get(get_canvas_handler))
        .route("/:id/elements", post(add_element_handler))
        .route("/:id/elements/:eid", put(update_element_handler))
        .route("/:id/elements/:eid", delete(delete_element_handler))
        .route("/:id/group", post(group_elements_handler))
        .route("/:id/layers", post(add_layer_handler))
        .route("/:id/export", post(export_canvas_handler))
        .route("/templates", get(get_templates_handler))
        .route("/assets", get(get_assets_handler))
        .with_state(state)
}

async fn create_canvas_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let organization_id = Uuid::nil();
    let user_id = Uuid::nil();
    let canvas = service.create_canvas(organization_id, user_id, request).await?;
    Ok(Json(canvas))
}

async fn get_canvas_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let canvas = service.get_canvas(canvas_id).await?;
    Ok(Json(canvas))
}

async fn add_element_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(request): Json<AddElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let user_id = Uuid::nil();
    let element = service.add_element(canvas_id, user_id, request).await?;
    Ok(Json(element))
}

async fn update_element_handler(
    State(state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let user_id = Uuid::nil();
    let element = service.update_element(canvas_id, element_id, user_id, request).await?;
    Ok(Json(element))
}

async fn delete_element_handler(
    State(state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let user_id = Uuid::nil();
    service.delete_element(canvas_id, element_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn group_elements_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(request): Json<GroupElementsRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let user_id = Uuid::nil();
    let group = service.group_elements(canvas_id, user_id, request).await?;
    Ok(Json(group))
}

async fn add_layer_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(request): Json<CreateLayerRequest>,
) -> Result<Json<Layer>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let user_id = Uuid::nil();
    let layer = service.add_layer(canvas_id, user_id, request).await?;
    Ok(Json(layer))
}

async fn export_canvas_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ExportResult>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let result = service.export_canvas(canvas_id, request).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct TemplatesQuery {
    category: Option<String>,
}

async fn get_templates_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TemplatesQuery>,
) -> Result<Json<Vec<CanvasTemplate>>, CanvasError> {
    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let templates = service.get_templates(query.category).await?;
    Ok(Json(templates))
}

#[derive(Debug, Deserialize)]
struct AssetsQuery {
    asset_type: Option<String>,
}

async fn get_assets_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssetsQuery>,
) -> Result<Json<Vec<AssetLibraryItem>>, CanvasError> {
    let asset_type = query.asset_type.and_then(|t| match t.as_str() {
        "icon" => Some(AssetType::Icon),
        "image" => Some(AssetType::Image),
        "illustration" => Some(AssetType::Illustration),
        "shape" => Some(AssetType::Shape),
        "component" => Some(AssetType::Component),
        _ => None,
    });

    let service = CanvasService::new(Arc::new(state.conn.clone()));
    let assets = service.get_asset_library(asset_type).await?;
    Ok(Json(assets))
}
