use crate::{config::EmailConfig, shared::state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use diesel::prelude::*;
use imap::types::Seq;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::info;
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub text: String,
    date: String,
    read: bool,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveDraftRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct SaveDraftResponse {
    pub success: bool,
    pub draft_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct GetLatestEmailRequest {
    pub from_email: String,
}

#[derive(Debug, Serialize)]
pub struct LatestEmailResponse {
    pub success: bool,
    pub email_text: Option<String>,
    pub message: String,
}

// Custom error type for email operations
struct EmailError(String);

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for EmailError {
    fn from(s: String) -> Self {
        EmailError(s)
    }
}

async fn internal_send_email(config: &EmailConfig, to: &str, subject: &str, body: &str) {
    let email = Message::builder()
        .from(config.from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();
    let creds = Credentials::new(config.username.clone(), config.password.clone());
    SmtpTransport::relay(&config.server)
        .unwrap()
        .port(config.port)
        .credentials(creds)
        .build()
        .send(&email)
        .unwrap();
}

pub async fn list_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<EmailResponse>>, EmailError> {
    let _config = state
        .config
        .as_ref()
        .ok_or_else(|| EmailError("Configuration not available".to_string()))?;

    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|e| EmailError(format!("Failed to create TLS connector: {:?}", e)))?;

    let client = imap::connect(
        (_config.email.server.as_str(), 993),
        _config.email.server.as_str(),
        &tls,
    )
    .map_err(|e| EmailError(format!("Failed to connect to IMAP: {:?}", e)))?;

    let mut session = client
        .login(&_config.email.username, &_config.email.password)
        .map_err(|e| EmailError(format!("Login failed: {:?}", e)))?;

    session
        .select("INBOX")
        .map_err(|e| EmailError(format!("Failed to select INBOX: {:?}", e)))?;

    let messages = session
        .search("ALL")
        .map_err(|e| EmailError(format!("Failed to search emails: {:?}", e)))?;

    let mut email_list = Vec::new();
    let recent_messages: Vec<_> = messages.iter().cloned().collect();
    let recent_messages: Vec<Seq> = recent_messages.into_iter().rev().take(20).collect();

    for seq in recent_messages {
        let fetch_result = session.fetch(seq.to_string(), "RFC822");
        let messages =
            fetch_result.map_err(|e| EmailError(format!("Failed to fetch email: {:?}", e)))?;

        for msg in messages.iter() {
            let body = msg
                .body()
                .ok_or_else(|| EmailError("No body found".to_string()))?;

            let parsed = parse_mail(body)
                .map_err(|e| EmailError(format!("Failed to parse email: {:?}", e)))?;

            let headers = parsed.get_headers();
            let subject = headers.get_first_value("Subject").unwrap_or_default();
            let from = headers.get_first_value("From").unwrap_or_default();
            let date = headers.get_first_value("Date").unwrap_or_default();

            let body_text = if let Some(body_part) = parsed
                .subparts
                .iter()
                .find(|p| p.ctype.mimetype == "text/plain")
            {
                body_part.get_body().unwrap_or_default()
            } else {
                parsed.get_body().unwrap_or_default()
            };

            let preview = body_text.lines().take(3).collect::<Vec<_>>().join(" ");
            let preview_truncated = if preview.len() > 150 {
                format!("{}...", &preview[..150])
            } else {
                preview
            };

            let (from_name, from_email) = parse_from_field(&from);
            email_list.push(EmailResponse {
                id: seq.to_string(),
                name: from_name,
                email: from_email,
                subject,
                text: preview_truncated,
                date,
                read: false,
                labels: vec![],
            });
        }
    }

    session.logout().ok();
    Ok(Json(email_list))
}

fn parse_from_field(from: &str) -> (String, String) {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            let name = from[..start].trim().trim_matches('"').to_string();
            let email = from[start + 1..end].to_string();
            return (name, email);
        }
    }
    (String::new(), from.to_string())
}

async fn save_email_draft(
    config: &EmailConfig,
    draft_data: &SaveDraftRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    let draft_id = uuid::Uuid::new_v4().to_string();
    Ok(draft_id)
}

pub async fn save_draft(
    State(state): State<Arc<AppState>>,
    Json(draft_data): Json<SaveDraftRequest>,
) -> Result<Json<SaveDraftResponse>, EmailError> {
    let config = state
        .config
        .as_ref()
        .ok_or_else(|| EmailError("Configuration not available".to_string()))?;

    match save_email_draft(&config.email, &draft_data).await {
        Ok(draft_id) => Ok(Json(SaveDraftResponse {
            success: true,
            draft_id: Some(draft_id),
            message: "Draft saved successfully".to_string(),
        })),
        Err(e) => Ok(Json(SaveDraftResponse {
            success: false,
            draft_id: None,
            message: format!("Failed to save draft: {}", e),
        })),
    }
}

async fn fetch_latest_email_from_sender(
    config: &EmailConfig,
    from_email: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((config.server.as_str(), 993), config.server.as_str(), &tls)?;
    let mut session = client.login(&config.username, &config.password)?;
    session.select("INBOX")?;

    let search_query = format!("FROM \"{}\"", from_email);
    let messages = session.search(&search_query)?;

    if let Some(&seq) = messages.last() {
        let fetch_result = session.fetch(seq.to_string(), "RFC822")?;
        for msg in fetch_result.iter() {
            if let Some(body) = msg.body() {
                let parsed = parse_mail(body)?;
                let body_text = if let Some(body_part) = parsed
                    .subparts
                    .iter()
                    .find(|p| p.ctype.mimetype == "text/plain")
                {
                    body_part.get_body().unwrap_or_default()
                } else {
                    parsed.get_body().unwrap_or_default()
                };
                session.logout().ok();
                return Ok(body_text);
            }
        }
    }

    session.logout().ok();
    Err("No email found from sender".into())
}

pub async fn get_latest_email_from(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetLatestEmailRequest>,
) -> Result<Json<LatestEmailResponse>, EmailError> {
    let config = state
        .config
        .as_ref()
        .ok_or_else(|| EmailError("Configuration not available".to_string()))?;

    match fetch_latest_email_from_sender(&config.email, &request.from_email).await {
        Ok(email_text) => Ok(Json(LatestEmailResponse {
            success: true,
            email_text: Some(email_text),
            message: "Email retrieved successfully".to_string(),
        })),
        Err(e) => Ok(Json(LatestEmailResponse {
            success: false,
            email_text: None,
            message: format!("Failed to retrieve email: {}", e),
        })),
    }
}

pub async fn send_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<(String, String, String)>,
) -> Result<StatusCode, EmailError> {
    let (to, subject, body) = payload;
    info!("To: {}", to);
    info!("Subject: {}", subject);
    info!("Body: {}", body);

    let config = state
        .config
        .as_ref()
        .ok_or_else(|| EmailError("Configuration not available".to_string()))?;

    internal_send_email(&config.email, &to, &subject, &body).await;
    Ok(StatusCode::OK)
}

pub async fn save_click(
    Path((campaign_id, email)): Path<(String, String)>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Log the click event
    info!(
        "Click tracked - Campaign: {}, Email: {}",
        campaign_id, email
    );

    // Return a 1x1 transparent GIF pixel
    let pixel: Vec<u8> = vec![
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
    ];

    (StatusCode::OK, [("content-type", "image/gif")], pixel)
}

pub async fn get_emails(
    Path(campaign_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> String {
    // Return placeholder response
    info!("Get emails requested for campaign: {}", campaign_id);
    "No emails tracked".to_string()
}
