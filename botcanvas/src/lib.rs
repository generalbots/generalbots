use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<diesel::PgConnection>>;

pub type GetBotContextFn = Arc<dyn Fn(&DbPool) -> (Uuid, Uuid) + Send + Sync>;

#[derive(Clone)]
pub struct CanvasState {
    pub pool: Arc<DbPool>,
    pub get_bot_context: GetBotContextFn,
}

diesel::table! {
    canvases (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        width -> Int4,
        height -> Int4,
        background_color -> Nullable<Varchar>,
        thumbnail_url -> Nullable<Text>,
        is_public -> Bool,
        is_template -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_elements (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        element_type -> Varchar,
        x -> Float8,
        y -> Float8,
        width -> Float8,
        height -> Float8,
        rotation -> Float8,
        z_index -> Int4,
        locked -> Bool,
        properties -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_collaborators (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        user_id -> Uuid,
        permission -> Varchar,
        added_by -> Nullable<Uuid>,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_versions (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        version_number -> Int4,
        name -> Nullable<Varchar>,
        elements_snapshot -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    canvas_comments (id) {
        id -> Uuid,
        canvas_id -> Uuid,
        element_id -> Nullable<Uuid>,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        x_position -> Nullable<Float8>,
        y_position -> Nullable<Float8>,
        resolved -> Bool,
        resolved_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(canvas_elements -> canvases (canvas_id));
diesel::joinable!(canvas_collaborators -> canvases (canvas_id));
diesel::joinable!(canvas_versions -> canvases (canvas_id));
diesel::joinable!(canvas_comments -> canvases (canvas_id));

diesel::allow_tables_to_appear_in_same_query!(
    canvases,
    canvas_elements,
    canvas_collaborators,
    canvas_versions,
    canvas_comments,
);

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = canvases)]
pub struct DbCanvas {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: i32,
    pub height: i32,
    pub background_color: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_public: bool,
    pub is_template: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = canvas_elements)]
pub struct DbCanvasElement {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub z_index: i32,
    pub locked: bool,
    pub properties: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = canvas_collaborators)]
pub struct DbCanvasCollaborator {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub added_by: Option<Uuid>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = canvas_versions)]
pub struct DbCanvasVersion {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub version_number: i32,
    pub name: Option<String>,
    pub elements_snapshot: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = canvas_comments)]
pub struct DbCanvasComment {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub element_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub x_position: Option<f64>,
    pub y_position: Option<f64>,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: i32,
    pub height: i32,
    pub background_color: String,
    pub thumbnail_url: Option<String>,
    pub is_public: bool,
    pub is_template: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub elements: Vec<CanvasElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasElement {
    pub id: Uuid,
    pub canvas_id: Uuid,
    pub element_type: ElementType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub z_index: i32,
    pub locked: bool,
    pub properties: ElementProperties,
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

impl ElementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Line => "line",
            Self::Arrow => "arrow",
            Self::FreehandPath => "freehand_path",
            Self::Text => "text",
            Self::Image => "image",
            Self::Sticky => "sticky",
            Self::Frame => "frame",
            Self::Connector => "connector",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "rectangle" => Self::Rectangle,
            "ellipse" => Self::Ellipse,
            "line" => Self::Line,
            "arrow" => Self::Arrow,
            "freehand_path" => Self::FreehandPath,
            "text" => Self::Text,
            "image" => Self::Image,
            "sticky" => Self::Sticky,
            "frame" => Self::Frame,
            "connector" => Self::Connector,
            _ => Self::Rectangle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_arrow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_arrow: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub element_count: i64,
    pub is_public: bool,
    pub is_template: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub background_color: Option<String>,
    pub is_template: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub background_color: Option<String>,
    pub is_public: Option<bool>,
    pub is_template: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub element_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
    pub x_position: Option<f64>,
    pub y_position: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AddCollaboratorRequest {
    pub user_id: Uuid,
    pub permission: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub is_public: Option<bool>,
    pub is_template: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UiListQuery {
    pub search: Option<String>,
    pub is_template: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CollaborationSession {
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub selection: Vec<Uuid>,
    pub connected_at: DateTime<Utc>,
}

fn db_to_canvas_element(db: DbCanvasElement) -> CanvasElement {
    let properties: ElementProperties =
        serde_json::from_value(db.properties).unwrap_or_default();
    CanvasElement {
        id: db.id,
        canvas_id: db.canvas_id,
        element_type: ElementType::from_str(&db.element_type),
        x: db.x,
        y: db.y,
        width: db.width,
        height: db.height,
        rotation: db.rotation,
        z_index: db.z_index,
        locked: db.locked,
        properties,
        created_by: db.created_by,
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

fn db_to_canvas(db: DbCanvas, elements: Vec<CanvasElement>) -> Canvas {
    Canvas {
        id: db.id,
        org_id: db.org_id,
        name: db.name,
        description: db.description,
        width: db.width,
        height: db.height,
        background_color: db.background_color.unwrap_or_else(|| "#ffffff".to_string()),
        thumbnail_url: db.thumbnail_url,
        is_public: db.is_public,
        is_template: db.is_template,
        created_by: db.created_by,
        created_at: db.created_at,
        updated_at: db.updated_at,
        elements,
    }
}

async fn list_canvases(
    State(state): State<Arc<CanvasState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CanvasSummary>>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_public) = query.is_public {
        q = q.filter(canvases::is_public.eq(is_public));
    }

    if let Some(is_template) = query.is_template {
        q = q.filter(canvases::is_template.eq(is_template));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            canvases::name
                .ilike(pattern.clone())
                .or(canvases::description.ilike(pattern)),
        );
    }

    let db_canvases: Vec<DbCanvas> = q
        .order(canvases::updated_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let mut summaries = Vec::with_capacity(db_canvases.len());
    for c in db_canvases {
        let element_count: i64 = canvas_elements::table
            .filter(canvas_elements::canvas_id.eq(c.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        summaries.push(CanvasSummary {
            id: c.id,
            name: c.name,
            description: c.description,
            thumbnail_url: c.thumbnail_url,
            element_count,
            is_public: c.is_public,
            is_template: c.is_template,
            created_at: c.created_at,
            updated_at: c.updated_at,
        });
    }

    Ok(Json(summaries))
}

async fn create_canvas(
    State(state): State<Arc<CanvasState>>,
    Json(req): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let user_id = Uuid::nil();

    let db_canvas = DbCanvas {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        width: req.width.unwrap_or(1920),
        height: req.height.unwrap_or(1080),
        background_color: Some(req.background_color.unwrap_or_else(|| "#ffffff".to_string())),
        thumbnail_url: None,
        is_public: false,
        is_template: req.is_template.unwrap_or(false),
        created_by: user_id,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(canvases::table)
        .values(&db_canvas)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    let canvas = db_to_canvas(db_canvas, vec![]);
    Ok(Json(canvas))
}

async fn get_canvas(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Canvas>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let db_canvas: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    let canvas = db_to_canvas(db_canvas, elements);

    Ok(Json(canvas))
}

async fn update_canvas(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<UpdateCanvasRequest>,
) -> Result<Json<Canvas>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let mut db_canvas: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    if let Some(name) = req.name {
        db_canvas.name = name;
    }
    if let Some(desc) = req.description {
        db_canvas.description = Some(desc);
    }
    if let Some(width) = req.width {
        db_canvas.width = width;
    }
    if let Some(height) = req.height {
        db_canvas.height = height;
    }
    if let Some(bg) = req.background_color {
        db_canvas.background_color = Some(bg);
    }
    if let Some(is_public) = req.is_public {
        db_canvas.is_public = is_public;
    }
    if let Some(is_template) = req.is_template {
        db_canvas.is_template = is_template;
    }
    db_canvas.updated_at = Utc::now();

    diesel::update(canvases::table.filter(canvases::id.eq(canvas_id)))
        .set(&db_canvas)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    let canvas = db_to_canvas(db_canvas, elements);

    Ok(Json(canvas))
}

async fn delete_canvas(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    diesel::delete(canvas_comments::table.filter(canvas_comments::canvas_id.eq(canvas_id)))
        .execute(&mut conn)
        .ok();

    diesel::delete(canvas_versions::table.filter(canvas_versions::canvas_id.eq(canvas_id)))
        .execute(&mut conn)
        .ok();

    diesel::delete(
        canvas_collaborators::table.filter(canvas_collaborators::canvas_id.eq(canvas_id)),
    )
    .execute(&mut conn)
    .ok();

    diesel::delete(canvas_elements::table.filter(canvas_elements::canvas_id.eq(canvas_id)))
        .execute(&mut conn)
        .ok();

    let deleted = diesel::delete(canvases::table.filter(canvases::id.eq(canvas_id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Canvas not found".to_string()))
    }
}

async fn list_elements(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<CanvasElement>>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let _: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    Ok(Json(elements))
}

async fn create_element(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<CreateElementRequest>,
) -> Result<Json<CanvasElement>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let _: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let now = Utc::now();
    let user_id = Uuid::nil();
    let id = Uuid::new_v4();

    let max_z: Option<i32> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .select(diesel::dsl::max(canvas_elements::z_index))
        .first(&mut conn)
        .ok()
        .flatten();

    let z_index = req.z_index.unwrap_or_else(|| max_z.unwrap_or(0) + 1);
    let properties = req.properties.unwrap_or_default();
    let properties_json =
        serde_json::to_value(&properties).unwrap_or_else(|_| serde_json::json!({}));

    let db_element = DbCanvasElement {
        id,
        canvas_id,
        element_type: req.element_type.as_str().to_string(),
        x: req.x,
        y: req.y,
        width: req.width,
        height: req.height,
        rotation: req.rotation.unwrap_or(0.0),
        z_index,
        locked: false,
        properties: properties_json,
        created_by: user_id,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(canvas_elements::table)
        .values(&db_element)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    diesel::update(canvases::table.filter(canvases::id.eq(canvas_id)))
        .set(canvases::updated_at.eq(now))
        .execute(&mut conn)
        .ok();

    let element = db_to_canvas_element(db_element);
    Ok(Json(element))
}

async fn update_element(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateElementRequest>,
) -> Result<Json<CanvasElement>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let mut db_element: DbCanvasElement = canvas_elements::table
        .filter(canvas_elements::id.eq(element_id))
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Element not found".to_string()))?;

    if let Some(x) = req.x {
        db_element.x = x;
    }
    if let Some(y) = req.y {
        db_element.y = y;
    }
    if let Some(width) = req.width {
        db_element.width = width;
    }
    if let Some(height) = req.height {
        db_element.height = height;
    }
    if let Some(rotation) = req.rotation {
        db_element.rotation = rotation;
    }
    if let Some(z_index) = req.z_index {
        db_element.z_index = z_index;
    }
    if let Some(locked) = req.locked {
        db_element.locked = locked;
    }
    if let Some(props) = req.properties {
        db_element.properties =
            serde_json::to_value(&props).unwrap_or_else(|_| serde_json::json!({}));
    }
    db_element.updated_at = Utc::now();

    diesel::update(canvas_elements::table.filter(canvas_elements::id.eq(element_id)))
        .set(&db_element)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    diesel::update(canvases::table.filter(canvases::id.eq(canvas_id)))
        .set(canvases::updated_at.eq(Utc::now()))
        .execute(&mut conn)
        .ok();

    let element = db_to_canvas_element(db_element);
    Ok(Json(element))
}

async fn delete_element(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, element_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let deleted = diesel::delete(
        canvas_elements::table
            .filter(canvas_elements::id.eq(element_id))
            .filter(canvas_elements::canvas_id.eq(canvas_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    if deleted > 0 {
        diesel::update(canvases::table.filter(canvases::id.eq(canvas_id)))
            .set(canvases::updated_at.eq(Utc::now()))
            .execute(&mut conn)
            .ok();
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Element not found".to_string()))
    }
}

async fn export_canvas(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let db_canvas: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    let canvas = db_to_canvas(db_canvas, elements);

    match req.format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&canvas)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {e}")))?;
            Ok(Json(ExportResponse {
                format: ExportFormat::Json,
                url: None,
                data: Some(json),
            }))
        }
        ExportFormat::Svg => {
            let svg = generate_svg(&canvas, req.background.unwrap_or(true));
            Ok(Json(ExportResponse {
                format: ExportFormat::Svg,
                url: None,
                data: Some(svg),
            }))
        }
        _ => Ok(Json(ExportResponse {
            format: req.format,
            url: Some(format!("/api/canvas/{canvas_id}/export/file")),
            data: None,
        })),
    }
}

fn generate_svg(canvas: &Canvas, include_background: bool) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"##,
        canvas.width, canvas.height, canvas.width, canvas.height
    );

    if include_background {
        svg.push_str(&format!(
            r##"<rect width="100%" height="100%" fill="{}"/>"##,
            canvas.background_color
        ));
    }

    for element in &canvas.elements {
        let transform = if element.rotation != 0.0 {
            format!(
                r##" transform="rotate({} {} {})""##,
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
                    r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                    element.x, element.y, element.width, element.height,
                    radius, fill, stroke, stroke_width, opacity, transform
                ));
            }
            ElementType::Ellipse => {
                svg.push_str(&format!(
                    r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
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
                    r##"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}" opacity="{}"{}>{}</text>"##,
                    element.x, element.y + font_size, font_size, font_family,
                    fill, opacity, transform, text
                ));
            }
            ElementType::FreehandPath => {
                if let Some(path_data) = &element.properties.path_data {
                    svg.push_str(&format!(
                        r##"<path d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                        path_data, stroke, stroke_width, opacity, transform
                    ));
                }
            }
            ElementType::Line | ElementType::Arrow => {
                let marker = if element.element_type == ElementType::Arrow {
                    r##" marker-end="url(#arrowhead)""##
                } else {
                    ""
                };
                svg.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" opacity="{}"{}{}/>"##,
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

async fn list_collaborators(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<DbCanvasCollaborator>>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let collaborators: Vec<DbCanvasCollaborator> = canvas_collaborators::table
        .filter(canvas_collaborators::canvas_id.eq(canvas_id))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(collaborators))
}

async fn add_collaborator(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<AddCollaboratorRequest>,
) -> Result<Json<DbCanvasCollaborator>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let _: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let now = Utc::now();
    let collaborator = DbCanvasCollaborator {
        id: Uuid::new_v4(),
        canvas_id,
        user_id: req.user_id,
        permission: req.permission.unwrap_or_else(|| "view".to_string()),
        added_by: None,
        added_at: now,
    };

    diesel::insert_into(canvas_collaborators::table)
        .values(&collaborator)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(collaborator))
}

async fn remove_collaborator(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let deleted = diesel::delete(
        canvas_collaborators::table
            .filter(canvas_collaborators::canvas_id.eq(canvas_id))
            .filter(canvas_collaborators::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Collaborator not found".to_string()))
    }
}

async fn list_comments(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<DbCanvasComment>>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let comments: Vec<DbCanvasComment> = canvas_comments::table
        .filter(canvas_comments::canvas_id.eq(canvas_id))
        .order(canvas_comments::created_at.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(comments))
}

async fn create_comment(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<DbCanvasComment>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let _: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let now = Utc::now();
    let user_id = Uuid::nil();

    let comment = DbCanvasComment {
        id: Uuid::new_v4(),
        canvas_id,
        element_id: req.element_id,
        parent_comment_id: req.parent_comment_id,
        author_id: user_id,
        content: req.content,
        x_position: req.x_position,
        y_position: req.y_position,
        resolved: false,
        resolved_by: None,
        resolved_at: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(canvas_comments::table)
        .values(&comment)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(comment))
}

async fn resolve_comment(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DbCanvasComment>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let now = Utc::now();
    let user_id = Uuid::nil();

    diesel::update(
        canvas_comments::table
            .filter(canvas_comments::id.eq(comment_id))
            .filter(canvas_comments::canvas_id.eq(canvas_id)),
    )
    .set((
        canvas_comments::resolved.eq(true),
        canvas_comments::resolved_by.eq(Some(user_id)),
        canvas_comments::resolved_at.eq(Some(now)),
        canvas_comments::updated_at.eq(now),
    ))
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let comment: DbCanvasComment = canvas_comments::table
        .filter(canvas_comments::id.eq(comment_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Comment not found".to_string()))?;

    Ok(Json(comment))
}

async fn list_versions(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<DbCanvasVersion>>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let versions: Vec<DbCanvasVersion> = canvas_versions::table
        .filter(canvas_versions::canvas_id.eq(canvas_id))
        .order(canvas_versions::version_number.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(versions))
}

async fn create_version(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<DbCanvasVersion>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let _: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let max_version: Option<i32> = canvas_versions::table
        .filter(canvas_versions::canvas_id.eq(canvas_id))
        .select(diesel::dsl::max(canvas_versions::version_number))
        .first(&mut conn)
        .ok()
        .flatten();

    let now = Utc::now();
    let user_id = Uuid::nil();
    let elements_snapshot =
        serde_json::to_value(&db_elements).unwrap_or_else(|_| serde_json::json!([]));

    let version = DbCanvasVersion {
        id: Uuid::new_v4(),
        canvas_id,
        version_number: max_version.unwrap_or(0) + 1,
        name: None,
        elements_snapshot,
        created_by: user_id,
        created_at: now,
    };

    diesel::insert_into(canvas_versions::table)
        .values(&version)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(version))
}

async fn get_collaboration_info(
    State(_state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Result<Json<Vec<CollaborationSession>>, (StatusCode, String)> {
    let _ = canvas_id;
    Ok(Json(vec![]))
}

pub fn configure_canvas_routes() -> Router<Arc<CanvasState>> {
    Router::new()
        .route("/api/canvas", get(list_canvases).post(create_canvas))
        .route(
            "/api/canvas/{canvas_id}",
            get(get_canvas).put(update_canvas).delete(delete_canvas),
        )
        .route(
            "/api/canvas/{canvas_id}/elements",
            get(list_elements).post(create_element),
        )
        .route(
            "/api/canvas/{canvas_id}/elements/{element_id}",
            put(update_element).delete(delete_element),
        )
        .route("/api/canvas/{canvas_id}/export", post(export_canvas))
        .route(
            "/api/canvas/{canvas_id}/collaborators",
            get(list_collaborators).post(add_collaborator),
        )
        .route(
            "/api/canvas/{canvas_id}/collaborators/{user_id}",
            axum::routing::delete(remove_collaborator),
        )
        .route(
            "/api/canvas/{canvas_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/api/canvas/{canvas_id}/comments/{comment_id}/resolve",
            put(resolve_comment),
        )
        .route(
            "/api/canvas/{canvas_id}/versions",
            get(list_versions).post(create_version),
        )
        .route(
            "/api/canvas/{canvas_id}/collaborate",
            get(get_collaboration_info),
        )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn render_empty_state(icon: &str, title: &str, description: &str) -> String {
    format!(
        r##"<div class="empty-state">
<div class="empty-icon">{icon}</div>
<h3>{title}</h3>
<p>{description}</p>
</div>"##
    )
}

fn render_canvas_card(canvas: &DbCanvas, element_count: i64) -> String {
    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_default();
    let bg_color = canvas
        .background_color
        .as_deref()
        .unwrap_or("#ffffff");
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = canvas.id;
    let template_badge = if canvas.is_template {
        r##"<span class="badge badge-info">Template</span>"##
    } else {
        ""
    };
    let public_badge = if canvas.is_public {
        r##"<span class="badge badge-success">Public</span>"##
    } else {
        ""
    };

    format!(
        r##"<div class="canvas-card" data-id="{id}">
<div class="canvas-preview" style="background-color: {bg_color};">
<div class="canvas-element-count">{element_count} elements</div>
</div>
<div class="canvas-info">
<h4 class="canvas-name">{name}</h4>
<p class="canvas-description">{description}</p>
<div class="canvas-meta">
<span class="canvas-updated">{updated}</span>
{template_badge}
{public_badge}
</div>
</div>
<div class="canvas-actions">
<button class="btn btn-sm btn-primary" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">
Open
</button>
<button class="btn btn-sm btn-secondary" hx-get="/api/ui/canvas/{id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
Settings
</button>
<button class="btn btn-sm btn-danger" hx-delete="/api/canvas/{id}" hx-confirm="Delete this canvas?" hx-swap="none">
Delete
</button>
</div>
</div>"##
    )
}

fn render_canvas_row(canvas: &DbCanvas, element_count: i64) -> String {
    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "-".to_string());
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = canvas.id;
    let status = if canvas.is_public { "Public" } else { "Private" };

    format!(
        r##"<tr class="canvas-row" data-id="{id}">
<td class="canvas-name">
<a href="#" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">{name}</a>
</td>
<td class="canvas-description">{description}</td>
<td class="canvas-elements">{element_count}</td>
<td class="canvas-status">{status}</td>
<td class="canvas-updated">{updated}</td>
<td class="canvas-actions">
<button class="btn btn-xs btn-primary" hx-get="/api/ui/canvas/{id}/editor" hx-target="#canvas-editor">Open</button>
<button class="btn btn-xs btn-danger" hx-delete="/api/canvas/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
</td>
</tr>"##
    )
}

pub async fn canvas_list(
    State(state): State<Arc<CanvasState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let mut q = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_template) = query.is_template {
        q = q.filter(canvases::is_template.eq(is_template));
    }

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            canvases::name
                .ilike(pattern.clone())
                .or(canvases::description.ilike(pattern)),
        );
    }

    let db_canvases: Vec<DbCanvas> = match q
        .order(canvases::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load canvases"));
        }
    };

    if db_canvases.is_empty() {
        return Html(render_empty_state(
            "🎨",
            "No Canvases",
            "Create your first canvas to get started",
        ));
    }

    let mut rows = String::new();
    for canvas in &db_canvases {
        let element_count: i64 = canvas_elements::table
            .filter(canvas_elements::canvas_id.eq(canvas.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        rows.push_str(&render_canvas_row(canvas, element_count));
    }

    Html(format!(
        r##"<table class="table canvas-table">
<thead>
<tr>
<th>Name</th>
<th>Description</th>
<th>Elements</th>
<th>Status</th>
<th>Updated</th>
<th>Actions</th>
</tr>
</thead>
<tbody>{rows}</tbody>
</table>"##
    ))
}

pub async fn canvas_cards(
    State(state): State<Arc<CanvasState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let mut q = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_template) = query.is_template {
        q = q.filter(canvases::is_template.eq(is_template));
    }

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            canvases::name
                .ilike(pattern.clone())
                .or(canvases::description.ilike(pattern)),
        );
    }

    let db_canvases: Vec<DbCanvas> = match q
        .order(canvases::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load canvases"));
        }
    };

    if db_canvases.is_empty() {
        return Html(render_empty_state(
            "🎨",
            "No Canvases",
            "Create your first canvas to get started",
        ));
    }

    let mut cards = String::new();
    for canvas in &db_canvases {
        let element_count: i64 = canvas_elements::table
            .filter(canvas_elements::canvas_id.eq(canvas.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        cards.push_str(&render_canvas_card(canvas, element_count));
    }

    Html(format!(r##"<div class="canvas-grid">{cards}</div>"##))
}

pub async fn canvas_count(State(state): State<Arc<CanvasState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn canvas_templates_count(State(state): State<Arc<CanvasState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let count: i64 = canvases::table
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .filter(canvases::is_template.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn canvas_detail(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Canvas not found"));
        }
    };

    let elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let name = html_escape(&canvas.name);
    let description = canvas
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");
    let created = canvas.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = canvas.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let element_count = elements.len();
    let status = if canvas.is_public { "Public" } else { "Private" };
    let template_status = if canvas.is_template { "Yes" } else { "No" };

    Html(format!(
        r##"<div class="canvas-detail">
<div class="canvas-header">
<h2>{name}</h2>
<p class="canvas-description">{description}</p>
</div>
<div class="canvas-stats">
<div class="stat">
<span class="stat-label">Elements</span>
<span class="stat-value">{element_count}</span>
</div>
<div class="stat">
<span class="stat-label">Size</span>
<span class="stat-value">{width}x{height}</span>
</div>
<div class="stat">
<span class="stat-label">Background</span>
<span class="stat-value" style="background-color: {bg_color}; padding: 2px 8px;">{bg_color}</span>
</div>
<div class="stat">
<span class="stat-label">Status</span>
<span class="stat-value">{status}</span>
</div>
<div class="stat">
<span class="stat-label">Template</span>
<span class="stat-value">{template_status}</span>
</div>
</div>
<div class="canvas-dates">
<span>Created: {created}</span>
<span>Updated: {updated}</span>
</div>
<div class="canvas-actions">
<button class="btn btn-primary" hx-get="/api/ui/canvas/{canvas_id}/editor" hx-target="#canvas-editor" hx-swap="innerHTML">
Open Editor
</button>
<button class="btn btn-secondary" hx-get="/api/canvas/{canvas_id}/export" hx-vals='{{"format":"svg"}}' hx-target="#export-result">
Export SVG
</button>
<button class="btn btn-secondary" hx-get="/api/canvas/{canvas_id}/export" hx-vals='{{"format":"json"}}' hx-target="#export-result">
Export JSON
</button>
</div>
<div id="export-result"></div>
</div>"##,
        width = canvas.width,
        height = canvas.height
    ))
}

pub async fn canvas_editor(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Canvas not found"));
        }
    };

    let name = html_escape(&canvas.name);
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");

    Html(format!(
        r##"<div class="canvas-editor" data-canvas-id="{canvas_id}">
<div class="editor-toolbar">
<div class="toolbar-left">
<span class="canvas-title">{name}</span>
</div>
<div class="toolbar-center">
<button class="tool-btn" data-tool="select" title="Select">
<span>🔲</span>
</button>
<button class="tool-btn" data-tool="rectangle" title="Rectangle">
<span>⬜</span>
</button>
<button class="tool-btn" data-tool="ellipse" title="Ellipse">
<span>⭕</span>
</button>
<button class="tool-btn" data-tool="line" title="Line">
<span>📏</span>
</button>
<button class="tool-btn" data-tool="text" title="Text">
<span>📝</span>
</button>
<button class="tool-btn" data-tool="freehand" title="Freehand">
<span>✏️</span>
</button>
<button class="tool-btn" data-tool="sticky" title="Sticky Note">
<span>📌</span>
</button>
</div>
<div class="toolbar-right">
<button class="btn btn-sm" hx-post="/api/canvas/{canvas_id}/versions" hx-swap="none">
Save Version
</button>
<button class="btn btn-sm btn-secondary" hx-get="/api/ui/canvas/{canvas_id}" hx-target="#canvas-detail">
Close
</button>
</div>
</div>
<div class="editor-workspace">
<div class="canvas-container" style="background-color: {bg_color};">
<svg id="canvas-svg"
width="{width}"
height="{height}"
viewBox="0 0 {width} {height}"
hx-get="/api/ui/canvas/{canvas_id}/elements"
hx-trigger="load"
hx-swap="innerHTML">
</svg>
</div>
<div class="properties-panel" id="properties-panel">
<h4>Properties</h4>
<p class="text-muted">Select an element to edit its properties</p>
</div>
</div>
</div>"##,
        width = canvas.width,
        height = canvas.height
    ))
}

pub async fn canvas_elements_svg(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };

    let elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let mut svg_elements = String::new();
    for el in &elements {
        let svg = render_element_svg(el);
        svg_elements.push_str(&svg);
    }

    Html(svg_elements)
}

fn render_element_svg(element: &DbCanvasElement) -> String {
    let id = element.id;
    let x = element.x;
    let y = element.y;
    let width = element.width;
    let height = element.height;
    let rotation = element.rotation;

    let transform = if rotation != 0.0 {
        format!(
            r##" transform="rotate({rotation} {} {})""##,
            x + width / 2.0,
            y + height / 2.0
        )
    } else {
        String::new()
    };

    let fill = element
        .properties
        .get("fill_color")
        .and_then(|v| v.as_str())
        .unwrap_or("transparent");
    let stroke = element
        .properties
        .get("stroke_color")
        .and_then(|v| v.as_str())
        .unwrap_or("#000000");
    let stroke_width = element
        .properties
        .get("stroke_width")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0);
    let opacity = element
        .properties
        .get("opacity")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    match element.element_type.as_str() {
        "rectangle" => {
            let corner_radius = element
                .properties
                .get("corner_radius")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" rx="{corner_radius}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "ellipse" => {
            let cx = x + width / 2.0;
            let cy = y + height / 2.0;
            let rx = width / 2.0;
            let ry = height / 2.0;
            format!(
                r##"<ellipse data-element-id="{id}" cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "line" | "arrow" => {
            let x2 = x + width;
            let y2 = y + height;
            let marker = if element.element_type == "arrow" {
                r##" marker-end="url(#arrowhead)""##
            } else {
                ""
            };
            format!(
                r##"<line data-element-id="{id}" x1="{x}" y1="{y}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{marker}{transform} class="canvas-element"/>"##
            )
        }
        "text" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let font_size = element
                .properties
                .get("font_size")
                .and_then(|v| v.as_f64())
                .unwrap_or(16.0);
            let font_family = element
                .properties
                .get("font_family")
                .and_then(|v| v.as_str())
                .unwrap_or("sans-serif");
            let text_y = y + font_size;
            format!(
                r##"<text data-element-id="{id}" x="{x}" y="{text_y}" font-size="{font_size}" font-family="{font_family}" fill="{fill}" opacity="{opacity}"{transform} class="canvas-element">{text}</text>"##,
                text = html_escape(text)
            )
        }
        "freehand_path" => {
            let path_data = element
                .properties
                .get("path_data")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!(
                r##"<path data-element-id="{id}" d="{path_data}" fill="none" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "sticky" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let bg = element
                .properties
                .get("fill_color")
                .and_then(|v| v.as_str())
                .unwrap_or("#ffeb3b");
            format!(
                r##"<g data-element-id="{id}" class="canvas-element sticky-note"{transform}>
<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{bg}" stroke="#000" stroke-width="1"/>
<text x="{text_x}" y="{text_y}" font-size="14" fill="#000">{text}</text>
</g>"##,
                text_x = x + 8.0,
                text_y = y + 24.0,
                text = html_escape(text)
            )
        }
        _ => format!(
            r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
        ),
    }
}

pub async fn canvas_settings(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let canvas: DbCanvas = match canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
    {
        Ok(c) => c,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Canvas not found"));
        }
    };

    let name = html_escape(&canvas.name);
    let description = canvas.description.as_deref().map(html_escape).unwrap_or_default();
    let bg_color = canvas.background_color.as_deref().unwrap_or("#ffffff");
    let is_public_checked = if canvas.is_public { "checked" } else { "" };
    let is_template_checked = if canvas.is_template { "checked" } else { "" };

    Html(format!(
        r##"<div class="modal-header">
<h3>Canvas Settings</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="canvas-settings-form" hx-put="/api/canvas/{canvas_id}" hx-swap="none" hx-on::after-request="closeModal()">
<div class="form-group">
<label>Name</label>
<input type="text" name="name" value="{name}" required />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="3">{description}</textarea>
</div>
<div class="form-row">
<div class="form-group">
<label>Width</label>
<input type="number" name="width" value="{width}" min="100" max="10000" />
</div>
<div class="form-group">
<label>Height</label>
<input type="number" name="height" value="{height}" min="100" max="10000" />
</div>
</div>
<div class="form-group">
<label>Background Color</label>
<input type="color" name="background_color" value="{bg_color}" />
</div>
<div class="form-group">
<label class="checkbox-label">
<input type="checkbox" name="is_public" value="true" {is_public_checked} />
Public (anyone can view)
</label>
</div>
<div class="form-group">
<label class="checkbox-label">
<input type="checkbox" name="is_template" value="true" {is_template_checked} />
Save as template
</label>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Save Changes</button>
</div>
</form>"##,
        width = canvas.width,
        height = canvas.height
    ))
}

pub async fn new_canvas_form(State(_state): State<Arc<CanvasState>>) -> Html<String> {
    Html(
        r##"<div class="modal-header">
<h3>New Canvas</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="canvas-form" hx-post="/api/canvas" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#canvas-list', 'refresh');">
<div class="form-group">
<label>Name</label>
<input type="text" name="name" placeholder="My Canvas" required />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="3" placeholder="Describe your canvas..."></textarea>
</div>
<div class="form-row">
<div class="form-group">
<label>Width</label>
<input type="number" name="width" value="1920" min="100" max="10000" />
</div>
<div class="form-group">
<label>Height</label>
<input type="number" name="height" value="1080" min="100" max="10000" />
</div>
</div>
<div class="form-group">
<label>Background Color</label>
<input type="color" name="background_color" value="#ffffff" />
</div>
<div class="form-group">
<label class="checkbox-label">
<input type="checkbox" name="is_template" value="true" />
Create as template
</label>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Create Canvas</button>
</div>
</form>"##
        .to_string(),
    )
}

pub fn configure_canvas_ui_routes() -> Router<Arc<CanvasState>> {
    Router::new()
        .route("/api/ui/canvas", get(canvas_list))
        .route("/api/ui/canvas/cards", get(canvas_cards))
        .route("/api/ui/canvas/count", get(canvas_count))
        .route("/api/ui/canvas/templates/count", get(canvas_templates_count))
        .route("/api/ui/canvas/new", get(new_canvas_form))
        .route("/api/ui/canvas/{canvas_id}", get(canvas_detail))
        .route("/api/ui/canvas/{canvas_id}/editor", get(canvas_editor))
        .route("/api/ui/canvas/{canvas_id}/elements", get(canvas_elements_svg))
        .route("/api/ui/canvas/{canvas_id}/settings", get(canvas_settings))
}
