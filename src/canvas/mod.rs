use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub elements: Vec<CanvasElement>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_public: bool,
    pub collaborators: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasElement {
    pub id: Uuid,
    pub element_type: ElementType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub properties: ElementProperties,
    pub z_index: i32,
    pub locked: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    FreehandPath,
    Text,
    Image,
    Sticky,
    Frame,
    Connector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementProperties {
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub text_align: Option<String>,
    pub image_url: Option<String>,
    pub path_data: Option<String>,
    pub corner_radius: Option<f64>,
    pub start_arrow: Option<String>,
    pub end_arrow: Option<String>,
}

impl Default for ElementProperties {
    fn default() -> Self {
        Self {
            fill_color: Some("#ffffff".to_string()),
            stroke_color: Some("#000000".to_string()),
            stroke_width: Some(2.0),
            opacity: Some(1.0),
            text: None,
            font_size: Some(16.0),
            font_family: Some("Inter".to_string()),
            text_align: Some("left".to_string()),
            image_url: None,
            path_data: None,
            corner_radius: Some(0.0),
            start_arrow: None,
            end_arrow: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub element_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub description: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background_color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background_color: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateElementRequest {
    pub element_type: ElementType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: Option<f64>,
    pub properties: Option<ElementProperties>,
    pub z_index: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateElementRequest {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
    pub properties: Option<ElementProperties>,
    pub z_index: Option<i32>,
    pub locked: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub scale: Option<f64>,
    pub background: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Png,
    Svg,
    Pdf,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub format: ExportFormat,
    pub url: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub selection: Vec<Uuid>,
    pub connected_at: DateTime<Utc>,
}

pub struct CanvasService {
    canvases: Arc<RwLock<HashMap<Uuid, Canvas>>>,
}

impl CanvasService {
    pub fn new() -> Self {
        Self {
            canvases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn list_canvases(&self, org_id: Uuid) -> Vec<CanvasSummary> {
        let canvases = self.canvases.read().await;
        canvases
            .values()
            .filter(|c| c.organization_id == org_id)
            .map(|c| CanvasSummary {
                id: c.id,
                name: c.name.clone(),
                description: c.description.clone(),
                thumbnail_url: None,
                element_count: c.elements.len(),
                created_at: c.created_at,
                updated_at: c.updated_at,
                is_public: c.is_public,
            })
            .collect()
    }

    pub async fn create_canvas(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        req: CreateCanvasRequest,
    ) -> Canvas {
        let now = Utc::now();
        let canvas = Canvas {
            id: Uuid::new_v4(),
            organization_id: org_id,
            name: req.name,
            description: req.description,
            width: req.width.unwrap_or(1920),
            height: req.height.unwrap_or(1080),
            background_color: req.background_color.unwrap_or_else(|| "#ffffff".to_string()),
            elements: vec![],
            created_by: user_id,
            created_at: now,
            updated_at: now,
            is_public: false,
            collaborators: vec![user_id],
        };

        let mut canvases = self.canvases.write().await;
        canvases.insert(canvas.id, canvas.clone());
        canvas
    }

    pub async fn get_canvas(&self, canvas_id: Uuid) -> Option<Canvas> {
        let canvases = self.canvases.read().await;
        canvases.get(&canvas_id).cloned()
    }

    pub async fn update_canvas(
        &self,
        canvas_id: Uuid,
        req: UpdateCanvasRequest,
    ) -> Option<Canvas> {
        let mut canvases = self.canvases.write().await;
        if let Some(canvas) = canvases.get_mut(&canvas_id) {
            if let Some(name) = req.name {
                canvas.name = name;
            }
            if let Some(desc) = req.description {
                canvas.description = Some(desc);
            }
            if let Some(width) = req.width {
                canvas.width = width;
            }
            if let Some(height) = req.height {
                canvas.height = height;
            }
            if let Some(bg) = req.background_color {
                canvas.background_color = bg;
            }
            if let Some(public) = req.is_public {
                canvas.is_public = public;
            }
            canvas.updated_at = Utc::now();
            return Some(canvas.clone());
        }
        None
    }

    pub async fn delete_canvas(&self, canvas_id: Uuid) -> bool {
        let mut canvases = self.canvases.write().await;
        canvases.remove(&canvas_id).is_some()
    }

    pub async fn add_element(
        &self,
        canvas_id: Uuid,
        user_id: Uuid,
        req: CreateElementRequest,
    ) -> Option<CanvasElement> {
        let mut canvases = self.canvases.write().await;
        if let Some(canvas) = canvases.get_mut(&canvas_id) {
            let now = Utc::now();
            let element = CanvasElement {
                id: Uuid::new_v4(),
                element_type: req.element_type,
                x: req.x,
                y: req.y,
                width: req.width,
                height: req.height,
                rotation: req.rotation.unwrap_or(0.0),
                properties: req.properties.unwrap_or_default(),
                z_index: req.z_index.unwrap_or(canvas.elements.len() as i32),
                locked: false,
                created_by: user_id,
                created_at: now,
                updated_at: now,
            };
            canvas.elements.push(element.clone());
            canvas.updated_at = now;
            return Some(element);
        }
        None
    }

    pub async fn update_element(
        &self,
        canvas_id: Uuid,
        element_id: Uuid,
        req: UpdateElementRequest,
    ) -> Option<CanvasElement> {
        let mut canvases = self.canvases.write().await;
        if let Some(canvas) = canvases.get_mut(&canvas_id) {
            if let Some(element) = canvas.elements.iter_mut().find(|e| e.id == element_id) {
                if let Some(x) = req.x {
                    element.x = x;
                }
                if let Some(y) = req.y {
                    element.y = y;
                }
                if let Some(width) = req.width {
                    element.width = width;
                }
                if let Some(height) = req.height {
                    element.height = height;
                }
                if let Some(rotation) = req.rotation {
                    element.rotation = rotation;
                }
                if let Some(props) = req.properties {
                    element.properties = props;
                }
                if let Some(z) = req.z_index {
                    element.z_index = z;
                }
                if let Some(locked) = req.locked {
                    element.locked = locked;
                }
                element.updated_at = Utc::now();
                canvas.updated_at = Utc::now();
                return Some(element.clone());
            }
        }
        None
    }

    pub async fn delete_element(&self, canvas_id: Uuid, element_id: Uuid) -> bool {
        let mut canvases = self.canvases.write().await;
        if let Some(canvas) = canvases.get_mut(&canvas_id) {
            let len_before = canvas.elements.len();
            canvas.elements.retain(|e| e.id != element_id);
            if canvas.elements.len() < len_before {
                canvas.updated_at = Utc::now();
                return true;
            }
        }
        false
    }

    pub async fn export_canvas(
        &self,
        canvas_id: Uuid,
        req: ExportRequest,
    ) -> Option<ExportResponse> {
        let canvases = self.canvases.read().await;
        let canvas = canvases.get(&canvas_id)?;

        match req.format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(canvas).ok()?;
                Some(ExportResponse {
                    format: ExportFormat::Json,
                    url: None,
                    data: Some(json),
                })
            }
            ExportFormat::Svg => {
                let svg = generate_svg(canvas, req.background.unwrap_or(true));
                Some(ExportResponse {
                    format: ExportFormat::Svg,
                    url: None,
                    data: Some(svg),
                })
            }
            _ => Some(ExportResponse {
                format: req.format,
                url: Some(format!("/api/canvas/{}/export/file", canvas_id)),
                data: None,
            }),
        }
    }
}

impl Default for CanvasService {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_svg(canvas: &Canvas, include_background: bool) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        canvas.width, canvas.height, canvas.width, canvas.height
    );

    if include_background {
        svg.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            canvas.background_color
        ));
    }

    for element in &canvas.elements {
        let transform = if element.rotation != 0.0 {
            format!(
                r#" transform="rotate({} {} {})""#,
                element.rotation,
                element.x + element.width / 2.0,
                element.y + element.height / 2.0
            )
        } else {
            String::new()
        };

        let fill = element
            .properties
            .fill_color
            .as_deref()
            .unwrap_or("transparent");
        let stroke = element
            .properties
            .stroke_color
            .as_deref()
            .unwrap_or("none");
        let stroke_width = element.properties.stroke_width.unwrap_or(1.0);
        let opacity = element.properties.opacity.unwrap_or(1.0);

        match element.element_type {
            ElementType::Rectangle => {
                let radius = element.properties.corner_radius.unwrap_or(0.0);
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                    element.x, element.y, element.width, element.height,
                    radius, fill, stroke, stroke_width, opacity, transform
                ));
            }
            ElementType::Ellipse => {
                svg.push_str(&format!(
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                    element.x + element.width / 2.0,
                    element.y + element.height / 2.0,
                    element.width / 2.0,
                    element.height / 2.0,
                    fill, stroke, stroke_width, opacity, transform
                ));
            }
            ElementType::Text => {
                let text = element.properties.text.as_deref().unwrap_or("");
                let font_size = element.properties.font_size.unwrap_or(16.0);
                let font_family = element
                    .properties
                    .font_family
                    .as_deref()
                    .unwrap_or("sans-serif");
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}" opacity="{}"{}>
                        {}
                    </text>"#,
                    element.x, element.y + font_size, font_size, font_family,
                    fill, opacity, transform, text
                ));
            }
            ElementType::FreehandPath => {
                if let Some(path_data) = &element.properties.path_data {
                    svg.push_str(&format!(
                        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                        path_data, stroke, stroke_width, opacity, transform
                    ));
                }
            }
            ElementType::Line | ElementType::Arrow => {
                let marker = if element.element_type == ElementType::Arrow {
                    r#" marker-end="url(#arrowhead)""#
                } else {
                    ""
                };
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" opacity="{}"{}{}/>"#,
                    element.x, element.y,
                    element.x + element.width, element.y + element.height,
                    stroke, stroke_width, opacity, marker, transform
                ));
            }
            _ => {}
        }
    }

    svg.push_str("</svg>");
    svg
}

#[derive(Debug, Serialize)]
pub struct CanvasError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for CanvasError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": self.error, "code": self.code})),
        )
            .into_response()
    }
}

async fn list_canvases(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<CanvasSummary>>, CanvasError> {
    let service = CanvasService::new();
    let org_id = Uuid::nil();
    let canvases = service.list_canvases(org_id).await;
    Ok(Json(canvases))
}

async fn create_canvas(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let canvas = service.create_canvas(org_id, user_id, req).await;
    Ok(Json(canvas))
}

async fn get_canvas(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new();
    let canvas = service.get_canvas(canvas_id).await.ok_or_else(|| CanvasError {
        error: "Canvas not found".to_string(),
        code: "CANVAS_NOT_FOUND".to_string(),
    })?;
    Ok(Json(canvas))
}

async fn update_canvas(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<UpdateCanvasRequest>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new();
    let canvas = service
        .update_canvas(canvas_id, req)
        .await
        .ok_or_else(|| CanvasError {
            error: "Canvas not found".to_string(),
            code: "CANVAS_NOT_FOUND".to_string(),
        })?;
    Ok(Json(canvas))
}

async fn delete_canvas(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<StatusCode, CanvasError> {
    let service = CanvasService::new();
    if service.delete_canvas(canvas_id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(CanvasError {
            error: "Canvas not found".to_string(),
            code: "CANVAS_NOT_FOUND".to_string(),
        })
    }
}

async fn list_elements(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<CanvasElement>>, CanvasError> {
    let service = CanvasService::new();
    let canvas = service.get_canvas(canvas_id).await.ok_or_else(|| CanvasError {
        error: "Canvas not found".to_string(),
        code: "CANVAS_NOT_FOUND".to_string(),
    })?;
    Ok(Json(canvas.elements))
}

async fn create_element(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<CreateElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new();
    let user_id = Uuid::nil();
    let element = service
        .add_element(canvas_id, user_id, req)
        .await
        .ok_or_else(|| CanvasError {
            error: "Canvas not found".to_string(),
            code: "CANVAS_NOT_FOUND".to_string(),
        })?;
    Ok(Json(element))
}

async fn update_element(
    State(_state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new();
    let element = service
        .update_element(canvas_id, element_id, req)
        .await
        .ok_or_else(|| CanvasError {
            error: "Element not found".to_string(),
            code: "ELEMENT_NOT_FOUND".to_string(),
        })?;
    Ok(Json(element))
}

async fn delete_element(
    State(_state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, CanvasError> {
    let service = CanvasService::new();
    if service.delete_element(canvas_id, element_id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(CanvasError {
            error: "Element not found".to_string(),
            code: "ELEMENT_NOT_FOUND".to_string(),
        })
    }
}

async fn export_canvas(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, CanvasError> {
    let service = CanvasService::new();
    let response = service
        .export_canvas(canvas_id, req)
        .await
        .ok_or_else(|| CanvasError {
            error: "Canvas not found".to_string(),
            code: "CANVAS_NOT_FOUND".to_string(),
        })?;
    Ok(Json(response))
}

async fn get_collaboration_info(
    State(_state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<CollaborationSession>>, CanvasError> {
    let _ = canvas_id;
    Ok(Json(vec![]))
}

pub fn configure_canvas_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/canvas", get(list_canvases).post(create_canvas))
        .route(
            "/api/canvas/:canvas_id",
            get(get_canvas).put(update_canvas).delete(delete_canvas),
        )
        .route(
            "/api/canvas/:canvas_id/elements",
            get(list_elements).post(create_element),
        )
        .route(
            "/api/canvas/:canvas_id/elements/:element_id",
            put(update_element).delete(delete_element),
        )
        .route("/api/canvas/:canvas_id/export", post(export_canvas))
        .route(
            "/api/canvas/:canvas_id/collaborate",
            get(get_collaboration_info),
        )
}
