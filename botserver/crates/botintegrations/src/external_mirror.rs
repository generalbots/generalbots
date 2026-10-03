//! OneDrive and Google Drive actions served from the shared external-drive
//! mirror.
//!
//! These two providers used to have their own OAuth flow, their own token
//! store in Vault and a direct call to the provider API. That made them a
//! *second* implementation of the same thing next to `botdrive::external`,
//! with the two disagreeing: the Vault copy never refreshed its cursor, so a
//! `drive.files.list` was a full listing against the live API on every call,
//! and nothing it returned was visible anywhere in the UI.
//!
//! Now both surfaces read one mirror. The account is connected once from
//! Drive → External (native change feed, encrypted tokens, per-branch
//! isolation) and these actions answer from `external_drive_files`, so an
//! action call costs a database read instead of a rate-limited API call, and
//! what a bot sees is exactly what the user sees in the Drive app.
//!
//! Scope comes from the connection scope (org/branch/bot resolved server-side
//! by the caller), never from the action parameters.

use crate::providers::{ActionOutcome, ERR_INVALID_REQUEST};
use crate::scope::ConnectionScope;
use crate::state::IntegrationState;
use botdrive::external::{files, Provider};
use serde_json::{json, Value};

/// Largest listing a single action call may return.
const MAX_LIMIT: i64 = 500;

/// Map an Integrations provider slug onto the external-drive backend.
pub fn mirror_provider(provider: &str) -> Option<Provider> {
    match provider {
        "drive" => Some(Provider::GoogleDrive),
        "onedrive" => Some(Provider::OneDrive),
        _ => None,
    }
}

/// True when the provider is served from the mirror rather than its own
/// connection. Used by the OAuth start handler to refuse a second credential
/// store for an account that is already connected centrally.
pub fn is_mirrored(provider: &str) -> bool {
    mirror_provider(provider).is_some()
}

/// Execute a mirrored action for `provider`.
///
/// Returns `None` when the slug is not mirrored, so the caller falls through to
/// the ordinary connection path.
pub async fn invoke(
    state: &IntegrationState,
    scope: &ConnectionScope,
    provider: &str,
    action: &str,
    params: &Value,
) -> Option<Result<ActionOutcome, String>> {
    let kind = mirror_provider(provider)?;
    Some(run(state, scope, kind, action, params).await)
}

async fn run(
    state: &IntegrationState,
    scope: &ConnectionScope,
    kind: Provider,
    action: &str,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let action = action.strip_prefix(&format!("{}.", kind_action_prefix(kind))).unwrap_or(action);
    let mut conn = state
        .pool
        .get()
        .map_err(|_| crate::providers::ERR_STORAGE_UNAVAILABLE.to_string())?;

    match action {
        "files.list" => {
            let prefix = params
                .get("path")
                .or_else(|| params.get("prefix"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let limit = params
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(100)
                .clamp(1, MAX_LIMIT);
            let rows = files::list_files(&mut conn, scope.branch_id, Some(kind), Some(prefix), limit)
                .map_err(|error| {
                    log::error!("external mirror listing failed for {}: {error}", kind.as_str());
                    crate::providers::ERR_STORAGE_UNAVAILABLE.to_string()
                })?;
            let payload: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "id": row.remote_id,
                        "name": row.name,
                        "path": row.path,
                        "mimeType": row.mime_type,
                        "size": row.size_bytes,
                        "isFolder": row.is_folder,
                        "modifiedTime": row.modified_at,
                    })
                })
                .collect();
            let count = payload.len();
            Ok(ActionOutcome {
                summary: format!("{count} file(s) mirrored for {}", kind.display_name()),
                data: json!({ "files": payload }),
                truncated: false,
            })
        }
        "files.get" => {
            let remote_id = params
                .get("id")
                .or_else(|| params.get("resource_id"))
                .or_else(|| params.get("file_id"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!("{ERR_INVALID_REQUEST}: id is required for files.get")
                })?;
            let row = files::get_file(&mut conn, scope.branch_id, kind, remote_id)
                .map_err(|error| {
                    log::error!("external mirror lookup failed for {}: {error}", kind.as_str());
                    crate::providers::ERR_STORAGE_UNAVAILABLE.to_string()
                })?
                .ok_or_else(|| {
                    format!("no mirrored file '{remote_id}' for {}", kind.display_name())
                })?;
            Ok(ActionOutcome {
                summary: format!("{} · {}", row.name, row.path),
                data: json!({
                    "id": row.remote_id,
                    "name": row.name,
                    "path": row.path,
                    "mimeType": row.mime_type,
                    "size": row.size_bytes,
                    "isFolder": row.is_folder,
                    "modifiedTime": row.modified_at,
                }),
                truncated: false,
            })
        }
        other => Err(format!(
            "{}: {other}",
            crate::providers::ERR_ACTION_NOT_AVAILABLE
        )),
    }
}

/// The action prefix the catalog uses for each provider (`drive.` / `onedrive.`).
fn kind_action_prefix(kind: Provider) -> &'static str {
    match kind {
        Provider::GoogleDrive => "drive",
        Provider::OneDrive => "onedrive",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_drive_providers_are_mirrored() {
        assert_eq!(mirror_provider("drive"), Some(Provider::GoogleDrive));
        assert_eq!(mirror_provider("onedrive"), Some(Provider::OneDrive));
        // Everything else keeps its own connection path.
        assert_eq!(mirror_provider("notion"), None);
        assert_eq!(mirror_provider("outlook"), None);
        assert!(is_mirrored("drive") && is_mirrored("onedrive"));
        assert!(!is_mirrored("box"));
    }

    #[test]
    fn action_names_are_stripped_before_dispatch() {
        assert_eq!(kind_action_prefix(Provider::GoogleDrive), "drive");
        assert_eq!(kind_action_prefix(Provider::OneDrive), "onedrive");
    }
}