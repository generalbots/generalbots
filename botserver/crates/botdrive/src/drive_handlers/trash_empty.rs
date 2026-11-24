use super::*;

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
            #[diesel(sql_type = Bool)]
            is_dir: bool,
        }
        let rows = match diesel::sql_query(
            "SELECT bucket, path, is_dir FROM drive_trash WHERE user_id = $1",
        )
        .bind::<Text, _>(&uid)
        .load::<Row>(&mut conn)
        {
            Ok(rows) => rows,
            Err(e) => {
                log::warn!("Failed to list trashed files to empty: {:?}", e);
                Vec::new()
            }
        };

        for row in &rows {
            if row.is_dir {
                // A trashed folder keeps every child under its prefix; deleting
                // the row path alone orphaned the whole tree in .trash/.
                let prefix = format!("{}/", row.path.trim_end_matches('/'));
                if let Ok(keys) = drive.list_objects(&row.bucket, Some(&prefix)).await {
                    for key in keys {
                        revoke_links_for_key(&state, &row.bucket, &key)?;
                        if let Err(e) = drive.delete_object(&row.bucket, &key).await {
                            log::warn!("Failed to delete trashed object {}/{}: {e}", row.bucket, key);
                        }
                    }
                }
            }
            revoke_links_for_key(&state, &row.bucket, &row.path)?;
            if let Err(e) = drive.delete_object(&row.bucket, &row.path).await {
                log::warn!("Failed to delete trashed object {}/{}: {e}", row.bucket, row.path);
            }
        }

        if let Err(e) = diesel::sql_query("DELETE FROM drive_trash WHERE user_id = $1")
            .bind::<Text, _>(&uid)
            .execute(&mut conn)
        {
            log::warn!("Failed to clear trash rows for user {uid}: {e}");
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}
