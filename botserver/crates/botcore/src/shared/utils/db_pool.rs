//! Process-wide database pool shared by every feature crate.
//!
//! This used to be copy-pasted as `src/db.rs` into 19 crates, so each one
//! opened and initialised its own pool. There is one `POOL` here now; crates
//! keep their `crate::db` module as a thin re-export so call sites do not
//! change.

use axum::http::StatusCode;
use std::sync::OnceLock;

use botsecurity_core::error_sanitizer::log_and_sanitize_tuple;

use super::{create_conn, DbPool};

static POOL: OnceLock<Result<DbPool, String>> = OnceLock::new();

/// Returns the process-wide pool, initialising it on first use.
pub fn pool() -> Result<&'static DbPool, (StatusCode, String)> {
    POOL.get_or_init(|| match create_conn() {
        Ok(p) => Ok(p),
        // #2753 — never cache a raw DSN/password in the process-wide error:
        // it is cloned into HTTP bodies by every caller of `pool()`.
        Err(e) => Err(log_and_sanitize_tuple(
            &e,
            "botcore::db_pool::init",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .1),
    })
    .as_ref()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.clone()))
}

/// Maps a Diesel error into an HTTP status / message pair (#2753).
///
/// The cause is logged server-side; the body is always the generic sanitizer
/// message. Sixty-plus handlers across the feature crates reach this helper
/// through their local `db` re-export, so fixing it here removes raw schema,
/// constraint and connection text from all of their response bodies.
pub fn map_diesel_err(e: diesel::result::Error) -> (StatusCode, String) {
    match e {
        diesel::result::Error::NotFound => {
            (StatusCode::NOT_FOUND, "Resource not found".to_string())
        }
        other => log_and_sanitize_tuple(
            &other,
            "botcore::db_pool::map_diesel_err",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
