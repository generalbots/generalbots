use super::*;

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
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-resource"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — scope from verified claims before exposing any comment content.
    if !has_collab_access(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
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
    .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);

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
            mentions: serde_json::from_str(&r.mentions).map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v),
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

#[cfg(feature = "mail")]
fn notification_email_enabled(conn: &mut PgConnection, user_id: &str, flag: &str) -> bool {
    #[derive(QueryableByName)]
    struct PrefRow {
        #[diesel(sql_type = Nullable<Text>)]
        pub(crate) value: Option<String>,
    }

    let row = diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences \
         WHERE user_id::text = $1 AND preference_key = 'notifications' LIMIT 1",
    )
    .bind::<Text, _>(user_id)
    .get_result::<PrefRow>(conn)
    .ok();
    let Some(row) = row else { return true };
    let Some(value) = row.value else { return true };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) else {
        return true;
    };
    parsed
        .get(flag)
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

/// Fetch users matching a raw @mention token together with all of their
/// organization memberships. Tenant filtering is intentionally *not* done in
/// SQL here: the pure [`super::notification_email::MentionTenant::in_scope`]
/// filter applies it uniformly to both the author and recipient lookups and is
/// unit-testable with a two-tenant fixture (#2899).
#[cfg(feature = "mail")]
fn mention_candidates(
    conn: &mut PgConnection,
    token: &str,
) -> Vec<super::notification_email::MentionCandidate> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Array<SqlUuid>)]
        org_ids: Vec<uuid::Uuid>,
    }
    diesel::sql_query(
        "SELECT u.id::text AS id, u.email, \
                COALESCE(array_agg(uo.org_id) FILTER (WHERE uo.org_id IS NOT NULL), '{}'::uuid[]) AS org_ids \
         FROM users u \
         LEFT JOIN user_organizations uo ON uo.user_id = u.id \
         WHERE LOWER(u.username) = LOWER($1) OR LOWER(u.email) = LOWER($1) \
            OR u.id::text = $1 \
         GROUP BY u.id, u.email",
    )
    .bind::<Text, _>(token)
    .load::<Row>(conn)
    .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v)
    .into_iter()
    .map(|row| super::notification_email::MentionCandidate {
        id: row.id,
        email: row.email,
        org_ids: row.org_ids,
    })
    .collect()
}

/// `POST /api/collab/comments` — create a comment (or reply via parent_id).
/// Deliver @mention notification emails through the platform notification
/// sender (compiled only with the `mail` feature, which pulls in `lettre`).
/// Runs on a blocking thread, is strictly fire-and-forget, and degrades to a
/// warning log on any failure — never panics and never blocks the comment
/// write.
#[cfg(feature = "mail")]
pub(crate) fn send_mention_emails(
    pool: botcore::shared::utils::DbPool,
    author_id: String,
    author_name: String,
    resource_type: String,
    resource_id: String,
    mentions: Vec<String>,
    body: String,
    caller_org_id: Option<uuid::Uuid>,
) {
    // #2899 — the tenant scope comes from the authenticated caller, never from
    // the mention token. Without a tenant, delivery is disabled (fail closed)
    // so a principal can never trigger a cross-tenant notification email.
    let Some(tenant) = super::notification_email::MentionTenant::from_caller(caller_org_id) else {
        warn!("mention email: no tenant scope for caller; delivery disabled (#2899)");
        return;
    };

    // Platform-owned sender only. Never use the author's or the recipient's
    // `user_email_accounts` mailbox as the sender: notification mail must not
    // leave through a personal Gmail/Outlook account, and delivery is simply
    // disabled when the workspace has no notification sender configured. The
    // recipient address itself comes from `notification_recipient` below.
    let Some(sender) = super::notification_email::NotificationSender::configured() else {
        warn!("mention email: no platform notification sender configured; delivery disabled");
        return;
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("mention email: db pool unavailable: {e}");
            return;
        }
    };

    // Resolve the author and every recipient through the same tenant-scoped
    // lookup. The SQL returns each matching user with all their organization
    // memberships; `tenant.in_scope` is the pure filter that discards users
    // outside the caller's tenant (#2899), including for self-exclusion.
    let author_candidates = mention_candidates(&mut conn, &author_id);
    let author = tenant.in_scope(&author_candidates).into_iter().next();
    let author_user_id = author.as_ref().map(|candidate| candidate.id.clone());
    let author_email = author
        .as_ref()
        .map(|candidate| candidate.email.to_lowercase());

    let mut recipients: Vec<super::notification_email::MentionCandidate> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for token in &mentions {
        let token = token.trim();
        if token.is_empty() || !seen.insert(token.to_lowercase()) {
            continue;
        }
        let candidates = mention_candidates(&mut conn, token);
        let Some(candidate) = tenant.in_scope(&candidates).into_iter().next() else {
            continue;
        };
        if author_user_id.as_deref() == Some(candidate.id.as_str())
            || author_email.as_deref() == Some(candidate.email.to_lowercase().as_str())
        {
            continue;
        }
        if recipients.iter().any(|r| r.id == candidate.id) {
            continue;
        }
        recipients.push((*candidate).clone());
    }

    if recipients.is_empty() {
        return;
    }

    let preview: String = body.chars().take(400).collect();

    for recipient in recipients {
        if !notification_email_enabled(&mut conn, &recipient.id, "email_mentions") {
            continue;
        }

        // #2094 — one message per recipient, in the language they picked.
        let locale = uuid::Uuid::parse_str(&recipient.id)
            .map(|user_id| botcore::i18n::locale_for_user(&mut conn, user_id))
            .unwrap_or(Default::default());
        let subject = botlib::i18n::translate(
            &locale,
            "email-mention-subject",
            &[("author", &author_name)],
        );
        let text = botlib::i18n::translate(
            &locale,
            "email-mention-body",
            &[
                ("author", &author_name),
                ("resource_type", &resource_type),
                ("resource_id", &resource_id),
                ("preview", &preview),
            ],
        );
        // #1853 — deliver to the mailbox the user configured (primary Mail
        // account or linked account); fall back to the profile address.
        let to = super::notification_email::notification_recipient(&mut conn, &recipient.id)
            .unwrap_or_else(|| recipient.email.clone());
        match sender.send(&to, &subject, &text) {
            Ok(()) => botlog::safe_info!("mention email sent to {to} (platform sender)"),
            Err(e) => warn!("mention email: send failed for {to}: {e}"),
        }
    }
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
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-resource"));
    }
    let body = req.body.trim().to_string();
    if body.is_empty() {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-comment-body-required"));
    }
    if body.chars().count() > 8000 {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-comment-too-long"));
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
            Err(_) => return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-parent")),
        },
    };

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — identity is bound from verified claims (actor id above); a
    // writer must also have a grant on the target resource.
    if !has_collab_access(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
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
    .map_err(|e| {
        err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

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
