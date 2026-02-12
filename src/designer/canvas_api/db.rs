use diesel::prelude::*;
use diesel::sql_types::{Bool, Double, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use uuid::Uuid;

use crate::designer::canvas_api::types::{Canvas, CanvasTemplate, Layer, CanvasElement};

#[derive(QueryableByName)]
pub struct CanvasRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub organization_id: Uuid,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Double)]
    pub width: f64,
    #[diesel(sql_type = Double)]
    pub height: f64,
    #[diesel(sql_type = Text)]
    pub background_color: String,
    #[diesel(sql_type = Bool)]
    pub grid_enabled: bool,
    #[diesel(sql_type = Integer)]
    pub grid_size: i32,
    #[diesel(sql_type = Bool)]
    pub snap_to_grid: bool,
    #[diesel(sql_type = Double)]
    pub zoom_level: f64,
    #[diesel(sql_type = Text)]
    pub elements_json: String,
    #[diesel(sql_type = Text)]
    pub layers_json: String,
    #[diesel(sql_type = DieselUuid)]
    pub created_by: Uuid,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
pub struct TemplateRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Text)]
    pub category: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub thumbnail_url: Option<String>,
    #[diesel(sql_type = Text)]
    pub canvas_data: String,
    #[diesel(sql_type = Bool)]
    pub is_system: bool,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub created_by: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn row_to_canvas(row: CanvasRow) -> Canvas {
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
