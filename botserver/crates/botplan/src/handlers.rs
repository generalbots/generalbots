//! Plan REST handlers (issue #2136).
//!
//! Every handler resolves the caller from a **verified** credential before it
//! touches the store, and every access to an existing plan is authorized
//! against its owner/members and tenant branch. A plan the caller cannot see
//! answers 404, so a guessed id never confirms its existence; a plan the caller
//! can see but not administer (membership) answers 403.
//!
//! `GET /api/plan/:plan_id` no longer creates anything: ids are minted
//! server-side by `POST /api/plan`, which is the only place a plan comes into
//! existence.

use super::*;
use crate::auth::{authorize, Access, Caller, PlanError};
use axum::extract::Path;
use axum::http::HeaderMap;

/// Loads a plan the caller is allowed to see, or fails closed.
pub(crate) async fn visible_plan(
    plan_id: &str,
    caller: &Caller,
    access: Access,
) -> Result<Plan, PlanError> {
    let plan = load_plan(plan_id).await.ok_or(PlanError::NotFound)?;
    authorize(&plan, caller, access)?;
    Ok(plan)
}

/// Applies `change` to an authorized plan under the write lock, then persists.
///
/// Authorization happens inside the same lock as the mutation, so a membership
/// revocation cannot slip between the check and the write.
async fn mutate_plan<F, T>(
    plan_id: &str,
    caller: &Caller,
    access: Access,
    change: F,
) -> Result<T, PlanError>
where
    F: FnOnce(&mut Plan) -> T,
{
    let outcome = {
        let mut store = get_store().write().await;
        let plan = store.get_mut(plan_id).ok_or(PlanError::NotFound)?;
        authorize(plan, caller, access)?;
        let value = change(plan);
        plan.updated_at = Utc::now();
        value
    };
    persist_all().await;
    Ok(outcome)
}

/// `POST /api/plan` — mints a plan owned by the verified caller.
pub async fn handle_create_plan(
    headers: HeaderMap,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let now = Utc::now();
    let plan = Plan {
        id: Uuid::new_v4().to_string(),
        title: req
            .title
            .map(|title| title.trim().to_string())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| "Novo Plano".to_string()),
        description: req.description,
        owner_id: caller.user_id.clone(),
        org_id: caller.org_id.map(|id| id.to_string()),
        branch_id: caller.branch_id.map(|id| id.to_string()),
        members: Vec::new(),
        tasks: HashMap::new(),
        created_at: now,
        updated_at: now,
    };
    {
        let mut store = get_store().write().await;
        store.insert(plan.id.clone(), plan.clone());
    }
    persist_all().await;
    log::info!("plan {} created by {}", plan.id, caller.user_id);
    Ok(Json(serde_json::json!({ "ok": true, "plan": plan })))
}

/// `GET /api/plan/:plan_id` — reads a plan the caller owns or belongs to.
pub async fn handle_get_plan(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let plan = visible_plan(&plan_id, &caller, Access::Read).await?;
    Ok(Json(serde_json::json!({ "ok": true, "plan": plan })))
}

/// `DELETE /api/plan/:plan_id` — owner deletes the plan and its snapshot.
pub async fn handle_delete_plan(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let removed = {
        let mut store = get_store().write().await;
        let plan = store.get(&plan_id).ok_or(PlanError::NotFound)?;
        authorize(plan, &caller, Access::Owner)?;
        store.remove(&plan_id).is_some()
    };
    delete_snapshot(&plan_id).await;
    Ok(Json(
        serde_json::json!({ "ok": true, "deleted": removed }),
    ))
}

/// `POST /api/plan/task` — adds a task to an existing, writable plan.
pub async fn handle_create_task(
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let now = Utc::now();
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title: req.title,
        description: None,
        status: req.status.unwrap_or_else(|| "todo".to_string()),
        priority: req.priority.unwrap_or_else(|| "medium".to_string()),
        assignee: req.assignee,
        start: req.start,
        end: req.end,
        progress: 0,
        tags: req.tags.unwrap_or_default(),
        depends_on: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by: Some(caller.user_id.clone()),
    };
    let created = task.clone();
    mutate_plan(&req.plan_id, &caller, Access::Write, move |plan| {
        plan.tasks.insert(created.id.clone(), created);
    })
    .await?;
    Ok(Json(serde_json::json!({ "ok": true, "task": task })))
}

/// `POST /api/plan/task/update` — patches a task of a writable plan.
pub async fn handle_update_task(
    headers: HeaderMap,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let updated = mutate_plan(&req.plan_id, &caller, Access::Write, move |plan| {
        let task = plan.tasks.get_mut(&req.task_id)?;
        if let Some(v) = req.title {
            task.title = v;
        }
        if let Some(v) = req.status {
            task.status = v;
        }
        if let Some(v) = req.priority {
            task.priority = v;
        }
        if let Some(v) = req.assignee {
            task.assignee = Some(v);
        }
        if let Some(v) = req.start {
            task.start = Some(v);
        }
        if let Some(v) = req.end {
            task.end = Some(v);
        }
        if let Some(v) = req.progress {
            task.progress = v.min(100);
        }
        if let Some(v) = req.tags {
            task.tags = v;
        }
        if let Some(v) = req.depends_on {
            task.depends_on = v;
        }
        task.updated_at = Utc::now();
        Some(task.clone())
    })
    .await?;
    match updated {
        Some(task) => Ok(Json(serde_json::json!({ "ok": true, "task": task }))),
        None => Ok(Json(
            serde_json::json!({ "ok": false, "error": "task not found" }),
        )),
    }
}

/// `POST /api/plan/task/depends` — replaces a task's dependency list.
pub async fn handle_set_dependencies(
    headers: HeaderMap,
    Json(req): Json<SetDependenciesRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let updated = mutate_plan(&req.plan_id, &caller, Access::Write, move |plan| {
        let task = plan.tasks.get_mut(&req.task_id)?;
        task.depends_on = req.depends_on;
        task.updated_at = Utc::now();
        Some(task.clone())
    })
    .await?;
    match updated {
        Some(task) => Ok(Json(serde_json::json!({ "ok": true, "task": task }))),
        None => Ok(Json(
            serde_json::json!({ "ok": false, "error": "task not found" }),
        )),
    }
}

/// `POST /api/plan/task/delete` — removes a task of a writable plan.
pub async fn handle_delete_task(
    headers: HeaderMap,
    Json(req): Json<DeleteTaskRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let removed = mutate_plan(&req.plan_id, &caller, Access::Write, move |plan| {
        plan.tasks.remove(&req.task_id).is_some()
    })
    .await?;
    if removed {
        Ok(Json(serde_json::json!({ "ok": true })))
    } else {
        Ok(Json(
            serde_json::json!({ "ok": false, "error": "task not found" }),
        ))
    }
}

/// `POST /api/plan/:plan_id/members` — owner adds a collaborator.
pub async fn handle_add_member(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let member = req.user_id.trim().to_string();
    if member.is_empty() {
        return Ok(Json(
            serde_json::json!({ "ok": false, "error": "user_id required" }),
        ));
    }
    let plan = mutate_plan(&plan_id, &caller, Access::Owner, move |plan| {
        if !plan.members.iter().any(|existing| existing == &member) {
            plan.members.push(member);
        }
        plan.clone()
    })
    .await?;
    Ok(Json(serde_json::json!({ "ok": true, "plan": plan })))
}

/// `DELETE /api/plan/:plan_id/members` — owner removes a collaborator.
pub async fn handle_remove_member(
    headers: HeaderMap,
    Path(plan_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<Value>, PlanError> {
    let caller = Caller::from_headers(&headers)?;
    let member = req.user_id.trim().to_string();
    let plan = mutate_plan(&plan_id, &caller, Access::Owner, move |plan| {
        plan.members.retain(|existing| existing != &member);
        plan.clone()
    })
    .await?;
    Ok(Json(serde_json::json!({ "ok": true, "plan": plan })))
}

#[cfg(test)]
#[path = "handler_tests.rs"]
mod handler_tests;
