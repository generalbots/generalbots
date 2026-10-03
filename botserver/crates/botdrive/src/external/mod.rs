//! External drives: OneDrive and Google Drive as first-class file backends.
//!
//! Before this module, both providers existed only as declarations: a read-only
//! `GenericAdapter` spec with two actions each, and a `botm365` surface whose
//! "connect" and "sync" handlers stamped timestamps without ever calling a
//! provider. Nothing was fetched, nothing was listed, nothing was visible.
//!
//! What lives here is the missing middle:
//!
//! * [`onedrive`] / [`gdrive`] — native change-feed clients (Graph `delta`,
//!   Drive `changes.list`), so a steady-state sync costs a few API calls
//!   instead of a full listing.
//! * [`db`] — one connection row per (branch, provider), every query bounded
//!   by `branch_id`.
//! * [`files`] — the mirrored listing the Drive app reads.
//! * [`crypto`] — tokens encrypted with the platform master key, and the
//!   signed OAuth `state` that binds a callback to the branch that started it.
//! * [`engine`] — one pass per connection plus the background scheduler.
//! * [`routes`] — the `/api/external-drives/*` HTTP surface the Drive app's
//!   External tab consumes.
//!
//! Mirroring is deliberately **metadata-only**: rows carry name, path, size,
//! type and revision, and the bytes are streamed from the provider on demand.
//! Duplicating whole accounts into MinIO would double the storage and re-create
//! the sync conflicts the native change feeds exist to avoid.

pub mod config;
pub mod crypto;
pub mod db;
pub mod engine;
pub mod files;
pub mod gdrive;
pub mod onedrive;
pub mod routes;
pub mod routes_files;
pub mod types;

pub use types::{Connection, DeltaPage, ExternalFile, Provider, SyncError, SyncReport};

/// Create the tables if they are missing. Called at boot so the first request
/// does not have to, and safe to call again because every statement is
/// `IF NOT EXISTS`.
pub fn ensure_schema(conn: &mut diesel::PgConnection) -> Result<(), String> {
    db::ensure_schema(conn)
}