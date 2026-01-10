use crate::shared::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use ppt_rs::{Pptx, Slide as PptSlide, TextBox, Shape, ShapeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

type SlideChannels = Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<SlideMessage>>>>;

static SLIDE_CHANNELS: std::sync::OnceLock<SlideChannels> = std::sync::OnceLock::new();

fn get_slide_channels() -> &'static SlideChannels {
    SLIDE_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideMessage {
    pub msg_type: String,
    pub presentation_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slide_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub slides: Vec<Slide>,
    pub theme: PresentationTheme,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: String,
    pub layout: String,
    pub elements: Vec<SlideElement>,
    pub background: SlideBackground,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<SlideTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideElement {
    pub id: String,
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub rotation: f64,
    pub content: ElementContent,
    pub style: ElementStyle,
    #[serde(default)]
    pub animations: Vec<Animation>,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_data: Option<TableData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideBackground {
    pub bg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient: Option<GradientStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_fit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStyle {
    pub gradient_type: String,
    pub angle: f64,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub color: String,
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideTransition {
    pub transition_type: String,
    pub duration: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub animation_type: String,
    pub trigger: String,
    pub duration: f64,
    pub delay: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationTheme {
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub text_light: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: String,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub rows: usize,
    pub cols: usize,
    pub cells: HashMap<String, TableCell>,
    pub col_widths: Vec<f64>,
    pub row_heights: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colspan: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rowspan: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ElementStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationMetadata {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub slide_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePresentationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub slides: Vec<Slide>,
    pub theme: PresentationTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadQuery {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSlideRequest {
    pub presentation_id: String,
    pub layout: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSlideRequest {
    pub presentation_id: String,
    pub slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSlideRequest {
    pub presentation_id: String,
    pub slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderSlidesRequest {
    pub presentation_id: String,
    pub slide_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element: SlideElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element: SlideElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyThemeRequest {
    pub presentation_id: String,
    pub theme: PresentationTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSlideNotesRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn configure_slides_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/slides/list", get(handle_list_presentations))
        .route("/api/slides/search", get(handle_search_presentations))
        .route("/api/slides/load", get(handle_load_presentation))
        .route("/api/slides/save", post(handle_save_presentation))
        .route("/api/slides/delete", post(handle_delete_presentation))
        .route("/api/slides/new", get(handle_new_presentation))
        .route("/api/slides/slide/add", post(handle_add_slide))
        .route("/api/slides/slide/delete", post(handle_delete_slide))
        .route("/api/slides/slide/duplicate", post(handle_duplicate_slide))
        .route("/api/slides/slide/reorder", post(handle_reorder_slides))
        .route("/api/slides/slide/notes", post(handle_update_slide_notes))
        .route("/api/slides/element/add", post(handle_add_element))
        .route("/api/slides/element/update", post(handle_update_element))
        .route("/api/slides/element/delete", post(handle_delete_element))
        .route("/api/slides/theme/apply", post(handle_apply_theme))
        .route("/api/slides/export", post(handle_export_presentation))
        .route("/api/slides/:id", get(handle_get_presentation_by_id))
        .route("/api/slides/:id/collaborators", get(handle_get_collaborators))
        .route("/ws/slides/:presentation_id", get(handle_slides_websocket))
}

fn get_user_presentations_path(user_id: &str) -> String {
    format!("users/{}/presentations", user_id)
}

fn get_current_user_id() -> String {
    "default-user".to_string()
}

async fn save_presentation_to_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation: &Presentation,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation.id
    );
    let content = serde_json::to_string_pretty(presentation)
        .map_err(|e| format!("Serialization error: {e}"))?;

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(content.into_bytes().into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save presentation: {e}"))?;

    Ok(())
}

async fn load_presentation_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation_id: &str,
) -> Result<Presentation, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to load presentation: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read presentation: {e}"))?
        .into_bytes();

    let presentation: Presentation = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Failed to parse presentation: {e}"))?;

    Ok(presentation)
}

async fn list_presentations_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<Vec<PresentationMetadata>, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let prefix = format!("{}/", get_user_presentations_path(user_id));

    let result = drive
        .list_objects_v2()
        .bucket("gbo")
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("Failed to list presentations: {e}"))?;

    let mut presentations = Vec::new();

    if let Some(contents) = result.contents {
        for obj in contents {
            if let Some(key) = obj.key {
                if key.ends_with(".json") {
                    let id = key
                        .split('/')
                        .last()
                        .unwrap_or("")
                        .trim_end_matches(".json")
                        .to_string();
                    if let Ok(pres) = load_presentation_from_drive(state, user_id, &id).await {
                        presentations.push(PresentationMetadata {
                            id: pres.id,
                            name: pres.name,
                            owner_id: pres.owner_id,
                            slide_count: pres.slides.len(),
                            created_at: pres.created_at,
                            updated_at: pres.updated_at,
                        });
                    }
                }
            }
        }
    }

    presentations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(presentations)
}

async fn delete_presentation_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation_id: &str,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );

    drive
        .delete_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to delete presentation: {e}"))?;

    Ok(())
}

fn create_default_theme() -> PresentationTheme {
    PresentationTheme {
        name: "Default".to_string(),
        colors: ThemeColors {
            primary: "#3b82f6".to_string(),
            secondary: "#64748b".to_string(),
            accent: "#f59e0b".to_string(),
            background: "#ffffff".to_string(),
            text: "#1e293b".to_string(),
            text_light: "#64748b".to_string(),
        },
        fonts: ThemeFonts {
            heading: "Inter".to_string(),
            body: "Inter".to_string(),
        },
    }
}

fn create_title_slide() -> Slide {
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "title".to_string(),
        elements: vec![
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 100.0,
                y: 200.0,
                width: 760.0,
                height: 100.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Presentation Title".to_string()),
                    html: None,
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    font_size: Some(48.0),
                    font_weight: Some("bold".to_string()),
                    text_align: Some("center".to_string()),
                    color: Some("#1e293b".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 1,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 100.0,
                y: 320.0,
                width: 760.0,
                height: 50.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Subtitle or Author Name".to_string()),
                    html: None,
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    font_size: Some(24.0),
                    text_align: Some("center".to_string()),
                    color: Some("#64748b".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 2,
                locked: false,
            },
        ],
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some("#ffffff".to_string()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: Some(SlideTransition {
            transition_type: "fade".to_string(),
            duration: 0.5,
            direction: None,
        }),
    }
}

fn create_content_slide(layout: &str) -> Slide {
    let elements = match layout {
        "title-content" => vec![
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 40.0,
                width: 860.0,
                height: 60.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Slide Title".to_string()),
                    ..Default::default()
                },
                style: ElementStyle {
                    font_size: Some(36.0),
                    font_weight: Some("bold".to_string()),
                    color: Some("#1e293b".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 1,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 120.0,
                width: 860.0,
                height: 400.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("• Click to add content\n• Add your bullet points here".to_string()),
                    ..Default::default()
                },
                style: ElementStyle {
                    font_size: Some(20.0),
                    color: Some("#374151".to_string()),
                    line_height: Some(1.6),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 2,
                locked: false,
            },
        ],
        "two-column" => vec![
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 40.0,
                width: 860.0,
                height: 60.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Slide Title".to_string()),
                    ..Default::default()
                },
                style: ElementStyle {
                    font_size: Some(36.0),
                    font_weight: Some("bold".to_string()),
                    color: Some("#1e293b".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 1,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 120.0,
                width: 410.0,
                height: 400.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Left column content".to_string()),
                    ..Default::default()
                },
                style: ElementStyle {
                    font_size: Some(18.0),
                    color: Some("#374151".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 2,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 500.0,
                y: 120.0,
                width: 410.0,
                height: 400.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Right column content".to_string()),
                    ..Default::default()
                },
                style: ElementStyle {
                    font_size: Some(18.0),
                    color: Some("#374151".to_string()),
                    ..Default::default()
                },
                animations: vec![],
                z_index: 3,
                locked: false,
            },
        ],
        "section" => vec![SlideElement {
            id: Uuid::new_v4().to_string(),
            element_type: "text".to_string(),
            x: 100.0,
            y: 220.0,
            width: 760.0,
            height: 100.0,
            rotation: 0.0,
            content: ElementContent {
                text: Some("Section Title".to_string()),
                ..Default::default()
            },
            style: ElementStyle {
                font_size: Some(48.0),
                font_weight: Some("bold".to_string()),
                text_align: Some("center".to_string()),
                color: Some("#1e293b".to_string()),
                ..Default::default()
            },
            animations: vec![],
            z_index: 1,
            locked: false,
        }],
        _ => vec![],
    };

    Slide {
        id: Uuid::new_v4().to_string(),
        layout: layout.to_string(),
        elements,
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some("#ffffff".to_string()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: Some(SlideTransition {
            transition_type: "fade".to_string(),
            duration: 0.5,
            direction: None,
        }),
    }
}

pub async fn handle_new_presentation(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let presentation = Presentation {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Presentation".to_string(),
        owner_id: get_current_user_id(),
        slides: vec![create_title_slide()],
        theme: create_default_theme(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(Json(presentation))
}

pub async fn handle_list_presentations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match list_presentations_from_drive(&state, &user_id).await {
        Ok(presentations) => Ok(Json(presentations)),
        Err(e) => {
            error!("Failed to list presentations: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_presentations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let presentations = match list_presentations_from_drive(&state, &user_id).await {
        Ok(p) => p,
        Err(_) => Vec::new(),
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        presentations
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        presentations
    };

    Ok(Json(filtered))
}

pub async fn handle_load_presentation(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoadQuery>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match load_presentation_from_drive(&state, &user_id, &query.id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_save_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SavePresentationRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let presentation_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let presentation = Presentation {
        id: presentation_id.clone(),
        name: req.name,
        owner_id: user_id.clone(),
        slides: req.slides,
        theme: req.theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: presentation_id,
        success: true,
        message: Some("Presentation saved".to_string()),
    }))
}

pub async fn handle_delete_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoadQuery>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    if let Err(e) = delete_presentation_from_drive(&state, &user_id, &req.id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.id,
        success: true,
        message: Some("Presentation deleted".to_string()),
    }))
}

pub async fn handle_get_presentation_by_id(
    State(state): State<Arc<AppState>>,
    Path(presentation_id): Path<String>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    match load_presentation_from_drive(&state, &user_id, &presentation_id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    }
}

pub async fn handle_add_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    let new_slide = create_content_slide(&req.layout);
    let position = req.position.unwrap_or(presentation.slides.len());
    presentation.slides.insert(position.min(presentation.slides.len()), new_slide);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "slideAdded", &user_id, Some(position), None).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Slide added".to_string()) }))
}

pub async fn handle_delete_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    presentation.slides.remove(req.slide_index);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "slideDeleted", &user_id, Some(req.slide_index), None).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Slide deleted".to_string()) }))
}

pub async fn handle_duplicate_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DuplicateSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    let mut duplicated = presentation.slides[req.slide_index].clone();
    duplicated.id = Uuid::new_v4().to_string();
    for element in &mut duplicated.elements {
        element.id = Uuid::new_v4().to_string();
    }
    presentation.slides.insert(req.slide_index + 1, duplicated);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "slideDuplicated", &user_id, Some(req.slide_index), None).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Slide duplicated".to_string()) }))
}

pub async fn handle_reorder_slides(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReorderSlidesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    let mut new_slides = Vec::new();
    for slide_id in &req.slide_order {
        if let Some(slide) = presentation.slides.iter().find(|s| &s.id == slide_id) {
            new_slides.push(slide.clone());
        }
    }

    if new_slides.len() == presentation.slides.len() {
        presentation.slides = new_slides;
        presentation.updated_at = Utc::now();

        if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
        }
    }

    broadcast_slide_change(&req.presentation_id, "slidesReordered", &user_id, None, None).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Slides reordered".to_string()) }))
}

pub async fn handle_update_slide_notes(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateSlideNotesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    presentation.slides[req.slide_index].notes = if req.notes.is_empty() { None } else { Some(req.notes) };
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Notes updated".to_string()) }))
}

pub async fn handle_add_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    presentation.slides[req.slide_index].elements.push(req.element.clone());
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "elementAdded", &user_id, Some(req.slide_index), Some(&req.element.id)).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Element added".to_string()) }))
}

pub async fn handle_update_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    let slide = &mut presentation.slides[req.slide_index];
    if let Some(pos) = slide.elements.iter().position(|e| e.id == req.element.id) {
        slide.elements[pos] = req.element.clone();
    } else {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Element not found" }))));
    }

    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "elementUpdated", &user_id, Some(req.slide_index), Some(&req.element.id)).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Element updated".to_string()) }))
}

pub async fn handle_delete_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid slide index" }))));
    }

    let slide = &mut presentation.slides[req.slide_index];
    slide.elements.retain(|e| e.id != req.element_id);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "elementDeleted", &user_id, Some(req.slide_index), Some(&req.element_id)).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Element deleted".to_string()) }))
}

pub async fn handle_apply_theme(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApplyThemeRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_from_drive(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    presentation.theme = req.theme;
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_slide_change(&req.presentation_id, "themeChanged", &user_id, None, None).await;
    Ok(Json(SaveResponse { id: req.presentation_id, success: true, message: Some("Theme applied".to_string()) }))
}

pub async fn handle_export_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let presentation = match load_presentation_from_drive(&state, &user_id, &req.id).await {
        Ok(p) => p,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    match req.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&presentation).unwrap_or_default();
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        "html" => {
            let html = export_to_html(&presentation);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html))
        }
        "pptx" => {
            match export_to_pptx(&presentation) {
                Ok(bytes) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    Ok(([(axum::http::header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.presentationml.presentation")], encoded))
                }
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e })))),
            }
        }
        _ => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Unsupported format" })))),
    }
}

fn export_to_pptx(presentation: &Presentation) -> Result<Vec<u8>, String> {
    let mut pptx = Pptx::new();

    for slide in &presentation.slides {
        let mut ppt_slide = PptSlide::new();

        for element in &slide.elements {
            match element.element_type.as_str() {
                "text" => {
                    let content = element.content.as_deref().unwrap_or("");
                    let x = element.x as f64;
                    let y = element.y as f64;
                    let width = element.width as f64;
                    let height = element.height as f64;

                    let mut text_box = TextBox::new(content)
                        .position(x, y)
                        .size(width, height);

                    if let Some(ref style) = element.style {
                        if let Some(size) = style.font_size {
                            text_box = text_box.font_size(size as f64);
                        }
                        if let Some(ref weight) = style.font_weight {
                            if weight == "bold" {
                                text_box = text_box.bold(true);
                            }
                        }
                        if let Some(ref color) = style.color {
                            text_box = text_box.font_color(color);
                        }
                    }

                    ppt_slide = ppt_slide.add_text_box(text_box);
                }
                "shape" => {
                    let shape_type = element.shape_type.as_deref().unwrap_or("rectangle");
                    let x = element.x as f64;
                    let y = element.y as f64;
                    let width = element.width as f64;
                    let height = element.height as f64;

                    let ppt_shape_type = match shape_type {
                        "ellipse" | "circle" => ShapeType::Ellipse,
                        "triangle" => ShapeType::Triangle,
                        _ => ShapeType::Rectangle,
                    };

                    let mut shape = Shape::new(ppt_shape_type)
                        .position(x, y)
                        .size(width, height);

                    if let Some(ref style) = element.style {
                        if let Some(ref fill) = style.background {
                            shape = shape.fill_color(fill);
                        }
                    }

                    ppt_slide = ppt_slide.add_shape(shape);
                }
                _ => {}
            }
        }

        pptx = pptx.add_slide(ppt_slide);
    }

    pptx.save_to_buffer().map_err(|e| format!("Failed to generate PPTX: {}", e))
}

fn export_to_html(presentation: &Presentation) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>{}</title>
<style>
.slide {{ width: 960px; height: 540px; position: relative; margin: 20px auto; background: #fff; box-shadow: 0 4px 6px rgba(0,0,0,0.1); overflow: hidden; }}
.element {{ position: absolute; }}
.element-text {{ white-space: pre-wrap; }}
</style></head><body>"#,
        presentation.name
    );

    for (i, slide) in presentation.slides.iter().enumerate() {
        let bg_color = slide.background.color.as_deref().unwrap_or("#ffffff");
        html.push_str(&format!(
            r#"<div class="slide" style="background:{}"><h4 style="position:absolute;top:5px;left:10px;color:#999;font-size:12px;">Slide {}</h4>"#,
            bg_color, i + 1
        ));

        for element in &slide.elements {
            let style = format!(
                "left:{}px;top:{}px;width:{}px;height:{}px;transform:rotate({}deg);",
                element.x, element.y, element.width, element.height, element.rotation
            );
            let extra_style = format!(
                "font-size:{}px;color:{};text-align:{};font-weight:{};",
                element.style.font_size.unwrap_or(16.0),
                element.style.color.as_deref().unwrap_or("#000"),
                element.style.text_align.as_deref().unwrap_or("left"),
                element.style.font_weight.as_deref().unwrap_or("normal")
            );

            match element.element_type.as_str() {
                "text" => {
                    let text = element.content.text.as_deref().unwrap_or("");
                    html.push_str(&format!(
                        r#"<div class="element element-text" style="{}{}">{}</div>"#,
                        style, extra_style, text
                    ));
                }
                "image" => {
                    if let Some(ref src) = element.content.src {
                        html.push_str(&format!(
                            r#"<img class="element" style="{}" src="{}" />"#,
                            style, src
                        ));
                    }
                }
                "shape" => {
                    let fill = element.style.fill.as_deref().unwrap_or("#3b82f6");
                    html.push_str(&format!(
                        r#"<div class="element" style="{}background:{};border-radius:4px;"></div>"#,
                        style, fill
                    ));
                }
                _ => {}
            }
        }
        html.push_str("</div>");
    }

    html.push_str("</body></html>");
    html
}

pub async fn handle_get_collaborators(
    Path(presentation_id): Path<String>,
) -> impl IntoResponse {
    let channels = get_slide_channels().read().await;
    let active = channels.contains_key(&presentation_id);
    Json(serde_json::json!({ "presentation_id": presentation_id, "collaborators": [], "active": active }))
}

pub async fn handle_slides_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(presentation_id): Path<String>,
) -> impl IntoResponse {
    info!("Slides WebSocket connection request for presentation: {}", presentation_id);
    ws.on_upgrade(move |socket| handle_slides_connection(socket, state, presentation_id))
}

async fn handle_slides_connection(socket: WebSocket, _state: Arc<AppState>, presentation_id: String) {
    let (mut sender, mut receiver) = socket.split();

    let channels = get_slide_channels();
    let rx = {
        let mut channels_write = channels.write().await;
        let tx = channels_write.entry(presentation_id.clone()).or_insert_with(|| broadcast::channel(256).0);
        tx.subscribe()
    };

    let user_id = format!("user-{}", &Uuid::new_v4().to_string()[..8]);
    let user_color = get_random_color();

    let welcome = serde_json::json!({
        "type": "connected",
        "presentation_id": presentation_id,
        "user_id": user_id,
        "user_color": user_color,
        "timestamp": Utc::now().to_rfc3339()
    });

    if sender.send(Message::Text(welcome.to_string())).await.is_err() {
        error!("Failed to send welcome message");
        return;
    }

    info!("User {} connected to presentation {}", user_id, presentation_id);
    broadcast_slide_change(&presentation_id, "userJoined", &user_id, None, None).await;

    let presentation_id_recv = presentation_id.clone();
    let user_id_recv = user_id.clone();
    let user_id_send = user_id.clone();

    let mut rx = rx;
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if msg.user_id != user_id_send {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        let slide_index = parsed.get("slideIndex").and_then(|v| v.as_u64()).map(|v| v as usize);
                        let element_id = parsed.get("elementId").and_then(|v| v.as_str()).map(String::from);

                        match msg_type {
                            "elementMove" | "elementResize" | "elementUpdate" | "slideChange" | "cursor" => {
                                broadcast_slide_change(&presentation_id_recv, msg_type, &user_id_recv, slide_index, element_id.as_deref()).await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    broadcast_slide_change(&presentation_id, "userLeft", &user_id, None, None).await;
    info!("User {} disconnected from presentation {}", user_id, presentation_id);
}

async fn broadcast_slide_change(
    presentation_id: &str,
    msg_type: &str,
    user_id: &str,
    slide_index: Option<usize>,
    element_id: Option<&str>,
) {
    let channels = get_slide_channels().read().await;
    if let Some(tx) = channels.get(presentation_id) {
        let msg = SlideMessage {
            msg_type: msg_type.to_string(),
            presentation_id: presentation_id.to_string(),
            user_id: user_id.to_string(),
            user_name: format!("User {}", &user_id[..8.min(user_id.len())]),
            user_color: get_random_color(),
            slide_index,
            element_id: element_id.map(String::from),
            data: None,
            timestamp: Utc::now(),
        };
        let _ = tx.send(msg);
    }
}

fn get_random_color() -> String {
    let colors = [
        "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
        "#ec4899", "#14b8a6", "#f97316", "#6366f1", "#84cc16",
    ];
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize % colors.len())
        .unwrap_or(0);
    colors[idx].to_string()
}
