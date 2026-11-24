//! Server-side access audit for Docs (#2452, criterion 4).
//!
//! The share dialog already records its `share` event in the generic
//! `collab_activity` table (see the `record_activity` helper in
//! `collab_routes::helpers` and `GET /api/activity`). This module writes the
//! two *read* sides of that lifecycle to the very same table, using the very
//! same column shape: a successful shared-link read becomes `view` and a
//! successful export becomes `export`, both with `resource_type = 'docs'`.
//!
//! Deliberately no new store: the audit timeline a reader sees through
//! `GET /api/activity?resource_type=docs&resource_id=...` is the one that gains
//! these rows. As with `record_activity`, an insert failure is logged and
//! never fails the read/export it trails; only *successful* operations call
//! into here, so a rejected request cannot emit a success event.
//!
//! Nothing content-bearing is recorded: no document body, no export bytes and
//! no capability token ever enter the event (the token is the bearer's secret;
//! the body is the document itself). The timestamp is the table's
//! `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` column, which the activity
//! reader already renders.

use crate::auth::DocUser;
use crate::state::DocState;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::Text;

/// Resource type the share dialog uses for a Docs document.
const RESOURCE_DOCS: &str = "docs";
/// A capability link is redeemed without a session: there is no verified
/// identity to attribute, so the actor is an explicit guest marker.
const ANONYMOUS_GUEST: &str = "anonymous-guest";
const ANONYMOUS_GUEST_NAME: &str = "Anonymous Guest";

/// The content-free event appended to `collab_activity`.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessAuditEvent {
    pub resource_type: &'static str,
    pub resource_id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: &'static str,
    pub payload: serde_json::Value,
}

impl AccessAuditEvent {
    /// A shared-link document read: anonymous actor, no token in the payload.
    pub fn shared_view(resource_id: &str) -> Self {
        Self {
            resource_type: RESOURCE_DOCS,
            resource_id: resource_id.to_string(),
            actor_id: ANONYMOUS_GUEST.to_string(),
            actor_name: ANONYMOUS_GUEST_NAME.to_string(),
            action: "view",
            payload: serde_json::json!({ "via": "shared-link" }),
        }
    }

    /// An export performed by a verified Docs user: only the format is kept.
    pub fn export(actor_id: &str, actor_name: &str, resource_id: &str, format: &str) -> Self {
        Self {
            resource_type: RESOURCE_DOCS,
            resource_id: resource_id.to_string(),
            actor_id: actor_id.to_string(),
            actor_name: actor_name.to_string(),
            action: "export",
            payload: serde_json::json!({ "format": format }),
        }
    }
}

/// Mirror of `collab_routes::helpers::record_activity`'s INSERT, with the
/// resource type fixed to Docs so a caller cannot attribute an event to a
/// foreign resource. `created_at`/`id` are filled by the table defaults.
fn append(conn: &mut PgConnection, event: &AccessAuditEvent) {
    let payload_json = match serde_json::to_string(&event.payload) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("docs access audit payload serialization failed: {e}");
            "{}".to_string()
        }
    };
    let result = diesel::sql_query(
        "INSERT INTO collab_activity \
         (resource_type, resource_id, actor_id, actor_name, action, payload) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<Text, _>(event.resource_type)
    .bind::<Text, _>(&event.resource_id)
    .bind::<Text, _>(&event.actor_id)
    .bind::<Text, _>(&event.actor_name)
    .bind::<Text, _>(event.action)
    .bind::<Text, _>(&payload_json)
    .execute(conn);
    if let Err(e) = result {
        log::warn!("docs access audit insert failed: {e}");
    }
}

fn record(state: &DocState, event: AccessAuditEvent) {
    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::warn!("docs access audit pool checkout failed: {e}");
            return;
        }
    };
    append(&mut conn, &event);
}

/// Call only after a shared-link document has actually been loaded.
pub fn record_shared_view(state: &DocState, resource_id: &str) {
    record(state, AccessAuditEvent::shared_view(resource_id));
}

/// Call only after the export bytes have actually been produced.
pub fn record_export_access(state: &DocState, user: &DocUser, resource_id: &str, format: &str) {
    record(
        state,
        AccessAuditEvent::export(user.id(), user.display_name(), resource_id, format),
    );
}

#[cfg(test)]
mod tests {
    use super::AccessAuditEvent;

    #[test]
    fn shared_view_is_anonymous_and_content_free() {
        let event = AccessAuditEvent::shared_view("doc-1");
        assert_eq!(event.resource_type, "docs");
        assert_eq!(event.resource_id, "doc-1");
        assert_eq!(event.actor_id, "anonymous-guest");
        assert_eq!(event.action, "view");
        assert!(event.payload.get("token").is_none());
        assert!(event.payload.get("content").is_none());
        assert_eq!(
            event.payload.get("via").and_then(|v| v.as_str()),
            Some("shared-link")
        );
    }

    #[test]
    fn export_keeps_only_owner_and_format() {
        let event = AccessAuditEvent::export("owner-1", "Alice", "doc-1", "pdf");
        assert_eq!(event.resource_type, "docs");
        assert_eq!(event.actor_id, "owner-1");
        assert_eq!(event.actor_name, "Alice");
        assert_eq!(event.action, "export");
        assert_eq!(
            event.payload.get("format").and_then(|v| v.as_str()),
            Some("pdf")
        );
        assert!(event.payload.as_object().is_some_and(|map| map.len() == 1));
    }
}
