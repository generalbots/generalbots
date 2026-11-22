//! Honest data-protection surface for the OS Settings panel (#2470). Status is
//! static: no host probing and no claim of LUKS, TPM, scheduled snapshots or
//! factory reset. `export` returns an archive manifest descriptor, `snapshots`
//! stages a job through the documented btrfs/Drive adapter contract and fails
//! closed (503) with no backend, `reset` requires an exact typed token yet
//! still returns 501. All actions are gated and audited.

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::audit_log::{record_audit_event, AuditEvent};
use crate::os::{authorize, deny};

/// Single-copy user data tree on the OS image. Descriptors only: this module
/// never reads from or writes to it.
pub const DATA_ROOT: &str = "/var/lib/gbdesk";

/// Planned retention counts, not evidence of a scheduler (none ships).
pub const RETENTION_DAILY_KEEP: u8 = 7;
pub const RETENTION_WEEKLY_KEEP: u8 = 4;
pub const RETENTION_MONTHLY_KEEP: u8 = 12;

pub const STATUS_CONTRACT_PATH: &str = "/api/os/data-protection";
pub const EXPORT_CONTRACT_PATH: &str = "/api/os/data-protection/export";
pub const RESET_CONTRACT_PATH: &str = "/api/os/data-protection/reset";
pub const SNAPSHOT_CONTRACT_PATH: &str = "/api/os/data-protection/snapshots";

pub const ARCHIVE_SCOPE_FULL: &str = "full-archive";
pub const ARCHIVE_FORMAT: &str = "tar.zst";

/// Exact typed confirmation required before reset will even report that its
/// backend is absent. The UI reads it from status instead of hardcoding it.
pub const RESET_CONFIRMATION_TOKEN: &str = "RESET THIS PC";

const ARCHIVE_INCLUDES: [&str; 4] = ["chat history", "documents", "bot workspace", "settings"];
const ARCHIVE_EXCLUDES: [&str; 3] = ["credentials", "session tokens", "encryption keys"];
const MAX_REASON_BYTES: usize = 200;
const EXPORT_DETAILS: [&str; 2] = ["invalid_scope", ARCHIVE_SCOPE_FULL];
const RESET_DETAILS: [&str; 2] = ["confirmation_missing", "typed_token_no_backend"];

type ApiResult = Result<Response, (StatusCode, Json<Value>)>;

pub(crate) fn configure_data_protection_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(STATUS_CONTRACT_PATH, get(data_protection_status))
        .route(EXPORT_CONTRACT_PATH, post(export_archive))
        .route(SNAPSHOT_CONTRACT_PATH, post(create_snapshot))
        .route(RESET_CONTRACT_PATH, post(reset_device))
}

/// Shell-mode + admin/owner + loopback gate; returns the actor for auditing.
fn gate(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> Result<AuthenticatedUser, (StatusCode, Json<Value>)> {
    let Extension(user) = user.ok_or_else(|| deny("Authentication required"))?;
    authorize(&user, &headers)?;
    Ok(user)
}

fn bad_request(error: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": error })))
}

fn respond(status: StatusCode, value: Value) -> Response {
    (status, Json(value)).into_response()
}

/// One audit row per action, including refused and unimplemented attempts.
fn audit(
    state: &Arc<AppState>,
    actor_id: uuid::Uuid,
    action: &'static str,
    target_type: &'static str,
    success: bool,
    details: &'static str,
) {
    record_audit_event(AuditEvent {
        state,
        event_type: "os_data_protection",
        actor_id,
        action,
        target_type: Some(target_type),
        target_id: None,
        success,
        details: Some(details),
    });
}

/// Receipt for a staged snapshot job. No bytes are copied by this build.
pub struct SnapshotReceipt {
    pub job_id: String,
    pub location: String,
}

/// OS-image snapshot contract. `btrfs` stages a read-only subvolume of
/// `DATA_ROOT`; `drive-archive` uploads a consistent `tar.zst` to Drive.
/// `prepare` is idempotent per reason, never deletes the source and returns
/// `None` when unprovisioned, so the endpoint fails closed.
pub trait SnapshotBackend {
    fn id(&self) -> &'static str;
    fn medium(&self) -> &'static str;
    fn prepare(&self, reason: &str) -> Option<SnapshotReceipt>;
}

/// Backends registered by the OS image. Empty here: this build registers none.
pub fn registered_snapshot_backends() -> &'static [&'static dyn SnapshotBackend] {
    &[]
}

fn retention_payload() -> Value {
    json!({
        "daily_keep": RETENTION_DAILY_KEEP,
        "weekly_keep": RETENTION_WEEKLY_KEEP,
        "monthly_keep": RETENTION_MONTHLY_KEEP,
        "unit": "snapshots",
        "note": "Planned policy only. No snapshots are created in this build."
    })
}

/// First prepared backend wins; `None` is mapped to 503 by the handler.
fn snapshot_descriptor(backends: &[&dyn SnapshotBackend], reason: &str) -> Option<Value> {
    for backend in backends {
        if let Some(receipt) = backend.prepare(reason) {
            let job = receipt.job_id;
            let medium = backend.medium();
            return Some(json!({
                "status": "staged",
                "adapter": backend.id(),
                "medium": medium,
                "job_id": job,
                "location": receipt.location,
                "source_root": DATA_ROOT,
                "retention": retention_payload(),
                "destructive": false,
                "message": format!("Snapshot job {job} staged on {} ({medium}); no snapshot was created by this build.", backend.id()),
                "note": "Adapter contract only. Restore and pruning are separate typed-confirmation actions supplied by the OS image."
            }));
        }
    }
    None
}

/// Stable, content-derived identifier; repeated manifests differ only in time.
fn manifest_id() -> String {
    let canonical = format!(
        "{DATA_ROOT}|{ARCHIVE_SCOPE_FULL}|{ARCHIVE_FORMAT}|{}|{}",
        ARCHIVE_INCLUDES.join(","),
        ARCHIVE_EXCLUDES.join(",")
    );
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

/// Offline-testable descriptor. Explicitly not an archive.
fn archive_manifest() -> Value {
    let id = manifest_id();
    json!({
        "manifest_id": id,
        "status": "descriptor_only",
        "generated_at": Utc::now().to_rfc3339(),
        "source_root": DATA_ROOT,
        "scope": ARCHIVE_SCOPE_FULL,
        "format": ARCHIVE_FORMAT,
        "includes": ARCHIVE_INCLUDES,
        "excludes": ARCHIVE_EXCLUDES,
        "bytes_read": 0,
        "destructive": false,
        "archive_created": false,
        "message": format!("Archive manifest {id} · {ARCHIVE_FORMAT} · {ARCHIVE_SCOPE_FULL} · descriptor only, no archive was created and no data left the device."),
        "note": "Lists exactly what a future export backend would include and exclude. Contains no user data and performs no file access."
    })
}

fn valid_reason(reason: &str) -> bool {
    let trimmed = reason.trim();
    !trimmed.is_empty()
        && trimmed.len() <= MAX_REASON_BYTES
        && !trimmed.chars().any(char::is_control)
}

fn reset_confirmed(confirmation: Option<&str>) -> bool {
    confirmation == Some(RESET_CONFIRMATION_TOKEN)
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub scope: String,
}
#[derive(Debug, Deserialize)]
pub struct SnapshotRequest {
    pub reason: String,
}
#[derive(Debug, Deserialize)]
pub struct ResetRequest {
    pub confirmation: Option<String>,
}

/// Static status payload: no host probing, so the answer cannot drift from
/// what this build actually ships.
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
            "retention_policy": retention_payload(),
            "contract": {
                "status": "adapter_contract_only",
                "method": "POST",
                "path": SNAPSHOT_CONTRACT_PATH,
                "request": { "reason": "string" },
                "adapters": ["btrfs", "drive-archive"],
                "result": "503 when no OS backend is registered; 202 with a staged job descriptor otherwise",
                "medium": "btrfs snapshot or Drive archive of the data root"
            }
        },
        "export": {
            "implemented": false,
            "enabled": false,
            "descriptor_available": true,
            "contract": {
                "status": "descriptor_only",
                "method": "POST",
                "path": EXPORT_CONTRACT_PATH,
                "request": { "scope": ARCHIVE_SCOPE_FULL },
                "result": "200 with a deterministic archive manifest; no archive is created and no data leaves the device",
                "includes": ARCHIVE_INCLUDES,
                "excludes": ARCHIVE_EXCLUDES
            }
        },
        "reset": {
            "implemented": false,
            "enabled": false,
            "confirmation_token": RESET_CONFIRMATION_TOKEN,
            "contract": {
                "status": "disabled",
                "method": "POST",
                "path": RESET_CONTRACT_PATH,
                "request": { "confirmation": "exact typed confirmation token required" },
                "result": "400 without the typed token; 501 with it. No wipe backend ships in this build."
            }
        }
    })
}

async fn data_protection_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gate(user, headers)?;
    Ok(respond(StatusCode::OK, status_payload()))
}

async fn export_archive(
    State(state): State<Arc<AppState>>,
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    Json(request): Json<ExportRequest>,
) -> ApiResult {
    let actor = gate(user, headers)?;
    let accepted = request.scope == ARCHIVE_SCOPE_FULL;
    audit(
        &state,
        actor.user_id,
        "create_archive_manifest",
        "archive_manifest",
        accepted,
        EXPORT_DETAILS[usize::from(accepted)],
    );
    if accepted {
        Ok(respond(StatusCode::OK, archive_manifest()))
    } else {
        Err(bad_request("unsupported_export_scope"))
    }
}

async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    Json(request): Json<SnapshotRequest>,
) -> ApiResult {
    let actor = gate(user, headers)?;
    let valid = valid_reason(&request.reason);
    let descriptor = if valid {
        snapshot_descriptor(registered_snapshot_backends(), &request.reason)
    } else {
        None
    };
    let staged = descriptor.is_some();
    let details = if !valid {
        "invalid_reason"
    } else if staged {
        "staged"
    } else {
        "backend_absent"
    };
    audit(
        &state,
        actor.user_id,
        "request_snapshot",
        "snapshot_job",
        staged,
        details,
    );
    if !valid {
        return Err(bad_request("invalid_reason"));
    }
    match descriptor {
        Some(value) => Ok(respond(StatusCode::ACCEPTED, value)),
        None => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "os_backend_unavailable",
                "status": "not_implemented",
                "message": "No btrfs or Drive snapshot backend is registered in this build. No snapshot was created.",
                "contract": SNAPSHOT_CONTRACT_PATH,
                "adapters": ["btrfs", "drive-archive"]
            })),
        )),
    }
}

async fn reset_device(
    State(state): State<Arc<AppState>>,
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    Json(request): Json<ResetRequest>,
) -> ApiResult {
    let actor = gate(user, headers)?;
    let confirmed = reset_confirmed(request.confirmation.as_deref());
    // No build ships a wipe backend, so this action never succeeds.
    audit(
        &state,
        actor.user_id,
        "request_factory_reset",
        "device",
        false,
        RESET_DETAILS[usize::from(confirmed)],
    );
    if !confirmed {
        return Err(bad_request("confirmation_mismatch"));
    }
    Ok(respond(
        StatusCode::NOT_IMPLEMENTED,
        json!({
            "error": "reset_backend_absent",
            "status": "not_implemented",
            "message": "Factory reset is disabled: no wipe-and-wizard backend ships in this build. Nothing was erased.",
            "contract": RESET_CONTRACT_PATH
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct B(bool);
    impl SnapshotBackend for B {
        fn id(&self) -> &'static str {
            "btrfs"
        }
        fn medium(&self) -> &'static str {
            "local-subvolume"
        }
        fn prepare(&self, _r: &str) -> Option<SnapshotReceipt> {
            if !self.0 {
                return None;
            }
            Some(SnapshotReceipt {
                job_id: "j".into(),
                location: DATA_ROOT.into(),
            })
        }
    }
    #[test]
    fn status_is_honest_and_publishes_contracts() {
        let p = status_payload();
        assert_eq!(p["status"], "not_implemented");
        assert_eq!(p["encryption"]["implemented"], false);
        assert_eq!(p["backup"]["implemented"], false);
        assert_eq!(p["export"]["enabled"], false);
        assert_eq!(p["export"]["descriptor_available"], true);
        assert_eq!(p["reset"]["enabled"], false);
        assert_eq!(p["reset"]["confirmation_token"], RESET_CONFIRMATION_TOKEN);
        assert_eq!(p["backup"]["contract"]["path"], SNAPSHOT_CONTRACT_PATH);
        assert_eq!(p["export"]["contract"]["path"], EXPORT_CONTRACT_PATH);
        assert_eq!(p["reset"]["contract"]["path"], RESET_CONTRACT_PATH);
        let r = retention_payload();
        assert_eq!(r["daily_keep"], RETENTION_DAILY_KEEP);
        let t = p.to_string().to_ascii_lowercase();
        assert!(!t.contains("tpm 2.0 sealing of the root volume is performed"));
        assert!(!t.contains("enabled\":true"));
    }
    #[test]
    fn manifest_is_deterministic_and_non_destructive() {
        let m = archive_manifest();
        assert_eq!(m["manifest_id"], archive_manifest()["manifest_id"]);
        assert_eq!(m["manifest_id"].as_str().map(str::len), Some(64));
        assert_eq!(m["status"], "descriptor_only");
        assert_eq!(m["archive_created"], false);
        assert_eq!(m["destructive"], false);
        assert_eq!(m["excludes"].as_array().map(Vec::len), Some(3));
    }
    #[test]
    fn snapshot_fails_closed_and_reports_a_prepared_backend() {
        assert!(registered_snapshot_backends().is_empty());
        assert!(snapshot_descriptor(registered_snapshot_backends(), "r").is_none());
        assert!(snapshot_descriptor(&[&B(false)], "r").is_none());
        let d = snapshot_descriptor(&[&B(true)], "r");
        assert!(d.is_some());
        if let Some(d) = d {
            assert_eq!(d["status"], "staged");
        }
    }
    #[test]
    fn inputs_are_validated() {
        assert!(valid_reason("nightly backup"));
        assert!(!valid_reason("   "));
        assert!(!valid_reason(&"x".repeat(MAX_REASON_BYTES + 1)));
        assert!(reset_confirmed(Some(RESET_CONFIRMATION_TOKEN)));
        assert!(!reset_confirmed(None));
    }
}
