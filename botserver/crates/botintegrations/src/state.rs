use botcore::shared::utils::DbPool;

use crate::secrets::ConnectionVault;

/// Per-feature state for the integration connection control plane (#939).
///
/// Reuses the shared database pool; legacy handlers keep using the module
/// level pool in `db.rs` untouched.
#[derive(Clone)]
pub struct IntegrationState {
    pub pool: DbPool,
    pub vault: ConnectionVault,
}

impl IntegrationState {
    pub fn new(pool: DbPool, vault: ConnectionVault) -> Self {
        Self { pool, vault }
    }
}
