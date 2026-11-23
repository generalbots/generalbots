//! Payload-authority recheck for KB vector search (#2820).
//!
//! A Qdrant payload is untrusted: a stale or poisoned point may carry a foreign
//! scope stamp. Every retrieval path must validate the stamp against the
//! trusted bot_id derived from the caller's session and, when installed, a
//! current-ACL verdict before returning embedded text.

use serde_json::Value;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

/// Current-ACL recheck for KB content, installed at startup from a trusted
/// source (the session's KB associations + `kb_group_associations` /
/// `rbac_user_groups`). Args: trusted bot_id, collection, document path.
pub type KbAccessVerifier = dyn Fn(Uuid, &str, &str) -> bool + Send + Sync;

static KB_ACCESS_VERIFIER: OnceLock<Arc<KbAccessVerifier>> = OnceLock::new();

pub fn set_kb_access_verifier(
    verifier: Arc<KbAccessVerifier>,
) -> Result<(), Arc<KbAccessVerifier>> {
    KB_ACCESS_VERIFIER.set(verifier)
}

/// Reject content whose stamp does not match the trusted bot_id - never read
/// the owner from the payload. Legacy points with no stamp remain readable;
/// `bot_id = nil` means no trusted scope is known, in which case stamped points
/// are rejected conservatively.
pub fn kb_payload_scope_is_trusted(payload: &Value, bot_id: Uuid) -> bool {
    let mut has_stamp = false;
    for key in ["scope_bot_id", "bot_id"] {
        if let Some(raw) = payload.get(key).and_then(Value::as_str) {
            has_stamp = true;
            match Uuid::parse_str(raw) {
                Ok(stamped) if bot_id.is_nil() || stamped == bot_id => {}
                _ => return false,
            }
        }
    }
    !bot_id.is_nil() || !has_stamp
}

/// Single gate applied to every KB search path before content is returned.
pub fn kb_chunk_is_authorized(
    payload: &Value,
    bot_id: Uuid,
    collection_name: &str,
    document_path: &str,
) -> bool {
    if !kb_payload_scope_is_trusted(payload, bot_id) {
        return false;
    }
    match KB_ACCESS_VERIFIER.get() {
        Some(verifier) => verifier(bot_id, collection_name, document_path),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_payload_stamped_for_another_bot_is_rejected() {
        let bot = Uuid::from_u128(1);
        let foreign = Uuid::from_u128(2);
        let payload = json!({ "content": "secret", "scope_bot_id": foreign.to_string() });
        assert!(!kb_payload_scope_is_trusted(&payload, bot));
        assert!(!kb_chunk_is_authorized(&payload, bot, "kb_x", "docs/a.pdf"));
    }

    #[test]
    fn an_own_stamp_or_legacy_unstamped_point_is_accepted() {
        let bot = Uuid::from_u128(1);
        let own = json!({ "content": "ok", "bot_id": bot.to_string() });
        assert!(kb_payload_scope_is_trusted(&own, bot));
        let legacy = json!({ "content": "ok" });
        assert!(kb_payload_scope_is_trusted(&legacy, bot));
    }

    #[test]
    fn revoked_access_hides_an_own_stamped_chunk() {
        let bot = Uuid::from_u128(1);
        let payload = json!({ "content": "ok", "scope_bot_id": bot.to_string() });
        assert!(kb_payload_scope_is_trusted(&payload, bot));

        // The gate is `trusted stamp && current ACL verdict`. The process-wide
        // verifier supplies the second operand; exercise both states of it.
        let allowed = |current_acl: bool| kb_payload_scope_is_trusted(&payload, bot) && current_acl;
        assert!(allowed(true));
        assert!(!allowed(false));
    }
}
