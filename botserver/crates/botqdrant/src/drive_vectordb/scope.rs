//! Server-derived scope stamping and permission recheck for vector payloads (#2820).
//!
//! A Qdrant payload is untrusted data: it can be stale (a Drive permission was
//! revoked after indexing) or poisoned by a peer that can reach the data plane.
//! Every point is therefore stamped at index time with identifiers taken from
//! trusted server state (the session/DB), never from the payload or the request.
//! Every query path re-validates that stamp and, when the caller installs one,
//! a *current* ACL verdict before returning any embedded text.

use serde_json::{Map, Value};
use std::sync::Arc;
use uuid::Uuid;

pub const SCOPE_BOT_ID: &str = "scope_bot_id";
pub const SCOPE_USER_ID: &str = "scope_user_id";
pub const SCOPE_ORG_ID: &str = "scope_org_id";
pub const SCOPE_BRANCH_ID: &str = "scope_branch_id";

/// Owner identifiers of a point, derived from trusted server state. Never build
/// one from a Qdrant payload or a request: that is exactly the trust inversion
/// this module exists to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorScope {
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
}

impl VectorScope {
    pub fn new(bot_id: Uuid, user_id: Uuid) -> Self {
        Self {
            bot_id,
            user_id,
            org_id: None,
            branch_id: None,
        }
    }

    pub fn with_org_id(mut self, org_id: Uuid) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_branch_id(mut self, branch_id: Uuid) -> Self {
        self.branch_id = Some(branch_id);
        self
    }

    /// Stamp trusted identifiers into a point payload. Only call this at index
    /// time, after the identifiers have been resolved from the server session.
    pub fn stamp(&self, payload: &mut Map<String, Value>) {
        payload.insert(SCOPE_BOT_ID.into(), Value::String(self.bot_id.to_string()));
        payload.insert(
            SCOPE_USER_ID.into(),
            Value::String(self.user_id.to_string()),
        );
        if let Some(org_id) = self.org_id {
            payload.insert(SCOPE_ORG_ID.into(), Value::String(org_id.to_string()));
        }
        if let Some(branch_id) = self.branch_id {
            payload.insert(SCOPE_BRANCH_ID.into(), Value::String(branch_id.to_string()));
        }
    }

    /// Server-derived Qdrant payload filter. Applied at query time so
    /// unstamped or foreign points are never considered, in addition to the
    /// client-side recheck in `authorizes_payload`.
    pub fn as_qdrant_filter(&self) -> Value {
        let mut must = vec![
            serde_json::json!({
                "key": SCOPE_BOT_ID,
                "match": { "value": self.bot_id.to_string() }
            }),
            serde_json::json!({
                "key": SCOPE_USER_ID,
                "match": { "value": self.user_id.to_string() }
            }),
        ];
        if let Some(org_id) = self.org_id {
            must.push(serde_json::json!({
                "key": SCOPE_ORG_ID,
                "match": { "value": org_id.to_string() }
            }));
        }
        if let Some(branch_id) = self.branch_id {
            must.push(serde_json::json!({
                "key": SCOPE_BRANCH_ID,
                "match": { "value": branch_id.to_string() }
            }));
        }
        Value::Object(Map::from_iter([("must".to_string(), Value::Array(must))]))
    }

    /// True only when the payload's stamp matches this trusted scope exactly.
    /// A missing or malformed stamp is rejected: the payload is never the
    /// authority for who may read a chunk.
    pub fn authorizes_payload(&self, payload: &Map<String, Value>) -> bool {
        let stamp_matches = |key: &str, expected: Uuid| -> bool {
            match payload.get(key).and_then(Value::as_str) {
                Some(raw) => matches!(Uuid::parse_str(raw), Ok(id) if id == expected),
                None => false,
            }
        };

        if !stamp_matches(SCOPE_BOT_ID, self.bot_id) || !stamp_matches(SCOPE_USER_ID, self.user_id)
        {
            return false;
        }
        if let Some(org_id) = self.org_id {
            if !stamp_matches(SCOPE_ORG_ID, org_id) {
                return false;
            }
        }
        if let Some(branch_id) = self.branch_id {
            if !stamp_matches(SCOPE_BRANCH_ID, branch_id) {
                return false;
            }
        }
        true
    }
}

/// Current ACL check for a document, supplied by the caller from a trusted
/// source (Drive/OtterIO object + `drive_shares`, or the KB group tables). It
/// receives the trusted scope plus the candidate document's path and bucket so
/// a process-wide verifier can still decide per user.
pub type AccessVerifier = dyn Fn(&VectorScope, &str, &str) -> bool + Send + Sync;

/// Decide whether one chunk may be returned. Pure so the revoke path is
/// testable without a running Qdrant: `verifier` supplies the *current* verdict
/// from the trusted source. With no verifier installed the stamp is still
/// enforced; only the per-document recheck is skipped.
pub fn chunk_is_authorized(
    scope: &VectorScope,
    payload: &Map<String, Value>,
    file_path: &str,
    bucket: &str,
    verifier: Option<&AccessVerifier>,
) -> bool {
    if !scope.authorizes_payload(payload) {
        return false;
    }
    match verifier {
        Some(check) => check(scope, file_path, bucket),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> VectorScope {
        VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2))
    }

    fn stamped(scope: &VectorScope) -> Map<String, Value> {
        let mut payload = Map::new();
        scope.stamp(&mut payload);
        payload
    }

    #[test]
    fn a_stamped_payload_from_the_owner_is_returned() {
        let scope = scope();
        let payload = stamped(&scope);
        assert!(scope.authorizes_payload(&payload));
        assert!(chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            None
        ));
    }

    #[test]
    fn revoking_access_hides_a_previously_indexed_chunk() {
        // The chunk was indexed while access was granted; the stale Qdrant
        // payload is unchanged when the trusted source revokes the grant.
        let scope = scope();
        let payload = stamped(&scope);
        let granted = std::sync::Arc::new(std::sync::Mutex::new(vec![(
            scope.user_id,
            "docs/a.pdf".to_string(),
        )]));
        let g = std::sync::Arc::clone(&granted);
        let current_acl = move |scope: &VectorScope, path: &str, _bucket: &str| {
            g.lock()
                .is_ok_and(|granted| granted.iter().any(|(uid, p)| *uid == scope.user_id && p == path))
        };

        assert!(chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            Some(&current_acl)
        ));

        // Revoked: only the trusted source changes, the payload does not.
        if let Ok(mut g) = granted.lock() {
            g.clear();
        }
        assert!(!chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            Some(&current_acl)
        ));
    }

    #[test]
    fn a_foreign_or_missing_stamp_is_never_authoritative() {
        let scope = scope();
        let mut payload = stamped(&scope);
        payload.insert(
            SCOPE_BOT_ID.into(),
            Value::String(Uuid::from_u128(99).to_string()),
        );
        assert!(!scope.authorizes_payload(&payload));
        assert!(!chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            None
        ));
        assert!(!scope.authorizes_payload(&Map::new()));
    }

    #[test]
    fn a_verifier_from_another_tenant_cannot_read_the_chunk() {
        let owner = scope();
        let attacker = VectorScope::new(Uuid::from_u128(10), Uuid::from_u128(11));
        let payload = stamped(&owner);
        assert!(!attacker.authorizes_payload(&payload));
        assert!(!chunk_is_authorized(
            &attacker,
            &payload,
            "docs/a.pdf",
            "bucket",
            None
        ));
    }

    #[test]
    fn the_query_filter_is_derived_only_from_the_trusted_scope() {
        let scope = scope().with_org_id(Uuid::from_u128(3));
        let filter = scope.as_qdrant_filter();
        let keys: Vec<&str> = filter["must"]
            .as_array()
            .map(|m| m.iter().filter_map(|c| c["key"].as_str()).collect())
            .unwrap_or_else(Vec::new);
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&SCOPE_BOT_ID));
        assert!(keys.contains(&SCOPE_USER_ID));
        assert!(keys.contains(&SCOPE_ORG_ID));
    }

    #[test]
    fn optional_org_and_branch_stamps_are_enforced() {
        let scope = scope()
            .with_org_id(Uuid::from_u128(3))
            .with_branch_id(Uuid::from_u128(4));
        let mut payload = stamped(&scope);
        assert!(scope.authorizes_payload(&payload));
        payload.insert(
            SCOPE_ORG_ID.into(),
            Value::String(Uuid::from_u128(77).to_string()),
        );
        assert!(!scope.authorizes_payload(&payload));
    }
}
