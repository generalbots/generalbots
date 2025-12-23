//! SEND TEMPLATE Keywords - Wrapper module
//!
//! This module serves as a wrapper for the messaging template functionality,
//! re-exporting the functions from the messaging module for backward compatibility.
//!
//! BASIC Keywords provided:
//! - SEND_TEMPLATE - Send a templated message to a single recipient
//! - SEND_TEMPLATE_TO - Send templated messages to multiple recipients (bulk)
//! - CREATE_TEMPLATE - Create or update a message template
//! - GET_TEMPLATE - Retrieve a message template
//! - LIST_TEMPLATES - List all available templates
//!
//! Supported Channels:
//! - email - Email messages
//! - whatsapp - WhatsApp messages
//! - sms - SMS text messages
//! - telegram - Telegram messages
//! - push - Push notifications
//!
//! Examples:
//!   ' Send a single templated email
//!   result = SEND_TEMPLATE("welcome", "user@example.com", "email")
//!
//!   ' Send with variables
//!   vars = #{"name": "John", "order_id": "12345"}
//!   result = SEND_TEMPLATE("order_confirmation", "user@example.com", "email", vars)
//!
//!   ' Send to multiple recipients
//!   recipients = ["user1@example.com", "user2@example.com", "user3@example.com"]
//!   result = SEND_TEMPLATE_TO("newsletter", recipients, "email")
//!
//!   ' Create a new template
//!   CREATE_TEMPLATE "welcome", "email", "Welcome!", "Hello {{name}}, welcome to our service!"
//!
//!   ' Retrieve a template
//!   template = GET_TEMPLATE("welcome")
//!   TALK "Template content: " + template.content

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::messaging::register_messaging_keywords;

/// Register all send template keywords
///
/// This function delegates to the messaging module's registration function,
/// providing a convenient alias for backward compatibility and clearer intent.
///
/// ## Keywords Registered
///
/// ### SEND_TEMPLATE
/// Send a templated message to a single recipient.
/// ```basic
/// result = SEND_TEMPLATE("template_name", "recipient", "channel")
/// result = SEND_TEMPLATE("template_name", "recipient", "channel", variables)
/// ```
///
/// ### SEND_TEMPLATE_TO
/// Send a templated message to multiple recipients (bulk sending).
/// ```basic
/// result = SEND_TEMPLATE_TO("template_name", recipients_array, "channel")
/// result = SEND_TEMPLATE_TO("template_name", recipients_array, "channel", variables)
/// ```
///
/// ### CREATE_TEMPLATE
/// Create or update a message template.
/// ```basic
/// CREATE_TEMPLATE "name", "channel", "content"
/// CREATE_TEMPLATE "name", "channel", "subject", "content"  ' For email with subject
/// ```
///
/// ### GET_TEMPLATE
/// Retrieve a message template by name.
/// ```basic
/// template = GET_TEMPLATE("template_name")
/// template = GET_TEMPLATE("template_name", "channel")
/// ```
///
/// ### LIST_TEMPLATES
/// List all available templates.
/// ```basic
/// templates = LIST_TEMPLATES()
/// FOR EACH t IN templates
///   TALK "Template: " + t
/// NEXT
/// ```
///
/// ## Template Variables
///
/// Templates support variable substitution using double curly braces:
/// ```
/// Hello {{name}}, your order {{order_id}} is ready!
/// ```
///
/// Variables are passed as a map:
/// ```basic
/// vars = #{"name": "John", "order_id": "12345"}
/// SEND_TEMPLATE "order_ready", "user@example.com", "email", vars
/// ```
pub fn register_send_template_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering send template keywords...");

    // Delegate to messaging module which contains the actual implementation
    register_messaging_keywords(state, user, engine);

    debug!("Send template keywords registered successfully");
}
