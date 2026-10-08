use super::*;

/// `DELETE /api/collab/comments/:id` — soft-delete (author or admin only).
pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let changed = if user.is_admin() || user.is_super_admin() {
        diesel::sql_query("UPDATE collab_comments SET deleted = TRUE WHERE id = $1")
            .bind::<SqlUuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    } else {
        diesel::sql_query(
            "UPDATE collab_comments SET deleted = TRUE WHERE id = $1 AND author_id = $2",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    };

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Comment not found"));
    }

    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            "delete",
            &serde_json::json!({ "comment_id": id.to_string() }),
        );
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `POST /api/collab/comments/:id/reactions` — toggle an emoji reaction
/// (adds if absent, removes if the same user already reacted with it).
pub async fn toggle_reaction(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ReactionBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let emoji = req.emoji.trim().to_string();
    if emoji.is_empty() || emoji.chars().count() > 8 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid emoji"));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let exists = diesel::sql_query(
        "SELECT 1 FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&emoji)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let added = if exists == 0 {
        diesel::sql_query(
            "INSERT INTO collab_comment_reactions (comment_id, user_id, emoji) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;
        true
    } else {
        diesel::sql_query(
            "DELETE FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;
        false
    };

    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            "reaction",
            &serde_json::json!({ "comment_id": id.to_string(), "emoji": emoji, "added": added }),
        );
    }

    Ok(Json(serde_json::json!({ "success": true, "added": added })))
}

// ---------------------------------------------------------------------------
// Resolve / read tracking (#863)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ResolveBody {
    pub resolved: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReadBody {
    pub resource_type: String,
    pub resource_id: String,
}

/// `POST /api/collab/comments/:id/resolve` — resolve or reopen a thread.
/// The comment author (or an admin) toggles the resolved state; reopening
/// clears the resolved_by/resolved_at audit fields.
pub async fn resolve_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ResolveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let resolved_by: Option<String> = if req.resolved { Some(uid.clone()) } else { None };
    let resolved_at: Option<chrono::DateTime<chrono::Utc>> =
        if req.resolved { Some(chrono::Utc::now()) } else { None };

    let changed = if user.is_admin() || user.is_super_admin() {
        diesel::sql_query(
            "UPDATE collab_comments \
             SET resolved = $1, resolved_by = $2, resolved_at = $3, updated_at = NOW() \
             WHERE id = $4",
        )
        .bind::<Bool, _>(req.resolved)
        .bind::<Nullable<Text>, _>(resolved_by)
        .bind::<Nullable<Timestamptz>, _>(resolved_at)
        .bind::<SqlUuid, _>(id)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    } else {
        diesel::sql_query(
            "UPDATE collab_comments \
             SET resolved = $1, resolved_by = $2, resolved_at = $3, updated_at = NOW() \
             WHERE id = $4 AND author_id = $5",
        )
        .bind::<Bool, _>(req.resolved)
        .bind::<Nullable<Text>, _>(resolved_by)
        .bind::<Nullable<Timestamptz>, _>(resolved_at)
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    };

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Comment not found or not yours to resolve"));
    }
    info!("collab comment {} set resolved={} by {uid}", id, req.resolved);

    let action = if req.resolved { "resolve" } else { "reopen" };
    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            action,
            &serde_json::json!({ "comment_id": id.to_string() }),
        );
    }

    Ok(Json(serde_json::json!({ "success": true, "resolved": req.resolved })))
}

/// `POST /api/collab/comments/read` — mark a resource's comments as read up to
/// now, so the unread badge resets when the user opens the panel.
pub async fn mark_comments_read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ReadBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    diesel::sql_query(
        "INSERT INTO collab_comment_reads (resource_type, resource_id, user_id, last_read_at) \
         VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (resource_type, resource_id, user_id) DO UPDATE SET last_read_at = NOW()",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&uid)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/comments/unread?resource_type=&resource_id=&include_children=`
/// — count of non-deleted comments (excluding the reader's own) created since
/// the reader last marked the resource read.
pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
    let ty_prefix = if params.include_children {
        format!("{}:%", params.resource_type)
    } else {
        String::new()
    };
    let id_prefix = if params.include_children {
        format!("{}:%", params.resource_id)
    } else {
        String::new()
    };
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        pub(crate) count: i64,
    }

    let row = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM collab_comments c \
         WHERE c.deleted = FALSE AND c.author_id <> $5 \
           AND ((c.resource_type = $1 AND c.resource_id = $2) \
             OR (c.resource_type LIKE $3 AND c.resource_id LIKE $4)) \
           AND c.created_at > COALESCE( \
                 (SELECT last_read_at FROM collab_comment_reads \
                  WHERE resource_type = $1 AND resource_id = $2 AND user_id = $5), \
                 TIMESTAMPTZ 'epoch')",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .bind::<Text, _>(&uid)
    .get_result::<CountRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "count": row.count })))
}

/// `POST /api/collab/presence` — heartbeat with optional typing flag.
pub async fn update_presence(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<PresenceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
    let name = collab_user_name(&user);
    let typing = req.typing.unwrap_or(false);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    diesel::sql_query(
        "INSERT INTO collab_presence (resource_type, resource_id, user_id, user_name, last_seen, typing) \
         VALUES ($1, $2, $3, $4, NOW(), $5) \
         ON CONFLICT (resource_type, resource_id, user_id) \
         DO UPDATE SET last_seen = NOW(), typing = $5, user_name = $4",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&name)
    .bind::<Bool, _>(typing)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/presence?resource_type=&resource_id=` — who is active
/// (heartbeat within the last 60 seconds) on the resource.
pub async fn list_presence(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<Vec<PresenceItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let rows = diesel::sql_query(
        "SELECT user_id, user_name, typing, last_seen FROM collab_presence \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND last_seen > NOW() - INTERVAL '60 seconds' \
         ORDER BY typing DESC, last_seen DESC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<PresenceRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<PresenceItem> = rows
        .into_iter()
        .map(|r| PresenceItem {
            user_id: r.user_id,
            user_name: r.user_name,
            typing: r.typing,
            last_seen: r.last_seen.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

/// `GET /api/activity?resource_type=&resource_id=&limit=&before=` — audit
/// timeline for a resource, newest first, cursor-paginated on `created_at`.
pub async fn list_activity(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ActivityQuery>,
) -> Result<Json<Vec<ActivityItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let before = params
        .before
        .as_deref()
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
        })
        .transpose()
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid `before` cursor"))?;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // A NULL `before` means "start from the newest"; the single 4-bind query
    // keeps both cases type-compatible with diesel.
    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, action, payload, created_at \
         FROM collab_activity \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND ($3::timestamptz IS NULL OR created_at < $3) \
         ORDER BY created_at DESC LIMIT $4",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Nullable<Timestamptz>, _>(before)
    .bind::<BigInt, _>(limit)
    .load::<ActivityRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<ActivityItem> = rows
        .into_iter()
        .map(|r| ActivityItem {
            id: r.id,
            actor_id: r.actor_id,
            actor_name: r.actor_name,
            action: r.action,
            payload: serde_json::from_str(&r.payload).unwrap_or(serde_json::Value::Null),
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

/// `POST /api/activity` — record an audit event from a frontend mutation
/// (edit/share/restore/transfer). The actor is always resolved server-side;
/// `action` is restricted to a small allow-list.
pub async fn record_activity_event(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RecordActivityBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    const ALLOWED: &[&str] = &[
        "create", "edit", "comment", "delete", "resolve", "reopen",
        "reaction", "share", "restore", "transfer",
    ];
    if !ALLOWED.contains(&req.action.as_str()) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid action"));
    }

    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    record_activity(
        &mut conn,
        &actor_id,
        &actor_name,
        &req.resource_type,
        &req.resource_id,
        &req.action,
        &req.payload,
    );

    Ok(Json(serde_json::json!({ "success": true })))
}
