use serde::{Deserialize, Serialize};

/// Enum representing different types of messages in the bot system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageType(pub i32);

impl MessageType {
    /// Regular message from external systems (WhatsApp, Instagram, etc.)
    pub const EXTERNAL: MessageType = MessageType(0);

    /// User message from web interface
    pub const USER: MessageType = MessageType(1);

    /// Bot response (can be regular content or event)
    pub const BOT_RESPONSE: MessageType = MessageType(2);

    /// Continue interrupted response
    pub const CONTINUE: MessageType = MessageType(3);

    /// Suggestion or command message
    pub const SUGGESTION: MessageType = MessageType(4);

    /// Context change notification
    pub const CONTEXT_CHANGE: MessageType = MessageType(5);
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        MessageType(value)
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        value.0
    }
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::USER
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.0 {
            0 => "EXTERNAL",
            1 => "USER",
            2 => "BOT_RESPONSE",
            3 => "CONTINUE",
            4 => "SUGGESTION",
            5 => "CONTEXT_CHANGE",
            _ => "UNKNOWN",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(i32::from(MessageType::USER), 1);
        assert_eq!(MessageType::from(2), MessageType::BOT_RESPONSE);
    }

    #[test]
    fn test_message_type_display() {
        assert_eq!(MessageType::USER.to_string(), "USER");
        assert_eq!(MessageType::BOT_RESPONSE.to_string(), "BOT_RESPONSE");
    }

    #[test]
    fn test_message_type_equality() {
        assert_eq!(MessageType::USER, MessageType(1));
        assert_ne!(MessageType::USER, MessageType::BOT_RESPONSE);
    }
}
