pub(crate) use axum::{ extract::State, http::StatusCode, response::{Html, IntoResponse, Json}, routing::{get, post}, Form, Router, };
pub(crate) use chrono::{NaiveDateTime, Utc};
pub(crate) use serde::Deserialize;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;
pub(crate) use botcore::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SaveEventRequest {
    pub title: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub all_day: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutotaskRequest {
    pub intent: Option<String>,
}

#[derive(Deserialize)]
pub struct DnsRecordRequest {
    pub hostname: Option<String>,
    pub record_type: Option<String>,
    pub target: Option<String>,
    pub ttl: Option<String>,
}

#[derive(Deserialize)]
pub struct DnsRemoveRequest {
    pub id: Option<String>,
}

mod html_escape;
mod handle_calendar_event_save;
mod handle_autotask_create;
mod handle_dns_search;
mod handle_dns_register;
mod handle_dns_remove;

use html_escape::*;
pub use handle_calendar_event_save::*;
pub use handle_autotask_create::*;
pub use handle_dns_search::*;
pub use handle_dns_register::*;
pub use handle_dns_remove::*;
