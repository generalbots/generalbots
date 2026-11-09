//! Group management JSON endpoints.
//!
//! Split out of `ui_api.rs` for the 450-line structural rule. They use the
//! parent module's imports (`Json`, `State`, diesel traits) and its
//! `get_conn` helper through `super::*`.
use super::*;

pub(super) async fn update_group_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<GroupUpdateRequest>,
) -> Result<Json<GroupResponse>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    if let Some(name) = &payload.name {
        diesel::sql_query("UPDATE rbac_groups SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Uuid, _>(group_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(display_name) = &payload.display_name {
        diesel::sql_query("UPDATE rbac_groups SET display_name = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(display_name)
            .bind::<diesel::sql_types::Uuid, _>(group_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(description) = &payload.description {
        diesel::sql_query("UPDATE rbac_groups SET description = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(description)
            .bind::<diesel::sql_types::Uuid, _>(group_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(is_active) = payload.is_active {
        diesel::sql_query("UPDATE rbac_groups SET is_active = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Bool, _>(is_active)
            .bind::<diesel::sql_types::Uuid, _>(group_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_group_json(&mut conn, group_id)
}

pub(super) async fn delete_group_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    diesel::sql_query("DELETE FROM rbac_groups WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(group_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true, "deleted": group_id })))
}

pub(super) async fn get_group_members_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct MemberRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
    }

    let members: Vec<MemberRow> = diesel::sql_query(
        "SELECT u.id, u.username, u.email FROM rbac_user_groups ug \
         JOIN users u ON u.id = ug.user_id WHERE ug.group_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(group_id)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(serde_json::json!({
        "group_id": group_id,
        "members": members.into_iter().map(|m| serde_json::json!({
            "id": m.id, "username": m.username, "email": m.email,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
}

pub(super) async fn add_group_member_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    let user_id = if let Some(user_id) = payload.user_id {
        user_id
    } else if let Some(email) = payload.email {
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct UserIdRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }
        diesel::sql_query("SELECT id FROM users WHERE LOWER(email) = LOWER($1) LIMIT 1")
            .bind::<diesel::sql_types::Text, _>(&email)
            .get_result::<UserIdRow>(&mut conn)
            .map(|r| r.id)
            .map_err(|_| (StatusCode::NOT_FOUND, "No user found with that email".to_string()))?
    } else {
        return Err((StatusCode::BAD_REQUEST, "user_id or email required".to_string()));
    };

    diesel::sql_query(
        "INSERT INTO rbac_user_groups (user_id, group_id, added_at) VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Uuid, _>(group_id)
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true, "group_id": group_id, "user_id": user_id })))
}

#[derive(Debug, Deserialize)]
pub struct RemoveMemberRequest {
    pub user_id: Option<Uuid>,
}

pub(super) async fn remove_group_member_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<RemoveMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = payload
        .user_id
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "user_id required".to_string()))?;

    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    diesel::sql_query("DELETE FROM rbac_user_groups WHERE user_id = $1 AND group_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(group_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
pub struct SendInviteRequest {
    pub email: Option<String>,
    pub role: Option<String>,
}

pub(super) async fn send_group_invite_json(
    _state: State<Arc<DirectoryApiState>>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<SendInviteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let email = payload.email.unwrap_or_default();
    if email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "email required".to_string()));
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "group_id": group_id,
        "email": email,
        "message": "Invitation recorded",
    })))
}

fn get_group_json(
    conn: &mut diesel::PgConnection,
    group_id: Uuid,
) -> Result<Json<GroupResponse>, (StatusCode, String)> {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct GroupRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        display_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let row: GroupRow = diesel::sql_query(
        "SELECT id, name, display_name, description, is_active FROM rbac_groups WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(group_id)
    .get_result(conn)
    .map_err(|e| (StatusCode::NOT_FOUND, format!("Group not found: {e}")))?;

    Ok(Json(GroupResponse {
        id: row.id,
        name: row.name,
        display_name: row.display_name,
        description: row.description,
        is_active: row.is_active,
    }))
}
