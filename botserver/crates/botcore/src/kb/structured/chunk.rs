use super::citation::{Citation, SourceRef};
use crate::kb::permissions::UserContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deterministic 64-bit FNV-1a over an explicit list of components.
///
/// The vector indexer uses this as the document-model-facing ID so that
/// re-indexing an unchanged section produces the same ID (idempotent upserts)
/// while an edit produces a new one.
pub fn stable_id(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for (position, part) in parts.iter().enumerate() {
        if position > 0 {
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

/// Access metadata attached to every extracted chunk. Values are copies of
/// the object ACL at index time; retrieval never re-reads the source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AclMetadata {
    pub tenant_id: String,
    pub is_public: bool,
    pub allowed_users: Vec<String>,
    pub allowed_roles: Vec<String>,
    pub allowed_groups: Vec<String>,
}

impl AclMetadata {
    pub fn private(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            is_public: false,
            allowed_users: Vec::new(),
            allowed_roles: Vec::new(),
            allowed_groups: Vec::new(),
        }
    }

    pub fn public(tenant_id: impl Into<String>) -> Self {
        Self {
            is_public: true,
            ..Self::private(tenant_id)
        }
    }

    pub fn with_users(mut self, users: Vec<String>) -> Self {
        self.allowed_users = users;
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.allowed_roles = roles;
        self
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.allowed_groups = groups;
        self
    }

    /// True only if the user may read this chunk. A denial is never surfaced
    /// as a "restricted result" by retrieval; the chunk is simply absent.
    pub fn allows(&self, user: &UserContext) -> bool {
        if self.tenant_id != Self::tenant_of(user) {
            return false;
        }
        if self.is_public {
            return true;
        }
        if !user.is_authenticated {
            return false;
        }
        let id = user.user_id.to_string();
        if self.allowed_users.iter().any(|value| value == &id) {
            return true;
        }
        if let Some(email) = &user.email {
            if self.allowed_users.iter().any(|value| value == email) {
                return true;
            }
        }
        self.allowed_roles.iter().any(|role| user.roles.contains(role))
            || self.allowed_groups.iter().any(|group| user.groups.contains(group))
    }

    fn tenant_of(user: &UserContext) -> String {
        match &user.organization_id {
            Some(id) => id.to_string(),
            None => String::new(),
        }
    }
}

/// A structure-aware chunk carrying its own stable ID and citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredChunk {
    pub id: String,
    pub document_id: String,
    pub file: String,
    pub version: String,
    pub tenant_id: String,
    pub heading_path: Vec<String>,
    pub source: SourceRef,
    pub content: String,
    pub acl: AclMetadata,
}

impl StructuredChunk {
    pub fn citation(&self) -> Citation {
        Citation::new(
            &self.document_id,
            &self.file,
            &self.version,
            self.source.clone(),
            self.heading_path.clone(),
        )
    }

    /// Payload seam for the vector indexer: the citation and the ACL travel
    /// with the vector, so retrieval can filter and cite without re-reading
    /// the source document.
    pub fn to_payload(&self) -> HashMap<String, serde_json::Value> {
        let mut payload = HashMap::new();
        payload.insert(
            "content".to_string(),
            serde_json::Value::String(self.content.clone()),
        );
        payload.insert(
            "document_path".to_string(),
            serde_json::Value::String(self.file.clone()),
        );
        payload.insert(
            "stable_id".to_string(),
            serde_json::Value::String(self.id.clone()),
        );
        payload.insert(
            "tenant_id".to_string(),
            serde_json::Value::String(self.tenant_id.clone()),
        );
        Self::insert_serialized(&mut payload, "citation", &self.citation());
        Self::insert_serialized(&mut payload, "acl", &self.acl);
        payload
    }

    fn insert_serialized<T: Serialize>(
        payload: &mut HashMap<String, serde_json::Value>,
        key: &str,
        value: &T,
    ) {
        match serde_json::to_value(value) {
            Ok(serialized) => {
                payload.insert(key.to_string(), serialized);
            }
            Err(error) => {
                log::error!("Could not serialize {key} into KB payload: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn parse_uuid(value: &str) -> Uuid {
        match Uuid::parse_str(value) {
            Ok(id) => id,
            Err(_) => Uuid::nil(),
        }
    }

    fn tenant() -> Uuid {
        parse_uuid("11111111-1111-1111-1111-111111111111")
    }

    fn user() -> Uuid {
        parse_uuid("33333333-3333-3333-3333-333333333333")
    }

    #[test]
    fn stable_id_is_deterministic_and_order_sensitive() {
        let first = stable_id(&["tenant", "contract", "clause=7.3", "text"]);
        let second = stable_id(&["tenant", "contract", "clause=7.3", "text"]);
        let other = stable_id(&["tenant", "contract", "text", "clause=7.3"]);
        assert_eq!(first, second);
        assert_eq!(first.len(), 16);
        assert_ne!(first, other);
    }

    #[test]
    fn private_acl_denies_anonymous_and_unlisted_users() {
        let acl = AclMetadata::private(tenant().to_string());
        let anonymous = UserContext {
            organization_id: Some(tenant()),
            ..UserContext::anonymous()
        };
        let user = UserContext::authenticated(user(), None, Some(tenant()));
        assert!(!acl.allows(&anonymous));
        assert!(!acl.allows(&user));
    }

    #[test]
    fn private_acl_allows_matched_role_group_and_user() {
        let acl = AclMetadata::private(tenant().to_string())
            .with_users(vec![user().to_string()])
            .with_roles(vec!["legal".to_string()]);
        let allowed = UserContext::authenticated(user(), None, Some(tenant()));
        assert!(acl.allows(&allowed));

        let by_role = UserContext::authenticated(
            parse_uuid("44444444-4444-4444-4444-444444444444"),
            None,
            Some(tenant()),
        )
        .with_roles(vec!["legal".to_string()]);
        assert!(acl.allows(&by_role));
    }

    #[test]
    fn payload_carries_citation_and_acl_without_document_bytes() {
        let chunk = StructuredChunk {
            id: "abc".to_string(),
            document_id: "doc".to_string(),
            file: "contract.docx".to_string(),
            version: "3".to_string(),
            tenant_id: tenant().to_string(),
            heading_path: vec!["Termination".to_string()],
            source: SourceRef::clause("7.3").with_page(12),
            content: "notice period".to_string(),
            acl: AclMetadata::private(tenant().to_string()),
        };
        let payload = chunk.to_payload();
        assert_eq!(payload["stable_id"], serde_json::json!("abc"));
        assert_eq!(payload["citation"]["file"], serde_json::json!("contract.docx"));
        assert_eq!(payload["acl"]["tenant_id"], serde_json::json!(tenant().to_string()));
        assert_eq!(payload["content"], serde_json::json!("notice period"));
    }

    #[test]
    fn acl_never_crosses_tenant_boundaries() {
        let other = parse_uuid("22222222-2222-2222-2222-222222222222");
        let acl = AclMetadata::public(tenant().to_string());
        let foreign = UserContext::authenticated(user(), None, Some(other));
        assert!(!acl.allows(&foreign));
    }
}
