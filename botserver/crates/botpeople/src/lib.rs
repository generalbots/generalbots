

pub mod models;
pub mod scope;
pub mod employee;
pub mod attendance;
pub mod payroll;
pub mod time_clock;
pub mod ui;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post, put},
    Json, Router,
};

use bigdecimal::BigDecimal;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod get_default_bot_fn;
mod org_chart_entry;
mod list_people;
mod get_team;
mod list_time_off;

pub use get_default_bot_fn::*;
pub use org_chart_entry::*;
pub use list_people::*;
pub use get_team::*;
pub use list_time_off::*;
