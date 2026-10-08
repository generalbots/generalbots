use super::*;

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/collab/comments?resource_type=&resource_id=` — threaded list
/// (top-level comments with inline replies), excluding soft-deleted rows.
pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<Vec<CommentItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

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
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

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
    .unwrap_or_default();

    let mut reactions: std::collections::HashMap<String, Vec<ReactionItem>> =
        std::collections::HashMap::new();
    for r in reaction_rows {
        reactions
            .entry(r.comment_id)
            .or_default()
            .push(ReactionItem { emoji: r.emoji, user_id: r.user_id });
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
            mentions: serde_json::from_str(&r.mentions).unwrap_or_default(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
            resolved: r.resolved,
            resolved_by: r.resolved_by,
            resolved_at: r.resolved_at.map(|d| d.to_rfc3339()),
            reactions: reactions.remove(&r.id).unwrap_or_default(),
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

/// `POST /api/collab/comments` — create a comment (or reply via parent_id).
/// Deliver @mention notification emails (compiled only with the `mail`
/// feature, which pulls in `lettre`). Runs on a blocking thread, is strictly
/// fire-and-forget, and degrades to a warning log on any failure — never
/// panics and never blocks the comment write.
#[cfg(feature = "mail")]
pub(crate) fn send_mention_emails(
    pool: botcore::shared::utils::DbPool,
    author_id: String,
    author_name: String,
    resource_type: String,
    resource_id: String,
    mentions: Vec<String>,
    body: String,
) {
    use base64::{engine::general_purpose, Engine as _};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    #[derive(QueryableByName)]
    struct AccountRow {
        #[diesel(sql_type = Text)]
        pub(crate) email: String,
        #[diesel(sql_type = Nullable<Text>)]
        pub(crate) display_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        pub(crate) smtp_port: i32,
        #[diesel(sql_type = Text)]
        pub(crate) smtp_server: String,
        #[diesel(sql_type = Text)]
        pub(crate) username: String,
        #[diesel(sql_type = Text)]
        pub(crate) password_encrypted: String,
    }

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("mention email: db pool unavailable: {e}");
            return;
        }
    };

    // The comment author's active SMTP account (author_id is their email or
    // user uuid — see collab_user_id). Prefer the primary account.
    let account: AccountRow = match diesel::sql_query(
        "SELECT uea.email, uea.display_name, uea.smtp_port, \
         uea.smtp_server, uea.username, uea.password_encrypted \
         FROM user_email_accounts uea \
         JOIN users u ON u.id = uea.user_id \
         WHERE (u.email = $1 OR u.id::text = $1) AND uea.is_active = true \
         ORDER BY uea.is_primary DESC LIMIT 1",
    )
    .bind::<Text, _>(&author_id)
    .get_result(&mut conn)
    {
        Ok(a) => a,
        Err(e) => {
            warn!("mention email: no active SMTP account for author: {e}");
            return;
        }
    };

    let password = match general_purpose::STANDARD.decode(&account.password_encrypted) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                warn!("mention email: invalid password encoding for account");
                return;
            }
        },
        // Legacy rows may store the password in clear text.
        Err(_) => account.password_encrypted.clone(),
    };

    let from = match account.display_name.as_deref().filter(|n| !n.is_empty()) {
        Some(name) => format!("{name} <{}>", account.email),
        None => account.email.clone(),
    };

    // Resolve each mention token to a real user email, dedupe, skip the author.
    let mut recipients: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for token in &mentions {
        let token = token.trim();
        if token.is_empty() || !seen.insert(token.to_lowercase()) {
            continue;
        }
        #[derive(QueryableByName)]
        struct EmailRow {
            #[diesel(sql_type = Text)]
            pub(crate) email: String,
        }
        let rows: Vec<EmailRow> = diesel::sql_query(
            "SELECT email FROM users \
             WHERE LOWER(username) = LOWER($1) OR LOWER(email) = LOWER($1) LIMIT 1",
        )
        .bind::<Text, _>(token)
        .get_results(&mut conn)
        .unwrap_or_default();
        if let Some(row) = rows.into_iter().next() {
            let email = row.email.to_lowercase();
            if email == account.email.to_lowercase() {
                continue; // never email the author about their own comment
            }
            if !recipients.iter().any(|r| r.to_lowercase() == email) {
                recipients.push(row.email);
            }
        }
    }

    if recipients.is_empty() {
        return;
    }

    let preview: String = body.chars().take(400).collect();
    let subject = format!("{author_name} mentioned you in a comment");
    let text = format!(
        "{author_name} mentioned you on {resource_type} ({resource_id}).\n\n\"{preview}\"\n\nOpen the app to view and reply."
    );

    for recipient in recipients {
        let msg = match Message::builder()
            .from(match from.parse() {
                Ok(m) => m,
                Err(e) => {
                    warn!("mention email: invalid from address: {e}");
                    continue;
                }
            })
            .to(match recipient.parse() {
                Ok(m) => m,
                Err(e) => {
                    warn!("mention email: invalid recipient: {e}");
                    continue;
                }
            })
            .subject(subject.clone())
            .body(text.clone())
        {
            Ok(m) => m,
            Err(e) => {
                warn!("mention email: build failed: {e}");
                continue;
            }
        };

        let mailer = match SmtpTransport::relay(&account.smtp_server) {
            Ok(b) => b
                .port(u16::try_from(account.smtp_port).unwrap_or(587))
                .credentials(Credentials::new(account.username.clone(), password.clone()))
                .build(),
            Err(e) => {
                warn!("mention email: SMTP relay failed: {e}");
                continue;
            }
        };

        match mailer.send(&msg) {
            Ok(_) => info!("mention email sent to {recipient} (from {author_name})"),
            Err(e) => warn!("mention email: send failed for {recipient}: {e}"),
        }
    }
}

/// `@mention` tokens are extracted and stored for notification/rendering.
pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateCommentBody>,
) -> Result<Json<CommentItem>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let body = req.body.trim().to_string();
    if body.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Comment body is required"));
    }
    if body.chars().count() > 8000 {
        return Err(err(StatusCode::BAD_REQUEST, "Comment too long (max 8000 chars)"));
    }

    let author_id = collab_user_id(&user);
    let author_name = collab_user_name(&user);
    let mentions = extract_mentions(&body);
    let mentions_json = serde_json::to_string(&mentions).unwrap_or_else(|_| "[]".to_string());
    let parent_id: Option<uuid::Uuid> = match req.parent_id.as_deref() {
        None | Some("") => None,
        Some(p) => match uuid::Uuid::parse_str(p) {
            Ok(id) => Some(id),
            Err(_) => return Err(err(StatusCode::BAD_REQUEST, "Invalid parent_id")),
        },
    };

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

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
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    info!("collab comment created by {author_id} on {}:{}", req.resource_type, req.resource_id);
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
        tokio::task::spawn_blocking(move || {
            send_mention_emails(
                pool,
                task_author_id,
                task_author_name,
                task_resource_type,
                task_resource_id,
                task_mentions,
                task_body,
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
