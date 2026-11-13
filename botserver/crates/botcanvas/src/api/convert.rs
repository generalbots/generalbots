use super::super::*;

pub(crate) fn db_to_canvas_element(db: DbCanvasElement) -> CanvasElement {
    let properties: ElementProperties = serde_json::from_value(db.properties).unwrap_or_default();
    CanvasElement {
        id: db.id,
        canvas_id: db.canvas_id,
        element_type: ElementType::from_label(&db.element_type),
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

pub(crate) fn db_to_canvas(db: DbCanvas, elements: Vec<CanvasElement>) -> Canvas {
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

#[cfg(test)]
#[path = "convert_tests.rs"]
mod tests;
