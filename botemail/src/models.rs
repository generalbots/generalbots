use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{
    Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid, Varchar,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::schema;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut PgConnection) -> (Uuid, String) + Send + Sync>;

pub type SecretsProvider = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
    pub secrets_provider: SecretsProvider,
}

#[derive(Debug, QueryableByName)]
pub struct EmailAccountBasicRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
}

#[derive(Debug, QueryableByName)]
pub struct ImapCredentialsRow {
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
}

#[derive(Debug, QueryableByName)]
pub struct SmtpCredentialsRow {
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub display_name: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
}

#[derive(Debug, QueryableByName)]
pub struct EmailSearchRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub from_address: String,
    #[diesel(sql_type = Text)]
    pub to_addresses: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub body_text: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct EmailSignatureRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub bot_id: Option<Uuid>,
    #[diesel(sql_type = Varchar)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub content_html: String,
    #[diesel(sql_type = Text)]
    pub content_plain: String,
    #[diesel(sql_type = Bool)]
    pub is_default: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, QueryableByName)]
pub struct EmailAccountRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

pub struct EmailError(pub String);

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for EmailError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

pub struct EmailData {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

pub struct EmailSummary {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub subject: String,
    pub preview: String,
    pub date: String,
    pub read: bool,
}

pub struct EmailContent {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

pub fn extract_user_from_session() -> Result<Uuid, String> {
    Ok(Uuid::new_v4())
}
