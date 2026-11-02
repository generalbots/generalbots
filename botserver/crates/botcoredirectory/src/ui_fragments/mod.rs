pub(crate) use crate::api::DirectoryApiState;
pub(crate) use axum::{ extract::{Path, Query, State}, http::StatusCode, response::{Html, IntoResponse}, routing::get, Router, };
pub(crate) use chrono::{DateTime, Utc};
pub(crate) use diesel::RunQueryDsl;
pub(crate) use serde::Deserialize;
pub(crate) use std::sync::Arc;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

mod configure_directory_ui_routes;
mod get_conn;
mod list_users_rows;
mod create_group;
mod group_detail;

pub use configure_directory_ui_routes::*;
use get_conn::*;
pub use list_users_rows::*;
pub use create_group::*;
pub use group_detail::*;
