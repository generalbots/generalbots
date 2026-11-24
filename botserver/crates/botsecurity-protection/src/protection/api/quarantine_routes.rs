//! Quarantine read/restore handlers (#2600).
//!
//! These endpoints expose file names, paths and payloads from host quarantine
//! stores, so every one of them demands a verified authenticated admin and
//! scopes owned records to the caller's tenant. Errors returned to clients are
//! static strings; details stay in the server log.

use super::*;
use axum::extract::Extension;
use botsecurity_auth::auth_api::{AuthenticatedUser, Role};
use uuid::Uuid;

/// Pure authorization predicate for the quarantine routes (#2600). Kept free
/// of I/O and framework types so the boundary is unit-testable without a
/// server or a token. `authenticated` is the identity the auth middleware
/// verified; `roles` are the DB-resolved RBAC roles on that identity. Reads and
/// restores require a real authenticated Admin/SuperAdmin — an anonymous or
/// non-admin caller never gets a quarantine listing or a restore.
fn quarantine_admin_authorized(authenticated: bool, roles: &[Role]) -> bool {
    authenticated
        && roles
            .iter()
            .any(|role| matches!(role, Role::Admin | Role::SuperAdmin))
}

/// Fail-closed guard applied by every quarantine handler. The `AuthenticatedUser`
/// extension is inserted only by the verified auth middleware, so a missing
/// extension is treated as unauthenticated rather than as an implicit admin.
fn require_quarantine_admin(
    user: Option<Extension<AuthenticatedUser>>,
) -> Result<AuthenticatedUser, (StatusCode, Json<ApiResponse<()>>)> {
    match user {
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required")),
        )),
        Some(Extension(user)) if !quarantine_admin_authorized(user.is_authenticated(), &user.roles) => {
            warn!("quarantine access refused: an authenticated admin is required");
            Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Admin access required for quarantine")),
            ))
        }
        Some(Extension(user)) => Ok(user),
    }
}

/// Pure read scoping: an owned record is listed only to its owning tenant;
/// legacy process-wide records stay visible to the admin console (#2600).
fn quarantine_entry_visible(
    entry: &hostile_quarantine::QuarantineEntry,
    org: Option<Uuid>,
) -> bool {
    match (&entry.organization_id, org) {
        (Some(recorded), Some(org)) => match Uuid::parse_str(recorded) {
            Ok(recorded) => recorded == org,
            Err(_) => false,
        },
        (None, _) => true,
        (Some(_), None) => false,
    }
}

/// Canonical malware quarantine: the hostile-file scan records written by the
/// upload gate and by ClamAV, not LMD's separate store (#2600). Admin only.
pub(super) async fn get_quarantine(
    user: Option<Extension<AuthenticatedUser>>,
) -> Result<Json<ApiResponse<Vec<hostile_quarantine::QuarantineEntry>>>, (StatusCode, Json<ApiResponse<()>>)>
{
    let admin = require_quarantine_admin(user)?;
    match hostile_quarantine::list_quarantine_records() {
        Ok(files) => {
            let files: Vec<hostile_quarantine::QuarantineEntry> = files
                .into_iter()
                .filter(|entry| quarantine_entry_visible(entry, admin.organization_id))
                .collect();
            Ok(Json(ApiResponse::success(files)))
        }
        Err(e) => {
            warn!(error = %e, "Failed to get quarantine list");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to get quarantine list")),
            ))
        }
    }
}

/// Restore is authenticated, tenant-scoped and re-scanned by the core store;
/// an infected payload or a path outside the store is refused (#2600).
pub(super) async fn restore_from_quarantine(
    Path(file_id): Path<String>,
    user: Option<Extension<AuthenticatedUser>>,
) -> Result<Json<ApiResponse<ActionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let admin = require_quarantine_admin(user)?;
    match hostile_quarantine::restore_quarantine_record_for_owner(&file_id, admin.organization_id) {
        Ok(()) => Ok(Json(ApiResponse::success(ActionResponse {
            success: true,
            message: format!("Quarantine record {file_id} re-scanned and restored"),
        }))),
        Err(e) => {
            warn!(error = %e, "Failed to restore file from quarantine");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Restore refused: re-scan failed or file still infected")),
            ))
        }
    }
}

/// LMD's own quarantine stays available under an explicitly LMD-labelled path.
/// Admin only — it exposes file names and paths from the host quarantine.
pub(super) async fn get_lmd_quarantine(
    user: Option<Extension<AuthenticatedUser>>,
) -> Result<Json<ApiResponse<Vec<super::lmd::QuarantinedFile>>>, (StatusCode, Json<ApiResponse<()>>)>
{
    require_quarantine_admin(user)?;
    match super::lmd::list_quarantined().await {
        Ok(files) => Ok(Json(ApiResponse::success(files))),
        Err(e) => {
            warn!(error = %e, "Failed to get LMD quarantine list");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to get LMD quarantine list")),
            ))
        }
    }
}

pub(super) async fn restore_lmd_quarantine(
    Path(file_id): Path<String>,
    user: Option<Extension<AuthenticatedUser>>,
) -> Result<Json<ApiResponse<ActionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_quarantine_admin(user)?;
    match super::lmd::restore_file(&file_id).await {
        Ok(()) => Ok(Json(ApiResponse::success(ActionResponse {
            success: true,
            message: format!("File {file_id} restored from LMD quarantine"),
        }))),
        Err(e) => {
            warn!(error = %e, "Failed to restore file from LMD quarantine");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to restore file from LMD quarantine")),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(owner: Option<&str>) -> hostile_quarantine::QuarantineEntry {
        hostile_quarantine::QuarantineEntry {
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            original_path: "uploads/eicar.txt".to_string(),
            quarantine_path: "/tmp/gb-hostile-quarantine/11111111-1111-4111-8111-111111111111.bin"
                .to_string(),
            size: 68,
            quarantined_at: Some("2026-11-23T00:00:00Z".to_string()),
            threat_name: Some("EICAR-STANDARD-ANTIVIRUS-TEST-FILE".to_string()),
            verdict: "infected".to_string(),
            scanner: "builtin-eicar-signature".to_string(),
            organization_id: owner.map(str::to_string),
        }
    }

    #[test]
    fn only_verified_admins_pass_the_quarantine_gate() {
        assert!(quarantine_admin_authorized(true, &[Role::Admin]));
        assert!(quarantine_admin_authorized(true, &[Role::SuperAdmin]));
        assert!(quarantine_admin_authorized(true, &[Role::User, Role::Admin]));
    }

    #[test]
    fn non_admin_roles_are_refused() {
        assert!(!quarantine_admin_authorized(true, &[]));
        assert!(!quarantine_admin_authorized(true, &[Role::User]));
        assert!(!quarantine_admin_authorized(true, &[Role::Moderator]));
        assert!(!quarantine_admin_authorized(true, &[Role::Service, Role::Bot]));
    }

    #[test]
    fn missing_identity_is_refused_even_with_an_admin_role() {
        assert!(!quarantine_admin_authorized(false, &[Role::Admin]));
        assert!(!quarantine_admin_authorized(false, &[Role::SuperAdmin]));
        assert!(!quarantine_admin_authorized(false, &[]));
    }

    #[test]
    fn quarantine_reads_are_scoped_to_the_owner_tenant() -> Result<(), String> {
        let org = Uuid::parse_str("22222222-2222-4222-8222-222222222222")
            .map_err(|error| error.to_string())?;
        let other = Uuid::parse_str("33333333-3333-4333-8333-333333333333")
            .map_err(|error| error.to_string())?;
        assert!(quarantine_entry_visible(&entry(None), Some(org)));
        assert!(quarantine_entry_visible(&entry(Some(&org.to_string())), Some(org)));
        assert!(!quarantine_entry_visible(
            &entry(Some(&other.to_string())),
            Some(org)
        ));
        assert!(!quarantine_entry_visible(&entry(Some("not-a-uuid")), Some(org)));
        Ok(())
    }
}
