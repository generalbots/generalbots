pub(crate) use crate::settings_billing;
pub(crate) use crate::settings_credentials;
pub(crate) use crate::settings_profile;
pub(crate) use crate::settings_webhooks;
pub(crate) use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
pub(crate) use botcore::shared::state::AppState;
pub(crate) use diesel::RunQueryDsl;
pub(crate) use serde::Deserialize;
pub(crate) use std::sync::Arc;

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TextRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    value: Option<String>,
}

#[derive(Deserialize)]
pub struct GroupCreateForm {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UserCreateForm {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

mod groups_create;
mod html_escape;
mod read_pref;
mod saas_jwt_secret;
mod users_create;

pub use groups_create::*;
pub use html_escape::*;
pub use read_pref::*;
pub use saas_jwt_secret::*;
pub use users_create::*;
