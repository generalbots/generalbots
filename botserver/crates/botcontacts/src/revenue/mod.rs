//! #2548 — revenue attribution / contact-segment bridge.
//!
//! CRM and Campaigns share one contact model. This module reads the campaign
//! evidence tables and the CRM tables through the existing branch pool, then
//! projects them into typed, auditable numbers. No table is created here and
//! no metric is guessed: missing denominators are `null`, not `0`.

pub mod api;
pub mod evaluate;
pub mod fragment;
pub mod model;
pub mod store;

#[cfg(test)]
mod tests;

pub use model::{
    AttributionInput, AttributionTouch, ContactRevenueBridge, DataQuality, DataQualityNote,
    EventCounts, EventKind, InfluencedDeal, MembershipSource, Money, RevenueAttributionReport,
    SegmentMembership, SuppressionRecord, SuppressionSource, TouchSource,
};
