//! Parallel sub-agent fan-out (#1694).
//!
//! `POST /api/runs/subagents` registers up to `MAX_SUBAGENTS` focused tasks
//! under one parent run and executes them concurrently through the configured
//! LLM provider. Each sub-agent is a durable row (`subagent_runs`) with its
//! own status/result, so progress survives the request and is queryable with
//! `GET /api/runs/subagents?parent_run_id=…`.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::bot::runs::subagents::{summarize, validate_tasks};

type ApiError = (StatusCode, Json<Value>);

fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn user_from_headers(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _roles) =
        crate::main_module::routes::anonymous_auth::resolve_token_user(token)
            .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid token"))?;
    Ok(crate::security::user_role::derive_stable_user_uuid(
        &claims_user,
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/runs/subagents", post(create).get(list))
}

#[derive(Debug, Deserialize)]
struct TaskBody {
    title: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    parent_run_id: String,
    thread_id: String,
    #[serde(default)]
    bot_id: Option<Uuid>,
    #[serde(default)]
    user_id: Option<Uuid>,
    #[serde(default)]
    session_id: Option<Uuid>,
    tasks: Vec<TaskBody>,
}

#[derive(Debug, diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct Row {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    result: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    created_at: String,
}

async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<Json<Value>, ApiError> {
    let caller = user_from_headers(&headers)?;
    let tasks: Vec<(String, String)> = body
        .tasks
        .iter()
        .map(|task| (task.title.clone(), task.prompt.clone()))
        .collect();
    validate_tasks(&tasks).map_err(|error| api_error(StatusCode::BAD_REQUEST, &error))?;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let mut ids = Vec::with_capacity(tasks.len());
    for (title, prompt) in &tasks {
        let row = diesel::sql_query(
            "INSERT INTO subagent_runs (parent_run_id, thread_id, title, prompt) \
             VALUES ($1, $2, $3, $4) RETURNING id::text AS id",
        )
        .bind::<diesel::sql_types::Text, _>(&body.parent_run_id)
        .bind::<diesel::sql_types::Text, _>(&body.thread_id)
        .bind::<diesel::sql_types::Text, _>(title)
        .bind::<diesel::sql_types::Text, _>(prompt)
        .get_result::<IdRow>(&mut conn)
        .map_err(|error| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("sub-agent insert failed: {error}"),
            )
        })?;
        ids.push(row.id);
    }

    let bot_uuid = body.bot_id.unwrap_or_else(Uuid::nil);
    let user_uuid = body.user_id.unwrap_or(caller);
    let session_uuid = body.session_id.unwrap_or_else(Uuid::nil);
    let provider = crate::core::bot::pipeline::enriched_llm::resolve_bot_llm_provider(
        &state,
        bot_uuid,
        user_uuid,
        session_uuid,
        "auto",
    )
    .await
    .ok()
    .flatten()
    .or_else(|| {
        state.llm_provider.clone().map(|provider| {
            (
                provider,
                std::env::var("LLM_MODEL").unwrap_or_default(),
                std::env::var("LLM_KEY").unwrap_or_default(),
            )
        })
    });
    let Some((llm, model, key)) = provider else {
        return Err(api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "no LLM provider is configured for sub-agents",
        ));
    };

    let tasks_for_run = tasks.clone();
    let ids_for_run = ids.clone();
    let thread = body.thread_id.clone();
    let state_for_run = state.clone();
    tokio::spawn(async move {
        let mut set = tokio::task::JoinSet::new();
        for ((title, prompt), id) in tasks_for_run.into_iter().zip(ids_for_run) {
            let llm = llm.clone();
            let model = model.clone();
            let key = key.clone();
            let pool = state_for_run.conn.clone();
            let thread = thread.clone();
            set.spawn(async move {
                update_status(&pool, &id, "running", None, None);
                publish(&thread, &id, &title, "running", None);
                let ask = format!(
                    "You are a focused sub-agent working on one part of a larger task. \
                     Task: {title}\n\n{prompt}\n\nAnswer only your part, concisely."
                );
                match llm
                    .generate(&ask, &serde_json::Value::Null, &model, &key)
                    .await
                {
                    Ok(text) => {
                        update_status(&pool, &id, "succeeded", Some(&text), None);
                        publish(&thread, &id, &title, "succeeded", Some(&text));
                    }
                    Err(error) => {
                        let message = error.to_string();
                        update_status(&pool, &id, "failed", None, Some(&message));
                        publish(&thread, &id, &title, "failed", Some(&message));
                    }
                }
            });
        }
        while set.join_next().await.is_some() {}
    });

    let total = ids.len();
    Ok(Json(json!({
        "ids": ids,
        "progress": { "total": total, "pending": total, "percent_done": 0 },
    })))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    parent_run_id: String,
}

async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let rows = diesel::sql_query(
        "SELECT id::text, title, status, result, error, created_at::text AS created_at \
         FROM subagent_runs WHERE parent_run_id = $1 ORDER BY created_at",
    )
    .bind::<diesel::sql_types::Text, _>(&query.parent_run_id)
    .load::<Row>(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("sub-agent load failed: {error}"),
        )
    })?;
    let statuses: Vec<&str> = rows.iter().map(|row| row.status.as_str()).collect();
    let progress = summarize(&statuses);
    Ok(Json(json!({
        "entries": rows.iter().map(|row| json!({
            "id": row.id,
            "title": row.title,
            "status": row.status,
            "result": row.result,
            "error": row.error,
            "created_at": row.created_at,
        })).collect::<Vec<_>>(),
        "progress": {
            "total": progress.total,
            "pending": progress.pending,
            "running": progress.running,
            "succeeded": progress.succeeded,
            "failed": progress.failed,
            "percent_done": progress.percent_done(),
            "overall": progress.overall(),
        },
    })))
}

fn update_status(
    pool: &DbPool,
    id: &str,
    status: &str,
    result: Option<&str>,
    error: Option<&str>,
) {
    let Ok(mut conn) = pool.get() else {
        return;
    };
    let _ = diesel::sql_query(
        "UPDATE subagent_runs SET status = $2, result = $3, error = $4, updated_at = NOW() \
         WHERE id = $1::uuid",
    )
    .bind::<diesel::sql_types::Text, _>(id)
    .bind::<diesel::sql_types::Text, _>(status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(result.map(str::to_string))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(error.map(str::to_string))
    .execute(&mut conn);
}

fn publish(thread_id: &str, id: &str, title: &str, status: &str, result: Option<&str>) {
    crate::core::bot::runs::bus::bus().publish(
        thread_id,
        json!({
            "type": "CUSTOM",
            "data": {
                "name": "gb.subagent",
                "payload": {
                    "id": id,
                    "title": title,
                    "status": status,
                    "result_excerpt": result.map(|text| text.chars().take(200).collect::<String>()),
                },
            },
        }),
    );
}
