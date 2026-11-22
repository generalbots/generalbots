//! #2548 — HTTP surface for the revenue-attribution bridge.
//!
//! All endpoints are read-only and branch-scoped through the shared
//! `branch_from_jwt` helper. Errors are returned as typed `ContactsError`s so
//! database details never reach the client.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use super::evaluate::build_report;
use super::model::{AttributionTouch, ContactRevenueBridge, RevenueAttributionReport};
use super::store;
use crate::{ContactsError, CrateState};

#[derive(Debug, Deserialize)]
pub struct RevenueQuery {
    pub campaign_id: Option<Uuid>,
    pub segment_id: Option<Uuid>,
}

/// `GET /api/crm/revenue/attribution` — honest, reproducible aggregate.
pub async fn attribution(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Query(query): Query<RevenueQuery>,
) -> Result<Json<RevenueAttributionReport>, ContactsError> {
    let input = store::load_input(&state, &headers, query.campaign_id, query.segment_id)?;
    Ok(Json(build_report(&input)))
}

/// `GET /api/crm/revenue/contacts/:id/bridge` — one contact, all channels,
/// every segment and deal it touched, with sources.
pub async fn contact_bridge(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ContactRevenueBridge>, ContactsError> {
    match store::load_contact_bridge(&state, &headers, id)? {
        Some(bridge) => Ok(Json(bridge)),
        None => Err(ContactsError::NotFound),
    }
}

/// `GET /api/crm/revenue/contacts/:id/timeline` — the reply/open/click
/// timeline that lands on a CRM contact record.
pub async fn contact_timeline(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AttributionTouch>>, ContactsError> {
    let bridge = match store::load_contact_bridge(&state, &headers, id)? {
        Some(value) => value,
        None => return Err(ContactsError::NotFound),
    };
    let mut touches = bridge.touches;
    touches.sort_by_key(|item| item.occurred_at);
    Ok(Json(touches))
}
