//! Honest data-protection surface for the OS Settings panel (#2470). Status is
//! static: no host probing and no claim of LUKS, TPM, scheduled snapshots or
//! factory reset. `export` returns an archive manifest descriptor, `snapshots`
//! stages a bounded, non-destructive Drive archive descriptor when Drive is
//! configured (otherwise it fails closed with 503), and `reset` requires an
//! exact typed token yet still returns 501. All actions are gated and audited.

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
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

// Bounded, non-destructive Drive descriptor adapter. Included here (via
// `#[path]`) so it reuses this module's descriptor helpers without widening
// their visibility, and so no sibling module has to be touched.
#[path = "os_data/snapshot.rs"]
mod drive_snapshot;

// Static status payload, split out to keep this module under the size cap.
#[path = "os_data/status.rs"]
mod status;
pub(crate) use status::status_payload;

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
/// `DATA_ROOT`; `drive-archive` describes a consistent `tar.zst` in Drive.
/// `prepare` returns `None` when a backend is unprovisioned, so the endpoint
/// fails closed. Backends never read, write or delete source data.
pub trait SnapshotBackend {
    fn id(&self) -> &'static str;
    fn medium(&self) -> &'static str;
    fn prepare(&self, reason: &str) -> Option<SnapshotReceipt>;

    /// Bounded, non-destructive archive manifest + retention descriptor. The
    /// default keeps only a digest of the caller-supplied reason, never the
    /// raw text. A backend may add bounded fields (e.g. a logical object key).
    fn manifest(&self, reason: &str) -> Value {
        let reason_digest = hex::encode(Sha256::digest(reason.as_bytes()));
        json!({
            "manifest_id": manifest_id(),
            "reason_digest": reason_digest,
            "status": "descriptor_only",
            "scope": ARCHIVE_SCOPE_FULL,
            "format": ARCHIVE_FORMAT,
            "source_root": DATA_ROOT,
            "retention": retention_payload(),
            "destructive": false,
            "archive_created": false,
            "bytes_copied": 0
        })
    }
}

/// Backends available for this request. The Drive descriptor adapter is
/// registered only when Drive is configured (`AppState.drive` is `Some`);
/// otherwise the slice is empty and the snapshot endpoint fails closed (503).
pub fn registered_snapshot_backends(drive_available: bool) -> Vec<&'static dyn SnapshotBackend> {
    if drive_available {
        vec![drive_snapshot::backend()]
    } else {
        Vec::new()
    }
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
                "manifest": backend.manifest(reason),
                "destructive": false,
                "message": format!("Snapshot job {job} staged on {} ({medium}); no archive was created and no data left the device.", backend.id()),
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
        // Drive configured => the descriptor adapter is registered; otherwise
        // no backend is available and the request fails closed below.
        snapshot_descriptor(
            &registered_snapshot_backends(state.drive.is_some()),
            &request.reason,
        )
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
                "message": "No snapshot backend is available: Drive is not configured and no btrfs backend ships in this build. Nothing was created or deleted.",
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
#[path = "os_data/protection_tests.rs"]
mod tests;
