//! #1441 — account create/update payloads. Split out of `requests.rs`, which
//! is at the 450-line budget; the suite edit form needs both shapes and the
//! shared module re-exports them as `crate::requests::*`.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub annual_revenue: Option<f64>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// #1441 — partial account update; absent fields are left untouched.
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub annual_revenue: Option<f64>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}
