
use crate::message_types::MessageType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl<T> ApiResponse<T> {
    #[must_use]
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
            code: None,
        }
    }

    #[must_use]
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: Some(message.into()),
            code: None,
        }
    }

    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            message: None,
            code: None,
        }
    }

    #[must_use]
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            message: None,
            code: Some(code.into()),
        }
    }

    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ApiResponse<U> {
        ApiResponse {
            success: self.success,
            data: self.data.map(f),
            error: self.error,
            message: self.message,
            code: self.code,
        }
    }

    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }
}

impl<T: Default> Default for ApiResponse<T> {
    fn default() -> Self {
        Self::success(T::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Session {
    #[must_use]
    pub fn new(user_id: Uuid, bot_id: Uuid, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            bot_id,
            title: title.into(),
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    #[must_use]
    pub const fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Utc::now() > exp)
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_expired()
    }

    #[must_use]
    pub fn remaining_time(&self) -> Option<chrono::Duration> {
        self.expires_at.map(|exp| exp - Utc::now())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub bot_id: String,
    pub user_id: String,
    pub session_id: String,
    pub channel: String,
    pub content: String,
    pub message_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_name: Option<String>,
}

impl UserMessage {
    #[must_use]
    pub fn text(
        bot_id: impl Into<String>,
        user_id: impl Into<String>,
        session_id: impl Into<String>,
        channel: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            bot_id: bot_id.into(),
            user_id: user_id.into(),
            session_id: session_id.into(),
            channel: channel.into(),
            content: content.into(),
            message_type: MessageType::USER,
            media_url: None,
            timestamp: Utc::now(),
            context_name: None,
        }
    }

    #[must_use]
    pub fn with_media(mut self, url: impl Into<String>) -> Self {
        self.media_url = Some(url.into());
        self
    }

    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context_name = Some(context.into());
        self
    }

    #[must_use]
    pub const fn has_media(&self) -> bool {
        self.media_url.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl Suggestion {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            context: None,
            action: None,
            icon: None,
        }
    }

    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    #[must_use]
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl<S: Into<String>> From<S> for Suggestion {
    fn from(text: S) -> Self {
        Self::new(text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResponse {
    pub bot_id: String,
    pub user_id: String,
    pub session_id: String,
    pub channel: String,
    pub content: String,
    pub message_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_token: Option<String>,
    pub is_complete: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<Suggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_name: Option<String>,
    #[serde(default)]
    pub context_length: usize,
    #[serde(default)]
    pub context_max_length: usize,
}

impl BotResponse {
    #[must_use]
    pub fn new(
        bot_id: impl Into<String>,
        session_id: impl Into<String>,
        user_id: impl Into<String>,
        content: impl Into<String>,
        channel: impl Into<String>,
    ) -> Self {
        Self {
            bot_id: bot_id.into(),
            user_id: user_id.into(),
            session_id: session_id.into(),
            channel: channel.into(),
            content: content.into(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        }
    }

    #[must_use]
    pub fn streaming(
        bot_id: impl Into<String>,
        session_id: impl Into<String>,
        user_id: impl Into<String>,
        channel: impl Into<String>,
        stream_token: impl Into<String>,
    ) -> Self {
        Self {
            bot_id: bot_id.into(),
            user_id: user_id.into(),
            session_id: session_id.into(),
            channel: channel.into(),
            content: String::new(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: Some(stream_token.into()),
            is_complete: false,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        }
    }

    #[must_use]
    pub fn with_suggestions<I, S>(mut self, suggestions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Suggestion>,
    {
        self.suggestions = suggestions.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn add_suggestion(mut self, suggestion: impl Into<Suggestion>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    #[must_use]
    pub fn with_context(
        mut self,
        name: impl Into<String>,
        length: usize,
        max_length: usize,
    ) -> Self {
        self.context_name = Some(name.into());
        self.context_length = length;
        self.context_max_length = max_length;
        self
    }

    pub fn append_content(&mut self, chunk: &str) {
        self.content.push_str(chunk);
    }

    #[must_use]
    pub const fn complete(mut self) -> Self {
        self.is_complete = true;
        self
    }

    #[must_use]
    pub const fn is_streaming(&self) -> bool {
        self.stream_token.is_some() && !self.is_complete
    }

    #[must_use]
    pub const fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }
}

impl Default for BotResponse {
    fn default() -> Self {
        Self {
            bot_id: String::new(),
            user_id: String::new(),
            session_id: String::new(),
            channel: String::new(),
            content: String::new(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub attachment_type: AttachmentType,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    Audio,
    Video,
    Document,
    File,
}

impl Attachment {
    #[must_use]
    pub fn new(attachment_type: AttachmentType, url: impl Into<String>) -> Self {
        Self {
            attachment_type,
            url: url.into(),
            mime_type: None,
            filename: None,
            size: None,
            thumbnail_url: None,
        }
    }

    #[must_use]
    pub fn image(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Image, url)
    }

    #[must_use]
    pub fn audio(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Audio, url)
    }

    #[must_use]
    pub fn video(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Video, url)
    }

    #[must_use]
    pub fn document(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Document, url)
    }

    #[must_use]
    pub fn file(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::File, url)
    }

    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    #[must_use]
    pub const fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    #[must_use]
    pub fn with_thumbnail(mut self, thumbnail_url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(thumbnail_url.into());
        self
    }

    #[must_use]
    pub const fn is_image(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Image)
    }

    #[must_use]
    pub const fn is_media(&self) -> bool {
        matches!(
            self.attachment_type,
            AttachmentType::Image | AttachmentType::Audio | AttachmentType::Video
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("something went wrong");
        assert!(!response.is_success());
        assert!(response.is_error());
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_api_response_map() {
        let response: ApiResponse<i32> = ApiResponse::success(42);
        let mapped = response.map(|n| n.to_string());
        assert_eq!(mapped.data, Some("42".to_string()));
    }

    #[test]
    fn test_session_creation() {
        let user_id = Uuid::new_v4();
        let bot_id = Uuid::new_v4();
        let session = Session::new(user_id, bot_id, "Test Session");

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.bot_id, bot_id);
        assert_eq!(session.title, "Test Session");
        assert!(session.is_active());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_user_message_creation() {
        let msg =
            UserMessage::text("bot1", "user1", "sess1", "web", "Hello!").with_context("greeting");

        assert_eq!(msg.content, "Hello!");
        assert_eq!(msg.message_type, MessageType::USER);
        assert_eq!(msg.context_name, Some("greeting".to_string()));
    }

    #[test]
    fn test_bot_response_creation() {
        let response = BotResponse::new("bot1", "sess1", "user1", "Hi there!", "web")
            .add_suggestion("Option 1")
            .add_suggestion("Option 2");

        assert!(response.is_complete);
        assert!(!response.is_streaming());
        assert!(response.has_suggestions());
        assert_eq!(response.suggestions.len(), 2);
    }

    #[test]
    fn test_bot_response_streaming() {
        let mut response = BotResponse::streaming("bot1", "sess1", "user1", "web", "token123");
        assert!(response.is_streaming());
        assert!(!response.is_complete);

        response.append_content("Hello ");
        response.append_content("World!");
        assert_eq!(response.content, "Hello World!");

        let response = response.complete();
        assert!(!response.is_streaming());
        assert!(response.is_complete);
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment::image("https://example.com/photo.jpg")
            .with_filename("photo.jpg")
            .with_size(1024)
            .with_mime_type("image/jpeg");

        assert!(attachment.is_image());
        assert!(attachment.is_media());
        assert_eq!(attachment.filename, Some("photo.jpg".to_string()));
        assert_eq!(attachment.size, Some(1024));
    }

    #[test]
    fn test_suggestion_from_string() {
        let suggestion: Suggestion = "Click here".into();
        assert_eq!(suggestion.text, "Click here");
        assert!(suggestion.context.is_none());
    }
}
