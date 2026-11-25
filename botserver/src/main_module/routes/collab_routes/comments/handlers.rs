use super::super::*;

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/collab/comments?resource_type=&resource_id=` — threaded list
/// (top-level comments with inline replies), excluding soft-deleted rows.
pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<Vec<CommentItem>>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — scope from verified claims before exposing any comment content.
    if !has_collab_access(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(
            locale,
            StatusCode::FORBIDDEN,
            "error-collab-owner-only",
        ));
    }

    // When children are not requested, the LIKE patterns bind as the empty
    // string (which matches nothing — resource_type/resource_id are never
    // empty), so the OR branch is inert and the query behaves like an exact
    // match. Binding the same four parameters in both cases keeps the two
    // diesel bind chains type-compatible.
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

    let rows = diesel::sql_query(
        "SELECT id::text, resource_type, resource_id, author_id, author_name, \
                parent_id::text, body, mentions, created_at, updated_at, \
                resolved, resolved_by, resolved_at \
         FROM collab_comments \
         WHERE deleted = FALSE \
           AND ((resource_type = $1 AND resource_id = $2) \
             OR (resource_type LIKE $3 AND resource_id LIKE $4)) \
         ORDER BY created_at ASC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .load::<CommentRow>(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let reaction_rows = diesel::sql_query(
        "SELECT comment_id::text, user_id, emoji FROM collab_comment_reactions \
         WHERE comment_id IN (SELECT id FROM collab_comments \
                              WHERE deleted = FALSE \
                                AND ((resource_type = $1 AND resource_id = $2) \
                                  OR (resource_type LIKE $3 AND resource_id LIKE $4)))",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .load::<ReactionRow>(&mut conn)
    .map_or_else(
        |_| {
            log::warn!("defaulting after error at {}:{}", file!(), line!());
            Default::default()
        },
        |v| v,
    );

    let mut reactions: std::collections::HashMap<String, Vec<ReactionItem>> =
        std::collections::HashMap::new();
    for r in reaction_rows {
        reactions
            .entry(r.comment_id)
            .or_default()
            .push(ReactionItem {
                emoji: r.emoji,
                user_id: r.user_id,
            });
    }

    let mut top: Vec<CommentItem> = Vec::new();
    let mut replies: Vec<CommentItem> = Vec::new();
    for r in rows {
        let item = CommentItem {
            id: r.id.clone(),
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            author_id: r.author_id,
            author_name: r.author_name,
            parent_id: r.parent_id.clone(),
            body: r.body,
            mentions: serde_json::from_str(&r.mentions).map_or_else(
                |_| {
                    log::warn!("defaulting after error at {}:{}", file!(), line!());
                    Default::default()
                },
                |v| v,
            ),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
            resolved: r.resolved,
            resolved_by: r.resolved_by,
            resolved_at: r.resolved_at.map(|d| d.to_rfc3339()),
            reactions: reactions.remove(&r.id).map_or_else(Default::default, |v| v),
            replies: Vec::new(),
        };
        if r.parent_id.is_some() {
            replies.push(item);
        } else {
            top.push(item);
        }
    }

    let mut reply_map: std::collections::HashMap<String, Vec<CommentItem>> =
        std::collections::HashMap::new();
    for reply in replies {
        if let Some(parent) = reply.parent_id.clone() {
            reply_map.entry(parent).or_default().push(reply);
        }
    }
    for comment in &mut top {
        if let Some(children) = reply_map.remove(&comment.id) {
            comment.replies = children;
        }
    }

    Ok(Json(top))
}

/// `@mention` tokens are extracted and stored for notification/rendering.
pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<CreateCommentBody>,
) -> Result<Json<CommentItem>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    let body = req.body.trim().to_string();
    if body.is_empty() {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-comment-body-required",
        ));
    }
    if body.chars().count() > 8000 {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-comment-too-long",
        ));
    }

    let author_id = collab_user_id(&user);
    let author_name = collab_user_name(&user);
    // #2899 — the authenticated caller's active organization is the only
    // trusted tenant scope for @mention email delivery.
    let caller_org_id = user.organization_id;
    let mentions = extract_mentions(&body);
    let mentions_json = serde_json::to_string(&mentions).unwrap_or_else(|_| "[]".to_string());
    let parent_id: Option<uuid::Uuid> = match req.parent_id.as_deref() {
        None | Some("") => None,
        Some(p) => match uuid::Uuid::parse_str(p) {
            Ok(id) => Some(id),
            Err(_) => {
                return Err(err(
                    locale,
                    StatusCode::BAD_REQUEST,
                    "error-collab-invalid-parent",
                ))
            }
        },
    };

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — identity is bound from verified claims (actor id above); a
    // writer must also have a grant on the target resource.
    if !has_collab_access(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(
            locale,
            StatusCode::FORBIDDEN,
            "error-collab-owner-only",
        ));
    }

    #[derive(QueryableByName)]
    struct CreatedRow {
        #[diesel(sql_type = Text)]
        pub(crate) id: String,
        #[diesel(sql_type = Timestamptz)]
        pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_comments \
         (resource_type, resource_id, author_id, author_name, parent_id, body, mentions) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&author_id)
    .bind::<Text, _>(&author_name)
    .bind::<Nullable<SqlUuid>, _>(parent_id)
    .bind::<Text, _>(&body)
    .bind::<Text, _>(&mentions_json)
    .get_result::<CreatedRow>(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    botlog::safe_info!(
        "collab comment created by {author_id} on {}:{}",
        req.resource_type,
        req.resource_id
    );
    if !mentions.is_empty() {
        info!("collab mentions: {:?}", mentions);
    }

    record_activity(
        &mut conn,
        &author_id,
        &author_name,
        &req.resource_type,
        &req.resource_id,
        "comment",
        &serde_json::json!({ "body_len": body.chars().count(), "reply": req.parent_id.is_some() }),
    );

    // Fire mention emails off-thread so a slow/missing SMTP server never
    // delays the comment write. Compiled only with the `mail` feature.
    #[cfg(feature = "mail")]
    if !mentions.is_empty() {
        let pool = state.conn.clone();
        let task_author_id = author_id.clone();
        let task_author_name = author_name.clone();
        let task_resource_type = req.resource_type.clone();
        let task_resource_id = req.resource_id.clone();
        let task_mentions = mentions.clone();
        let task_body = body.clone();
        let task_caller_org_id = caller_org_id;
        tokio::task::spawn_blocking(move || {
            send_mention_emails(
                pool,
                task_author_id,
                task_author_name,
                task_resource_type,
                task_resource_id,
                task_mentions,
                task_body,
                task_caller_org_id,
            );
        });
    }

    Ok(Json(CommentItem {
        id: created.id,
        resource_type: req.resource_type,
        resource_id: req.resource_id,
        author_id,
        author_name,
        parent_id: req.parent_id,
        body,
        mentions,
        created_at: created.created_at.to_rfc3339(),
        updated_at: created.created_at.to_rfc3339(),
        resolved: false,
        resolved_by: None,
        resolved_at: None,
        reactions: Vec::new(),
        replies: Vec::new(),
    }))
}
