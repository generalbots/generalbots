use serde::Deserialize;
use diesel::prelude::*;
use diesel::sql_types::Text;

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub object: String,
    pub entry: Vec<WebhookEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEntry {
    pub id: String,
    pub time: Option<i64>,
    pub messaging: Vec<MessagingEvent>,
}

#[derive(Debug, Deserialize)]
pub struct MessagingEvent {
    pub sender: Sender,
    pub recipient: Recipient,
    pub timestamp: Option<i64>,
    pub message: Option<IncomingMessage>,
    pub postback: Option<Postback>,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Recipient {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub mid: String,
    pub text: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Deserialize)]
pub struct Postback {
    pub payload: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub payload: Option<AttachmentPayload>,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentPayload {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyParams {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[derive(Debug, QueryableByName)]
pub struct BotRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
}
