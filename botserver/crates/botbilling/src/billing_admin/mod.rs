//! Admin billing endpoints: usage, payment methods, address, invoices,
//! retention offers and subscription lifecycle (upgrade/pause/cancel/accept).
//! The admin billing page (port 3000 `/admin/billing`) consumes these.

pub(crate) use axum::{ extract::State, http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Json, Router, };
pub(crate) use bigdecimal::{BigDecimal, ToPrimitive};
pub(crate) use chrono::Utc;
pub(crate) use diesel::prelude::*;
pub(crate) use serde::Deserialize;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;
pub(crate) use crate::api::BillingApiState;

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(Deserialize)]
pub struct OfferForm {
    pub offer: Option<String>,
}

#[derive(Deserialize)]
pub struct SubscriptionRequest {
    pub plan_id: Option<String>,
    pub reason: Option<String>,
}

mod bd_to_f64;
mod handle_upgrade;
mod handle_subscription_upgrade;

pub use bd_to_f64::*;
pub use handle_upgrade::*;
pub use handle_subscription_upgrade::*;
