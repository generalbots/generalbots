use super::*;
use axum::body::Bytes;
use axum::http::{header, HeaderMap};

pub async fn list_objectives(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Query(query): Query<ListObjectivesQuery>,
) -> Result<Json<Vec<Objective>>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, branch_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut db_query = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::branch_id.eq(branch_id))
            .into_boxed();

        if let Some(owner_id) = query.owner_id {
            db_query = db_query.filter(okr_objectives::owner_id.eq(owner_id));
        }
        if let Some(status) = query.status {
            db_query = db_query.filter(okr_objectives::status.eq(status));
        }
        if let Some(period) = query.period {
            // #2032 — the sidebar speaks `q2-2026` while older rows stored a
            // bare `Q2`. Match the canonical form first and fall back to the
            // legacy spelling so neither a new row nor an old one disappears
            // from the list.
            db_query = match legacy_period_alias(&period) {
                Some(legacy) => db_query.filter(
                    okr_objectives::period
                        .eq(period.clone())
                        .or(okr_objectives::period.eq(legacy)),
                ),
                None => db_query.filter(okr_objectives::period.eq(period)),
            };
        }
        if let Some(parent_id) = query.parent_id {
            db_query = db_query.filter(okr_objectives::parent_id.eq(parent_id));
        }

        db_query = db_query.order(okr_objectives::created_at.desc());

        if let Some(limit) = query.limit {
            db_query = db_query.limit(limit);
        }
        if let Some(offset) = query.offset {
            db_query = db_query.offset(offset);
        }

        db_query
            .load::<ObjectiveRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let objectives: Vec<Objective> = result.into_iter().map(record_to_objective).collect();
    Ok(Json(objectives))
}
/// #2032 — `q2-2026` has a legacy twin (`Q2`) written before the year became
/// part of the period. Returns the legacy spelling so the list can match
/// either.
fn legacy_period_alias(period: &str) -> Option<String> {
    let (head, _year) = period.split_once('-')?;
    let alias = match head.to_ascii_lowercase().as_str() {
        "q1" => "Q1",
        "q2" => "Q2",
        "q3" => "Q3",
        "q4" => "Q4",
        "h1" => "H1",
        "h2" => "H2",
        "annual" => "annual",
        _ => return None,
    };
    Some(alias.to_string())
}

/// #2032 — the New Objective fragment is a plain HTMX form, so the browser
/// sends `application/x-www-form-urlencoded` while the JS path sends JSON.
/// Accept both instead of rejecting one with 415. Blank form fields are dropped
/// so an empty `<input type="date">` does not fail `NaiveDate`.
fn decode_create_request(
    headers: &HeaderMap,
    body: &Bytes,
) -> Result<CreateObjectiveRequest, GoalsError> {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("json"))
        .unwrap_or(false);

    if is_json {
        return serde_json::from_slice(body)
            .map_err(|error| GoalsError::Validation(format!("invalid JSON body: {error}")));
    }

    let pairs: Vec<(String, String)> = serde_urlencoded::from_bytes(body)
        .map_err(|error| GoalsError::Validation(format!("invalid form body: {error}")))?;
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        if value.trim().is_empty() {
            continue;
        }
        map.insert(key, serde_json::Value::String(value));
    }
    serde_json::from_value(serde_json::Value::Object(map))
        .map_err(|error| GoalsError::Validation(format!("invalid objective: {error}")))
}

pub async fn create_objective(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    user: AuthenticatedUser,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Objective>, GoalsError> {
    let req = decode_create_request(&headers, &body)?;
    let (pool, get_bot_context) = state;
    let (org_id, branch_id) = get_bot_context();
    let owner_id = req.owner_id.unwrap_or(user.user_id);
    let owner_name = Some(user.username.clone());
    let now = Utc::now();

    let tags: Vec<Option<String>> = req.tags.unwrap_or_default().into_iter().map(Some).collect();

    let bot_id = bot_for_branch(&pool, branch_id);
    let new_objective = ObjectiveRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        owner_id,
        parent_id: req.parent_id,
        title: req.title.clone(),
        description: req.description.clone(),
        period: req.period.clone(),
        period_start: req.period_start,
        period_end: req.period_end,
        status: "draft".to_string(),
        progress: BigDecimal::from(0),
        visibility: req.visibility.as_ref().map(|v| v.to_str()).unwrap_or("team").to_string(),
        weight: BigDecimal::from(1),
        tags,
        created_at: now,
        updated_at: now,
        branch_id,
    };

    let record = new_objective.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        diesel::insert_into(okr_objectives::table)
            .values(&new_objective)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;
        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created objective: {} ({})", record.title, record.id);
    let mut obj = record_to_objective(record);
    obj.owner_name = owner_name;
    Ok(Json(obj))
}
pub async fn get_objective(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Objective>, GoalsError> {
    let (pool, _) = state;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_objectives::table
            .find(objective_id)
            .first::<ObjectiveRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    result
        .map(record_to_objective)
        .ok_or_else(|| GoalsError::NotFound("Objective not found".to_string()))
        .map(Json)
}
pub async fn update_objective(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<UpdateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let (pool, _) = state;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut objective = okr_objectives::table
            .find(objective_id)
            .first::<ObjectiveRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))?
            .ok_or_else(|| GoalsError::NotFound("Objective not found".to_string()))?;

        if let Some(title) = req.title {
            objective.title = title;
        }
        if let Some(description) = req.description {
            objective.description = Some(description);
        }
        if let Some(status) = req.status {
            objective.status = status.to_str().to_string();
        }
        if let Some(visibility) = req.visibility {
            objective.visibility = visibility.to_str().to_string();
        }
        if let Some(period_start) = req.period_start {
            objective.period_start = Some(period_start);
        }
        if let Some(period_end) = req.period_end {
            objective.period_end = Some(period_end);
        }
        if let Some(tags) = req.tags {
            objective.tags = tags.into_iter().map(Some).collect();
        }
        objective.updated_at = Utc::now();

        diesel::update(okr_objectives::table.find(objective_id))
            .set(&objective)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(objective)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Updated objective: {} ({})", result.title, result.id);
    Ok(Json(record_to_objective(result)))
}
pub async fn delete_objective(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let (pool, _) = state;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        let deleted = diesel::delete(okr_objectives::table.find(objective_id))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("Deleted objective: {objective_id}");
            Ok::<_, GoalsError>(())
        } else {
            Err(GoalsError::NotFound("Objective not found".to_string()))
        }
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}
pub async fn list_key_results(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Vec<KeyResult>>, GoalsError> {
    let (pool, _) = state;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_key_results::table
            .filter(okr_key_results::objective_id.eq(objective_id))
            .order(okr_key_results::created_at.asc())
            .load::<KeyResultRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let key_results: Vec<KeyResult> = result.into_iter().map(record_to_key_result).collect();
    Ok(Json(key_results))
}
pub async fn create_key_result(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    user: AuthenticatedUser,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<CreateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, branch_id) = get_bot_context();
    let owner_id = user.user_id;
    let now = Utc::now();

    let start_value = req.start_value.unwrap_or(0.0);

    let bot_id = bot_for_branch(&pool, branch_id);
    let new_kr = KeyResultRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        objective_id,
        owner_id,
        title: req.title.clone(),
        description: req.description.clone(),
        metric_type: req.metric_type.to_str().to_string(),
        start_value: BigDecimal::try_from(start_value).unwrap_or_else(|_| BigDecimal::from(0)),
        target_value: BigDecimal::try_from(req.target_value).unwrap_or_else(|_| BigDecimal::from(0)),
        current_value: BigDecimal::try_from(start_value).unwrap_or_else(|_| BigDecimal::from(0)),
        unit: req.unit.clone(),
        weight: BigDecimal::try_from(req.weight.unwrap_or(1.0) as f64).unwrap_or_else(|_| BigDecimal::from(1)),
        status: "not_started".to_string(),
        due_date: req.due_date,
        scoring_type: "linear".to_string(),
        created_at: now,
        updated_at: now,
        branch_id,
    };

    let record = new_kr.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        diesel::insert_into(okr_key_results::table)
            .values(&new_kr)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;
        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created key result: {} ({})", record.title, record.id);
    Ok(Json(record_to_key_result(record)))
}
