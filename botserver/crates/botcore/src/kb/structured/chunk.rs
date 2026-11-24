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
        self.allowed_roles
            .iter()
            .any(|role| user.roles.contains(role))
            || self
                .allowed_groups
                .iter()
                .any(|group| user.groups.contains(group))
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
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["contracts".to_string()]);
        let by_user = UserContext::authenticated(user(), None, Some(tenant()));
        assert!(acl.allows(&by_user));

        let by_role = UserContext::authenticated(
            parse_uuid("44444444-4444-4444-4444-444444444444"),
            None,
            Some(tenant()),
        )
        .with_roles(vec!["legal".to_string()]);
        assert!(acl.allows(&by_role));

        let by_group = UserContext::authenticated(
            parse_uuid("55555555-5555-5555-5555-555555555555"),
            None,
            Some(tenant()),
        )
        .with_groups(vec!["contracts".to_string()]);
        assert!(acl.allows(&by_group));
    }

    #[test]
    fn payload_carries_citation_and_acl_without_document_bytes() {
        let acl = AclMetadata::private(tenant().to_string())
            .with_users(vec![user().to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["contracts".to_string()]);
        let chunk = StructuredChunk {
            id: "abc".to_string(),
            document_id: "doc".to_string(),
            file: "contract.docx".to_string(),
            version: "3".to_string(),
            tenant_id: tenant().to_string(),
            heading_path: vec!["Termination".to_string()],
            source: SourceRef::clause("7.3").with_page(12),
            content: "notice period".to_string(),
            acl: acl.clone(),
        };
        let payload = chunk.to_payload();

        assert_eq!(payload["stable_id"], serde_json::json!("abc"));
        assert_eq!(payload["document_path"], serde_json::json!("contract.docx"));
        assert_eq!(payload["content"], serde_json::json!("notice period"));
        assert_eq!(
            payload["tenant_id"],
            serde_json::json!(tenant().to_string())
        );

        assert_eq!(payload["citation"]["document_id"], serde_json::json!("doc"));
        assert_eq!(
            payload["citation"]["file"],
            serde_json::json!("contract.docx")
        );
        assert_eq!(payload["citation"]["version"], serde_json::json!("3"));
        assert_eq!(
            payload["citation"]["source"]["kind"],
            serde_json::json!("clause")
        );
        assert_eq!(
            payload["citation"]["source"]["locator"],
            serde_json::json!("7.3")
        );
        assert_eq!(payload["citation"]["source"]["page"], serde_json::json!(12));
        assert_eq!(
            payload["citation"]["heading_path"],
            serde_json::json!(["Termination"])
        );

        assert_eq!(
            payload["acl"]["tenant_id"],
            serde_json::json!(tenant().to_string())
        );
        assert_eq!(payload["acl"]["is_public"], serde_json::json!(false));
        assert_eq!(
            payload["acl"]["allowed_users"],
            serde_json::json!([user().to_string()])
        );
        assert_eq!(
            payload["acl"]["allowed_roles"],
            serde_json::json!(["legal"])
        );
        assert_eq!(
            payload["acl"]["allowed_groups"],
            serde_json::json!(["contracts"])
        );
        let expected_acl = match serde_json::to_value(&acl) {
            Ok(value) => value,
            Err(e) => {
                log::warn!("Failed to serialize ACL for assertion: {e}");
                serde_json::Value::Null
            }
        };
        assert_eq!(payload["acl"], expected_acl);
    }

    #[test]
    fn acl_never_crosses_tenant_boundaries() {
        let other = parse_uuid("22222222-2222-2222-2222-222222222222");
        let acl = AclMetadata::public(tenant().to_string());
        let foreign = UserContext::authenticated(user(), None, Some(other));
        assert!(!acl.allows(&foreign));
    }

    #[test]
    fn public_acl_allows_same_tenant_users_including_anonymous() {
        let acl = AclMetadata::public(tenant().to_string());
        let anonymous = UserContext {
            organization_id: Some(tenant()),
            ..UserContext::anonymous()
        };
        let stranger = UserContext::authenticated(user(), None, Some(tenant()));
        assert!(acl.allows(&anonymous));
        assert!(acl.allows(&stranger));
    }

    #[test]
    fn tenant_check_precedes_public_and_membership() {
        let other = parse_uuid("22222222-2222-2222-2222-222222222222");
        let acl = AclMetadata::public(tenant().to_string())
            .with_users(vec![user().to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["contracts".to_string()]);
        let foreign_user = UserContext::authenticated(user(), None, Some(other))
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["contracts".to_string()]);
        let foreign_anonymous = UserContext {
            organization_id: Some(other),
            ..UserContext::anonymous()
        };
        assert!(!acl.allows(&foreign_user));
        assert!(!acl.allows(&foreign_anonymous));
    }

    #[test]
    fn user_membership_matches_id_or_email() {
        let by_id = AclMetadata::private(tenant().to_string()).with_users(vec![user().to_string()]);
        assert!(by_id.allows(&UserContext::authenticated(user(), None, Some(tenant()))));

        let email = "legal@example.test".to_string();
        let by_email = AclMetadata::private(tenant().to_string()).with_users(vec![email.clone()]);
        let email_user = UserContext::authenticated(
            parse_uuid("66666666-6666-6666-6666-666666666666"),
            Some(email),
            Some(tenant()),
        );
        assert!(by_email.allows(&email_user));
    }

    #[test]
    fn private_acl_denies_stranger_with_unmatched_identity() {
        let acl = AclMetadata::private(tenant().to_string())
            .with_users(vec![user().to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["contracts".to_string()]);
        let stranger = UserContext::authenticated(
            parse_uuid("77777777-7777-7777-7777-777777777777"),
            Some("stranger@example.test".to_string()),
            Some(tenant()),
        )
        .with_roles(vec!["sales".to_string()])
        .with_groups(vec!["guests".to_string()]);
        assert!(!acl.allows(&stranger));

        let anonymous = UserContext {
            organization_id: Some(tenant()),
            ..UserContext::anonymous()
        };
        assert!(!acl.allows(&anonymous));
    }
}
