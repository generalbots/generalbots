use super::*;

pub(crate) fn owner_or_admin(
    conn: &mut PgConnection,
    user: &AuthenticatedUser,
    resource_type: &str,
    resource_id: &str,
) -> bool {
    user.is_admin() || user.is_super_admin() || is_owner(conn, user, resource_type, resource_id)
}

/// `GET /api/collab/permissions?resource_type=&resource_id=` — list grants.
/// Owner (or admin) only.
pub async fn list_permissions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PermissionsQuery>,
) -> Result<Json<Vec<PermissionItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !owner_or_admin(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }
    let rows = diesel::sql_query(
        "SELECT grantee_type, grantee_id, role FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2 ORDER BY created_at ASC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<PermissionRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| PermissionItem {
                grantee_type: r.grantee_type,
                grantee_id: r.grantee_id,
                role: r.role,
            })
            .collect(),
    ))
}

/// `POST /api/collab/permissions` — add/update a grant. The first sharer of a
/// resource becomes its owner; only the owner (or admin) may grant afterwards.
pub async fn grant_permission(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<GrantBody>,
) -> Result<Json<PermissionItem>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    if !valid_grantee_type(&req.grantee_type) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid grantee_type"));
    }
    if !valid_editable_role(&req.role) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid role"));
    }
    let grantee = req.grantee_id.trim().to_string();
    if grantee.is_empty() || grantee.len() > 255 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid grantee_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !ensure_owner(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }

    diesel::sql_query(
        "INSERT INTO resource_permissions \
         (resource_type, resource_id, grantee_type, grantee_id, role) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (resource_type, resource_id, grantee_type, grantee_id) \
         DO UPDATE SET role = $5, updated_at = NOW()",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&req.grantee_type)
    .bind::<Text, _>(&grantee)
    .bind::<Text, _>(&req.role)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    record_activity(
        &mut conn,
        &collab_user_id(&user),
        &collab_user_name(&user),
        &req.resource_type,
        &req.resource_id,
        "share",
        &serde_json::json!({ "grantee": grantee, "role": req.role }),
    );

    Ok(Json(PermissionItem {
        grantee_type: req.grantee_type,
        grantee_id: grantee,
        role: req.role,
    }))
}

/// `DELETE /api/collab/permissions` — revoke a grant (owner only; the owner
/// grant itself is not revocable here — use transfer).
pub async fn revoke_permission(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RevokeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let grantee = req.grantee_id.trim().to_string();
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !owner_or_admin(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }

    let changed = diesel::sql_query(
        "DELETE FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2 AND grantee_type = $3 AND grantee_id = $4 \
           AND NOT (grantee_type = 'user' AND role = 'owner')",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&req.grantee_type)
    .bind::<Text, _>(&grantee)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Grant not found"));
    }
    record_activity(
        &mut conn,
        &collab_user_id(&user),
        &collab_user_name(&user),
        &req.resource_type,
        &req.resource_id,
        "share",
        &serde_json::json!({ "revoked": grantee }),
    );
    Ok(Json(serde_json::json!({ "success": true })))
}

/// `POST /api/collab/permissions/transfer` — transfer ownership to another
/// user. Owner (or admin) only; records a `transfer` audit event.
pub async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<TransferBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let new_owner = req.new_owner_id.trim().to_string();
    if new_owner.is_empty() || new_owner.len() > 255 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid new_owner_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !owner_or_admin(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }

    diesel::sql_query(
        "DELETE FROM resource_permissions \
         WHERE resource_type = $1 AND resource_id = $2 AND grantee_type = 'user' AND role = 'owner'",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    diesel::sql_query(
        "INSERT INTO resource_permissions \
         (resource_type, resource_id, grantee_type, grantee_id, role) \
         VALUES ($1, $2, 'user', $3, 'owner') \
         ON CONFLICT (resource_type, resource_id, grantee_type, grantee_id) \
         DO UPDATE SET role = 'owner', updated_at = NOW()",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&new_owner)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    record_activity(
        &mut conn,
        &collab_user_id(&user),
        &collab_user_name(&user),
        &req.resource_type,
        &req.resource_id,
        "transfer",
        &serde_json::json!({ "new_owner": new_owner }),
    );
    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/permissions/me?resource_type=&resource_id=` — the caller's
/// effective role (or null when they have no access). Docs/Slides use this to
/// gate the editor into read-only mode for viewers.
pub async fn my_role(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PermissionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let role = effective_role(&mut conn, &user, &params.resource_type, &params.resource_id);
    Ok(Json(serde_json::json!({
        "role": role,
        "user_id": collab_user_id(&user),
    })))
}

/// `POST /api/collab/links` — create a shareable link granting a role to
/// anyone with the link, optionally expiring.
pub async fn create_link(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<LinkBody>,
) -> Result<Json<LinkItem>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    if !valid_editable_role(&req.role) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid role"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !ensure_owner(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }

    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let expires_at = req
        .expires_in_hours
        .filter(|h| *h > 0)
        .map(|h| chrono::Utc::now() + chrono::Duration::hours(h));
    let created_at = chrono::Utc::now();

    diesel::sql_query(
        "INSERT INTO resource_links (token, resource_type, resource_id, role, expires_at, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<Text, _>(&token)
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&req.role)
    .bind::<Nullable<Timestamptz>, _>(expires_at)
    .bind::<Text, _>(&collab_user_id(&user))
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    record_activity(
        &mut conn,
        &collab_user_id(&user),
        &collab_user_name(&user),
        &req.resource_type,
        &req.resource_id,
        "share",
        &serde_json::json!({ "link_role": req.role }),
    );

    Ok(Json(LinkItem {
        token,
        role: req.role,
        expires_at: expires_at.map(|d| d.to_rfc3339()),
        created_at: created_at.to_rfc3339(),
    }))
}

/// `GET /api/collab/links?resource_type=&resource_id=` — list active links.
pub async fn list_links(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PermissionsQuery>,
) -> Result<Json<Vec<LinkItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    if !owner_or_admin(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }
    let rows = diesel::sql_query(
        "SELECT token, role, expires_at, created_at FROM resource_links \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND (expires_at IS NULL OR expires_at > NOW()) \
         ORDER BY created_at DESC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<LinkRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| LinkItem {
                token: r.token,
                role: r.role,
                expires_at: r.expires_at.map(|d| d.to_rfc3339()),
                created_at: r.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

/// `DELETE /api/collab/links/:token` — revoke a link (owner of its resource
/// only).
pub async fn revoke_link(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct LinkResource {
        #[diesel(sql_type = Text)]
        pub(crate) resource_type: String,
        #[diesel(sql_type = Text)]
        pub(crate) resource_id: String,
    }
    let target = diesel::sql_query(
        "SELECT resource_type, resource_id FROM resource_links WHERE token = $1",
    )
    .bind::<Text, _>(&token)
    .load::<LinkResource>(&mut conn)
    .map_err(|e| err(StatusCode::NOT_FOUND, &format!("link: {e}")))?
    .into_iter()
    .next()
    .ok_or_else(|| err(StatusCode::NOT_FOUND, "Link not found"))?;

    if !owner_or_admin(&mut conn, &user, &target.resource_type, &target.resource_id) {
        return Err(err(StatusCode::FORBIDDEN, "Owner only"));
    }
    diesel::sql_query("DELETE FROM resource_links WHERE token = $1")
        .bind::<Text, _>(&token)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true })))
}
