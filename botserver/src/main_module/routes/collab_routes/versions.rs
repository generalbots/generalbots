use super::*;

/// `POST /api/collab/versions` — snapshot a document/presentation. Unchanged
/// content (same SHA-256 as the latest snapshot) is deduped into a no-op.
pub async fn snapshot_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SnapshotBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    if req.content.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Content is required"));
    }
    if req.content.len() > 2_000_000 {
        return Err(err(StatusCode::BAD_REQUEST, "Content too large"));
    }
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let content_hash = sha256_hex(&req.content);
    let name = req.name.trim().to_string();
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct LatestHash {
        #[diesel(sql_type = Text)]
        pub(crate) content_hash: String,
    }
    let latest_hash = diesel::sql_query(
        "SELECT content_hash FROM collab_versions \
         WHERE resource_type = $1 AND resource_id = $2 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .load::<LatestHash>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    .into_iter()
    .next();

    if let Some(latest) = latest_hash {
        if latest.content_hash == content_hash {
            return Ok(Json(serde_json::json!({ "skipped": true })));
        }
    }

    #[derive(QueryableByName)]
    struct CreatedVersion {
        #[diesel(sql_type = Text)]
        pub(crate) id: String,
        #[diesel(sql_type = Timestamptz)]
        pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_versions \
         (resource_type, resource_id, actor_id, actor_name, content, content_hash, name) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&actor_id)
    .bind::<Text, _>(&actor_name)
    .bind::<Text, _>(&req.content)
    .bind::<Text, _>(&content_hash)
    .bind::<Text, _>(&name)
    .get_result::<CreatedVersion>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "id": created.id,
        "actor_id": actor_id,
        "actor_name": actor_name,
        "name": name,
        "content_hash": content_hash,
        "size": req.content.len(),
        "created_at": created.created_at.to_rfc3339(),
    })))
}

/// `GET /api/collab/versions?resource_type=&resource_id=&limit=` — version
/// metadata (no content), newest first.
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<VersionsQuery>,
) -> Result<Json<Vec<VersionItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, content_hash, name, \
                octet_length(content)::bigint AS size, created_at \
         FROM collab_versions \
         WHERE resource_type = $1 AND resource_id = $2 \
         ORDER BY created_at DESC LIMIT $3",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<BigInt, _>(limit)
    .load::<VersionListRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<VersionItem> = rows
        .into_iter()
        .map(|r| VersionItem {
            id: r.id,
            actor_id: r.actor_id,
            actor_name: r.actor_name,
            name: r.name,
            content_hash: r.content_hash,
            size: r.size,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

/// `GET /api/collab/versions/:id` — full content of a single snapshot.
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VersionDetail>, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let row = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, content, content_hash, name, created_at \
         FROM collab_versions WHERE id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<VersionDetailRow>(&mut conn)
    .map_err(|e| err(StatusCode::NOT_FOUND, &format!("version: {e}")))?;

    Ok(Json(VersionDetail {
        id: row.id,
        actor_id: row.actor_id,
        actor_name: row.actor_name,
        name: row.name,
        size: row.content.len() as i64,
        content: row.content,
        content_hash: row.content_hash,
        created_at: row.created_at.to_rfc3339(),
    }))
}

/// `POST /api/collab/versions/:id/restore` — create a NEW current version from
/// an older snapshot's content. History is append-only; nothing is destroyed.
pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VersionDetail>, (StatusCode, Json<serde_json::Value>)> {
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let old = diesel::sql_query(
        "SELECT resource_type, resource_id, content, content_hash, name \
         FROM collab_versions WHERE id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<RestoreRow>(&mut conn)
    .map_err(|e| err(StatusCode::NOT_FOUND, &format!("version: {e}")))?;

    #[derive(QueryableByName)]
    struct CreatedVersion {
        #[diesel(sql_type = Text)]
        pub(crate) id: String,
        #[diesel(sql_type = Timestamptz)]
        pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_versions \
         (resource_type, resource_id, actor_id, actor_name, content, content_hash, name) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&old.resource_type)
    .bind::<Text, _>(&old.resource_id)
    .bind::<Text, _>(&actor_id)
    .bind::<Text, _>(&actor_name)
    .bind::<Text, _>(&old.content)
    .bind::<Text, _>(&old.content_hash)
    .bind::<Text, _>(&old.name)
    .get_result::<CreatedVersion>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    record_activity(
        &mut conn,
        &actor_id,
        &actor_name,
        &old.resource_type,
        &old.resource_id,
        "restore",
        &serde_json::json!({ "from_version": id.to_string() }),
    );

    Ok(Json(VersionDetail {
        id: created.id,
        actor_id,
        actor_name,
        name: old.name,
        size: old.content.len() as i64,
        content: old.content,
        content_hash: old.content_hash,
        created_at: created.created_at.to_rfc3339(),
    }))
}

/// `POST /api/collab/versions/:id/name` — name/rename a milestone version.
pub async fn name_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<NameBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let name = req.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 255 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid name"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let changed = diesel::sql_query("UPDATE collab_versions SET name = $1 WHERE id = $2")
        .bind::<Text, _>(&name)
        .bind::<SqlUuid, _>(id)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Version not found"));
    }
    info!("version {id} named by {}", collab_user_id(&user));
    Ok(Json(serde_json::json!({ "success": true, "name": name })))
}

/// `GET /api/collab/mentions/me?limit=` — comments that @mention the current
/// user (matched against their username and email), newest first. This backs
/// the in-app "you were mentioned" notification without any SMTP dependency.
pub async fn my_mentions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<MentionsQuery>,
) -> Result<Json<Vec<CommentItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = collab_user_id(&user);
    let uname = collab_user_name(&user);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let rows = diesel::sql_query(
        "SELECT id::text, resource_type, resource_id, author_id, author_name, \
                parent_id::text, body, mentions, created_at, updated_at, \
                resolved, resolved_by, resolved_at \
         FROM collab_comments \
         WHERE deleted = FALSE \
           AND (mentions::jsonb ? $1 OR mentions::jsonb ? $2) \
         ORDER BY created_at DESC LIMIT $3",
    )
    .bind::<Text, _>(&uname)
    .bind::<Text, _>(&uid)
    .bind::<BigInt, _>(limit)
    .load::<CommentRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<CommentItem> = rows
        .into_iter()
        .map(|r| CommentItem {
            id: r.id,
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            author_id: r.author_id,
            author_name: r.author_name,
            parent_id: r.parent_id,
            body: r.body,
            mentions: serde_json::from_str(&r.mentions).unwrap_or_default(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
            resolved: r.resolved,
            resolved_by: r.resolved_by,
            resolved_at: r.resolved_at.map(|d| d.to_rfc3339()),
            reactions: Vec::new(),
            replies: Vec::new(),
        })
        .collect();

    Ok(Json(items))
}
