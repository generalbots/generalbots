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
//!
//! #2452 criterion 4 — client address: the event and its read DTO carry an
//! optional `ip`. This module never invents one; the request layer supplied it
//! (or did not), so `None` is stored as SQL NULL. The IP is never written to a
//! log line.

use crate::auth::DocUser;
use crate::state::DocState;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text};
use serde::Serialize;

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
    /// Caller-supplied client address. `None` means the request layer had no
    /// IP to pass; it is recorded as SQL NULL, never fabricated here.
    pub ip: Option<String>,
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
            ip: None,
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
            ip: None,
        }
    }

    /// Additive attach point for the request layer's client IP. Existing
    /// constructors stay unchanged so concurrent callers are not broken; a
    /// caller with a resolved address calls `.with_ip(Some(addr))`.
    #[must_use]
    pub fn with_ip(mut self, ip: Option<&str>) -> Self {
        self.ip = ip.map(|addr| addr.to_string());
        self
    }
}

/// Read-side mirror of a `collab_activity` row, now including `ip`.
///
/// The live `/api/activity` SELECT and its `ActivityRow`/`ActivityItem` live in
/// `collab_routes`; this is the Docs-side shape for projecting the same row so
/// the address is not lost between storage and the reader.
#[derive(Debug, Clone, PartialEq, QueryableByName)]
pub struct AccessAuditRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub actor_id: String,
    #[diesel(sql_type = Text)]
    pub actor_name: String,
    #[diesel(sql_type = Text)]
    pub action: String,
    #[diesel(sql_type = Text)]
    pub payload: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub ip: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Serialized audit item returned to a reader. `ip` is part of the response so
/// "who read what, from where" is visible in the same row as actor and time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccessAuditItem {
    pub id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub ip: Option<String>,
    pub created_at: String,
}

impl From<AccessAuditRow> for AccessAuditItem {
    fn from(row: AccessAuditRow) -> Self {
        Self {
            id: row.id,
            actor_id: row.actor_id,
            actor_name: row.actor_name,
            action: row.action,
            payload: serde_json::from_str(&row.payload).unwrap_or(serde_json::Value::Null),
            ip: row.ip,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

/// IP-aware insert. Requires the `ip` column added to `collab_activity` by a
/// migration; the fallback below keeps older deployments writing audit rows.
const INSERT_WITH_IP: &str = "INSERT INTO collab_activity \
     (resource_type, resource_id, actor_id, actor_name, action, payload, ip) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)";
const INSERT_WITHOUT_IP: &str = "INSERT INTO collab_activity \
     (resource_type, resource_id, actor_id, actor_name, action, payload) \
     VALUES ($1, $2, $3, $4, $5, $6)";

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
    let result = diesel::sql_query(INSERT_WITH_IP)
        .bind::<Text, _>(event.resource_type)
        .bind::<Text, _>(&event.resource_id)
        .bind::<Text, _>(&event.actor_id)
        .bind::<Text, _>(&event.actor_name)
        .bind::<Text, _>(event.action)
        .bind::<Text, _>(&payload_json)
        .bind::<Nullable<Text>, _>(event.ip.as_deref())
        .execute(conn);
    if let Err(e) = result {
        log::warn!("docs access audit IP-aware insert failed: {e}");
        // A deployment whose collab_activity predates the ip column must not
        // lose the audit row entirely: fall back to the original column shape.
        let fallback = diesel::sql_query(INSERT_WITHOUT_IP)
            .bind::<Text, _>(event.resource_type)
            .bind::<Text, _>(&event.resource_id)
            .bind::<Text, _>(&event.actor_id)
            .bind::<Text, _>(&event.actor_name)
            .bind::<Text, _>(event.action)
            .bind::<Text, _>(&payload_json)
            .execute(conn);
        if let Err(e) = fallback {
            log::warn!("docs access audit insert failed: {e}");
        }
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
    record_shared_view_with_ip(state, resource_id, None);
}

/// IP-aware variant: the request layer passes the resolved client address, or
/// `None` when no address could be established. Never fabricate one here.
pub fn record_shared_view_with_ip(state: &DocState, resource_id: &str, ip: Option<&str>) {
    record(
        state,
        AccessAuditEvent::shared_view(resource_id).with_ip(ip),
    );
}

/// Call only after the export bytes have actually been produced.
pub fn record_export_access(state: &DocState, user: &DocUser, resource_id: &str, format: &str) {
    record_export_access_with_ip(state, user, resource_id, format, None);
}

/// IP-aware variant, additive to `record_export_access`.
pub fn record_export_access_with_ip(
    state: &DocState,
    user: &DocUser,
    resource_id: &str,
    format: &str,
    ip: Option<&str>,
) {
    record(
        state,
        AccessAuditEvent::export(user.id(), user.display_name(), resource_id, format).with_ip(ip),
    );
}

#[cfg(test)]
mod tests {
    use super::{AccessAuditEvent, AccessAuditItem, AccessAuditRow};

    /// RFC 5737 TEST-NET-3: documentation-only address, never a real client.
    const TEST_IP: &str = "203.0.113.7";

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
        assert_eq!(event.ip, None);
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
        assert_eq!(event.ip, None);
    }

    #[test]
    fn event_with_ip_carries_the_callers_address() {
        let view = AccessAuditEvent::shared_view("doc-1").with_ip(Some(TEST_IP));
        assert_eq!(view.ip.as_deref(), Some(TEST_IP));
        let export =
            AccessAuditEvent::export("owner-1", "Alice", "doc-1", "pdf").with_ip(Some(TEST_IP));
        assert_eq!(export.ip.as_deref(), Some(TEST_IP));
    }

    #[test]
    fn event_without_ip_stays_none() {
        assert_eq!(AccessAuditEvent::shared_view("doc-1").ip, None);
        assert_eq!(
            AccessAuditEvent::export("owner-1", "Alice", "doc-1", "pdf").ip,
            None
        );
        assert_eq!(
            AccessAuditEvent::shared_view("doc-1").with_ip(None).ip,
            None
        );
    }

    /// A row built from an event keeps the address through read + DTO.
    fn row_from(event: &AccessAuditEvent) -> AccessAuditRow {
        AccessAuditRow {
            id: "row-1".to_string(),
            actor_id: event.actor_id.clone(),
            actor_name: event.actor_name.clone(),
            action: event.action.to_string(),
            payload: serde_json::to_string(&event.payload).unwrap_or_else(|_| "{}".to_string()),
            ip: event.ip.clone(),
            created_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn row_and_dto_round_trip_a_present_ip() {
        let event = AccessAuditEvent::shared_view("doc-1").with_ip(Some(TEST_IP));
        let item: AccessAuditItem = row_from(&event).into();
        assert_eq!(item.ip.as_deref(), Some(TEST_IP));
        let json = serde_json::to_value(&item).unwrap_or(serde_json::Value::Null);
        assert_eq!(json.get("ip").and_then(|v| v.as_str()), Some(TEST_IP));
    }

    #[test]
    fn row_and_dto_round_trip_an_absent_ip() {
        let event = AccessAuditEvent::shared_view("doc-1");
        let item: AccessAuditItem = row_from(&event).into();
        assert_eq!(item.ip, None);
        let json = serde_json::to_value(&item).unwrap_or(serde_json::Value::Null);
        assert!(json.get("ip").is_some_and(serde_json::Value::is_null));
    }
}
