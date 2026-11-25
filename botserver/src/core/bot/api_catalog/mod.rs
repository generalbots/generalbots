//! LLM-facing api command catalog (unified UI/API surface for chat).
//!
//! On web the LLM drives the UI via `__ui_plan__`; on chat/WhatsApp the
//! backend REST surface is otherwise unreachable. This module exposes:
//!
//! - a compact set of **executable commands** (whitelisted, run in-process
//!   against backend state) injected into every channel's system prompt;
//! - the full **API endpoint catalog** (method + path + one-line summary)
//!   searched on demand through `api.find` — the LLM learns the whole
//!   surface without a giant system prompt;
//! - `apps.find` (suite app registry search) and `web.search` (DuckDuckGo),
//!   so the model can decide between UI automation, backend data and the web.
//!
//! The LLM invokes a command by emitting a single
//! `{"__api_call__": {"name": ..., "params": {...}, "compose": true}}`
//! JSON block as the first line of its reply. General-knowledge questions
//! must be answered directly — no api call.

pub(crate) use crate::apps::registry;
pub(crate) use base64::Engine;
pub(crate) use bigdecimal::BigDecimal;
pub(crate) use botcore::shared::state::AppState;
pub(crate) use diesel::RunQueryDsl;
pub(crate) use serde_json::{Value, json};
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;

pub const API_CALL_TRIGGER: &str = "\"__api_call__\":";

/// The declarative command model lives in `crate::apps::commands`. We reuse
/// that single source of truth here so the chat/WhatsApp prompt, the palette
/// and the discovery commands all agree on the same per-app actions.
pub use crate::apps::commands::{AppCommand, command_by_name};

#[derive(Debug, Clone, Copy)]
pub struct ApiEndpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
}

/// Truthfulness label attached to an endpoint before it reaches the LLM.
///
/// `not_implemented` routes are kept in the source inventory so the route
/// contract guard can still assert that the gap is declared with an owner, but
/// they are filtered out of `api.find` — advertising them makes the model plan
/// actions that return 501. Legacy inventory files only know method/path/
/// summary, so the label is derived here instead of being a required field on
/// `ApiEndpoint` (which would break every existing literal).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EndpointStatus {
    #[default]
    Stable,
    Beta,
    NotImplemented,
}

/// Manifest-facing catalog entry: an inventory route plus its status.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CatalogEndpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    #[serde(default)]
    pub status: EndpointStatus,
}

/// Every handler that terminates in `StatusCode::NOT_IMPLEMENTED`, as
/// `(method, path, owner)`. The owner is the crate that must implement the
/// route (or remove it from the inventory) before it may be advertised.
pub const NOT_IMPLEMENTED_ENDPOINTS: &[(&str, &str, &str)] = &[
    (
        "POST",
        "/api/external-drives/connect",
        "botdrive/src/external/routes/mod.rs:132",
    ),
    (
        "POST",
        "/api/external-drives/upload",
        "botdrive/src/external/routes_upload.rs:50",
    ),
    (
        "POST",
        "/api/security/protection/:tool/uninstall",
        "botsecurity-protection/src/protection/api/configure_protection_routes.rs:107",
    ),
    (
        "GET",
        "/scim/v2/Me",
        "botcoredirectory/src/scim/server/configure_scim_routes.rs:85",
    ),
    (
        "POST",
        "/api/m365/connect",
        "botm365/src/handlers.rs:278",
    ),
    (
        "POST",
        "/api/o365/connect",
        "botm365/src/handlers.rs:278",
    ),
    (
        "POST",
        "/conversations/:id/whiteboard/create",
        "botmeet/src/conversations.rs:288",
    ),
    (
        "POST",
        "/conversations/:id/whiteboard/collaborate",
        "botmeet/src/conversations.rs:303",
    ),
    (
        "GET",
        "/api/meet/minutes/:id/export/pdf",
        "botmeet/src/minutes/handlers.rs:313",
    ),
    (
        "POST",
        "/api/auth/2fa/verify",
        "botcoredirectory/src/auth_routes/logout.rs:292",
    ),
    (
        "POST",
        "/api/auth/2fa/resend",
        "botcoredirectory/src/auth_routes/resend_2fa.rs:8",
    ),
];

/// Normalise `:param`, `{param}` and `*param` to `{param}` so path style never
/// decides whether a route is filtered (the inventory mixes all three).
fn canonical_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut rest = path;
    while let Some(pos) = rest.find(|c| c == ':' || c == '*') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 1..];
        let end = after.find('/').unwrap_or(after.len());
        out.push('{');
        out.push_str(&after[..end]);
        out.push('}');
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Status of a documented endpoint. Unknown routes default to `Stable` so the
/// existing 1300+ inventory entries stay truthful by default; only the explicit
/// list above is downgraded.
pub fn endpoint_status(method: &str, path: &str) -> EndpointStatus {
    let path = canonical_path(path);
    if NOT_IMPLEMENTED_ENDPOINTS
        .iter()
        .any(|(m, p, _)| *m == method && canonical_path(p) == path)
    {
        EndpointStatus::NotImplemented
    } else {
        EndpointStatus::Stable
    }
}

impl CatalogEndpoint {
    pub fn from_endpoint(endpoint: &ApiEndpoint) -> Self {
        Self {
            method: endpoint.method,
            path: endpoint.path,
            summary: endpoint.summary,
            status: endpoint_status(endpoint.method, endpoint.path),
        }
    }

    pub fn is_available(&self) -> bool {
        self.status != EndpointStatus::NotImplemented
    }
}

#[derive(diesel::QueryableByName)]
struct ProjectNameRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    project_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_name: String,
}

#[derive(diesel::QueryableByName)]
struct PersonRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    person_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    first_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    last_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    email: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
}

#[derive(diesel::QueryableByName)]
struct InvoiceRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    invoice_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    invoice_number: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    customer_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total: BigDecimal,
}

#[derive(diesel::QueryableByName)]
struct PaymentRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    payment_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    payment_number: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    payer_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    amount: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    invoice_id: Option<Uuid>,
}

#[derive(diesel::QueryableByName)]
struct QuoteRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    quote_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    quote_number: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    customer_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total: BigDecimal,
}

#[derive(diesel::QueryableByName)]
struct ProductRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    product_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    sku: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Text)]
    product_type: String,
    #[diesel(sql_type = diesel::sql_types::Int4)]
    stock_quantity: i32,
}

#[derive(diesel::QueryableByName)]
struct TicketRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    ticket_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    ticket_number: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    subject: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    priority: String,
}

mod archive_drive_files;
mod branch_scope;
mod command_auth;
mod commands_list;
#[cfg(feature = "database")]
mod database_verbs;
mod docs_verbs;
mod execute_command;
mod execute_integrations;
mod find_api_entries;
mod is_api_call_trigger;
mod list_people;
mod sheet_verbs;
#[cfg(feature = "sources")]
mod skills;
// The installed-skill listing is the fallback used only when the source-backed
// `skills.list`/`skills.find` verbs are not compiled in (#2153 warnings).
#[cfg(not(feature = "sources"))]
mod skills_list;

use archive_drive_files::*;
pub(crate) use branch_scope::*;
pub use commands_list::*;
pub use execute_command::*;
use find_api_entries::*;
pub use is_api_call_trigger::*;
use list_people::*;
#[cfg(not(feature = "sources"))]
use skills_list::*;

#[cfg(test)]
mod route_contract_tests {
    use super::*;

    #[test]
    fn route_contract_status_ignores_param_style() {
        let path = "/api/security/protection/{tool}/uninstall";
        assert_eq!(endpoint_status("POST", path), EndpointStatus::NotImplemented);
        assert_eq!(
            endpoint_status("POST", "/api/security/protection/:tool/uninstall"),
            EndpointStatus::NotImplemented
        );
    }

    #[test]
    fn route_contract_status_defaults_to_stable() {
        assert_eq!(endpoint_status("GET", "/api/canvas"), EndpointStatus::Stable);
        let endpoint = ApiEndpoint {
            method: "GET",
            path: "/api/canvas",
            summary: "test",
        };
        assert!(CatalogEndpoint::from_endpoint(&endpoint).is_available());
    }

    #[test]
    fn route_contract_every_gap_has_an_owner() {
        for (method, path, owner) in NOT_IMPLEMENTED_ENDPOINTS {
            assert!(!owner.is_empty(), "{method} {path} has no owner");
        }
    }
}
