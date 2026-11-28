use super::*;

pub async fn delete_bot(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateBotRequest>,
) -> Result<Json<DeleteBotResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin_user(&user) {
        return Err(err(StatusCode::FORBIDDEN, "Administrator access required"));
    }

    let bot_name = req.name.trim().to_lowercase();
    if bot_name.is_empty()
        || !bot_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Invalid bot name. Use lowercase letters, numbers, and hyphens (3-50 chars).",
        ));
    }
    if bot_name.len() < 3 || bot_name.len() > 50 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "Bot name must be between 3 and 50 characters.",
        ));
    }

    // Tenant isolation (#1387): an org admin may only delete bots inside its
    // own tenant. The client supplies the name, so without this check any
    // org admin could destroy ANY tenant's bot. The bot's real location is
    // resolved from its row: an org bot's objects live inside its branch's
    // `{branch}.gbai/` workspace in the shared `{tenant}.gborg` bucket, so the
    // check validates that bucket, not a `{bot}.gbai` one.
    let location = match state.conn.get() {
        Ok(mut conn) => botbasic_core::utils::bot_drive_location_for_name(&mut conn, &bot_name),
        Err(e) => {
            return Err(err(
                StatusCode::SERVICE_UNAVAILABLE,
                &format!("Database unavailable: {e}"),
            ));
        }
    };

    if !is_platform_admin(&state, &user) {
        let allowed = allowed_buckets_for(&state, &user);
        if !allowed.iter().any(|a| a == &location.bucket) {
            warn!(
                "drive: denied bot delete '{}' for user {:?} (not a member)",
                bot_name, user.email
            );
            return Err(err(StatusCode::FORBIDDEN, "Access denied for this bot"));
        }
    }

    let bucket_name = location.bucket.clone();
    let drive = get_drive(&state)?;

    // Only this bot's own objects are removed: the `{…}{bot}.gb` tree inside
    // its branch workspace. Deleting the bucket — or the whole branch
    // workspace — would destroy every sibling bot in the same org.
    let objects = drive
        .list_objects(&bucket_name, Some(&location.bot_tree_prefix(&bot_name)))
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to list bucket objects: {e}"),
            )
        })?;

    if !objects.is_empty() {
        drive
            .delete_objects(&bucket_name, objects)
            .await
            .map_err(|e| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Failed to delete bot objects: {e}"),
                )
            })?;
    }

    botlog::safe_info!("Bot '{}' deleted by user {}", bot_name, user.user_id);
    Ok(Json(DeleteBotResponse {
        status: "deleted".to_string(),
    }))
}
pub async fn open_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileBody>,
) -> Result<Json<OpenFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = &req.path;
    let ext = path.rsplit('.').next().unwrap_or("");
    let bucket = req.bucket.as_deref().unwrap_or("");
    let (app, url) = match ext {
        "bas" => (
            "designer".to_string(),
            format!("/suite/designer.html?file={path}&bucket={bucket}"),
        ),
        "txt" | "md" | "json" | "xml" | "html" | "css" | "js" => (
            "editor".to_string(),
            format!("/suite/editor.html?file={path}&bucket={bucket}"),
        ),
        // #2254 — a PDF opens in Docs as an editable document: the importer
        // reads its text back into paragraphs. The Drive preview modal (context
        // menu → Preview) still renders the original pages.
        "pdf" => (
            "docs".to_string(),
            format!("/suite/docs/docs.html?file={path}&bucket={bucket}"),
        ),
        "csv" | "xls" | "xlsx" | "ods" => (
            "sheets".to_string(),
            format!("/suite/sheet/sheet.html?bucket={bucket}&path={path}"),
        ),
        "docx" | "doc" | "odt" | "rtf" => (
            "docs".to_string(),
            format!("/suite/docs/docs.html?file={path}&bucket={bucket}"),
        ),
        // Presentations open in the Slides app (the app imports the file
        // from Drive through the existing /api/slides/import handler).
        "pptx" | "ppt" => (
            "slides".to_string(),
            format!("/suite/slides/slides.html?bucket={bucket}&path={path}"),
        ),
        "draw" => (
            "canvas".to_string(),
            format!("/suite/canvas/canvas.html?bucket={bucket}&path={path}"),
        ),
        // An SVG opened from Drive lands in the Canvas app, which imports it
        // as editable elements (or keeps it as background when it uses
        // constructs the canvas cannot model — #1708).
        "svg" => (
            "canvas".to_string(),
            format!("/suite/canvas/canvas.html?bucket={bucket}&path={path}"),
        ),
        // #1306 — images open the Photo Editor APP WINDOW (never a raw tab).
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "avif" => (
            "photo-editor".to_string(),
            format!("/suite/photos/photos-editor.html?bucket={bucket}&path={path}"),
        ),
        _ => (
            "preview".to_string(),
            format!("/suite/docs/docs.html?file={path}&bucket={bucket}"),
        ),
    };
    Ok(Json(OpenFileResponse { app, url }))
}
pub async fn quota(
    State(state): State<Arc<AppState>>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<serde_json::Value>)> {
    if let Some(quota) = cached_quota() {
        return Ok(Json(quota));
    }

    let drive = get_drive(&state)?;

    let mut total_size: u64 = 0;
    let mut total_buckets: u64 = 0;

    if let Ok(bucket_names) = drive.list_all_buckets().await {
        let mut tasks = tokio::task::JoinSet::new();
        for bname in bucket_names {
            let repo = Arc::clone(drive);
            tasks.spawn(async move {
                match repo.list_objects_with_metadata(&bname, None).await {
                    Ok(objects) => Some(visible_used_bytes(&objects)),
                    Err(e) => {
                        warn!("quota: listing bucket '{bname}' failed: {e}");
                        None
                    }
                }
            });
            if tasks.len() >= QUOTA_SWEEP_CONCURRENCY {
                if let Some(Ok(Some(bytes))) = tasks.join_next().await {
                    total_size = total_size.saturating_add(bytes);
                    total_buckets = total_buckets.saturating_add(1);
                }
            }
        }
        while let Some(joined) = tasks.join_next().await {
            if let Ok(Some(bytes)) = joined {
                total_size = total_size.saturating_add(bytes);
                total_buckets = total_buckets.saturating_add(1);
            }
        }
    }

    let total_bytes: u64 = (total_buckets + 1).saturating_mul(10 * 1024 * 1024 * 1024);
    let used = total_size;
    let available = total_bytes.saturating_sub(used);
    let percentage = if total_bytes > 0 {
        (used as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    let quota = QuotaResponse {
        used_bytes: used,
        total_bytes,
        available_bytes: available,
        percentage_used: percentage,
    };
    store_quota(&quota);

    Ok(Json(quota))
}
pub async fn list_favorites(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<StarItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            created_at: String,
        }
        let rows = match diesel::sql_query(
            "SELECT id::text, bucket, path, created_at::text FROM drive_stars WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        {
            Ok(rows) => rows,
            Err(e) => {
                log::warn!("Failed to list starred files: {:?}", e);
                Vec::new()
            }
        };

        let items: Vec<StarItem> = rows
            .into_iter()
            .map(|r| StarItem {
                id: r.id,
                bucket: r.bucket,
                path: r.path,
                created_at: r.created_at,
            })
            .collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}
pub async fn toggle_star(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<StarToggleBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = req.bucket.as_deref().unwrap_or("default");
    let path = &req.path;

    // A star only counts when the row was really written; the old handler
    // swallowed pool/SQL errors and always answered success, so the Starred
    // list stayed empty with no feedback.
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let result = if req.starred {
        diesel::sql_query(
            "INSERT INTO drive_stars (user_id, bucket, path) VALUES ($1, $2, $3) ON CONFLICT (user_id, bucket, path) DO NOTHING"
        )
        .bind::<Text, _>(uid)
        .bind::<Text, _>(bucket)
        .bind::<Text, _>(path)
        .execute(&mut conn)
    } else {
        diesel::sql_query(
            "DELETE FROM drive_stars WHERE user_id = $1 AND bucket = $2 AND path = $3",
        )
        .bind::<Text, _>(uid)
        .bind::<Text, _>(bucket)
        .bind::<Text, _>(path)
        .execute(&mut conn)
    };
    result.map_err(|e| {
        botlog::safe_error!("drive star toggle failed: {e}");
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not update the star",
        )
    })?;

    Ok(Json(SuccessResponse { success: true }))
}
pub async fn list_shared(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<ShareItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            owner_id: String,
            #[diesel(sql_type = Text)]
            recipient_id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            permissions: String,
            #[diesel(sql_type = Text)]
            created_at: String,
        }
        let rows = match diesel::sql_query(
            "SELECT id::text, owner_id, recipient_id, bucket, path, permissions, created_at::text FROM drive_shares WHERE recipient_id = $1 ORDER BY created_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        {
            Ok(rows) => rows,
            Err(e) => {
                log::warn!("Failed to list shared files: {:?}", e);
                Vec::new()
            }
        };

        let items: Vec<ShareItem> = rows
            .into_iter()
            .map(|r| ShareItem {
                id: r.id,
                owner_id: r.owner_id,
                recipient_id: r.recipient_id,
                bucket: r.bucket,
                path: r.path,
                permissions: r.permissions,
                created_at: r.created_at,
            })
            .collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}
/// Pure ownership predicate for `share_folder` (#2668). The `owner_id` on a
/// share row must be the authenticated caller: a client `user_id` is accepted
/// only when it equals the caller's id, or for a platform admin acting for
/// support tooling (the delegation `effective_uid` already allows). Fail closed.
pub(super) fn share_owner_allowed(
    caller_id: &str,
    requested_owner: Option<&str>,
    platform_admin: bool,
) -> bool {
    match requested_owner {
        None => true,
        Some(requested) => requested == caller_id || platform_admin,
    }
}

pub async fn share_folder(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateShareBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let caller_id = get_user_id(&user);
    let platform_admin = is_platform_admin(&state, &user);

    // #2668 — never trust a client-supplied owner: a non-admin naming another
    // user is forging a share attributed to that victim. Refuse, fail closed.
    if !share_owner_allowed(&caller_id, req.user_id.as_deref(), platform_admin) {
        warn!(
            "drive: denied share with forged owner id {:?} for user {:?}",
            req.user_id, user.email
        );
        return Err(err(
            StatusCode::FORBIDDEN,
            "Cannot create a share as another user",
        ));
    }
    let owner_id = effective_uid(&state, &user, req.user_id.as_deref());

    let scope = req.scope.unwrap_or_else(FileScope::default);
    // #2668 — validate bucket against the caller's tenant membership like
    // every other write handler, and pin user scope to the verified owner.
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(owner_id.as_str()),
        None,
        &user,
    )?;
    // Normalize path to strip traversal (`..`) before persisting.
    let path = normalize_path(&req.path);
    if path.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "A share path is required"));
    }
    let recipient_id = req.recipient_id.trim();
    if recipient_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "A recipient is required"));
    }
    let permissions = req.permissions.as_deref().unwrap_or("read");

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    diesel::sql_query(
        "INSERT INTO drive_shares (owner_id, recipient_id, bucket, path, permissions) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (owner_id, recipient_id, bucket, path) DO UPDATE SET permissions = $5"
    )
    .bind::<Text, _>(owner_id)
    .bind::<Text, _>(recipient_id)
    .bind::<Text, _>(bucket)
    .bind::<Text, _>(path)
    .bind::<Text, _>(permissions)
    .execute(&mut conn)
    .map_err(|e| {
        botlog::safe_error!("drive share insert failed: {e}");
        err(StatusCode::INTERNAL_SERVER_ERROR, "Could not share the folder")
    })?;

    Ok(Json(SuccessResponse { success: true }))
}
