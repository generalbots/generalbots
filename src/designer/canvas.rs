use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Double, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: f64,
    pub height: f64,
    pub background_color: String,
    pub grid_enabled: bool,
    pub grid_size: i32,
    pub snap_to_grid: bool,
    pub zoom_level: f64,
    pub elements: Vec<CanvasElement>,
    pub layers: Vec<Layer>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasElement {
    pub id: Uuid,
    pub element_type: ElementType,
    pub layer_id: Uuid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub opacity: f64,
    pub visible: bool,
    pub locked: bool,
    pub name: Option<String>,
    pub style: ElementStyle,
    pub properties: ElementProperties,
    pub z_index: i32,
    pub parent_id: Option<Uuid>,
    pub children: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Polygon,
    Path,
    Text,
    Image,
    Icon,
    Group,
    Frame,
    Component,
    Html,
    Svg,
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rectangle => write!(f, "rectangle"),
            Self::Ellipse => write!(f, "ellipse"),
            Self::Line => write!(f, "line"),
            Self::Arrow => write!(f, "arrow"),
            Self::Polygon => write!(f, "polygon"),
            Self::Path => write!(f, "path"),
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::Icon => write!(f, "icon"),
            Self::Group => write!(f, "group"),
            Self::Frame => write!(f, "frame"),
            Self::Component => write!(f, "component"),
            Self::Html => write!(f, "html"),
            Self::Svg => write!(f, "svg"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementStyle {
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
    pub shadow: Option<ShadowStyle>,
    pub blur: Option<f64>,
    pub border_radius: Option<BorderRadius>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillStyle {
    pub fill_type: FillType,
    pub color: Option<String>,
    pub gradient: Option<Gradient>,
    pub pattern: Option<PatternFill>,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillType {
    Solid,
    LinearGradient,
    RadialGradient,
    Pattern,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub angle: f64,
    pub center_x: Option<f64>,
    pub center_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFill {
    pub pattern_type: String,
    pub scale: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: String,
    pub width: f64,
    pub dash_array: Option<Vec<f64>>,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub spread: f64,
    pub inset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderRadius {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl BorderRadius {
    pub fn uniform(radius: f64) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementProperties {
    pub text_content: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub text_align: Option<TextAlign>,
    pub vertical_align: Option<VerticalAlign>,
    pub line_height: Option<f64>,
    pub letter_spacing: Option<f64>,
    pub text_decoration: Option<String>,
    pub text_color: Option<String>,
    pub image_url: Option<String>,
    pub image_fit: Option<ImageFit>,
    pub icon_name: Option<String>,
    pub icon_set: Option<String>,
    pub html_content: Option<String>,
    pub svg_content: Option<String>,
    pub path_data: Option<String>,
    pub points: Option<Vec<Point>>,
    pub arrow_start: Option<ArrowHead>,
    pub arrow_end: Option<ArrowHead>,
    pub component_id: Option<Uuid>,
    pub component_props: Option<HashMap<String, serde_json::Value>>,
    pub constraints: Option<Constraints>,
    pub auto_layout: Option<AutoLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowHead {
    None,
    Triangle,
    Circle,
    Diamond,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub horizontal: ConstraintType,
    pub vertical: ConstraintType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    Fixed,
    Min,
    Max,
    Center,
    Scale,
    Stretch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLayout {
    pub direction: LayoutDirection,
    pub spacing: f64,
    pub padding_top: f64,
    pub padding_right: f64,
    pub padding_bottom: f64,
    pub padding_left: f64,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub z_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub canvas_data: serde_json::Value,
    pub is_system: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLibraryItem {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub url: Option<String>,
    pub svg_content: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub is_system: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Icon,
    Image,
    Illustration,
    Shape,
    Component,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub description: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub template_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub background_color: Option<String>,
    pub grid_enabled: Option<bool>,
    pub grid_size: Option<i32>,
    pub snap_to_grid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddElementRequest {
    pub element_type: ElementType,
    pub layer_id: Option<Uuid>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub style: Option<ElementStyle>,
    pub properties: Option<ElementProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateElementRequest {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub opacity: Option<f64>,
    pub visible: Option<bool>,
    pub locked: Option<bool>,
    pub name: Option<String>,
    pub style: Option<ElementStyle>,
    pub properties: Option<ElementProperties>,
    pub z_index: Option<i32>,
    pub layer_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveElementRequest {
    pub delta_x: f64,
    pub delta_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeElementRequest {
    pub width: f64,
    pub height: f64,
    pub anchor: ResizeAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupElementsRequest {
    pub element_ids: Vec<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignElementsRequest {
    pub element_ids: Vec<Uuid>,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    CenterHorizontal,
    Right,
    Top,
    CenterVertical,
    Bottom,
    DistributeHorizontal,
    DistributeVertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLayerRequest {
    pub name: String,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLayerRequest {
    pub name: Option<String>,
    pub visible: Option<bool>,
    pub locked: Option<bool>,
    pub opacity: Option<f64>,
    pub blend_mode: Option<BlendMode>,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub quality: Option<i32>,
    pub scale: Option<f64>,
    pub background: Option<bool>,
    pub element_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Png,
    Jpg,
    Svg,
    Pdf,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub data: String,
    pub content_type: String,
    pub filename: String,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignRequest {
    pub prompt: String,
    pub context: Option<AiDesignContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignContext {
    pub selected_elements: Option<Vec<Uuid>>,
    pub canvas_state: Option<serde_json::Value>,
    pub style_preferences: Option<StylePreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreferences {
    pub color_palette: Option<Vec<String>>,
    pub font_families: Option<Vec<String>>,
    pub design_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignResponse {
    pub success: bool,
    pub elements_created: Vec<CanvasElement>,
    pub elements_modified: Vec<Uuid>,
    pub message: String,
    pub html_preview: Option<String>,
    pub svg_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEvent {
    pub event_type: CanvasEventType,
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasEventType {
    ElementAdded,
    ElementUpdated,
    ElementDeleted,
    ElementMoved,
    ElementResized,
    ElementsGrouped,
    ElementsUngrouped,
    LayerAdded,
    LayerUpdated,
    LayerDeleted,
    CanvasUpdated,
    SelectionChanged,
    CursorMoved,
    UndoPerformed,
    RedoPerformed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedoState {
    pub canvas_id: Uuid,
    pub undo_stack: Vec<CanvasSnapshot>,
    pub redo_stack: Vec<CanvasSnapshot>,
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    pub id: Uuid,
    pub elements: Vec<CanvasElement>,
    pub layers: Vec<Layer>,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

#[derive(QueryableByName)]
struct CanvasRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    organization_id: Uuid,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Double)]
    width: f64,
    #[diesel(sql_type = Double)]
    height: f64,
    #[diesel(sql_type = Text)]
    background_color: String,
    #[diesel(sql_type = Bool)]
    grid_enabled: bool,
    #[diesel(sql_type = Integer)]
    grid_size: i32,
    #[diesel(sql_type = Bool)]
    snap_to_grid: bool,
    #[diesel(sql_type = Double)]
    zoom_level: f64,
    #[diesel(sql_type = Text)]
    elements_json: String,
    #[diesel(sql_type = Text)]
    layers_json: String,
    #[diesel(sql_type = DieselUuid)]
    created_by: Uuid,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct TemplateRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Text)]
    category: String,
    #[diesel(sql_type = Nullable<Text>)]
    thumbnail_url: Option<String>,
    #[diesel(sql_type = Text)]
    canvas_data: String,
    #[diesel(sql_type = Bool)]
    is_system: bool,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    created_by: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

pub struct CanvasService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    event_sender: broadcast::Sender<CanvasEvent>,
}

impl CanvasService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self { pool, event_sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CanvasEvent> {
        self.event_sender.subscribe()
    }

    pub async fn create_canvas(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        request: CreateCanvasRequest,
    ) -> Result<Canvas, CanvasError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            CanvasError::DatabaseConnection
        })?;

        let id = Uuid::new_v4();
        let width = request.width.unwrap_or(1920.0);
        let height = request.height.unwrap_or(1080.0);

        let default_layer = Layer {
            id: Uuid::new_v4(),
            name: "Layer 1".to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            z_index: 0,
        };

        let elements: Vec<CanvasElement> = Vec::new();
        let layers = vec![default_layer.clone()];

        let elements_json = serde_json::to_string(&elements).unwrap_or_else(|_| "[]".to_string());
        let layers_json = serde_json::to_string(&layers).unwrap_or_else(|_| "[]".to_string());

        let sql = r#"
            INSERT INTO designer_canvases (
                id, organization_id, name, description, width, height,
                background_color, grid_enabled, grid_size, snap_to_grid, zoom_level,
                elements_json, layers_json, created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, '#ffffff', TRUE, 10, TRUE, 1.0,
                $7, $8, $9, NOW(), NOW()
            )
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(organization_id)
            .bind::<Text, _>(&request.name)
            .bind::<Nullable<Text>, _>(request.description.as_deref())
            .bind::<Double, _>(width)
            .bind::<Double, _>(height)
            .bind::<Text, _>(&elements_json)
            .bind::<Text, _>(&layers_json)
            .bind::<DieselUuid, _>(user_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to create canvas: {e}");
                CanvasError::CreateFailed
            })?;

        info!("Created canvas {} for org {}", id, organization_id);

        Ok(Canvas {
            id,
            organization_id,
            name: request.name,
            description: request.description,
            width,
            height,
            background_color: "#ffffff".to_string(),
            grid_enabled: true,
            grid_size: 10,
            snap_to_grid: true,
            zoom_level: 1.0,
            elements,
            layers,
            created_by: user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn get_canvas(&self, canvas_id: Uuid) -> Result<Canvas, CanvasError> {
        let mut conn = self.pool.get().map_err(|_| CanvasError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, organization_id, name, description, width, height,
                   background_color, grid_enabled, grid_size, snap_to_grid, zoom_level,
                   elements_json, layers_json, created_by, created_at, updated_at
            FROM designer_canvases WHERE id = $1
        "#;

        let rows: Vec<CanvasRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(canvas_id)
            .load(&mut conn)
            .map_err(|e| {
                error!("Failed to get canvas: {e}");
                CanvasError::DatabaseConnection
            })?;

        let row = rows.into_iter().next().ok_or(CanvasError::NotFound)?;
        Ok(self.row_to_canvas(row))
    }

    pub async fn add_element(
        &self,
        canvas_id: Uuid,
        user_id: Uuid,
        request: AddElementRequest,
    ) -> Result<CanvasElement, CanvasError> {
        let mut canvas = self.get_canvas(canvas_id).await?;

        let layer_id = request.layer_id.unwrap_or_else(|| {
            canvas.layers.first().map(|l| l.id).unwrap_or_else(Uuid::new_v4)
        });

        let max_z = canvas.elements.iter().map(|e| e.z_index).max().unwrap_or(0);

        let element = CanvasElement {
            id: Uuid::new_v4(),
            element_type: request.element_type,
            layer_id,
            x: request.x,
            y: request.y,
            width: request.width,
            height: request.height,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
            visible: true,
            locked: false,
            name: None,
            style: request.style.unwrap_or_default(),
            properties: request.properties.unwrap_or_default(),
            z_index: max_z + 1,
            parent_id: None,
            children: Vec::new(),
        };

        canvas.elements.push(element.clone());
        self.save_canvas_elements(canvas_id, &canvas.elements).await?;

        self.broadcast_event(CanvasEventType::ElementAdded, canvas_id, user_id, serde_json::json!({
            "element_id": element.id,
            "element_type": element.element_type.to_string()
        }));

        Ok(element)
    }

    pub async fn update_element(
        &self,
        canvas_id: Uuid,
        element_id: Uuid,
        user_id: Uuid,
        request: UpdateElementRequest,
    ) -> Result<CanvasElement, CanvasError> {
        let mut canvas = self.get_canvas(canvas_id).await?;

        let element = canvas
            .elements
            .iter_mut()
            .find(|e| e.id == element_id)
            .ok_or(CanvasError::ElementNotFound)?;

        if element.locked {
            return Err(CanvasError::ElementLocked);
        }

        if let Some(x) = request.x {
            element.x = x;
        }
        if let Some(y) = request.y {
            element.y = y;
        }
        if let Some(w) = request.width {
            element.width = w;
        }
        if let Some(h) = request.height {
            element.height = h;
        }
        if let Some(r) = request.rotation {
            element.rotation = r;
        }
        if let Some(sx) = request.scale_x {
            element.scale_x = sx;
        }
        if let Some(sy) = request.scale_y {
            element.scale_y = sy;
        }
        if let Some(o) = request.opacity {
            element.opacity = o;
        }
        if let Some(v) = request.visible {
            element.visible = v;
        }
        if let Some(l) = request.locked {
            element.locked = l;
        }
        if let Some(n) = request.name {
            element.name = Some(n);
        }
        if let Some(s) = request.style {
            element.style = s;
        }
        if let Some(p) = request.properties {
            element.properties = p;
        }
        if let Some(z) = request.z_index {
            element.z_index = z;
        }
        if let Some(lid) = request.layer_id {
            element.layer_id = lid;
        }

        let updated_element = element.clone();
        self.save_canvas_elements(canvas_id, &canvas.elements).await?;

        self.broadcast_event(CanvasEventType::ElementUpdated, canvas_id, user_id, serde_json::json!({
            "element_id": element_id
        }));

        Ok(updated_element)
    }

    pub async fn delete_element(
        &self,
        canvas_id: Uuid,
        element_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CanvasError> {
        let mut canvas = self.get_canvas(canvas_id).await?;

        let idx = canvas
            .elements
            .iter()
            .position(|e| e.id == element_id)
            .ok_or(CanvasError::ElementNotFound)?;

        if canvas.elements[idx].locked {
            return Err(CanvasError::ElementLocked);
        }

        canvas.elements.remove(idx);
        self.save_canvas_elements(canvas_id, &canvas.elements).await?;

        self.broadcast_event(CanvasEventType::ElementDeleted, canvas_id, user_id, serde_json::json!({
            "element_id": element_id
        }));

        Ok(())
    }

    pub async fn group_elements(
        &self,
        canvas_id: Uuid,
        user_id: Uuid,
        request: GroupElementsRequest,
    ) -> Result<CanvasElement, CanvasError> {
        let mut canvas = self.get_canvas(canvas_id).await?;

        let elements_to_group: Vec<&CanvasElement> = canvas
            .elements
            .iter()
            .filter(|e| request.element_ids.contains(&e.id))
            .collect();

        if elements_to_group.is_empty() {
            return Err(CanvasError::InvalidInput("No elements to group".to_string()));
        }

        let min_x = elements_to_group.iter().map(|e| e.x).fold(f64::INFINITY, f64::min);
        let min_y = elements_to_group.iter().map(|e| e.y).fold(f64::INFINITY, f64::min);
        let max_x = elements_to_group.iter().map(|e| e.x + e.width).fold(f64::NEG_INFINITY, f64::max);
        let max_y = elements_to_group.iter().map(|e| e.y + e.height).fold(f64::NEG_INFINITY, f64::max);

        let group_id = Uuid::new_v4();
        let layer_id = elements_to_group.first().map(|e| e.layer_id).unwrap_or_else(Uuid::new_v4);
        let max_z = canvas.elements.iter().map(|e| e.z_index).max().unwrap_or(0);

        for element in canvas.elements.iter_mut() {
            if request.element_ids.contains(&element.id) {
                element.parent_id = Some(group_id);
            }
        }

        let group = CanvasElement {
            id: group_id,
            element_type: ElementType::Group,
            layer_id,
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
            visible: true,
            locked: false,
            name: request.name,
            style: ElementStyle::default(),
            properties: ElementProperties::default(),
            z_index: max_z + 1,
            parent_id: None,
            children: request.element_ids.clone(),
        };

        canvas.elements.push(group.clone());
        self.save_canvas_elements(canvas_id, &canvas.elements).await?;

        self.broadcast_event(CanvasEventType::ElementsGrouped, canvas_id, user_id, serde_json::json!({
            "group_id": group_id,
            "element_ids": request.element_ids
        }));

        Ok(group)
    }

    pub async fn add_layer(
        &self,
        canvas_id: Uuid,
        user_id: Uuid,
        request: CreateLayerRequest,
    ) -> Result<Layer, CanvasError> {
        let mut canvas = self.get_canvas(canvas_id).await?;

        let max_z = canvas.layers.iter().map(|l| l.z_index).max().unwrap_or(0);

        let layer = Layer {
            id: Uuid::new_v4(),
            name: request.name,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            z_index: request.z_index.unwrap_or(max_z + 1),
        };

        canvas.layers.push(layer.clone());
        self.save_canvas_layers(canvas_id, &canvas.layers).await?;

        self.broadcast_event(CanvasEventType::LayerAdded, canvas_id, user_id, serde_json::json!({
            "layer_id": layer.id
        }));

        Ok(layer)
    }

    pub async fn export_canvas(
        &self,
        canvas_id: Uuid,
        request: ExportRequest,
    ) -> Result<ExportResult, CanvasError> {
        let canvas = self.get_canvas(canvas_id).await?;

        let scale = request.scale.unwrap_or(1.0);
        let width = canvas.width * scale;
        let height = canvas.height * scale;

        let (data, content_type, ext) = match request.format {
            ExportFormat::Svg => {
                let svg = self.generate_svg(&canvas, &request)?;
                (svg, "image/svg+xml", "svg")
            }
            ExportFormat::Html => {
                let html = self.generate_html(&canvas, &request)?;
                (html, "text/html", "html")
            }
            ExportFormat::Png | ExportFormat::Jpg | ExportFormat::Pdf => {
                let svg = self.generate_svg(&canvas, &request)?;
                (svg, "image/svg+xml", "svg")
            }
        };

        Ok(ExportResult {
            format: request.format,
            data,
            content_type: content_type.to_string(),
            filename: format!("{}.{}", canvas.name, ext),
            width,
            height,
        })
    }

    fn generate_svg(&self, canvas: &Canvas, request: &ExportRequest) -> Result<String, CanvasError> {
        let scale = request.scale.unwrap_or(1.0);
        let width = canvas.width * scale;
        let height = canvas.height * scale;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, canvas.width, canvas.height
        );

        if request.background.unwrap_or(true) {
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{}"/>"#,
                canvas.background_color
            ));
        }

        let mut sorted_elements = canvas.elements.clone();
        sorted_elements.sort_by_key(|e| e.z_index);

        for element in sorted_elements.iter().filter(|e| e.visible) {
            svg.push_str(&self.element_to_svg(element));
        }

        svg.push_str("</svg>");
        Ok(svg)
    }

    fn element_to_svg(&self, element: &CanvasElement) -> String {
        let transform = if element.rotation != 0.0 || element.scale_x != 1.0 || element.scale_y != 1.0 {
            format!(
                r#" transform="translate({},{}) rotate({}) scale({},{})""#,
                element.x + element.width / 2.0,
                element.y + element.height / 2.0,
                element.rotation,
                element.scale_x,
                element.scale_y
            )
        } else {
            String::new()
        };

        let opacity = if element.opacity < 1.0 {
            format!(r#" opacity="{}""#, element.opacity)
        } else {
            String::new()
        };

        let fill = element.style.fill.as_ref().map(|f| {
            match f.fill_type {
                FillType::Solid => f.color.clone().unwrap_or_else(|| "#000000".to_string()),
                FillType::None => "none".to_string(),
                _ => "#000000".to_string(),
            }
        }).unwrap_or_else(|| "#000000".to_string());

        let stroke = element.style.stroke.as_ref().map(|s| {
            format!(r#" stroke="{}" stroke-width="{}""#, s.color, s.width)
        }).unwrap_or_default();

        match element.element_type {
            ElementType::Rectangle => {
                let rx = element.style.border_radius.as_ref().map(|r| r.top_left).unwrap_or(0.0);
                format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}"{}{}{}/>"#,
                    element.x, element.y, element.width, element.height, rx, fill, stroke, opacity, transform
                )
            }
            ElementType::Ellipse => {
                format!(
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"{}{}{}/>"#,
                    element.x + element.width / 2.0,
                    element.y + element.height / 2.0,
                    element.width / 2.0,
                    element.height / 2.0,
                    fill, stroke, opacity, transform
                )
            }
            ElementType::Line => {
                format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{}{}{}/>"#,
                    element.x, element.y,
                    element.x + element.width,
                    element.y + element.height,
                    stroke, opacity, transform
                )
            }
            ElementType::Text => {
                let text = element.properties.text_content.as_deref().unwrap_or("");
                let font_size = element.properties.font_size.unwrap_or(16.0);
                let font_family = element.properties.font_family.as_deref().unwrap_or("sans-serif");
                let text_color = element.properties.text_color.as_deref().unwrap_or("#000000");
                format!(
                    r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}"{}{}>{}text>"#,
                    element.x, element.y + font_size, font_size, font_family, text_color, opacity, transform, text
                )
            }
            ElementType::Image => {
                let url = element.properties.image_url.as_deref().unwrap_or("");
                format!(
                    r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"{}{}/>"#,
                    element.x, element.y, element.width, element.height, url, opacity, transform
                )
            }
            ElementType::Svg => {
                element.properties.svg_content.clone().unwrap_or_default()
            }
            ElementType::Path => {
                let d = element.properties.path_data.as_deref().unwrap_or("");
                format!(
                    r#"<path d="{}" fill="{}"{}{}{}/"#,
                    d, fill, stroke, opacity, transform
                )
            }
            _ => String::new(),
        }
    }

    fn generate_html(&self, canvas: &Canvas, request: &ExportRequest) -> Result<String, CanvasError> {
        let svg = self.generate_svg(canvas, request)?;

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ margin: 0; padding: 0; display: flex; justify-content: center; align-items: center; min-height: 100vh; background: #f0f0f0; }}
        .canvas-container {{ background: white; box-shadow: 0 4px 20px rgba(0,0,0,0.1); }}
    </style>
</head>
<body>
    <div class="canvas-container">
        {}
    </div>
</body>
</html>"#,
            canvas.name, svg
        );

        Ok(html)
    }

    async fn save_canvas_elements(&self, canvas_id: Uuid, elements: &[CanvasElement]) -> Result<(), CanvasError> {
        let mut conn = self.pool.get().map_err(|_| CanvasError::DatabaseConnection)?;

        let elements_json = serde_json::to_string(elements).unwrap_or_else(|_| "[]".to_string());

        diesel::sql_query("UPDATE designer_canvases SET elements_json = $1, updated_at = NOW() WHERE id = $2")
            .bind::<Text, _>(&elements_json)
            .bind::<DieselUuid, _>(canvas_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to save elements: {e}");
                CanvasError::UpdateFailed
            })?;

        Ok(())
    }

    async fn save_canvas_layers(&self, canvas_id: Uuid, layers: &[Layer]) -> Result<(), CanvasError> {
        let mut conn = self.pool.get().map_err(|_| CanvasError::DatabaseConnection)?;

        let layers_json = serde_json::to_string(layers).unwrap_or_else(|_| "[]".to_string());

        diesel::sql_query("UPDATE designer_canvases SET layers_json = $1, updated_at = NOW() WHERE id = $2")
            .bind::<Text, _>(&layers_json)
            .bind::<DieselUuid, _>(canvas_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to save layers: {e}");
                CanvasError::UpdateFailed
            })?;

        Ok(())
    }

    fn broadcast_event(&self, event_type: CanvasEventType, canvas_id: Uuid, user_id: Uuid, data: serde_json::Value) {
        let event = CanvasEvent {
            event_type,
            canvas_id,
            user_id,
            data,
            timestamp: Utc::now(),
        };
        let _ = self.event_sender.send(event);
    }

    fn row_to_canvas(&self, row: CanvasRow) -> Canvas {
        let elements: Vec<CanvasElement> = serde_json::from_str(&row.elements_json).unwrap_or_default();
        let layers: Vec<Layer> = serde_json::from_str(&row.layers_json).unwrap_or_default();

        Canvas {
            id: row.id,
            organization_id: row.organization_id,
            name: row.name,
            description: row.description,
            width: row.width,
            height: row.height,
            background_color: row.background_color,
            grid_enabled: row.grid_enabled,
            grid_size: row.grid_size,
            snap_to_grid: row.snap_to_grid,
            zoom_level: row.zoom_level,
            elements,
            layers,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    pub async fn get_templates(&self, category: Option<String>) -> Result<Vec<CanvasTemplate>, CanvasError> {
        let mut conn = self.pool.get().map_err(|_| CanvasError::DatabaseConnection)?;

        let sql = match category {
            Some(ref cat) => format!(
                "SELECT id, name, description, category, thumbnail_url, canvas_data, is_system, created_by, created_at FROM designer_templates WHERE category = '{}' ORDER BY name",
                cat
            ),
            None => "SELECT id, name, description, category, thumbnail_url, canvas_data, is_system, created_by, created_at FROM designer_templates ORDER BY category, name".to_string(),
        };

        let rows: Vec<TemplateRow> = diesel::sql_query(&sql)
            .load(&mut conn)
            .unwrap_or_default();

        let templates = rows
            .into_iter()
            .map(|row| CanvasTemplate {
                id: row.id,
                name: row.name,
                description: row.description,
                category: row.category,
                thumbnail_url: row.thumbnail_url,
                canvas_data: serde_json::from_str(&row.canvas_data).unwrap_or(serde_json::json!({})),
                is_system: row.is_system,
                created_by: row.created_by,
                created_at: row.created_at,
            })
            .collect();

        Ok(templates)
    }

    pub async fn get_asset_library(&self, asset_type: Option<AssetType>) -> Result<Vec<AssetLibraryItem>, CanvasError> {
        let icons = vec![
            AssetLibraryItem { id: Uuid::new_v4(), name: "Bot".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../botui/ui/suite/assets/icons/gb-bot.svg").to_string()), category: "General Bots".to_string(), tags: vec!["bot".to_string(), "assistant".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Analytics".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["analytics".to_string(), "chart".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Calendar".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["calendar".to_string(), "date".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Chat".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["chat".to_string(), "message".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Drive".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["drive".to_string(), "files".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Mail".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["mail".to_string(), "email".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Meet".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["meet".to_string(), "video".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Tasks".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some("<svg></svg>".to_string()), category: "General Bots".to_string(), tags: vec!["tasks".to_string(), "todo".to_string()], is_system: true },
        ];

        let filtered = match asset_type {
            Some(t) => icons.into_iter().filter(|i| i.asset_type == t).collect(),
            None => icons,
        };

        Ok(filtered)
    }
}

#[derive(Debug, Clone)]
pub enum CanvasError {
    DatabaseConnection,
    NotFound,
    ElementNotFound,
    ElementLocked,
    CreateFailed,
    UpdateFailed,
    DeleteFailed,
    ExportFailed(String),
    InvalidInput(String),
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::NotFound => write!(f, "Canvas not found"),
            Self::ElementNotFound => write!(f, "Element not found"),
            Self::ElementLocked => write!(f, "Element is locked"),
            Self::CreateFailed => write!(f, "Failed to create"),
            Self::UpdateFailed => write!(f, "Failed to update"),
            Self::DeleteFailed => write!(f, "Failed to delete"),
            Self::ExportFailed(msg) => write!(f, "Export failed: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for CanvasError {}

impl IntoResponse for CanvasError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound | Self::ElementNotFound => StatusCode::NOT_FOUND,
            Self::ElementLocked => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn create_canvas_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS designer_canvases (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        description TEXT,
        width DOUBLE PRECISION NOT NULL DEFAULT 1920,
        height DOUBLE PRECISION NOT NULL DEFAULT 1080,
        background_color TEXT NOT NULL DEFAULT '#ffffff',
        grid_enabled BOOLEAN NOT NULL DEFAULT TRUE,
        grid_size INTEGER NOT NULL DEFAULT 10,
        snap_to_grid BOOLEAN NOT NULL DEFAULT TRUE,
        zoom_level DOUBLE PRECISION NOT NULL DEFAULT 1.0,
        elements_json TEXT NOT NULL DEFAULT '[]',
        layers_json TEXT NOT NULL DEFAULT '[]',
        created_by UUID NOT NULL REFERENCES users(id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS designer_templates (
        id UUID PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        category TEXT NOT NULL,
        thumbnail_url TEXT,
        canvas_data TEXT NOT NULL DEFAULT '{}',
        is_system BOOLEAN NOT NULL DEFAULT FALSE,
        created_by UUID REFERENCES users(id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_designer_canvases_org ON designer_canvases(organization_id);
    CREATE INDEX IF NOT EXISTS idx_designer_templates_category ON designer_templates(category);
    "#
}

pub fn canvas_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
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
    organization_id: Uuid,
    user_id: Uuid,
    Json(request): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let canvas = service.create_canvas(organization_id, user_id, request).await?;
    Ok(Json(canvas))
}

async fn get_canvas_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Canvas>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let canvas = service.get_canvas(canvas_id).await?;
    Ok(Json(canvas))
}

async fn add_element_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    user_id: Uuid,
    Json(request): Json<AddElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let element = service.add_element(canvas_id, user_id, request).await?;
    Ok(Json(element))
}

async fn update_element_handler(
    State(state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
    Json(request): Json<UpdateElementRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let element = service.update_element(canvas_id, element_id, user_id, request).await?;
    Ok(Json(element))
}

async fn delete_element_handler(
    State(state): State<Arc<AppState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
    user_id: Uuid,
) -> Result<StatusCode, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    service.delete_element(canvas_id, element_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn group_elements_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    user_id: Uuid,
    Json(request): Json<GroupElementsRequest>,
) -> Result<Json<CanvasElement>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let group = service.group_elements(canvas_id, user_id, request).await?;
    Ok(Json(group))
}

async fn add_layer_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    user_id: Uuid,
    Json(request): Json<CreateLayerRequest>,
) -> Result<Json<Layer>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
    let layer = service.add_layer(canvas_id, user_id, request).await?;
    Ok(Json(layer))
}

async fn export_canvas_handler(
    State(state): State<Arc<AppState>>,
    Path(canvas_id): Path<Uuid>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ExportResult>, CanvasError> {
    let service = CanvasService::new(state.conn.clone());
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
    let service = CanvasService::new(state.conn.clone());
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

    let service = CanvasService::new(state.conn.clone());
    let assets = service.get_asset_library(asset_type).await?;
    Ok(Json(assets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_display() {
        assert_eq!(ElementType::Rectangle.to_string(), "rectangle");
        assert_eq!(ElementType::Ellipse.to_string(), "ellipse");
        assert_eq!(ElementType::Text.to_string(), "text");
    }

    #[test]
    fn test_border_radius_uniform() {
        let radius = BorderRadius::uniform(10.0);
        assert_eq!(radius.top_left, 10.0);
        assert_eq!(radius.top_right, 10.0);
        assert_eq!(radius.bottom_right, 10.0);
        assert_eq!(radius.bottom_left, 10.0);
    }

    #[test]
    fn test_blend_mode_default() {
        let mode = BlendMode::default();
        assert_eq!(mode, BlendMode::Normal);
    }

    #[test]
    fn test_canvas_error_display() {
        assert_eq!(CanvasError::NotFound.to_string(), "Canvas not found");
        assert_eq!(CanvasError::ElementLocked.to_string(), "Element is locked");
    }

    #[test]
    fn test_element_style_default() {
        let style = ElementStyle::default();
        assert!(style.fill.is_none());
        assert!(style.stroke.is_none());
        assert!(style.opacity.is_none());
    }
}
