//! Honest data-protection status for the OS Settings panel (#2470).
//!
//! This module reports what the image actually does. It deliberately does
//! **not** claim LUKS/TPM sealing, scheduled snapshots, a full-archive export
//! or a factory reset: none of those backends ship in this build. Every action
//! is exposed in the UI as a labelled, disabled control and the intended
//! contract is published here so a future implementation has a stable shape.
//!
//! Only the read-only status endpoint is registered. No destructive operation
//! is reachable from this module.

use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::os::{authorize, deny};

/// Root of the single-copy user data tree on the OS image (#2470). Status
/// only: nothing in this module reads from or writes to it.
pub const DATA_ROOT: &str = "/var/lib/gbdesk";

/// Planned retention policy, expressed as a count of snapshots per tier.
/// These are constants for the documented contract, not evidence that a
/// snapshot scheduler exists (it does not in this build).
pub const RETENTION_DAILY_KEEP: u8 = 7;
pub const RETENTION_WEEKLY_KEEP: u8 = 4;
pub const RETENTION_MONTHLY_KEEP: u8 = 12;

/// Documented (not implemented) request surfaces for the disabled actions.
pub const EXPORT_CONTRACT_PATH: &str = "/api/os/data-protection/export";
pub const RESET_CONTRACT_PATH: &str = "/api/os/data-protection/reset";
pub const SNAPSHOT_CONTRACT_PATH: &str = "/api/os/data-protection/snapshots";

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

pub(crate) fn configure_data_protection_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/os/data-protection", get(data_protection_status))
}

fn gated(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let Extension(user) = user.ok_or_else(|| deny("Authentication required"))?;
    authorize(&user, &headers)
}

/// Pure, offline-testable status payload. Kept free of host probing so the
/// answer never drifts from what is actually shipped in this build.
fn status_payload() -> Value {
    json!({
        "status": "not_implemented",
        "shell_mode_only": true,
        "data_root": DATA_ROOT,
        "encryption": {
            "implemented": false,
            "luks_mapping": false,
            "tpm_sealed": false,
            "recovery_key_export": false,
            "note": "No LUKS mapping, no systemd-cryptenroll provisioning and no TPM sealing are implemented in this image. User data at rest is not encrypted by General Bots."
        },
        "backup": {
            "implemented": false,
            "enabled": false,
            "root": DATA_ROOT,
            "schedule": "none",
            "last_snapshot": null,
            "restore_available": false,
            "retention_policy": {
                "daily_keep": RETENTION_DAILY_KEEP,
                "weekly_keep": RETENTION_WEEKLY_KEEP,
                "monthly_keep": RETENTION_MONTHLY_KEEP,
                "unit": "snapshots",
                "note": "Planned policy only. No snapshots are created in this build."
            },
            "contract": {
                "status": "planned",
                "method": "POST",
                "path": SNAPSHOT_CONTRACT_PATH,
                "request": { "reason": "string", "confirmation": "string" },
                "result": "202 with a job id; restore is a separate typed-confirmation action",
                "medium": "btrfs snapshot or Drive archive of the data root"
            }
        },
        "export": {
            "implemented": false,
            "enabled": false,
            "contract": {
                "status": "planned",
                "method": "POST",
                "path": EXPORT_CONTRACT_PATH,
                "request": { "scope": "full-archive", "confirmation": "string" },
                "result": "202 with a job id; the archive is downloaded from a job-status endpoint",
                "includes": ["chat history", "documents", "bot workspace", "settings"],
                "excludes": ["credentials", "session tokens", "encryption keys"]
            }
        },
        "reset": {
            "implemented": false,
            "enabled": false,
            "contract": {
                "status": "planned",
                "method": "POST",
                "path": RESET_CONTRACT_PATH,
                "request": { "confirmation": "string", "reinstall": true },
                "result": "wipe the data root then re-run the first-run wizard; never executed in this build"
            }
        }
    })
}

async fn data_protection_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(status_payload()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_no_encryption_or_protection_backends() {
        let payload = status_payload();
        assert_eq!(payload["status"], "not_implemented");
        assert_eq!(payload["encryption"]["implemented"], false);
        assert_eq!(payload["encryption"]["luks_mapping"], false);
        assert_eq!(payload["encryption"]["tpm_sealed"], false);
        assert_eq!(payload["backup"]["implemented"], false);
        assert_eq!(payload["backup"]["restore_available"], false);
        assert_eq!(payload["export"]["enabled"], false);
        assert_eq!(payload["reset"]["enabled"], false);
    }

    #[test]
    fn retention_and_contracts_are_published() {
        let payload = status_payload();
        let policy = &payload["backup"]["retention_policy"];
        assert_eq!(policy["daily_keep"], RETENTION_DAILY_KEEP);
        assert_eq!(policy["weekly_keep"], RETENTION_WEEKLY_KEEP);
        assert_eq!(policy["monthly_keep"], RETENTION_MONTHLY_KEEP);
        assert_eq!(
            payload["backup"]["contract"]["path"],
            SNAPSHOT_CONTRACT_PATH
        );
        assert_eq!(payload["export"]["contract"]["path"], EXPORT_CONTRACT_PATH);
        assert_eq!(payload["reset"]["contract"]["path"], RESET_CONTRACT_PATH);
        assert_eq!(payload["data_root"], DATA_ROOT);
    }

    #[test]
    fn no_false_tpm_or_luks_claim_in_any_text() {
        let text = status_payload().to_string().to_ascii_lowercase();
        assert!(!text.contains("tpm 2.0 sealing of the root volume is performed"));
        assert!(!text.contains("enabled\":true"));
    }
}
