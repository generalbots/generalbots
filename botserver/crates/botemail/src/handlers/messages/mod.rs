pub(crate) use axum::{ extract::{Path, State}, Json, };
pub(crate) use base64::{engine::general_purpose, Engine as _};
pub(crate) use diesel::prelude::*;
pub(crate) use lettre::{ transport::smtp::authentication::{Credentials, Mechanism}, Message, SmtpTransport, Transport, };
#[cfg(feature = "mail")]
pub(crate) use mailparse::{parse_mail, MailHeaderMap};
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;
pub(crate) use crate::models::{ extract_user_from_session, AppState, EmailError, SmtpCredentialsRow, };
#[cfg(feature = "mail")]
pub(crate) use crate::models::ImapCredentialsRow;
pub(crate) use crate::types::{ ApiResponse, EmailResponse, FolderInfo, ListEmailsRequest, SaveDraftRequest, SaveDraftResponse, SendEmailRequest, EmailTrackingParams, };
pub(crate) use crate::handlers::tracking;

#[derive(Debug, serde::Deserialize)]
pub struct BulkActionRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkMarkReadRequest {
    pub ids: Vec<Uuid>,
    pub is_read: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkAddLabelRequest {
    pub ids: Vec<Uuid>,
    pub label_name: String,
}

mod decrypt_password;
mod list_folders;
mod archive_emails_bulk;
mod create_label;
mod add_label_to_emails_bulk;

pub use decrypt_password::*;
pub use list_folders::*;
pub use archive_emails_bulk::*;
pub use create_label::*;
pub use add_label_to_emails_bulk::*;
