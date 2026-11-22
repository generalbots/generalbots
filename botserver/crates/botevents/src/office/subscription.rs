//! Webhook subscriptions, filters, secret rotation and native subscriptions.
//!
//! An empty filter vector means "no restriction" for that dimension. Filters
//! are ANDed together; tags match when any event tag intersects the filter.

use chrono::{DateTime, Utc};

use super::event::DomainEvent;
use super::taxonomy::OfficeEventType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookSecret {
    pub id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    /// Only the active secret signs new deliveries. Previous secrets remain
    /// in the list for a verification grace window during rotation.
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebhookSubscription {
    pub id: String,
    pub url: String,
    pub secrets: Vec<WebhookSecret>,
    pub event_types: Vec<OfficeEventType>,
    pub apps: Vec<String>,
    pub workspaces: Vec<String>,
    pub tags: Vec<String>,
    pub include_content: bool,
    pub fields: Option<Vec<String>>,
    pub active: bool,
}

impl WebhookSubscription {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        url: impl Into<String>,
        secret_id: impl Into<String>,
        secret_value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            secrets: vec![WebhookSecret {
                id: secret_id.into(),
                value: secret_value.into(),
                created_at: Utc::now(),
                active: true,
            }],
            event_types: Vec::new(),
            apps: Vec::new(),
            workspaces: Vec::new(),
            tags: Vec::new(),
            include_content: false,
            fields: None,
            active: true,
        }
    }

    #[must_use]
    pub fn with_event_types(mut self, event_types: Vec<OfficeEventType>) -> Self {
        self.event_types = event_types;
        self
    }

    #[must_use]
    pub fn with_apps(mut self, apps: Vec<impl Into<String>>) -> Self {
        self.apps = apps.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_workspaces(mut self, workspaces: Vec<impl Into<String>>) -> Self {
        self.workspaces = workspaces.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_content(mut self, include_content: bool) -> Self {
        self.include_content = include_content;
        self
    }

    #[must_use]
    pub fn with_fields(mut self, fields: Vec<impl Into<String>>) -> Self {
        self.fields = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    /// The secret used to sign new deliveries.
    #[must_use]
    pub fn active_secret(&self) -> Option<&WebhookSecret> {
        self.secrets.iter().find(|secret| secret.active)
    }

    /// Whether a signature made with `(id, value)` is still acceptable. Old
    /// secrets remain valid until explicitly removed after the grace window.
    #[must_use]
    pub fn secret_is_valid(&self, id: &str, value: &str) -> bool {
        self.secrets
            .iter()
            .any(|secret| secret.id == id && secret.value == value)
    }

    /// Rotates in a new signing secret without invalidating signatures still
    /// in flight with the previous one.
    pub fn rotate_secret(
        &mut self,
        new_id: impl Into<String>,
        new_value: impl Into<String>,
        at: DateTime<Utc>,
    ) {
        for secret in &mut self.secrets {
            secret.active = false;
        }
        self.secrets.push(WebhookSecret {
            id: new_id.into(),
            value: new_value.into(),
            created_at: at,
            active: true,
        });
    }

    /// Drops a retired secret once the verification grace window ends.
    pub fn retire_secret(&mut self, id: &str) {
        self.secrets.retain(|secret| secret.id != id);
    }

    #[must_use]
    pub fn matches(&self, event: &DomainEvent) -> bool {
        self.active
            && type_matches(&self.event_types, event.event_type)
            && list_matches(&self.apps, &event.app)
            && list_matches(&self.workspaces, &event.workspace_id)
            && tags_match(&self.tags, &event.tags)
    }
}

/// In-platform subscription used by suite automations. Delivery is an
/// in-memory queue handled by `NativeDispatcher`; no HTTP request is made.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeSubscription {
    pub id: String,
    pub bot_id: String,
    pub event_types: Vec<OfficeEventType>,
    pub apps: Vec<String>,
    pub workspaces: Vec<String>,
    pub active: bool,
}

impl NativeSubscription {
    #[must_use]
    pub fn new(id: impl Into<String>, bot_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bot_id: bot_id.into(),
            event_types: Vec::new(),
            apps: Vec::new(),
            workspaces: Vec::new(),
            active: true,
        }
    }

    #[must_use]
    pub fn with_event_types(mut self, event_types: Vec<OfficeEventType>) -> Self {
        self.event_types = event_types;
        self
    }

    #[must_use]
    pub fn with_apps(mut self, apps: Vec<impl Into<String>>) -> Self {
        self.apps = apps.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_workspaces(mut self, workspaces: Vec<impl Into<String>>) -> Self {
        self.workspaces = workspaces.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn matches(&self, event: &DomainEvent) -> bool {
        self.active
            && type_matches(&self.event_types, event.event_type)
            && list_matches(&self.apps, &event.app)
            && list_matches(&self.workspaces, &event.workspace_id)
    }
}

fn type_matches(filter: &[OfficeEventType], event_type: OfficeEventType) -> bool {
    filter.is_empty() || filter.contains(&event_type)
}

fn list_matches(filter: &[String], value: &str) -> bool {
    filter.is_empty() || filter.iter().any(|item| item == value)
}

fn tags_match(filter: &[String], tags: &[String]) -> bool {
    filter.is_empty() || tags.iter().any(|tag| filter.contains(tag))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{NativeSubscription, WebhookSubscription};
    use crate::office::event::DomainEvent;
    use crate::office::taxonomy::OfficeEventType;

    fn event(event_type: OfficeEventType) -> DomainEvent {
        DomainEvent::new("evt-1", event_type, "ws-1", "doc-1", "usr-1").with_tag("finance")
    }

    #[test]
    fn filters_combine_with_and_semantics() {
        let subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "v1")
            .with_event_types(vec![OfficeEventType::DocCommentAdded])
            .with_apps(vec!["documents"])
            .with_workspaces(vec!["ws-1"])
            .with_tags(vec!["finance"]);
        assert!(subscription.matches(&event(OfficeEventType::DocCommentAdded)));
        assert!(!subscription.matches(&event(OfficeEventType::DocCreated)));
        assert!(!subscription
            .clone()
            .with_workspaces(vec!["ws-other"])
            .matches(&event(OfficeEventType::DocCommentAdded)));
        assert!(!subscription
            .clone()
            .with_tags(vec!["legal"])
            .matches(&event(OfficeEventType::DocCommentAdded)));
    }

    #[test]
    fn empty_filters_match_everything() {
        let subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "v1");
        assert!(subscription.matches(&event(OfficeEventType::SheetsExported)));
    }

    #[test]
    fn secret_rotation_keeps_old_secret_valid_during_grace() {
        let mut subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "old-value");
        let at = Utc::now();
        subscription.rotate_secret("k2", "new-value", at);
        assert_eq!(
            subscription.active_secret().map(|secret| secret.id.as_str()),
            Some("k2")
        );
        assert!(subscription.secret_is_valid("k1", "old-value"));
        assert!(subscription.secret_is_valid("k2", "new-value"));
        subscription.retire_secret("k1");
        assert!(!subscription.secret_is_valid("k1", "old-value"));
    }

    #[test]
    fn native_subscription_filters_by_event_type() {
        let subscription = NativeSubscription::new("n1", "bot-1")
            .with_event_types(vec![OfficeEventType::SheetsExported]);
        assert!(subscription.matches(&event(OfficeEventType::SheetsExported)));
        assert!(!subscription.matches(&event(OfficeEventType::DocCreated)));
    }
}
