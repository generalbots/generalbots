//! Server-derived scoping for knowledge-base collections and Qdrant filters.
//!
//! A caller only supplies a friendly KB name. The owning bot and branch come
//! from verified server state (a signed JWT claim resolved to a bot row, or a
//! server-side session). Both the physical Qdrant collection name and the
//! retrieval filter are bound to that pair, so two tenants that create the
//! same friendly name can never read each other's vectors.
//!
//! Every helper fails closed: without a verified, non-nil bot/branch pair no
//! collection name is produced and no filter is handed to a query builder.

use serde_json::Value;
use uuid::Uuid;

/// Payload key stamped on every KB point with the owning bot id at index time.
pub const KB_SCOPE_PAYLOAD_KEY: &str = "bot_id";

/// A knowledge-base tenant scope resolved from verified server-side ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KbTenantScope {
    pub bot_id: Uuid,
    pub branch_id: Uuid,
}

impl KbTenantScope {
    /// Nil ids are never a valid tenant; reject them so scope can never be
    /// confused with a global/default lookup.
    pub fn new(bot_id: Uuid, branch_id: Uuid) -> Option<Self> {
        if bot_id.is_nil() || branch_id.is_nil() {
            None
        } else {
            Some(Self { bot_id, branch_id })
        }
    }

    /// Deterministic, tenant-scoped Qdrant collection name. The bot and branch
    /// ids make identically named collections from different tenants distinct.
    pub fn collection_name(&self, name: &str) -> String {
        format!(
            "kb_{}_{}_{}",
            self.bot_id,
            self.branch_id,
            sanitize_collection_suffix(name)
        )
    }

    /// Mandatory payload filter for every retrieval performed on this scope.
    pub fn filter(&self) -> Value {
        serde_json::json!({
            "must": [
                { "key": KB_SCOPE_PAYLOAD_KEY, "match": { "value": self.bot_id.to_string() } }
            ]
        })
    }
}

/// Build a scope from optional ids; `None` when either id is absent or nil.
pub fn scope_from_ids(bot_id: Option<Uuid>, branch_id: Option<Uuid>) -> Option<KbTenantScope> {
    KbTenantScope::new(bot_id?, branch_id?)
}

/// Sanitize a caller-supplied friendly name into a safe collection suffix.
pub fn sanitize_collection_suffix(name: &str) -> String {
    let mut suffix = String::new();
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            suffix.push(ch);
        } else {
            suffix.push('_');
        }
    }
    let suffix = suffix.trim_matches('_');
    if suffix.is_empty() {
        "unnamed".to_string()
    } else {
        suffix.to_string()
    }
}

/// Qdrant `filter` body pinning retrieval to one verified bot.
///
/// Returns `None` when no verified bot id is available. Callers MUST return no
/// results on `None` rather than issuing an unfiltered query.
pub fn filter_for_bot(bot_id: Option<Uuid>) -> Option<Value> {
    let bot_id = bot_id?;
    if bot_id.is_nil() {
        return None;
    }
    Some(serde_json::json!({
        "must": [
            { "key": KB_SCOPE_PAYLOAD_KEY, "match": { "value": bot_id.to_string() } }
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT_A: &str = "11111111-1111-4111-8111-111111111111";
    const BOT_B: &str = "22222222-2222-4222-8222-222222222222";
    const BRANCH_A: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const BRANCH_B: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

    fn id(value: &str) -> Uuid {
        Uuid::parse_str(value).unwrap()
    }

    #[test]
    fn two_tenants_never_share_a_collection_name() {
        let tenant_a = KbTenantScope::new(id(BOT_A), id(BRANCH_A)).unwrap();
        let tenant_b = KbTenantScope::new(id(BOT_B), id(BRANCH_B)).unwrap();
        let name_a = tenant_a.collection_name("hr");
        let name_b = tenant_b.collection_name("hr");
        assert_ne!(name_a, name_b);
        assert!(name_a.contains(BOT_A) && name_a.contains(BRANCH_A));
        assert!(name_b.contains(BOT_B) && name_b.contains(BRANCH_B));
    }

    #[test]
    fn two_tenants_never_share_a_filter() {
        let tenant_a = KbTenantScope::new(id(BOT_A), id(BRANCH_A)).unwrap();
        let tenant_b = KbTenantScope::new(id(BOT_B), id(BRANCH_B)).unwrap();
        assert_eq!(tenant_a.filter()["must"][0]["match"]["value"], BOT_A);
        assert_eq!(tenant_b.filter()["must"][0]["match"]["value"], BOT_B);
        assert_eq!(tenant_a.filter()["must"][0]["key"], KB_SCOPE_PAYLOAD_KEY);
    }

    #[test]
    fn missing_scope_never_yields_a_collection_or_filter() {
        assert_eq!(scope_from_ids(None, Some(id(BRANCH_A))), None);
        assert_eq!(scope_from_ids(Some(id(BOT_A)), None), None);
        assert_eq!(KbTenantScope::new(Uuid::nil(), id(BRANCH_A)), None);
        assert_eq!(KbTenantScope::new(id(BOT_A), Uuid::nil()), None);
        assert_eq!(filter_for_bot(None), None);
        assert_eq!(filter_for_bot(Some(Uuid::nil())), None);
    }

    #[test]
    fn friendly_name_is_sanitized_and_never_empty() {
        assert_eq!(sanitize_collection_suffix("My HR Docs"), "my_hr_docs");
        assert_eq!(sanitize_collection_suffix("../secrets"), "secrets");
        assert_eq!(sanitize_collection_suffix("  "), "unnamed");
    }
}
