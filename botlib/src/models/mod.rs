pub mod api;
pub mod attachment;
pub mod messaging;
pub mod session;
pub mod workflow;

pub use api::ApiResponse;
pub use attachment::{Attachment, AttachmentType};
pub use messaging::{BotResponse, Suggestion, Switcher, UserMessage};
pub use session::{Session, UserSession};
pub use workflow::{TriggerKind, WorkflowExecution};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_types::MessageType;

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
        let user_id = uuid::Uuid::new_v4();
        let bot_id = uuid::Uuid::new_v4();
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
