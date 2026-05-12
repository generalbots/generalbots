use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramProfile {
    pub id: String,
    pub name: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramWebhookPayload {
    pub object: String,
    pub entry: Vec<InstagramEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramEntry {
    pub id: String,
    pub time: i64,
    #[serde(default)]
    pub messaging: Option<Vec<InstagramMessaging>>,
    #[serde(default)]
    pub changes: Option<Vec<InstagramChange>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramMessaging {
    pub sender: InstagramUser,
    pub recipient: InstagramUser,
    pub timestamp: i64,
    #[serde(default)]
    pub message: Option<InstagramMessage>,
    #[serde(default)]
    pub postback: Option<InstagramPostback>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramUser {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramMessage {
    pub mid: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub attachments: Option<Vec<InstagramAttachment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub payload: InstagramAttachmentPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramAttachmentPayload {
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramPostback {
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramChange {
    pub field: String,
    pub value: serde_json::Value,
}

pub fn create_quick_reply(text: &str, replies: Vec<(&str, &str)>) -> serde_json::Value {
    let quick_replies: Vec<serde_json::Value> = replies
        .into_iter()
        .map(|(title, payload)| {
            serde_json::json!({
                "content_type": "text",
                "title": title,
                "payload": payload
            })
        })
        .collect();

    serde_json::json!({
        "text": text,
        "quick_replies": quick_replies
    })
}

pub fn create_generic_template(elements: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "attachment": {
            "type": "template",
            "payload": {
                "template_type": "generic",
                "elements": elements
            }
        }
    })
}

pub fn create_media_template(media_type: &str, attachment_id: &str) -> serde_json::Value {
    serde_json::json!({
        "attachment": {
            "type": "template",
            "payload": {
                "template_type": "media",
                "elements": [{
                    "media_type": media_type,
                    "attachment_id": attachment_id
                }]
            }
        }
    })
}
