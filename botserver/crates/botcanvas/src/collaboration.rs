use super::*;
use crate::auth::CanvasUser;
use crate::tenant_scope::{
    authorization_error, authorize_canvas_action, load_canvas_grants, load_scoped_canvas,
    CanvasAction, CanvasApiError,
};

fn internal(context: &str, error: impl std::fmt::Display) -> CanvasApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{context}: {error}"))
}

/// Workspace-level collaborator (distinct users granted access to any canvas
/// in the caller's scope, with their permission role) — #1248.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceCollaborator {
    pub email: String,
    pub username: String,
    pub role: String,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct WorkspaceCollaboratorRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    email: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    role: String,
}

impl WorkspaceCollaboratorRow {
    fn into_collaborator(self) -> WorkspaceCollaborator {
        WorkspaceCollaborator {
            email: self.email,
            username: self.username,
            role: self.role,
        }
    }
}

/// #1248 — workspace collaborators for the Canvas app's `loadCollaborators()`:
/// `GET /api/canvas/collaborators` (no canvas id). Returns every user granted
/// access to any canvas in the caller's `(org, bot)` scope, deduplicated.
/// #2735: the scope comes from `get_bot_context`, never from the request.
pub(crate) async fn list_workspace_collaborators(
    State(state): State<Arc<CanvasState>>,
    caller: CanvasUser,
) -> Result<Json<Vec<WorkspaceCollaborator>>, CanvasApiError> {
    log::debug!(
        "canvas workspace collaborators requested by {}",
        caller.user_id
    );
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);

    let rows = diesel::sql_query(
        "SELECT DISTINCT u.email AS email, u.username AS username, cc.permission AS role \
         FROM canvas_collaborators cc \
         JOIN canvases c ON c.id = cc.canvas_id \
         JOIN users u ON u.id = cc.user_id \
         WHERE c.org_id = $1 AND c.bot_id = $2 \
         ORDER BY u.email",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .load::<WorkspaceCollaboratorRow>(&mut conn)
    .map_err(|e| internal("Query error", e))?;

    Ok(Json(
        rows.into_iter().map(|r| r.into_collaborator()).collect(),
    ))
}

pub(crate) async fn list_collaborators(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
) -> Result<Json<Vec<DbCanvasCollaborator>>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::ListCollaborators,
        caller.user_id,
        &[],
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let collaborators = canvas_collaborators::table
        .filter(canvas_collaborators::canvas_id.eq(canvas_id))
        .load(&mut conn)
        .map_err(|e| internal("Query error", e))?;

    Ok(Json(collaborators))
}

pub(crate) async fn add_collaborator(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
    Json(req): Json<AddCollaboratorRequest>,
) -> Result<Json<DbCanvasCollaborator>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    let grants = load_canvas_grants(&mut conn, canvas_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::AddCollaborator,
        caller.user_id,
        &grants,
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    // #2735: `req.user_id` is untrusted input. The grantee and the actor are
    // both the verified caller, so a forged body cannot grant access to a
    // different identity (nor to another tenant's canvas, refused above).
    let permission = match req
        .permission
        .as_deref()
        .map(str::trim)
        .map(|value| value.to_ascii_lowercase())
    {
        Some(value) if matches!(value.as_str(), "view" | "edit" | "editor" | "owner") => value,
        _ => "view".to_string(),
    };
    let collaborator = DbCanvasCollaborator {
        id: Uuid::new_v4(),
        canvas_id,
        user_id: caller.user_id,
        permission,
        added_by: Some(caller.user_id),
        added_at: Utc::now(),
    };

    diesel::insert_into(canvas_collaborators::table)
        .values(&collaborator)
        .execute(&mut conn)
        .map_err(|e| internal("Insert error", e))?;

    Ok(Json(collaborator))
}

pub(crate) async fn remove_collaborator(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, user_id)): Path<(Uuid, Uuid)>,
    caller: CanvasUser,
) -> Result<StatusCode, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    let grants = load_canvas_grants(&mut conn, canvas_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::RemoveCollaborator,
        caller.user_id,
        &grants,
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let deleted = diesel::delete(
        canvas_collaborators::table
            .filter(canvas_collaborators::canvas_id.eq(canvas_id))
            .filter(canvas_collaborators::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    .map_err(|e| internal("Delete error", e))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Collaborator not found".to_string()))
    }
}

pub(crate) async fn list_comments(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
) -> Result<Json<Vec<DbCanvasComment>>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::ListComments,
        caller.user_id,
        &[],
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let comments = canvas_comments::table
        .filter(canvas_comments::canvas_id.eq(canvas_id))
        .order(canvas_comments::created_at.asc())
        .load(&mut conn)
        .map_err(|e| internal("Query error", e))?;

    Ok(Json(comments))
}

pub(crate) async fn create_comment(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<DbCanvasComment>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    let grants = load_canvas_grants(&mut conn, canvas_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::CreateComment,
        caller.user_id,
        &grants,
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let now = Utc::now();
    let comment = DbCanvasComment {
        id: Uuid::new_v4(),
        canvas_id,
        element_id: req.element_id,
        parent_comment_id: req.parent_comment_id,
        author_id: caller.user_id,
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
        .map_err(|e| internal("Insert error", e))?;

    Ok(Json(comment))
}

pub(crate) async fn resolve_comment(
    State(state): State<Arc<CanvasState>>,
    Path((canvas_id, comment_id)): Path<(Uuid, Uuid)>,
    caller: CanvasUser,
) -> Result<Json<DbCanvasComment>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    let grants = load_canvas_grants(&mut conn, canvas_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::ResolveComment,
        caller.user_id,
        &grants,
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let now = Utc::now();
    diesel::update(
        canvas_comments::table
            .filter(canvas_comments::id.eq(comment_id))
            .filter(canvas_comments::canvas_id.eq(canvas_id)),
    )
    .set((
        canvas_comments::resolved.eq(true),
        canvas_comments::resolved_by.eq(Some(caller.user_id)),
        canvas_comments::resolved_at.eq(Some(now)),
        canvas_comments::updated_at.eq(now),
    ))
    .execute(&mut conn)
    .map_err(|e| internal("Update error", e))?;

    let comment = canvas_comments::table
        .filter(canvas_comments::id.eq(comment_id))
        .filter(canvas_comments::canvas_id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Comment not found".to_string()))?;

    Ok(Json(comment))
}

pub(crate) async fn list_versions(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
) -> Result<Json<Vec<DbCanvasVersion>>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::ListVersions,
        caller.user_id,
        &[],
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    let versions = canvas_versions::table
        .filter(canvas_versions::canvas_id.eq(canvas_id))
        .order(canvas_versions::version_number.desc())
        .load(&mut conn)
        .map_err(|e| internal("Query error", e))?;

    Ok(Json(versions))
}

pub(crate) async fn create_version(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
) -> Result<Json<DbCanvasVersion>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    let grants = load_canvas_grants(&mut conn, canvas_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::CreateVersion,
        caller.user_id,
        &grants,
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

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
    let elements_snapshot =
        serde_json::to_value(&db_elements).unwrap_or_else(|_| serde_json::json!([]));

    let version = DbCanvasVersion {
        id: Uuid::new_v4(),
        canvas_id,
        version_number: max_version.unwrap_or(0) + 1,
        name: None,
        elements_snapshot,
        created_by: caller.user_id,
        created_at: now,
    };

    diesel::insert_into(canvas_versions::table)
        .values(&version)
        .execute(&mut conn)
        .map_err(|e| internal("Insert error", e))?;

    Ok(Json(version))
}

pub(crate) async fn get_collaboration_info(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    caller: CanvasUser,
) -> Result<Json<Vec<CollaborationSession>>, CanvasApiError> {
    let (org_id, bot_id) = (state.get_bot_context)(&state.pool);
    let mut conn = state.pool.get().map_err(|e| internal("DB error", e))?;
    let canvas = load_scoped_canvas(&mut conn, canvas_id, org_id, bot_id)?;
    authorize_canvas_action(
        Some(&canvas),
        CanvasAction::ListCollaborators,
        caller.user_id,
        &[],
        org_id,
        bot_id,
    )
    .map_err(authorization_error)?;

    Ok(Json(vec![]))
}
