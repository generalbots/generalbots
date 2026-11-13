use super::super::*;
use super::convert::{db_to_canvas, db_to_canvas_element};

pub(crate) async fn list_canvases(
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
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Query error: {e}"),
            )
        })?;

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

pub(crate) async fn create_canvas(
    State(state): State<Arc<CanvasState>>,
    Json(req): Json<CreateCanvasRequest>,
) -> Result<Json<Canvas>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let branch_id = branch_for_bot(&mut conn, bot_id);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let user_id = Uuid::nil();

    let db_canvas = DbCanvas {
        id,
        branch_id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        width: req.width.unwrap_or(1920),
        height: req.height.unwrap_or(1080),
        background_color: Some(
            req.background_color
                .unwrap_or_else(|| "#ffffff".to_string()),
        ),
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
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Insert error: {e}"),
            )
        })?;

    let canvas = db_to_canvas(db_canvas, vec![]);
    Ok(Json(canvas))
}

pub(crate) async fn get_canvas(
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

pub(crate) async fn update_canvas(
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
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Update error: {e}"),
            )
        })?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    let canvas = db_to_canvas(db_canvas, elements);

    Ok(Json(canvas))
}

pub(crate) async fn delete_canvas(
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
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Delete error: {e}"),
            )
        })?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Canvas not found".to_string()))
    }
}

/// Branch that owns `bot`; the branch-scoped canvas/social rows are NOT NULL
/// against `branches`, so inserts resolve it here (#1517).
fn branch_for_bot(conn: &mut diesel::PgConnection, bot_id: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    diesel::sql_query("SELECT branch_id FROM bots WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<Row>(conn)
        .map(|r| r.branch_id)
        .unwrap_or_else(|_| Uuid::nil())
}
