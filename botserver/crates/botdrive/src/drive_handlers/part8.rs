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
    pub scope: Option<FileScope>,
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
    pub scope: Option<FileScope>,
    #[serde(default)]
    pub user_id: Option<String>,
}

fn versions_prefix(scope: &FileScope, uid: &str, key: &str) -> String {
    // `resolve_scope_prefix` already ends with `/` for user scope and is empty
    // for bot scope; concatenating keeps a single separator either way.
    format!("{VERSIONS_ROOT}/{}{key}/", resolve_scope_prefix(scope, uid))
}

/// A restore may only copy from the file's own history. The stored version key
/// must extend the prefix (an exact prefix is a directory, not a revision).
fn version_belongs_to_file(allowed_prefix: &str, version: &str) -> bool {
    version.len() > allowed_prefix.len() && version.starts_with(allowed_prefix)
}

/// Copies the current object aside before it is overwritten. Called by the
/// upload handler; failures are logged and never block the write itself.
pub async fn snapshot_before_write(
    drive: &Arc<dyn botlib::traits::DriveRepository>,
    bucket: &str,
    key: &str,
    scope: &FileScope,
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
    scope: &FileScope,
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
    if !version_belongs_to_file(&allowed_prefix, &req.version) {
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

#[cfg(test)]
mod tests {
    use super::{version_belongs_to_file, versions_prefix};
    use crate::drive_types::FileScope;

    /// History lives under the caller's own scope and under the file's own key,
    /// so another user (or another file) can never be restored over this one.
    #[test]
    fn history_prefix_is_scoped_to_user_and_file() {
        assert_eq!(
            versions_prefix(&FileScope::User, "u-1", "users/u-1/docs/a.txt"),
            ".versions/users/u-1/users/u-1/docs/a.txt/"
        );
        assert_eq!(
            versions_prefix(&FileScope::Bot, "", "docs/a.txt"),
            ".versions/docs/a.txt/"
        );
        assert!(!versions_prefix(&FileScope::User, "u-1", "docs/a.txt")
            .contains(".versions//"));
    }

    #[test]
    fn restore_accepts_only_a_revision_of_this_file() {
        let prefix = versions_prefix(&FileScope::User, "u-1", "users/u-1/docs/a.txt");
        assert!(version_belongs_to_file(
            &prefix,
            &format!("{prefix}20261101T120000000Z")
        ));
        assert!(
            !version_belongs_to_file(&prefix, ".versions/users/u-1/users/u-1/docs/b.txt/20261101T120000000Z"),
            "another file's history must be refused"
        );
        assert!(
            !version_belongs_to_file(&prefix, ".versions/users/u-2/users/u-2/docs/a.txt/20261101T120000000Z"),
            "another user's history must be refused"
        );
        assert!(!version_belongs_to_file(&prefix, &prefix), "the directory itself is not a revision");
    }
}
