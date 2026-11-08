//! #1955 — REST surface of the local sync manager.

use axum::{extract::Path as AxumPath, http::StatusCode, Json};
use serde_json::{json, Value};

use super::*;

pub async fn list_jobs() -> Json<Value> {
    let jobs = tokio::task::spawn_blocking(load).await.unwrap_or_default();
    Json(json!({ "success": true, "jobs": jobs }))
}

pub async fn status() -> Json<Value> {
    match tokio::task::spawn_blocking(status_snapshot).await {
        Ok(value) => Json(value),
        Err(error) => Json(json!({ "success": false, "error": format!("status task: {error}") })),
    }
}

pub async fn create_job(Json(input): Json<JobInput>) -> (StatusCode, Json<Value>) {
    let local = clean_dir(&input.local_path, "local folder");
    let drive = clean_dir(&input.drive_path, "Drive folder");
    let (local, drive) = match (local, drive) {
        (Ok(local), Ok(drive)) => (local, drive),
        (Err(error), _) | (_, Err(error)) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error })))
        }
    };
    let name = input.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "a job name is required" })),
        );
    }
    let job = SyncJob {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        local_path: local.display().to_string(),
        drive_path: drive.display().to_string(),
        direction: input
            .direction
            .unwrap_or_else(|| "push".to_string())
            .to_lowercase(),
        interval_minutes: input.interval_minutes.unwrap_or(0),
        enabled: input.enabled.unwrap_or(true),
        last_run: None,
        last_status: "idle".to_string(),
        last_error: None,
        last_summary: None,
        last_duration_ms: None,
        conflicts: Vec::new(),
    };
    let mut jobs = load();
    if jobs.iter().any(|existing| existing.name == job.name) {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "success": false, "error": format!("a sync job named '{}' already exists", job.name) })),
        );
    }
    let mut next = jobs.clone();
    next.push(job.clone());
    if let Err(error) = save(&next) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": error })),
        );
    }
    jobs.clear();
    (StatusCode::OK, Json(json!({ "success": true, "job": job })))
}

pub async fn update_job(
    AxumPath(id): AxumPath<String>,
    Json(input): Json<JobInput>,
) -> (StatusCode, Json<Value>) {
    let mut jobs = load();
    let Some(index) = jobs.iter().position(|job| job.id == id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": format!("sync job {id} not found") })),
        );
    };
    let local = match clean_dir(&input.local_path, "local folder") {
        Ok(path) => path,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error }))),
    };
    let drive = match clean_dir(&input.drive_path, "Drive folder") {
        Ok(path) => path,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error }))),
    };
    let mut job = jobs[index].clone();
    job.name = input.name.trim().to_string();
    job.local_path = local.display().to_string();
    job.drive_path = drive.display().to_string();
    if let Some(direction) = input.direction {
        job.direction = direction.to_lowercase();
    }
    if let Some(interval) = input.interval_minutes {
        job.interval_minutes = interval;
    }
    if let Some(enabled) = input.enabled {
        job.enabled = enabled;
    }
    jobs[index] = job.clone();
    match save(&jobs) {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true, "job": job }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": error })),
        ),
    }
}

pub async fn delete_job(AxumPath(id): AxumPath<String>) -> (StatusCode, Json<Value>) {
    let mut jobs = load();
    let before = jobs.len();
    jobs.retain(|job| job.id != id);
    if jobs.len() == before {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": format!("sync job {id} not found") })),
        );
    }
    match save(&jobs) {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": error })),
        ),
    }
}

pub async fn run_job(AxumPath(id): AxumPath<String>) -> (StatusCode, Json<Value>) {
    let joined = tokio::task::spawn_blocking(move || run_job_blocking(&id)).await;
    match joined {
        Ok(Ok(job)) => (
            StatusCode::OK,
            Json(json!({ "success": true, "job": job, "state": job_last_status(&job) })),
        ),
        Ok(Err(error)) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": format!("run task: {error}") })),
        ),
    }
}

fn job_last_status(job: &SyncJob) -> &str {
    if job.last_status.is_empty() {
        "idle"
    } else {
        &job.last_status
    }
}

pub async fn list_conflicts(AxumPath(id): AxumPath<String>) -> (StatusCode, Json<Value>) {
    let joined = tokio::task::spawn_blocking(move || -> Result<Vec<SyncConflict>, String> {
        let jobs = load();
        let job = jobs
            .iter()
            .find(|job| job.id == id)
            .ok_or_else(|| format!("sync job {id} not found"))?;
        let local = clean_dir(&job.local_path, "local folder")?;
        let drive = clean_dir(&job.drive_path, "Drive folder")?;
        rsync_engine::scan_conflicts(&local, &drive)
    })
    .await;
    match joined {
        Ok(Ok(conflicts)) => (
            StatusCode::OK,
            Json(json!({ "success": true, "conflicts": conflicts })),
        ),
        Ok(Err(error)) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": format!("conflict task: {error}") })),
        ),
    }
}

pub async fn resolve_conflict(
    AxumPath(id): AxumPath<String>,
    Json(input): Json<ResolveInput>,
) -> (StatusCode, Json<Value>) {
    let joined =
        tokio::task::spawn_blocking(move || resolve_blocking(&id, &input.path, &input.resolution))
            .await;
    match joined {
        Ok(Ok(job)) => (StatusCode::OK, Json(json!({ "success": true, "job": job }))),
        Ok(Err(error)) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": error }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": format!("resolve task: {error}") })),
        ),
    }
}
