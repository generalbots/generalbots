//! File version history (GB6-14).
//!
//! Every overwrite of an existing object first copies the previous revision to
//! `.versions/{user}/{path}/{timestamp}` in the same bucket. The history is
//! therefore durable (it survives a reload, a restart and a different device)
//! unlike the old localStorage-only snapshots.

use super::*;
use serde::Deserialize;

const VERSIONS_ROOT: &str = ".versions";
const VERSIONS_KEPT: usize = 20;

#[derive(Debug, Deserialize)]
pub struct VersionListBody {
    pub path: String,
    #[serde(default)]
    pub bucket: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionRestoreBody {
    pub path: String,
    pub version: String,
    #[serde(default)]
    pub bucket: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

fn versions_prefix(scope: &str, uid: &str, key: &str) -> String {
    format!("{VERSIONS_ROOT}/{}/{}/", user_scope::resolve_scope_prefix(scope, uid), key)
}

/// Copies the current object aside before it is overwritten. Called by the
/// upload handler; failures are logged and never block the write itself.
pub async fn snapshot_before_write(
    drive: &Arc<dyn botlib::traits::DriveRepository>,
    bucket: &str,
    key: &str,
    scope: &str,
    uid: &str,
) {
    match drive.get_object(bucket, key).await {
        Ok(existing) if !existing.is_empty() => {
            let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string();
            let version_key = format!("{}{}", versions_prefix(scope, uid, key), stamp);
            if let Err(error) = drive.copy_object(bucket, key, &version_key).await {
                warn!("version snapshot failed for {key}: {error}");
                return;
            }
            prune_versions(drive, bucket, scope, uid, key).await;
        }
        _ => {}
    }
}

/// Keeps the newest `VERSIONS_KEPT` revisions; the list is already sorted by
/// key, and the timestamp format sorts lexicographically.
async fn prune_versions(
    drive: &Arc<dyn botlib::traits::DriveRepository>,
    bucket: &str,
    scope: &str,
    uid: &str,
    key: &str,
) {
    let prefix = versions_prefix(scope, uid, key);
    let Ok(mut objects) = drive.list_objects_with_metadata(bucket, Some(&prefix)).await else {
        return;
    };
    if objects.len() <= VERSIONS_KEPT {
        return;
    }
    objects.sort_by(|a, b| a.key.cmp(&b.key));
    let excess = objects.len() - VERSIONS_KEPT;
    for object in objects.into_iter().take(excess) {
        if let Err(error) = drive.delete_object(bucket, &object.key).await {
            warn!("could not prune version {}: {error}", object.key);
        }
    }
}

/// POST /api/files/versions/list
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<VersionListBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.clone().unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let key = format!(
        "{}{}",
        resolve_scope_prefix(&scope, &uid),
        normalize_path(&req.path)
    );
    let prefix = versions_prefix(&scope, &uid, &key);
    let objects = drive
        .list_objects_with_metadata(&bucket, Some(&prefix))
        .await
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
    let mut versions: Vec<serde_json::Value> = objects
        .into_iter()
        .map(|object| {
            let stamp = object
                .key
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_string();
            serde_json::json!({
                "version": object.key,
                "stamp": stamp,
                "size": object.size,
            })
        })
        .collect();
    versions.sort_by(|a, b| b["stamp"].as_str().cmp(&a["stamp"].as_str()));
    Ok(Json(serde_json::json!({
        "path": req.path,
        "versions": versions,
    })))
}

/// POST /api/files/versions/restore
pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<VersionRestoreBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.clone().unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let key = format!(
        "{}{}",
        resolve_scope_prefix(&scope, &uid),
        normalize_path(&req.path)
    );
    // The version must live under this file's own history: a caller cannot ask
    // the endpoint to copy an arbitrary object over a target.
    let allowed_prefix = versions_prefix(&scope, &uid, &key);
    if !req.version.starts_with(&allowed_prefix) {
        return Err(err(StatusCode::BAD_REQUEST, "version does not belong to this file"));
    }
    // Keep the revision we are about to replace.
    snapshot_before_write(drive, &bucket, &key, &scope, &uid).await;
    drive
        .copy_object(&bucket, &req.version, &key)
        .await
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
    Ok(Json(serde_json::json!({ "success": true, "path": req.path })))
}
