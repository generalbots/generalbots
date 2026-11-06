//! Process-wide database pool shared by every feature crate.
//!
//! This used to be copy-pasted as `src/db.rs` into 19 crates, so each one
//! opened and initialised its own pool. There is one `POOL` here now; crates
//! keep their `crate::db` module as a thin re-export so call sites do not
//! change.

use axum::http::StatusCode;
use std::sync::OnceLock;

use super::{create_conn, DbPool};

static POOL: OnceLock<Result<DbPool, String>> = OnceLock::new();

/// Returns the process-wide pool, initialising it on first use.
pub fn pool() -> Result<&'static DbPool, (StatusCode, String)> {
    POOL.get_or_init(|| match create_conn() {
        Ok(p) => Ok(p),
        Err(e) => Err(format!("DB connection init failed: {e}")),
    })
    .as_ref()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.clone()))
}

/// Maps a Diesel error into an HTTP status / message pair.
pub fn map_diesel_err(e: diesel::result::Error) -> (StatusCode, String) {
    match e {
        diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        other => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {other}")),
    }
}
