use super::*;

pub async fn list_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<TrashItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            user_id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            original_path: String,
            #[diesel(sql_type = Bool)]
            is_dir: bool,
            #[diesel(sql_type = BigInt)]
            size: i64,
            #[diesel(sql_type = Text)]
            deleted_at: String,
            #[diesel(sql_type = Text)]
            expires_at: String,
        }
        let rows = diesel::sql_query(
            "SELECT id::text, user_id, bucket, path, original_path, is_dir, size::bigint, deleted_at::text, expires_at::text FROM drive_trash WHERE user_id = $1 ORDER BY deleted_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        let items: Vec<TrashItem> = rows.into_iter().map(|r| TrashItem {
            id: r.id,
            user_id: r.user_id,
            bucket: r.bucket,
            path: r.path,
            original_path: r.original_path,
            is_dir: r.is_dir,
            size: r.size as u64,
            deleted_at: r.deleted_at,
            expires_at: r.expires_at,
        }).collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}
pub async fn trash_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DeleteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket =
        resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None, &user)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let relative = normalize_path(&req.path);
    let key = format!("{prefix}{relative}");
    // #1640 — a distinct stamp per delete keeps repeated deletes of the same
    // path (and same-named files from different folders) from colliding.
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string();
    let trash_base = format!(".trash/{uid}/{stamp}");

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // A file is trashed as one object; a folder is trashed recursively and
    // recorded as a single row so restoring it brings the tree back together.
    if let Ok(data) = drive.get_object(&bucket, &key).await {
        let trash_key = format!("{trash_base}/{relative}");
        let size = data.len() as i64;
        drive
            .put_object(&bucket, &trash_key, data, None)
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Trash move failed: {e}")))?;
        let _ = drive.delete_object(&bucket, &key).await;
        let _ = diesel::sql_query(
            "INSERT INTO drive_trash (user_id, bucket, path, original_path, is_dir, size) \
             VALUES ($1, $2, $3, $4, false, $5)",
        )
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&bucket)
        .bind::<Text, _>(&trash_key)
        .bind::<Text, _>(&key)
        .bind::<BigInt, _>(size)
        .execute(&mut conn);
        botlog::safe_info!("File moved to trash: {key} -> {trash_key}");
        return Ok(Json(SuccessResponse { success: true }));
    }

    let folder_prefix = format!("{}/", key.trim_end_matches('/'));
    let keys = drive
        .list_objects(&bucket, Some(&folder_prefix))
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?;
    if keys.is_empty() {
        return Err(err(StatusCode::NOT_FOUND, "File not found"));
    }
    let mut total = 0i64;
    for child in keys {
        if child.ends_with('/') {
            continue;
        }
        let relative_child = child
            .strip_prefix(&prefix)
            .unwrap_or(child.as_str())
            .to_string();
        let Ok(data) = drive.get_object(&bucket, &child).await else {
            continue;
        };
        total += data.len() as i64;
        let dest = format!("{trash_base}/{relative_child}");
        if drive.put_object(&bucket, &dest, data, None).await.is_ok() {
            let _ = drive.delete_object(&bucket, &child).await;
        }
    }
    let _ = diesel::sql_query(
        "INSERT INTO drive_trash (user_id, bucket, path, original_path, is_dir, size) \
         VALUES ($1, $2, $3, $4, true, $5)",
    )
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&bucket)
    .bind::<Text, _>(&format!("{trash_base}/{relative}"))
    .bind::<Text, _>(&key)
    .bind::<BigInt, _>(total)
    .execute(&mut conn);

    botlog::safe_info!("Folder moved to trash: {key} -> {trash_base}/{relative}");
    Ok(Json(SuccessResponse { success: true }))
}
pub async fn restore_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RestoreTrashBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        bucket: String,
        #[diesel(sql_type = Text)]
        path: String,
        #[diesel(sql_type = Text)]
        original_path: String,
        #[diesel(sql_type = Bool)]
        is_dir: bool,
    }
    let rows = diesel::sql_query(
        "SELECT bucket, path, original_path, is_dir FROM drive_trash \
         WHERE id::text = $1 AND user_id = $2",
    )
    .bind::<Text, _>(&req.id)
    .bind::<Text, _>(&uid)
    .load::<Row>(&mut conn)
    .unwrap_or_default();

    let Some(row) = rows.into_iter().next() else {
        return Err(err(StatusCode::NOT_FOUND, "Trash item not found"));
    };

    if row.is_dir {
        let prefix = format!("{}/", row.path.trim_end_matches('/'));
        let keys = drive.list_objects(&row.bucket, Some(&prefix)).await.unwrap_or_default();
        for child in keys {
            if child.ends_with('/') {
                continue;
            }
            let relative = child.strip_prefix(&prefix).unwrap_or("");
            if relative.is_empty() {
                continue;
            }
            let destination = format!("{}/{}", row.original_path.trim_end_matches('/'), relative);
            let Ok(data) = drive.get_object(&row.bucket, &child).await else {
                continue;
            };
            // #1640 — restoring over a live file must not destroy it silently.
            snapshot_before_write(drive, &row.bucket, &destination, &scope, &uid).await;
            let _ = drive.put_object(&row.bucket, &destination, data, None).await;
            let _ = drive.delete_object(&row.bucket, &child).await;
        }
    } else if let Ok(data) = drive.get_object(&row.bucket, &row.path).await {
        snapshot_before_write(drive, &row.bucket, &row.original_path, &scope, &uid).await;
        let _ = drive
            .put_object(&row.bucket, &row.original_path, data, None)
            .await;
        let _ = drive.delete_object(&row.bucket, &row.path).await;
    }

    let _ = diesel::sql_query("DELETE FROM drive_trash WHERE id::text = $1 AND user_id = $2")
        .bind::<Text, _>(&req.id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn);

    botlog::safe_info!("Trash item restored: {}", row.original_path);
    Ok(Json(SuccessResponse { success: true }))
}

/// Deletes trash rows whose 30-day retention has elapsed. Returns the number
/// of rows purged. Called by the background loop started in
/// [`spawn_trash_purge`] and safe to run by hand.
pub async fn purge_expired_trash(state: &Arc<AppState>) -> Result<usize, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive storage not configured".to_string())?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("db pool: {e}"))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        bucket: String,
        #[diesel(sql_type = Text)]
        path: String,
        #[diesel(sql_type = Bool)]
        is_dir: bool,
    }
    let rows = diesel::sql_query(
        "SELECT id::text, bucket, path, is_dir FROM drive_trash WHERE expires_at <= NOW()",
    )
    .load::<Row>(&mut conn)
    .unwrap_or_default();
    if rows.is_empty() {
        return Ok(0);
    }

    let mut purged = 0usize;
    for row in &rows {
        if row.is_dir {
            let prefix = format!("{}/", row.path.trim_end_matches('/'));
            if let Ok(keys) = drive.list_objects(&row.bucket, Some(&prefix)).await {
                for key in keys {
                    let _ = drive.delete_object(&row.bucket, &key).await;
                }
            }
        } else {
            let _ = drive.delete_object(&row.bucket, &row.path).await;
        }
        let deleted = diesel::sql_query("DELETE FROM drive_trash WHERE id::text = $1")
            .bind::<Text, _>(&row.id)
            .execute(&mut conn)
            .unwrap_or(0);
        purged += deleted;
    }
    botlog::safe_info!("trash purge: removed {purged} expired row(s)");
    Ok(purged)
}

/// #1640 — hourly retention job. The interval is configurable for tests; the
/// default matches the 30-day promise (an expired row never survives long).
pub fn spawn_trash_purge(state: Arc<AppState>) {
    tokio::spawn(async move {
        let secs: u64 = std::env::var("GB_TRASH_PURGE_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value >= 30)
            .unwrap_or(3600);
        log::info!("[trash] purge loop started (every {secs}s)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            if let Err(e) = purge_expired_trash(&state).await {
                log::warn!("[trash] purge failed: {e}");
            }
        }
    });
}

pub async fn empty_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<EmptyTrashBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let uid = effective_uid(&state, &user, req.user_id.as_deref());

    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
        }
        let rows = diesel::sql_query(
            "SELECT bucket, path FROM drive_trash WHERE user_id = $1"
        )
        .bind::<Text, _>(&uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        for row in &rows {
            let _ = drive.delete_object(&row.bucket, &row.path).await;
        }

        let _ = diesel::sql_query("DELETE FROM drive_trash WHERE user_id = $1")
            .bind::<Text, _>(&uid)
            .execute(&mut conn);
    }

    Ok(Json(SuccessResponse { success: true }))
}
pub async fn upload_file_binary(
    State(state): State<Arc<AppState>>,
    body: String,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    let data = body.into_bytes();
    let bucket = state.bucket_name.clone();
    let key = format!("uploads/{}", uuid::Uuid::new_v4());

    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Upload failed: {e}")))?;

    Ok(Json(SuccessResponse { success: true }))
}
