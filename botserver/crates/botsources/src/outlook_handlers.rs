use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use super::outlook::{EmailMessage, EmailImportance, EmailRecipient, OutlookService};
use super::sharepoint::SharePointClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMessagesRequest {
    pub access_token: String,
    pub tenant_id: String,
    pub top: u32,
}

pub async fn list_messages(
    Json(req): Json<ListMessagesRequest>,
) -> Result<Json<Vec<EmailMessage>>, (StatusCode, String)> {
    let client = SharePointClient::new(req.tenant_id, req.access_token);
    let url = OutlookService::new(client.clone())
        .me_messages_url(req.top)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<EmailMessage>, String> {
        let resp = client
            .http_client
            .get(&url)
            .header("Authorization", client.build_auth_header())
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        if let Some(arr) = body.get("value").and_then(|v| v.as_array()) {
            for entry in arr {
                let from = entry
                    .get("from")
                    .and_then(|f| f.get("emailAddress"))
                    .map(|a| EmailRecipient {
                        name: a.get("name").and_then(|n| n.as_str()).map(String::from),
                        address: a
                            .get("address")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                    .unwrap_or(EmailRecipient {
                        name: None,
                        address: String::new(),
                    });
                let to_vec = |k: &str| -> Vec<EmailRecipient> {
                    entry
                        .get(k)
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .map(|a| EmailRecipient {
                                    name: a
                                        .get("emailAddress")
                                        .and_then(|x| x.get("name"))
                                        .and_then(|n| n.as_str())
                                        .map(String::from),
                                    address: a
                                        .get("emailAddress")
                                        .and_then(|x| x.get("address"))
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                };
                let importance = match entry.get("importance").and_then(|v| v.as_str()) {
                    Some("high") => EmailImportance::High,
                    Some("low") => EmailImportance::Low,
                    _ => EmailImportance::Normal,
                };
                let received = entry
                    .get("receivedDateTime")
                    .and_then(|t| t.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now);
                let sent = entry
                    .get("sentDateTime")
                    .and_then(|t| t.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&chrono::Utc));
                out.push(EmailMessage {
                    id: entry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    conversation_id: entry
                        .get("conversationId")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    subject: entry
                        .get("subject")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    body_preview: entry
                        .get("bodyPreview")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    body_html: entry
                        .get("body")
                        .and_then(|b| b.get("content"))
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    body_text: None,
                    from,
                    to: to_vec("toRecipients"),
                    cc: to_vec("ccRecipients"),
                    bcc: to_vec("bccRecipients"),
                    received_at: received,
                    sent_at: sent,
                    is_read: entry
                        .get("isRead")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    has_attachments: entry
                        .get("hasAttachments")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    importance,
                    categories: entry
                        .get("categories")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    folder: entry
                        .get("parentFolderId")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                });
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(result))
}
