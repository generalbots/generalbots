//! Access-control helpers for `UserDriveVectorDB` (#2820).
//!
//! Kept out of the main file to respect the 450-line limit. All identifiers
//! used here come from the trusted `VectorScope` built at construction time.

use super::{AccessVerifier, UserDriveVectorDB, VectorScope};
use serde_json::Map;
use std::sync::Arc;
use uuid::Uuid;

impl UserDriveVectorDB {
    pub fn with_org_id(mut self, org_id: Uuid) -> Self {
        self.scope.org_id = Some(org_id);
        self
    }

    pub fn with_branch_id(mut self, branch_id: Uuid) -> Self {
        self.scope.branch_id = Some(branch_id);
        self
    }

    pub fn with_access_verifier(mut self, verifier: Arc<AccessVerifier>) -> Self {
        self.access_verifier = Some(verifier);
        self
    }

    pub fn scope(&self) -> &VectorScope {
        &self.scope
    }

    /// Single gate used by every search path: enforce the trusted server stamp
    /// and, when installed, the caller's current-ACL verifier before any
    /// embedded text is returned.
    pub(crate) fn authorize_chunk(
        &self,
        payload: &Map<String, serde_json::Value>,
        file_path: &str,
        bucket: &str,
    ) -> bool {
        super::scope::chunk_is_authorized(
            &self.scope,
            payload,
            file_path,
            bucket,
            self.access_verifier.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_db_authorizes_only_its_own_trusted_stamp() {
        let bot = Uuid::from_u128(1);
        let user = Uuid::from_u128(2);
        let db = UserDriveVectorDB::new(user, bot, std::path::PathBuf::from("/tmp/vdb"));
        let mut payload = Map::new();
        db.scope().stamp(&mut payload);
        assert!(db.authorize_chunk(&payload, "docs/a.pdf", "bucket"));

        // The same collection contents read under a different user's scope are
        // rejected even when the caller claims ownership in the payload.
        let other =
            UserDriveVectorDB::new(Uuid::from_u128(9), bot, std::path::PathBuf::from("/tmp/x"));
        assert!(!other.authorize_chunk(&payload, "docs/a.pdf", "bucket"));
    }

    #[test]
    fn installing_a_verifier_rechecks_current_access() {
        let db = UserDriveVectorDB::new(
            Uuid::from_u128(2),
            Uuid::from_u128(1),
            std::path::PathBuf::from("/tmp/vdb"),
        );
        let mut payload = Map::new();
        db.scope().stamp(&mut payload);

        let allowed = std::sync::Mutex::new(true);
        let verifier: Arc<AccessVerifier> =
            Arc::new(move |_scope, _path, _bucket| allowed.lock().is_ok_and(|flag| *flag));
        let db = db.with_access_verifier(verifier);
        assert!(db.authorize_chunk(&payload, "docs/a.pdf", "bucket"));

        // Revoked at the trusted source; the Qdrant payload is unchanged.
        // (Covered end-to-end in scope.rs; here we assert the db delegates.)
        let denied = UserDriveVectorDB::new(
            Uuid::from_u128(2),
            Uuid::from_u128(1),
            std::path::PathBuf::from("/tmp/vdb"),
        )
        .with_access_verifier(Arc::new(|_scope, _path, _bucket| false));
        assert!(!denied.authorize_chunk(&payload, "docs/a.pdf", "bucket"));
    }
}
