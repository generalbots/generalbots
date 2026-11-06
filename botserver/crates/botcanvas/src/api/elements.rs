use super::super::*;
use super::convert::db_to_canvas_element;

pub(crate) async fn list_elements(
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

pub(crate) async fn create_element(
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

pub(crate) async fn update_element(
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

pub(crate) async fn delete_element(
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
