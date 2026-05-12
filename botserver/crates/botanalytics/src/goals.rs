use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put},
    Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use diesel::prelude::*;
use log::info;
use std::sync::Arc;
use uuid::Uuid;

use crate::goals_types::*;
use crate::schema::{okr_checkins, okr_key_results, okr_objectives, okr_templates};
use crate::{AuthenticatedUser, DbPool, GetBotContextFn};

pub async fn list_objectives(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Query(query): Query<ListObjectivesQuery>,
) -> Result<Json<Vec<Objective>>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut db_query = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(owner_id) = query.owner_id {
            db_query = db_query.filter(okr_objectives::owner_id.eq(owner_id));
        }
        if let Some(status) = query.status {
            db_query = db_query.filter(okr_objectives::status.eq(status));
        }
        if let Some(period) = query.period {
            db_query = db_query.filter(okr_objectives::period.eq(period));
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

pub async fn create_objective(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    user: AuthenticatedUser,
    Json(req): Json<CreateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, bot_id) = get_bot_context();
    let owner_id = req.owner_id.unwrap_or(user.user_id);
    let owner_name = Some(user.username.clone());
    let now = Utc::now();

    let tags: Vec<Option<String>> = req.tags.unwrap_or_default().into_iter().map(Some).collect();

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
    let (org_id, bot_id) = get_bot_context();
    let owner_id = user.user_id;
    let now = Utc::now();

    let start_value = req.start_value.unwrap_or(0.0);

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

pub async fn update_key_result(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<UpdateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let (pool, _) = state;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut kr = okr_key_results::table
            .find(key_result_id)
            .first::<KeyResultRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))?
            .ok_or_else(|| GoalsError::NotFound("Key result not found".to_string()))?;

        if let Some(title) = req.title {
            kr.title = title;
        }
        if let Some(description) = req.description {
            kr.description = Some(description);
        }
        if let Some(target_value) = req.target_value {
            kr.target_value = BigDecimal::try_from(target_value).unwrap_or_else(|_| BigDecimal::from(0));
        }
        if let Some(current_value) = req.current_value {
            kr.current_value = BigDecimal::try_from(current_value).unwrap_or_else(|_| BigDecimal::from(0));
        }
        if let Some(weight) = req.weight {
            kr.weight = BigDecimal::try_from(weight as f64).unwrap_or_else(|_| BigDecimal::from(1));
        }
        if let Some(due_date) = req.due_date {
            kr.due_date = Some(due_date);
        }
        if let Some(status) = req.status {
            kr.status = status.to_str().to_string();
        }
        kr.updated_at = Utc::now();

        diesel::update(okr_key_results::table.find(key_result_id))
            .set(&kr)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(kr)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Updated key result: {} ({})", result.title, result.id);
    Ok(Json(record_to_key_result(result)))
}

pub async fn delete_key_result(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let (pool, _) = state;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        let deleted = diesel::delete(okr_key_results::table.find(key_result_id))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("Deleted key result: {key_result_id}");
            Ok::<_, GoalsError>(())
        } else {
            Err(GoalsError::NotFound("Key result not found".to_string()))
        }
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn create_check_in(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    user: AuthenticatedUser,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<CreateCheckInRequest>,
) -> Result<Json<CheckIn>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, bot_id) = get_bot_context();
    let user_id = user.user_id;
    let now = Utc::now();

    let pool_clone = pool.clone();
    let previous_value = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        okr_key_results::table
            .find(key_result_id)
            .select(okr_key_results::current_value)
            .first::<BigDecimal>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let new_checkin = CheckInRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        key_result_id,
        user_id,
        previous_value,
        new_value: BigDecimal::try_from(req.new_value).unwrap_or_else(|_| BigDecimal::from(0)),
        note: req.note.clone(),
        confidence: req.confidence.as_ref().map(|c| c.to_str().to_string()),
        blockers: req.blockers.clone(),
        created_at: now,
    };

    let record = new_checkin.clone();
    let new_val = req.new_value;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        diesel::insert_into(okr_checkins::table)
            .values(&new_checkin)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        diesel::update(okr_key_results::table.find(key_result_id))
            .set((
                okr_key_results::current_value.eq(BigDecimal::try_from(new_val).unwrap_or_else(|_| BigDecimal::from(0))),
                okr_key_results::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created check-in for key result: {key_result_id}");
    Ok(Json(record_to_checkin(record)))
}

pub async fn get_check_in_history(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<Vec<CheckIn>>, GoalsError> {
    let (pool, _) = state;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_checkins::table
            .filter(okr_checkins::key_result_id.eq(key_result_id))
            .order(okr_checkins::created_at.desc())
            .load::<CheckInRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let history: Vec<CheckIn> = result.into_iter().map(record_to_checkin).collect();
    Ok(Json(history))
}

pub async fn get_dashboard(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
) -> Result<Json<GoalsDashboard>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let total: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let completed: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("completed"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let at_risk: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("at_risk"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let objectives = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .select(okr_objectives::progress)
            .load::<BigDecimal>(&mut conn)
            .unwrap_or_default();

        let avg_progress = if objectives.is_empty() {
            0.0
        } else {
            let sum: f32 = objectives.iter().map(|p| p.to_f32().unwrap_or(0.0)).sum();
            sum / objectives.len() as f32
        };

        let upcoming_krs = okr_key_results::table
            .filter(okr_key_results::org_id.eq(org_id))
            .filter(okr_key_results::bot_id.eq(bot_id))
            .filter(okr_key_results::due_date.is_not_null())
            .order(okr_key_results::due_date.asc())
            .limit(5)
            .load::<KeyResultRecord>(&mut conn)
            .unwrap_or_default();

        let upcoming_check_ins: Vec<UpcomingCheckIn> = upcoming_krs.into_iter().map(|kr| {
            UpcomingCheckIn {
                key_result_id: kr.id,
                key_result_title: kr.title,
                objective_title: String::new(),
                due_date: kr.due_date,
            }
        }).collect();

        Ok::<_, GoalsError>(GoalsDashboard {
            total_objectives: total,
            completed_objectives: completed,
            at_risk_objectives: at_risk,
            average_progress: avg_progress,
            upcoming_check_ins,
            recent_activity: vec![],
        })
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(result))
}

pub async fn get_alignment(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
) -> Result<Json<Vec<AlignmentNode>>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let objectives = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::parent_id.is_null())
            .load::<ObjectiveRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        let nodes: Vec<AlignmentNode> = objectives.into_iter().map(|obj| {
            let key_results = okr_key_results::table
                .filter(okr_key_results::objective_id.eq(obj.id))
                .load::<KeyResultRecord>(&mut conn)
                .unwrap_or_default()
                .into_iter()
                .map(record_to_key_result)
                .collect();

            AlignmentNode {
                objective: record_to_objective(obj),
                key_results,
                children: vec![],
            }
        }).collect();

        Ok::<_, GoalsError>(nodes)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(result))
}

pub async fn ai_suggest(
    Json(_req): Json<AISuggestRequest>,
) -> Result<Json<Vec<AISuggestion>>, GoalsError> {
    let suggestions = vec![
        AISuggestion {
            objective: ObjectiveTemplate {
                title: "Improve customer satisfaction".to_string(),
                description: "Enhance customer experience across all touchpoints".to_string(),
            },
            key_results: vec![
                KeyResultTemplate {
                    title: "Increase NPS score".to_string(),
                    metric_type: MetricType::Number,
                    suggested_target: Some(50.0),
                },
                KeyResultTemplate {
                    title: "Reduce support ticket resolution time".to_string(),
                    metric_type: MetricType::Number,
                    suggested_target: Some(24.0),
                },
            ],
            rationale: "Customer satisfaction directly impacts retention and growth".to_string(),
        },
    ];
    Ok(Json(suggestions))
}

pub async fn list_templates(
    State(state): State<(Arc<DbPool>, GetBotContextFn)>,
) -> Result<Json<Vec<GoalTemplate>>, GoalsError> {
    let (pool, get_bot_context) = state;
    let (org_id, _bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        okr_templates::table
            .filter(okr_templates::org_id.eq(org_id).or(okr_templates::is_system.eq(true)))
            .order(okr_templates::name.asc())
            .load::<TemplateRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let templates: Vec<GoalTemplate> = result.into_iter().map(|t| {
        let objective_template: ObjectiveTemplate = serde_json::from_value(t.objective_template)
            .unwrap_or(ObjectiveTemplate { title: String::new(), description: String::new() });
        let key_result_templates: Vec<KeyResultTemplate> = serde_json::from_value(t.key_result_templates)
            .unwrap_or_default();

        GoalTemplate {
            id: t.id,
            organization_id: t.org_id,
            name: t.name,
            description: t.description,
            category: t.category,
            objective_template,
            key_result_templates,
            is_system: t.is_system,
            created_at: t.created_at,
        }
    }).collect();

    Ok(Json(templates))
}

pub fn configure_goals_routes() -> Router<(Arc<DbPool>, GetBotContextFn)> {
    Router::new()
        .route("/api/goals/objectives", get(list_objectives).post(create_objective))
        .route("/api/goals/objectives/:id", get(get_objective).put(update_objective).delete(delete_objective))
        .route("/api/goals/objectives/:id/key-results", get(list_key_results).post(create_key_result))
        .route("/api/goals/key-results/:id", put(update_key_result).delete(delete_key_result))
        .route("/api/goals/key-results/:id/check-in", post(create_check_in))
        .route("/api/goals/key-results/:id/history", get(get_check_in_history))
        .route("/api/goals/dashboard", get(get_dashboard))
        .route("/api/goals/alignment", get(get_alignment))
        .route("/api/goals/templates", get(list_templates))
        .route("/api/goals/ai/suggest", post(ai_suggest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_objective_record_creation() {
        let now = Utc::now();
        let objective = ObjectiveRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            parent_id: None,
            title: "Test Objective".to_string(),
            description: Some("Test description".to_string()),
            period: "Q1 2025".to_string(),
            period_start: None,
            period_end: None,
            status: "draft".to_string(),
            progress: BigDecimal::from(0),
            visibility: "team".to_string(),
            weight: BigDecimal::from(1),
            tags: vec![Some("test".to_string())],
            created_at: now,
            updated_at: now,
        };

        assert_eq!(objective.title, "Test Objective");
        assert_eq!(objective.status, "draft");
        assert_eq!(objective.progress, BigDecimal::from(0));
    }

    #[test]
    fn test_key_result_record_creation() {
        let now = Utc::now();
        let key_result = KeyResultRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            objective_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            title: "Test Key Result".to_string(),
            description: Some("Test KR description".to_string()),
            metric_type: "numeric".to_string(),
            start_value: BigDecimal::from(0),
            target_value: BigDecimal::from(100),
            current_value: BigDecimal::from(0),
            unit: Some("units".to_string()),
            weight: BigDecimal::from(1),
            status: "not_started".to_string(),
            due_date: None,
            scoring_type: "linear".to_string(),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(key_result.title, "Test Key Result");
        assert_eq!(key_result.metric_type, "numeric");
        assert_eq!(key_result.target_value, BigDecimal::from(100));
        assert_eq!(key_result.status, "not_started");
    }

    #[test]
    fn test_check_in_record_creation() {
        let now = Utc::now();
        let check_in = CheckInRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            key_result_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            previous_value: Some(BigDecimal::from(0)),
            new_value: BigDecimal::from(50),
            note: Some("Progress update".to_string()),
            confidence: Some("high".to_string()),
            blockers: Some("No blockers".to_string()),
            created_at: now,
        };

        assert_eq!(check_in.new_value, BigDecimal::from(50));
        assert_eq!(check_in.confidence, Some("high".to_string()));
    }

    #[test]
    fn test_goals_error_display() {
        let db_error = GoalsError::Database("Connection failed".to_string());
        assert!(format!("{}", db_error).contains("Database error"));

        let not_found = GoalsError::NotFound("Objective not found".to_string());
        assert!(format!("{}", not_found).contains("not found"));

        let validation = GoalsError::Validation("Invalid input".to_string());
        assert!(format!("{}", validation).contains("Validation error"));
    }
}
