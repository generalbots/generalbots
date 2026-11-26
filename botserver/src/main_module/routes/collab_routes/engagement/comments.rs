use super::super::*;

/// `DELETE /api/collab/comments/:id` — soft-delete (author or admin only).
pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let changed = if user.is_admin() || user.is_super_admin() {
        diesel::sql_query("UPDATE collab_comments SET deleted = TRUE WHERE id = $1")
            .bind::<SqlUuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?
    } else {
        diesel::sql_query(
            "UPDATE collab_comments SET deleted = TRUE WHERE id = $1 AND author_id = $2",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn)
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?
    };

    if changed == 0 {
        return Err(err(
            locale,
            StatusCode::NOT_FOUND,
            "error-collab-comment-not-found",
        ));
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
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<ReactionBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    let emoji = req.emoji.trim().to_string();
    if emoji.is_empty() || emoji.chars().count() > 8 {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-emoji",
        ));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let exists = diesel::sql_query(
        "SELECT 1 FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&emoji)
    .execute(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let added = if exists == 0 {
        diesel::sql_query(
            "INSERT INTO collab_comment_reactions (comment_id, user_id, emoji) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
        true
    } else {
        diesel::sql_query(
            "DELETE FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
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
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<ResolveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let resolved_by: Option<String> = if req.resolved {
        Some(uid.clone())
    } else {
        None
    };
    let resolved_at: Option<chrono::DateTime<chrono::Utc>> = if req.resolved {
        Some(chrono::Utc::now())
    } else {
        None
    };

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
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?
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
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?
    };

    if changed == 0 {
        return Err(err(
            locale,
            StatusCode::NOT_FOUND,
            "error-collab-comment-resolve-forbidden",
        ));
    }
    botlog::safe_info!(
        "collab comment {} set resolved={} by {uid}",
        id,
        req.resolved
    );

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

    Ok(Json(
        serde_json::json!({ "success": true, "resolved": req.resolved }),
    ))
}

/// `POST /api/collab/comments/read` — mark a resource's comments as read up to
/// now, so the unread badge resets when the user opens the panel.
pub async fn mark_comments_read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<ReadBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    diesel::sql_query(
        "INSERT INTO collab_comment_reads (resource_type, resource_id, user_id, last_read_at) \
         VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (resource_type, resource_id, user_id) DO UPDATE SET last_read_at = NOW()",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&uid)
    .execute(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Optional query for the unread badge (#2946). Unlike `CommentQuery` — used
/// by the list/read handlers, where a resource is mandatory — the shell polls
/// this endpoint on every app open, before any app establishes a collab scope.
/// Missing or blank resource parts are therefore not a client error here.
#[derive(Debug, Deserialize)]
pub struct UnreadQuery {
    resource_type: Option<String>,
    resource_id: Option<String>,
    #[serde(default)]
    include_children: bool,
}

/// Pure validation of the optional unread query: only a named resource of a
/// known shape counts as a collab scope. `None`/blank (the shell's scope-less
/// poll) and malformed values are treated as "no scope" instead of a 400
/// (#2946).
pub(crate) fn unread_params_valid(resource_type: Option<&str>, resource_id: Option<&str>) -> bool {
    sanitize_resource(resource_type.unwrap_or(""), resource_id.unwrap_or(""))
}

/// Pure decision for the badge: run the scoped count only when the query names
/// a valid resource *and* the caller has access to it. A foreign-tenant
/// resource is answered with a zero count, so no cross-tenant count leaks and
/// no error is raised in the console (#2946).
pub(crate) fn unread_should_count(
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    has_access: bool,
) -> bool {
    has_access && unread_params_valid(resource_type, resource_id)
}

/// `GET /api/collab/comments/unread?resource_type=&resource_id=&include_children=`
/// — count of non-deleted comments (excluding the reader's own) created since
/// the reader last marked the resource read. A session with no collab scope
/// gets a valid zero payload (200) rather than a 400 (#2946).
pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<UnreadQuery>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    let ty = params.resource_type.as_deref();
    let id = params.resource_id.as_deref();
    // #2946 — the shell preloads this poll for apps that never use comments.
    // Return before touching the pool: no scope means no unread comments.
    if !unread_params_valid(ty, id) {
        return Ok(Json(serde_json::json!({ "count": 0 })));
    }
    let (ty, id) = match (ty, id) {
        (Some(ty), Some(id)) => (ty, id),
        _ => return Ok(Json(serde_json::json!({ "count": 0 }))),
    };
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // Keep the same verified-claims tenant gate as list_comments/create_comment
    // (#2768): a foreign resource yields a zero count, never a leaked count.
    if !unread_should_count(
        Some(ty),
        Some(id),
        has_collab_access(&mut conn, &user, ty, id),
    ) {
        return Ok(Json(serde_json::json!({ "count": 0 })));
    }

    let uid = collab_user_id(&user);
    let ty_prefix = if params.include_children {
        format!("{ty}:%")
    } else {
        String::new()
    };
    let id_prefix = if params.include_children {
        format!("{id}:%")
    } else {
        String::new()
    };

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
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .bind::<Text, _>(&uid)
    .get_result::<CountRow>(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "count": row.count })))
}

#[cfg(test)]
mod unread_tests {
    use super::{unread_params_valid, unread_should_count};

    #[test]
    fn missing_or_blank_scope_is_not_a_collab_scope() {
        assert!(!unread_params_valid(None, None));
        assert!(!unread_params_valid(Some(""), Some("")));
        assert!(!unread_params_valid(Some("docs"), None));
        assert!(!unread_params_valid(None, Some("doc-1")));
        assert!(!unread_params_valid(Some("  "), Some("doc-1")));
        assert!(!unread_should_count(None, None, true));
    }

    #[test]
    fn valid_scope_is_counted_when_access_is_granted() {
        assert!(unread_params_valid(Some("docs"), Some("doc-1")));
        assert!(unread_params_valid(Some("sheet:cell"), Some("cell-1")));
        assert!(unread_should_count(Some("docs"), Some("doc-1"), true));
        assert!(unread_should_count(
            Some("sheet:cell"),
            Some("cell-1"),
            true
        ));
    }

    #[test]
    fn foreign_tenant_resource_never_produces_a_badge_count() {
        assert!(!unread_should_count(
            Some("docs"),
            Some("foreign-doc"),
            false
        ));
        assert!(!unread_should_count(Some("docs"), Some("doc-1"), false));
    }

    #[test]
    fn malformed_resource_parameters_are_treated_as_no_scope() {
        assert!(!unread_params_valid(Some("docs;drop"), Some("doc-1")));
        assert!(!unread_should_count(Some("docs;drop"), Some("doc-1"), true));
        assert!(!unread_params_valid(Some("docs"), Some(&"r".repeat(256))));
    }
}
