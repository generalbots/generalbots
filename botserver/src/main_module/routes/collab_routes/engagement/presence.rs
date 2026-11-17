use super::super::*;

/// `POST /api/collab/presence` — heartbeat with optional typing flag.
pub async fn update_presence(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<PresenceBody>,
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
    let name = collab_user_name(&user);
    let typing = req.typing.unwrap_or(false);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

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
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/presence?resource_type=&resource_id=` — who is active
/// (heartbeat within the last 60 seconds) on the resource.
pub async fn list_presence(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<Vec<PresenceItem>>, (StatusCode, Json<serde_json::Value>)> {
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

    let rows = diesel::sql_query(
        "SELECT user_id, user_name, typing, last_seen FROM collab_presence \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND last_seen > NOW() - INTERVAL '60 seconds' \
         ORDER BY typing DESC, last_seen DESC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<PresenceRow>(&mut conn)
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

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
