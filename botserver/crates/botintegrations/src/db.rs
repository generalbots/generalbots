use axum::http::StatusCode;
use botcore::shared::utils::DbPool;
use std::sync::OnceLock;

static POOL: OnceLock<Result<DbPool, String>> = OnceLock::new();

pub fn pool() -> Result<&'static DbPool, (StatusCode, String)> {
    POOL.get_or_init(|| match botcore::shared::utils::create_conn() {
        Ok(p) => Ok(p),
        Err(e) => Err(format!("DB connection init failed: {e}")),
    })
    .as_ref()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.clone()))
}

pub fn map_diesel_err(e: diesel::result::Error) -> (StatusCode, String) {
    match e {
        diesel::result::Error::NotFound => {
            (StatusCode::NOT_FOUND, "Resource not found".to_string())
        }
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {other}"),
        ),
    }
}
