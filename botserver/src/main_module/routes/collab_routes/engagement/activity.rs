use super::super::*;

/// `GET /api/activity?resource_type=&resource_id=&limit=&before=` — audit
/// timeline for a resource, newest first, cursor-paginated on `created_at`.
pub async fn list_activity(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ActivityQuery>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<Vec<ActivityItem>>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let before = params
        .before
        .as_deref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Utc)))
        .transpose()
        .map_err(|_| {
            err(
                locale,
                StatusCode::BAD_REQUEST,
                "error-collab-invalid-cursor",
            )
        })?;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — the audit timeline can contain sensitive payloads; never expose
    // it for a resource the caller cannot access.
    if !has_collab_access(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
    }

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
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

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
    request_locale: botcore::i18n::RequestLocale,
    Json(req): Json<RecordActivityBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ));
    }
    const ALLOWED: &[&str] = &[
        "create", "edit", "comment", "delete", "resolve", "reopen", "reaction", "share", "restore",
        "transfer",
    ];
    if !ALLOWED.contains(&req.action.as_str()) {
        return Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-action",
        ));
    }

    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    // #2768 — a client-supplied event must not be attributable to a foreign
    // resource; the actor is verified but the resource must be gated too.
    if !has_collab_access(&mut conn, &user, &req.resource_type, &req.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
    }

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
