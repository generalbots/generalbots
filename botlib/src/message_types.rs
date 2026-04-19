use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageType(pub i32);

impl MessageType {
    pub const EXTERNAL: Self = Self(0);

    pub const USER: Self = Self(1);

    pub const BOT_RESPONSE: Self = Self(2);

    pub const CONTINUE: Self = Self(3);

    pub const SUGGESTION: Self = Self(4);

    pub const CONTEXT_CHANGE: Self = Self(5);

    pub const TOOL_EXEC: Self = Self(6);
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        value.0
    }
}

impl Default for MessageType {
    fn default() -> Self {
        Self::USER
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
            6 => "TOOL_EXEC",
            _ => "UNKNOWN",
        };
        write!(f, "{name}")
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
