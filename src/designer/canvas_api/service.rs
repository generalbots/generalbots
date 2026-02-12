use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz, Uuid as DieselUuid};
use log::error;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::designer::canvas_api::db::{CanvasRow, TemplateRow, row_to_canvas};
use crate::designer::canvas_api::types::*;
use crate::designer::canvas_api::error::CanvasError;

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
            .bind::<diesel::sql_types::Nullable<Text>, _>(request.description.as_deref())
            .bind::<diesel::sql_types::Double, _>(width)
            .bind::<diesel::sql_types::Double, _>(height)
            .bind::<Text, _>(&elements_json)
            .bind::<Text, _>(&layers_json)
            .bind::<DieselUuid, _>(user_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to create canvas: {e}");
                CanvasError::CreateFailed
            })?;

        log::info!("Created canvas {} for org {}", id, organization_id);

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
        Ok(row_to_canvas(row))
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
                    r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}"{}{}>{}</text>"#,
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
                    r#"<path d="{}" fill="{}"{}{}{}"/>"#,
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
            AssetLibraryItem { id: Uuid::new_v4(), name: "Bot".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-bot.svg").to_string()), category: "General Bots".to_string(), tags: vec!["bot".to_string(), "assistant".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Analytics".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-analytics.svg").to_string()), category: "General Bots".to_string(), tags: vec!["analytics".to_string(), "chart".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Calendar".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-calendar.svg").to_string()), category: "General Bots".to_string(), tags: vec!["calendar".to_string(), "date".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Chat".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-chat.svg").to_string()), category: "General Bots".to_string(), tags: vec!["chat".to_string(), "message".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Drive".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-drive.svg").to_string()), category: "General Bots".to_string(), tags: vec!["drive".to_string(), "files".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Mail".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-mail.svg").to_string()), category: "General Bots".to_string(), tags: vec!["mail".to_string(), "email".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Meet".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-meet.svg").to_string()), category: "General Bots".to_string(), tags: vec!["meet".to_string(), "video".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Tasks".to_string(), asset_type: AssetType::Icon, url: None, svg_content: Some(include_str!("../../../../../botui/ui/suite/assets/icons/gb-tasks.svg").to_string()), category: "General Bots".to_string(), tags: vec!["tasks".to_string(), "todo".to_string()], is_system: true },
        ];

        let filtered = match asset_type {
            Some(t) => icons.into_iter().filter(|i| i.asset_type == t).collect(),
            None => icons,
        };

        Ok(filtered)
    }
}
