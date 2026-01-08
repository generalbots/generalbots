use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    pub messaging_service_sid: Option<String>,
    pub status_callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub sid: String,
    pub account_sid: String,
    pub from: String,
    pub to: String,
    pub body: String,
    pub status: MessageStatus,
    pub direction: MessageDirection,
    pub date_created: Option<DateTime<Utc>>,
    pub date_sent: Option<DateTime<Utc>>,
    pub date_updated: Option<DateTime<Utc>>,
    pub price: Option<String>,
    pub price_unit: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub num_segments: Option<u32>,
    pub num_media: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Queued,
    Sending,
    Sent,
    Delivered,
    Undelivered,
    Failed,
    Receiving,
    Received,
    Accepted,
    Scheduled,
    Read,
    PartiallyDelivered,
    Canceled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MessageDirection {
    Inbound,
    OutboundApi,
    OutboundCall,
    OutboundReply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsRequest {
    pub to: String,
    pub body: String,
    pub media_url: Option<Vec<String>>,
    pub status_callback: Option<String>,
    pub max_price: Option<String>,
    pub validity_period: Option<u32>,
    pub schedule_type: Option<ScheduleType>,
    pub send_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingWebhook {
    pub message_sid: String,
    pub account_sid: String,
    pub from: String,
    pub to: String,
    pub body: String,
    pub num_media: Option<u32>,
    pub num_segments: Option<u32>,
    pub sms_sid: Option<String>,
    pub sms_status: Option<String>,
    pub api_version: Option<String>,
    pub from_city: Option<String>,
    pub from_state: Option<String>,
    pub from_zip: Option<String>,
    pub from_country: Option<String>,
    pub to_city: Option<String>,
    pub to_state: Option<String>,
    pub to_zip: Option<String>,
    pub to_country: Option<String>,
    pub media_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCallback {
    pub message_sid: String,
    pub message_status: MessageStatus,
    pub account_sid: String,
    pub from: String,
    pub to: String,
    pub api_version: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioPhoneNumber {
    pub sid: String,
    pub phone_number: String,
    pub friendly_name: String,
    pub capabilities: PhoneCapabilities,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneCapabilities {
    pub sms: bool,
    pub mms: bool,
    pub voice: bool,
    pub fax: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub id: Uuid,
    pub phone_number: String,
    pub messages: Vec<ConversationMessage>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub sid: String,
    pub direction: MessageDirection,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub status: MessageStatus,
}

pub struct TwilioSmsChannel {
    config: TwilioConfig,
    http_client: Client,
    conversations: Arc<RwLock<HashMap<String, ConversationContext>>>,
    base_url: String,
}

impl TwilioSmsChannel {
    pub fn new(config: TwilioConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            base_url: "https://api.twilio.com/2010-04-01".to_string(),
        }
    }

    pub async fn send_sms(&self, request: SendSmsRequest) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages.json",
            self.base_url, self.config.account_sid
        );

        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("To", request.to.clone());
        params.insert("Body", request.body.clone());

        if let Some(ref msid) = self.config.messaging_service_sid {
            params.insert("MessagingServiceSid", msid.clone());
        } else {
            params.insert("From", self.config.from_number.clone());
        }

        if let Some(ref callback) = request.status_callback.or(self.config.status_callback_url.clone()) {
            params.insert("StatusCallback", callback.clone());
        }

        if let Some(ref media_urls) = request.media_url {
            for (i, url) in media_urls.iter().enumerate() {
                params.insert(Box::leak(format!("MediaUrl{}", i).into_boxed_str()), url.clone());
            }
        }

        if let Some(ref max_price) = request.max_price {
            params.insert("MaxPrice", max_price.clone());
        }

        if let Some(validity) = request.validity_period {
            params.insert("ValidityPeriod", validity.to_string());
        }

        if let Some(ref schedule_type) = request.schedule_type {
            params.insert("ScheduleType", "fixed".to_string());
            if let Some(send_at) = request.send_at {
                params.insert("SendAt", send_at.to_rfc3339());
            }
        }

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Unknown error".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        let sms_message = self.convert_response_to_message(message);

        self.update_conversation(&request.to, &sms_message, MessageDirection::OutboundApi)
            .await;

        Ok(sms_message)
    }

    pub async fn send_bulk_sms(
        &self,
        recipients: Vec<String>,
        body: &str,
    ) -> Vec<Result<SmsMessage, TwilioError>> {
        let mut results = Vec::with_capacity(recipients.len());

        for recipient in recipients {
            let request = SendSmsRequest {
                to: recipient,
                body: body.to_string(),
                media_url: None,
                status_callback: None,
                max_price: None,
                validity_period: None,
                schedule_type: None,
                send_at: None,
            };

            let result = self.send_sms(request).await;
            results.push(result);

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        results
    }

    pub async fn get_message(&self, message_sid: &str) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            self.base_url, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Message not found".to_string(),
                    more_info: None,
                    status: 404,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        Ok(self.convert_response_to_message(message))
    }

    pub async fn list_messages(
        &self,
        to: Option<&str>,
        from: Option<&str>,
        date_sent: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<Vec<SmsMessage>, TwilioError> {
        let mut url = format!(
            "{}/Accounts/{}/Messages.json",
            self.base_url, self.config.account_sid
        );

        let mut query_params = Vec::new();
        if let Some(to_number) = to {
            query_params.push(format!("To={}", urlencoding::encode(to_number)));
        }
        if let Some(from_number) = from {
            query_params.push(format!("From={}", urlencoding::encode(from_number)));
        }
        if let Some(date) = date_sent {
            query_params.push(format!("DateSent={}", urlencoding::encode(date)));
        }
        if let Some(size) = page_size {
            query_params.push(format!("PageSize={}", size));
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to list messages".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let list_response: MessageListResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        let messages = list_response
            .messages
            .into_iter()
            .map(|m| self.convert_response_to_message(m))
            .collect();

        Ok(messages)
    }

    pub async fn delete_message(&self, message_sid: &str) -> Result<(), TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            self.base_url, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .delete(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to delete message".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        Ok(())
    }

    pub async fn cancel_scheduled_message(&self, message_sid: &str) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            self.base_url, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&[("Status", "canceled")])
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to cancel message".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        Ok(self.convert_response_to_message(message))
    }

    pub fn parse_incoming_webhook(&self, params: &HashMap<String, String>) -> Result<IncomingWebhook, TwilioError> {
        let message_sid = params
            .get("MessageSid")
            .ok_or_else(|| TwilioError::ParseError("Missing MessageSid".to_string()))?
            .clone();

        let account_sid = params
            .get("AccountSid")
            .ok_or_else(|| TwilioError::ParseError("Missing AccountSid".to_string()))?
            .clone();

        let from = params
            .get("From")
            .ok_or_else(|| TwilioError::ParseError("Missing From".to_string()))?
            .clone();

        let to = params
            .get("To")
            .ok_or_else(|| TwilioError::ParseError("Missing To".to_string()))?
            .clone();

        let body = params.get("Body").cloned().unwrap_or_default();

        let num_media = params
            .get("NumMedia")
            .and_then(|s| s.parse().ok());

        let num_segments = params
            .get("NumSegments")
            .and_then(|s| s.parse().ok());

        let mut media_urls = Vec::new();
        if let Some(count) = num_media {
            for i in 0..count {
                if let Some(url) = params.get(&format!("MediaUrl{}", i)) {
                    media_urls.push(url.clone());
                }
            }
        }

        Ok(IncomingWebhook {
            message_sid,
            account_sid,
            from,
            to,
            body,
            num_media,
            num_segments,
            sms_sid: params.get("SmsSid").cloned(),
            sms_status: params.get("SmsStatus").cloned(),
            api_version: params.get("ApiVersion").cloned(),
            from_city: params.get("FromCity").cloned(),
            from_state: params.get("FromState").cloned(),
            from_zip: params.get("FromZip").cloned(),
            from_country: params.get("FromCountry").cloned(),
            to_city: params.get("ToCity").cloned(),
            to_state: params.get("ToState").cloned(),
            to_zip: params.get("ToZip").cloned(),
            to_country: params.get("ToCountry").cloned(),
            media_urls,
        })
    }

    pub fn parse_status_callback(&self, params: &HashMap<String, String>) -> Result<StatusCallback, TwilioError> {
        let message_sid = params
            .get("MessageSid")
            .ok_or_else(|| TwilioError::ParseError("Missing MessageSid".to_string()))?
            .clone();

        let message_status_str = params
            .get("MessageStatus")
            .ok_or_else(|| TwilioError::ParseError("Missing MessageStatus".to_string()))?;

        let message_status = parse_message_status(message_status_str)?;

        let account_sid = params
            .get("AccountSid")
            .ok_or_else(|| TwilioError::ParseError("Missing AccountSid".to_string()))?
            .clone();

        let from = params.get("From").cloned().unwrap_or_default();
        let to = params.get("To").cloned().unwrap_or_default();

        let error_code = params
            .get("ErrorCode")
            .and_then(|s| s.parse().ok());

        Ok(StatusCallback {
            message_sid,
            message_status,
            account_sid,
            from,
            to,
            api_version: params.get("ApiVersion").cloned(),
            error_code,
            error_message: params.get("ErrorMessage").cloned(),
        })
    }

    pub async fn handle_incoming_message(&self, webhook: IncomingWebhook) -> ConversationContext {
        let message = SmsMessage {
            sid: webhook.message_sid.clone(),
            account_sid: webhook.account_sid.clone(),
            from: webhook.from.clone(),
            to: webhook.to.clone(),
            body: webhook.body.clone(),
            status: MessageStatus::Received,
            direction: MessageDirection::Inbound,
            date_created: Some(Utc::now()),
            date_sent: Some(Utc::now()),
            date_updated: Some(Utc::now()),
            price: None,
            price_unit: None,
            error_code: None,
            error_message: None,
            num_segments: webhook.num_segments,
            num_media: webhook.num_media,
        };

        self.update_conversation(&webhook.from, &message, MessageDirection::Inbound)
            .await
    }

    pub fn generate_twiml_response(&self, message: Option<&str>, media_url: Option<&str>) -> String {
        let mut twiml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Response>");

        if let Some(msg) = message {
            if let Some(media) = media_url {
                twiml.push_str(&format!(
                    "\n  <Message>\n    <Body>{}</Body>\n    <Media>{}</Media>\n  </Message>",
                    escape_xml(msg),
                    escape_xml(media)
                ));
            } else {
                twiml.push_str(&format!(
                    "\n  <Message>{}</Message>",
                    escape_xml(msg)
                ));
            }
        }

        twiml.push_str("\n</Response>");
        twiml
    }

    pub async fn get_conversation(&self, phone_number: &str) -> Option<ConversationContext> {
        let conversations = self.conversations.read().await;
        conversations.get(phone_number).cloned()
    }

    pub async fn get_phone_numbers(&self) -> Result<Vec<TwilioPhoneNumber>, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/IncomingPhoneNumbers.json",
            self.base_url, self.config.account_sid
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to list phone numbers".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let list_response: PhoneNumberListResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        let numbers = list_response
            .incoming_phone_numbers
            .into_iter()
            .map(|p| TwilioPhoneNumber {
                sid: p.sid,
                phone_number: p.phone_number,
                friendly_name: p.friendly_name,
                capabilities: PhoneCapabilities {
                    sms: p.capabilities.sms.unwrap_or(false),
                    mms: p.capabilities.mms.unwrap_or(false),
                    voice: p.capabilities.voice.unwrap_or(false),
                    fax: p.capabilities.fax.unwrap_or(false),
                },
                status: p.status,
            })
            .collect();

        Ok(numbers)
    }

    pub fn validate_webhook_signature(
        &self,
        signature: &str,
        url: &str,
        params: &HashMap<String, String>,
    ) -> bool {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        let mut sorted_params: Vec<(&String, &String)> = params.iter().collect();
        sorted_params.sort_by(|a, b| a.0.cmp(b.0));

        let mut data = url.to_string();
        for (key, value) in sorted_params {
            data.push_str(key);
            data.push_str(value);
        }

        let mut mac = match Hmac::<Sha1>::new_from_slice(self.config.auth_token.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        mac.update(data.as_bytes());
        let result = mac.finalize();
        let computed_signature = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            result.into_bytes(),
        );

        signature == computed_signature
    }

    async fn update_conversation(
        &self,
        phone_number: &str,
        message: &SmsMessage,
        direction: MessageDirection,
    ) -> ConversationContext {
        let mut conversations = self.conversations.write().await;

        let context = conversations
            .entry(phone_number.to_string())
            .or_insert_with(|| ConversationContext {
                id: Uuid::new_v4(),
                phone_number: phone_number.to_string(),
                messages: Vec::new(),
                created_at: Utc::now(),
                last_message_at: Utc::now(),
                metadata: HashMap::new(),
            });

        context.messages.push(ConversationMessage {
            sid: message.sid.clone(),
            direction,
            body: message.body.clone(),
            timestamp: Utc::now(),
            status: message.status,
        });

        context.last_message_at = Utc::now();

        context.clone()
    }

    fn convert_response_to_message(&self, response: TwilioMessageResponse) -> SmsMessage {
        SmsMessage {
            sid: response.sid,
            account_sid: response.account_sid,
            from: response.from.unwrap_or_default(),
            to: response.to,
            body: response.body.unwrap_or_default(),
            status: parse_message_status(&response.status).unwrap_or(MessageStatus::Queued),
            direction: parse_direction(&response.direction.unwrap_or_default()),
            date_created: response.date_created.and_then(|d| DateTime::parse_from_rfc2822(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            date_sent: response.date_sent.and_then(|d| DateTime::parse_from_rfc2822(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            date_updated: response.date_updated.and_then(|d| DateTime::parse_from_rfc2822(&d).ok().map(|dt| dt.with_timezone(&Utc))),
            price: response.price,
            price_unit: response.price_unit,
            error_code: response.error_code,
            error_message: response.error_message,
            num_segments: response.num_segments.and_then(|s| s.parse().ok()),
            num_media: response.num_media.and_then(|s| s.parse().ok()),
        }
    }
}

fn parse_message_status(status: &str) -> Result<MessageStatus, TwilioError> {
    match status.to_lowercase().as_str() {
        "queued" => Ok(MessageStatus::Queued),
        "sending" => Ok(MessageStatus::Sending),
        "sent" => Ok(MessageStatus::Sent),
        "delivered" => Ok(MessageStatus::Delivered),
        "undelivered" => Ok(MessageStatus::Undelivered),
        "failed" => Ok(MessageStatus::Failed),
        "receiving" => Ok(MessageStatus::Receiving),
        "received" => Ok(MessageStatus::Received),
        "accepted" => Ok(MessageStatus::Accepted),
        "scheduled" => Ok(MessageStatus::Scheduled),
        "read" => Ok(MessageStatus::Read),
        "partially_delivered" => Ok(MessageStatus::PartiallyDelivered),
        "canceled" => Ok(MessageStatus::Canceled),
        _ => Err(TwilioError::ParseError(format!("Unknown status: {}", status))),
    }
}

fn parse_direction(direction: &str) -> MessageDirection {
    match direction.to_lowercase().as_str() {
        "inbound" => MessageDirection::Inbound,
        "outbound-api" => MessageDirection::OutboundApi,
        "outbound-call" => MessageDirection::OutboundCall,
        "outbound-reply" => MessageDirection::OutboundReply,
        _ => MessageDirection::OutboundApi,
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[derive(Debug, Deserialize)]
struct TwilioMessageResponse {
    sid: String,
    account_sid: String,
    from: Option<String>,
    to: String,
    body: Option<String>,
    status: String,
    direction: Option<String>,
    date_created: Option<String>,
    date_sent: Option<String>,
    date_updated: Option<String>,
    price: Option<String>,
    price_unit: Option<String>,
    error_code: Option<i32>,
    error_message: Option<String>,
    num_segments: Option<String>,
    num_media: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageListResponse {
    messages: Vec<TwilioMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct PhoneNumberListResponse {
    incoming_phone_numbers: Vec<PhoneNumberResponse>,
}

#[derive(Debug, Deserialize)]
struct PhoneNumberResponse {
    sid: String,
    phone_number: String,
    friendly_name: String,
    capabilities: CapabilitiesResponse,
    status: String,
}

#[derive(Debug, Deserialize)]
struct CapabilitiesResponse {
    sms: Option<bool>,
    mms: Option<bool>,
    voice: Option<bool>,
    fax: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioApiError {
    pub code: i32,
    pub message: String,
    pub more_info: Option<String>,
    pub status: i32,
}

#[derive(Debug, Clone)]
pub enum TwilioError {
    NetworkError(String),
    ApiError(TwilioApiError),
    ParseError(String),
    ConfigError(String),
    InvalidSignature,
}

impl std::fmt::Display for TwilioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::ApiError(e) => write!(f, "Twilio API error {}: {}", e.code, e.message),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::ConfigError(e) => write!(f, "Configuration error: {}", e),
            Self::InvalidSignature => write!(f, "Invalid webhook signature"),
        }
    }
}

impl std::error::Error for TwilioError {}

pub fn create_twilio_config(
    account_sid: &str,
    auth_token: &str,
    from_number: &str,
) -> TwilioConfig {
    TwilioConfig {
        account_sid: account_sid.to_string(),
        auth_token: auth_token.to_string(),
        from_number: from_number.to_string(),
        webhook_url: None,
        status_callback_url: None,
        messaging_service_sid: None,
    }
}
