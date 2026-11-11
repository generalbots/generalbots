//! #1955 — REST surface of the local sync manager.

use axum::{
    extract::Path as AxumPath,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};

use super::*;

/// Localized error for the sync surface (#2094). `success` and the absence of a
/// machine `code` are unchanged; only the human text is translated.
fn failed(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
    args: &[(&str, &str)],
) -> (StatusCode, Json<Value>) {
    let message = if args.is_empty() {
        botcore::i18n::t(locale, key)
    } else {
        botcore::i18n::t_args(locale, key, args)
    };
    (status, Json(json!({ "success": false, "error": message })))
}

pub async fn list_jobs() -> Json<Value> {
    let jobs = tokio::task::spawn_blocking(load).await.unwrap_or_default();
    Json(json!({ "success": true, "jobs": jobs }))
}

pub async fn status(headers: HeaderMap) -> Json<Value> {
    let locale = botcore::i18n::locale_from_headers(&headers);
    match tokio::task::spawn_blocking(status_snapshot).await {
        Ok(value) => Json(value),
        Err(error) => {
            let message = botcore::i18n::t_args(
                &locale,
                "error-drive-sync-task-failed",
                &[("task", "status"), ("error", error.to_string().as_str())],
            );
            Json(json!({ "success": false, "error": message }))
        }
    }
}

pub async fn create_job(
    headers: HeaderMap,
    Json(input): Json<JobInput>,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let local = clean_dir(
        &locale,
        &input.local_path,
        "error-drive-sync-field-local-folder",
    );
    let drive = clean_dir(
        &locale,
        &input.drive_path,
        "error-drive-sync-field-drive-folder",
    );
    let (local, drive) = match (local, drive) {
        (Ok(local), Ok(drive)) => (local, drive),
        (Err(error), _) | (_, Err(error)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": error })),
            )
        }
    };
    let name = input.name.trim();
    if name.is_empty() {
        return failed(
            &locale,
            StatusCode::BAD_REQUEST,
            "error-drive-sync-job-name-required",
            &[],
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
        return failed(
            &locale,
            StatusCode::CONFLICT,
            "error-drive-sync-job-exists",
            &[("name", job.name.as_str())],
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
    headers: HeaderMap,
    Json(input): Json<JobInput>,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut jobs = load();
    let Some(index) = jobs.iter().position(|job| job.id == id) else {
        return failed(
            &locale,
            StatusCode::NOT_FOUND,
            "error-drive-sync-job-not-found",
            &[("id", id.as_str())],
        );
    };
    let local = match clean_dir(
        &locale,
        &input.local_path,
        "error-drive-sync-field-local-folder",
    ) {
        Ok(path) => path,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": error })),
            )
        }
    };
    let drive = match clean_dir(
        &locale,
        &input.drive_path,
        "error-drive-sync-field-drive-folder",
    ) {
        Ok(path) => path,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": error })),
            )
        }
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

pub async fn delete_job(
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut jobs = load();
    let before = jobs.len();
    jobs.retain(|job| job.id != id);
    if jobs.len() == before {
        return failed(
            &locale,
            StatusCode::NOT_FOUND,
            "error-drive-sync-job-not-found",
            &[("id", id.as_str())],
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

pub async fn run_job(
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let for_run = locale.clone();
    let joined = tokio::task::spawn_blocking(move || run_job_blocking(&for_run, &id)).await;
    match joined {
        Ok(Ok(job)) => (
            StatusCode::OK,
            Json(json!({ "success": true, "job": job, "state": job_last_status(&job) })),
        ),
        Ok(Err(error)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": error })),
        ),
        Err(error) => failed(
            &locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-drive-sync-task-failed",
            &[("task", "run"), ("error", error.to_string().as_str())],
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

pub async fn list_conflicts(
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let for_scan = locale.clone();
    let joined = tokio::task::spawn_blocking(move || -> Result<Vec<SyncConflict>, String> {
        let jobs = load();
        let job = jobs.iter().find(|job| job.id == id).ok_or_else(|| {
            botcore::i18n::t_args(
                &for_scan,
                "error-drive-sync-job-not-found",
                &[("id", id.as_str())],
            )
        })?;
        let local = clean_dir(
            &for_scan,
            &job.local_path,
            "error-drive-sync-field-local-folder",
        )?;
        let drive = clean_dir(
            &for_scan,
            &job.drive_path,
            "error-drive-sync-field-drive-folder",
        )?;
        rsync_engine::scan_conflicts(&local, &drive)
    })
    .await;
    match joined {
        Ok(Ok(conflicts)) => (
            StatusCode::OK,
            Json(json!({ "success": true, "conflicts": conflicts })),
        ),
        Ok(Err(error)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": error })),
        ),
        Err(error) => failed(
            &locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-drive-sync-task-failed",
            &[("task", "conflict"), ("error", error.to_string().as_str())],
        ),
    }
}

pub async fn resolve_conflict(
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(input): Json<ResolveInput>,
) -> (StatusCode, Json<Value>) {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let for_resolve = locale.clone();
    let joined = tokio::task::spawn_blocking(move || {
        resolve_blocking(&for_resolve, &id, &input.path, &input.resolution)
    })
    .await;
    match joined {
        Ok(Ok(job)) => (StatusCode::OK, Json(json!({ "success": true, "job": job }))),
        Ok(Err(error)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": error })),
        ),
        Err(error) => failed(
            &locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-drive-sync-task-failed",
            &[("task", "resolve"), ("error", error.to_string().as_str())],
        ),
    }
}
