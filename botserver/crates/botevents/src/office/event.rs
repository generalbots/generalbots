//! The metadata-only domain event and the privacy-aware payload builder.
//!
//! `DomainEvent` is what the Office apps emit. It may carry an `EventContent`
//! (body text) internally, but that content is never serialized into a webhook
//! or native payload unless the subscription opts in and `ContentAcl` grants
//! access. The default ACL denies every request.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::taxonomy::{OfficeEventType, PAYLOAD_SCHEMA_VERSION};

/// Optional document text. Kept in the event for automation that runs inside
/// the trust boundary; excluded from delivery payloads by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventContent {
    pub title: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

impl EventContent {
    #[must_use]
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            excerpt: None,
        }
    }

    #[must_use]
    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }
}

/// A single versioned Office event. IDs and metadata are always present;
/// content is opt-in and only leaves the process through a gated builder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Stable unique id, also the first half of the idempotency key.
    pub event_id: String,
    pub event_type: OfficeEventType,
    pub schema_version: u32,
    pub app: String,
    pub workspace_id: String,
    pub doc_id: String,
    pub actor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<EventContent>,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent {
    #[must_use]
    pub fn new(
        event_id: impl Into<String>,
        event_type: OfficeEventType,
        workspace_id: impl Into<String>,
        doc_id: impl Into<String>,
        actor_id: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type,
            schema_version: PAYLOAD_SCHEMA_VERSION,
            app: event_type.app().to_string(),
            workspace_id: workspace_id.into(),
            doc_id: doc_id.into(),
            actor_id: actor_id.into(),
            correlation_id: None,
            tags: Vec::new(),
            metadata: Map::new(),
            content: None,
            occurred_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    #[must_use]
    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_content(mut self, content: EventContent) -> Self {
        self.content = Some(content);
        self
    }

    /// The idempotency key used for one subscription's delivery. Replaying the
    /// same event to the same endpoint reuses this key.
    #[must_use]
    pub fn idempotency_key(&self, subscription_id: &str) -> String {
        format!("{}:{}", self.event_id, subscription_id)
    }

    /// Structural checks before the event is emitted.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.event_id.trim().is_empty() {
            issues.push("event_id is required".to_string());
        }
        if self.workspace_id.trim().is_empty() {
            issues.push("workspace_id is required".to_string());
        }
        if self.doc_id.trim().is_empty() {
            issues.push("doc_id is required".to_string());
        }
        if self.actor_id.trim().is_empty() {
            issues.push("actor_id is required".to_string());
        }
        if self.event_id.chars().count() > 128 {
            issues.push("event_id exceeds 128 characters".to_string());
        }
        if self.schema_version != PAYLOAD_SCHEMA_VERSION {
            issues.push(format!(
                "unsupported payload schema version {}",
                self.schema_version
            ));
        }
        if let Some(content) = &self.content {
            if content.body.trim().is_empty() {
                issues.push("content.body must not be empty".to_string());
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

/// Authorisation check for opt-in content. Implementations must verify the
/// acting user's ACL on the target document; the default denies everything.
pub trait ContentAcl: Send + Sync {
    fn can_read_content(&self, actor_id: &str, workspace_id: &str, doc_id: &str) -> bool;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAllContent;

impl ContentAcl for DenyAllContent {
    fn can_read_content(&self, actor_id: &str, workspace_id: &str, doc_id: &str) -> bool {
        // Consume the request scope explicitly: this policy denies regardless.
        let scope = format!("{actor_id}:{workspace_id}:{doc_id}");
        drop(scope);
        false
    }
}

/// Test/demo ACL. Production code supplies its own implementation backed by
/// the document permission store.
#[derive(Debug, Clone, Copy, Default)]
pub struct AllowAllContent;

impl ContentAcl for AllowAllContent {
    fn can_read_content(&self, actor_id: &str, workspace_id: &str, doc_id: &str) -> bool {
        // Consume the request scope explicitly: this policy is a test/demo
        // grant; production supplies a real ACL check.
        let scope = format!("{actor_id}:{workspace_id}:{doc_id}");
        drop(scope);
        true
    }
}

/// Builds the JSON body delivered to a subscriber. Content is included only
/// when both `include_content` and the ACL allow it. An optional field
/// allowlist performs payload minimisation.
pub struct DeliveryPayloadBuilder<'a> {
    include_content: bool,
    acl: &'a dyn ContentAcl,
    fields: Option<Vec<String>>,
}

impl<'a> DeliveryPayloadBuilder<'a> {
    #[must_use]
    pub fn new(include_content: bool, acl: &'a dyn ContentAcl) -> Self {
        Self {
            include_content,
            acl,
            fields: None,
        }
    }

    #[must_use]
    pub fn with_fields(mut self, fields: Option<Vec<String>>) -> Self {
        self.fields = fields;
        self
    }

    pub fn build(&self, event: &DomainEvent) -> Result<Value, Vec<String>> {
        let mut map = Map::new();
        put(
            &mut map,
            &self.fields,
            "event_id",
            Value::String(event.event_id.clone()),
        );
        put(
            &mut map,
            &self.fields,
            "type",
            Value::String(event.event_type.as_str().to_string()),
        );
        put(
            &mut map,
            &self.fields,
            "schema_version",
            Value::from(event.schema_version),
        );
        put(
            &mut map,
            &self.fields,
            "app",
            Value::String(event.app.clone()),
        );
        put(
            &mut map,
            &self.fields,
            "workspace_id",
            Value::String(event.workspace_id.clone()),
        );
        put(
            &mut map,
            &self.fields,
            "doc_id",
            Value::String(event.doc_id.clone()),
        );
        put(
            &mut map,
            &self.fields,
            "actor_id",
            Value::String(event.actor_id.clone()),
        );
        put(
            &mut map,
            &self.fields,
            "occurred_at",
            Value::String(event.occurred_at.to_rfc3339()),
        );
        put(&mut map, &self.fields, "tags", as_value(&event.tags)?);
        put(&mut map, &self.fields, "metadata", as_value(&event.metadata)?);
        if let Some(correlation) = &event.correlation_id {
            put(
                &mut map,
                &self.fields,
                "correlation_id",
                Value::String(correlation.clone()),
            );
        }

        if self.include_content
            && self
                .acl
                .can_read_content(&event.actor_id, &event.workspace_id, &event.doc_id)
        {
            if let Some(content) = &event.content {
                put(&mut map, &self.fields, "content", as_value(content)?);
            }
        }
        Ok(Value::Object(map))
    }
}

fn as_value<T: Serialize>(value: &T) -> Result<Value, Vec<String>> {
    serde_json::to_value(value).map_err(|error| vec![format!("payload serialization failed: {error}")])
}

fn put(
    map: &mut Map<String, Value>,
    fields: &Option<Vec<String>>,
    key: &str,
    value: Value,
) {
    if fields
        .as_ref()
        .map_or(true, |allowed| allowed.iter().any(|name| name == key))
    {
        map.insert(key.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        AllowAllContent, ContentAcl, DenyAllContent, DeliveryPayloadBuilder, DomainEvent,
        EventContent,
    };
    use crate::office::taxonomy::OfficeEventType;

    fn event_with_body() -> DomainEvent {
        DomainEvent::new(
            "evt-1",
            OfficeEventType::DocCommentAdded,
            "ws-1",
            "doc-1",
            "usr-1",
        )
        .with_content(EventContent::new(
            "Confidential quarterly report",
            "The secret body text must not leak",
        ))
    }

    struct ActorAcl;

    impl ContentAcl for ActorAcl {
        fn can_read_content(&self, actor_id: &str, workspace_id: &str, doc_id: &str) -> bool {
            actor_id == "usr-1" && workspace_id == "ws-1" && doc_id == "doc-1"
        }
    }

    #[test]
    fn content_never_leaks_by_default() {
        let payload = DeliveryPayloadBuilder::new(false, &AllowAllContent)
            .build(&event_with_body())
            .unwrap_or(json!({}));
        let text = payload.to_string();
        assert!(payload.get("content").is_none());
        assert!(!text.contains("secret body text"));
        assert_eq!(payload["doc_id"], "doc-1");
        assert_eq!(payload["actor_id"], "usr-1");
        assert_eq!(payload["type"], "doc.comment.added");
    }

    #[test]
    fn opt_in_without_acl_still_hides_content() {
        let payload = DeliveryPayloadBuilder::new(true, &DenyAllContent)
            .build(&event_with_body())
            .unwrap_or(json!({}));
        assert!(payload.get("content").is_none());
        assert!(!payload.to_string().contains("secret body text"));
    }

    #[test]
    fn opt_in_with_authorised_acl_includes_content() {
        let payload = DeliveryPayloadBuilder::new(true, &ActorAcl)
            .build(&event_with_body())
            .unwrap_or(json!({}));
        assert_eq!(payload["content"]["body"], "The secret body text must not leak");
    }

    #[test]
    fn field_allowlist_minimises_payload() {
        let payload = DeliveryPayloadBuilder::new(false, &DenyAllContent)
            .with_fields(Some(vec![
                "event_id".to_string(),
                "doc_id".to_string(),
                "type".to_string(),
            ]))
            .build(&event_with_body())
            .unwrap_or(json!({}));
        let object = payload.as_object().map(|map| map.len()).unwrap_or(0);
        assert_eq!(object, 3);
        assert!(payload.get("workspace_id").is_none());
        assert!(payload.get("actor_id").is_none());
    }

    #[test]
    fn event_validation_and_idempotency_key() {
        let event = event_with_body();
        assert!(event.validate().is_ok());
        assert_eq!(event.idempotency_key("sub-1"), "evt-1:sub-1");
        let invalid = DomainEvent::new("", OfficeEventType::DocCreated, "", "", "");
        assert!(invalid.validate().is_err());
    }
}
