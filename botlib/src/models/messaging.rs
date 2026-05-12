use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::message_types::MessageType;

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_switchers: Vec<String>,
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
            active_switchers: Vec::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Switcher {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl Switcher {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            prompt: None,
            color: None,
            icon: None,
        }
    }

    #[must_use]
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    #[must_use]
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub switchers: Vec<Switcher>,
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
            switchers: Vec::new(),
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
            switchers: Vec::new(),
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
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        }
    }
}
